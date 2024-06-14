use std::collections::BTreeSet;

use binprot::BinProtRead;
use mina_p2p_messages::{gossip, v2};

use super::{pb, P2pNetworkPubsubAction, P2pNetworkPubsubClientState, P2pNetworkPubsubState};

impl P2pNetworkPubsubState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkPubsubAction>) {
        match action.action() {
            P2pNetworkPubsubAction::NewStream {
                incoming: true,
                peer_id,
                addr,
                protocol,
                ..
            } => {
                let state =
                    self.clients
                        .entry(*peer_id)
                        .or_insert_with(|| P2pNetworkPubsubClientState {
                            protocol: *protocol,
                            addr: *addr,
                            outgoing_stream_id: None,
                            topics: BTreeSet::default(),
                            message: pb::Rpc {
                                subscriptions: vec![],
                                publish: vec![],
                                control: None,
                            },
                            buffer: vec![],
                        });
                state.protocol = *protocol;
                state.addr = *addr;
            }
            P2pNetworkPubsubAction::NewStream {
                incoming: false,
                peer_id,
                stream_id,
                addr,
                protocol,
            } => {
                let state =
                    self.clients
                        .entry(*peer_id)
                        .or_insert_with(|| P2pNetworkPubsubClientState {
                            protocol: *protocol,
                            addr: *addr,
                            outgoing_stream_id: Some(*stream_id),
                            topics: BTreeSet::default(),
                            message: pb::Rpc {
                                subscriptions: vec![],
                                publish: vec![],
                                control: None,
                            },
                            buffer: vec![],
                        });
                state.outgoing_stream_id = Some(*stream_id);
                state.protocol = *protocol;
                state.addr = *addr;

                self.servers.insert(*peer_id, ());
            }
            P2pNetworkPubsubAction::IncomingData { peer_id, data, .. } => {
                self.incoming_snarks.clear();
                self.incoming_transactions.take();
                let Some(state) = self.clients.get_mut(peer_id) else {
                    return;
                };
                let slice = if state.buffer.is_empty() {
                    &**data
                } else {
                    state.buffer.extend_from_slice(data);
                    &state.buffer
                };
                match <pb::Rpc as prost::Message>::decode_length_delimited(slice) {
                    Ok(v) => {
                        state.buffer.clear();
                        for subscription in v.subscriptions {
                            if subscription.subscribe() {
                                state.topics.insert(subscription.topic_id().to_owned());
                            } else {
                                state.topics.remove(subscription.topic_id());
                            }
                        }
                        for message in v.publish {
                            if let Some(signature) = &message.signature {
                                // skip recently seen message
                                if !self.seen.contains(signature) {
                                    self.seen.push_back(signature.clone());
                                    // keep only last 256 to avoid memory leak
                                    if self.seen.len() > 256 {
                                        self.seen.pop_front();
                                    }
                                } else {
                                    continue;
                                }
                            }
                            // TODO: verify signature
                            self.clients
                                .iter_mut()
                                .filter(|(c, state)| {
                                    // don't send back to who sent this
                                    **c != *peer_id && state.topics.contains(&message.topic)
                                })
                                .for_each(|(_, state)| state.message.publish.push(message.clone()));

                            if let Some(data) = message.data {
                                if data.len() <= 8 {
                                    continue;
                                }
                                let mut slice = &data[8..];
                                match gossip::GossipNetMessageV2::binprot_read(&mut slice) {
                                    Ok(gossip::GossipNetMessageV2::NewState(block)) => {
                                        self.incoming_block = Some((*peer_id, block));
                                    }
                                    Ok(gossip::GossipNetMessageV2::SnarkPoolDiff {
                                        message,
                                        nonce,
                                    }) => {
                                        if let v2::NetworkPoolSnarkPoolDiffVersionedStableV2::AddSolvedWork(work) = message {
                                            self.incoming_snarks.push((work.1.into(), nonce.as_u32()));
                                        }
                                    }
                                    Ok(gossip::GossipNetMessageV2::TransactionPoolDiff {
                                        message,
                                        nonce,
                                    }) => {
                                        let v2::NetworkPoolTransactionPoolDiffVersionedStableV2(txs) = message;
                                        self.incoming_transactions = Some((txs, nonce.as_u32()));
                                    }
                                    Err(err) => {
                                        dbg!(err);
                                    }
                                }
                            }
                        }
                        // TODO: handle control messages
                    }
                    Err(err) => {
                        // bad way to check the error, but `prost` doesn't provide better
                        if err.to_string().contains("buffer underflow") && state.buffer.is_empty() {
                            state.buffer = data.to_vec();
                        }
                        dbg!(err);
                    }
                }
            }
            P2pNetworkPubsubAction::OutgoingMessage { peer_id, .. } => {
                if let Some(v) = self.clients.get_mut(peer_id) {
                    v.message.subscriptions.clear();
                    v.message.publish.clear();
                    v.message.control = None;
                }
            }
            P2pNetworkPubsubAction::Broadcast { .. } => {}
            P2pNetworkPubsubAction::Sign {
                seqno,
                author,
                data,
                topic,
            } => {
                self.seq += 1;

                let libp2p_peer_id = libp2p_identity::PeerId::from(*author);
                self.to_sign.push_back(pb::Message {
                    from: Some(libp2p_peer_id.to_bytes()),
                    data: Some(data.0.clone().into_vec()),
                    seqno: Some((*seqno).to_be_bytes().to_vec()),
                    topic: topic.clone(),
                    signature: None,
                    key: None,
                });
            }
            P2pNetworkPubsubAction::BroadcastSigned { signature } => {
                if let Some(mut message) = self.to_sign.pop_front() {
                    message.signature = Some(signature.clone().0.to_vec());
                    self.clients
                        .values_mut()
                        .filter(|state| state.topics.contains(&message.topic))
                        .for_each(|state| state.message.publish.push(message.clone()));
                }
            }
            P2pNetworkPubsubAction::OutgoingData { .. } => {}
        }
    }
}
