use std::collections::btree_map::Entry;

use binprot::BinProtRead;
use mina_p2p_messages::{gossip, v2};

use super::{
    p2p_network_pubsub_state::P2pNetworkPubsubClientMeshAddingState, pb, P2pNetworkPubsubAction,
    P2pNetworkPubsubClientState, P2pNetworkPubsubClientTopicState, P2pNetworkPubsubState,
};

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
                let entry = self.clients.entry(*peer_id);
                // preserve it
                let outgoing_stream_id = match &entry {
                    Entry::Occupied(v) => v.get().outgoing_stream_id,
                    Entry::Vacant(_) => None,
                };
                let state = entry.or_insert_with(|| P2pNetworkPubsubClientState {
                    protocol: *protocol,
                    addr: *addr,
                    outgoing_stream_id,
                    message: pb::Rpc {
                        subscriptions: vec![],
                        publish: vec![],
                        control: None,
                    },
                    buffer: vec![],
                });
                state.protocol = *protocol;
                state.addr = *addr;

                self.topics
                    .insert(super::TOPIC.to_owned(), Default::default());
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
            }
            P2pNetworkPubsubAction::IncomingData {
                peer_id,
                data,
                seen_limit,
                ..
            } => {
                self.incoming_transactions.clear();
                self.incoming_snarks.clear();
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
                        // println!(
                        //     "(pubsub) this <- {peer_id}, {:?}, {:?}, {}",
                        //     v.subscriptions,
                        //     v.control,
                        //     v.publish.len()
                        // );

                        for subscription in v.subscriptions {
                            let topic_id = subscription.topic_id().to_owned();
                            let topic = self.topics.entry(topic_id).or_default();

                            if subscription.subscribe() {
                                if let Entry::Vacant(v) = topic.entry(*peer_id) {
                                    v.insert(P2pNetworkPubsubClientTopicState::default());
                                }
                            } else {
                                topic.remove(peer_id);
                            }
                        }
                        for message in v.publish {
                            let message_id = self.mcache.put(message.clone());
                            let topic = self.topics.entry(message.topic.clone()).or_default();
                            if let Some(signature) = &message.signature {
                                // skip recently seen message
                                if !self.seen.contains(signature) {
                                    self.seen.push_back(signature.clone());
                                    // keep only last `n` to avoid memory leak
                                    if self.seen.len() > *seen_limit {
                                        self.seen.pop_front();
                                    }
                                } else {
                                    continue;
                                }
                            }
                            // TODO: verify signature
                            self.clients
                                .iter_mut()
                                .filter(|(c, _)| {
                                    // don't send back to who sent this
                                    **c != *peer_id
                                })
                                .for_each(|(c, state)| {
                                    let Some(topic_state) = topic.get(c) else {
                                        return;
                                    };
                                    if topic_state.on_mesh() {
                                        state.message.publish.push(message.clone())
                                    } else {
                                        let ctr = state
                                            .message
                                            .control
                                            .get_or_insert_with(Default::default);
                                        ctr.ihave.push(pb::ControlIHave {
                                            topic_id: Some(message.topic.clone()),
                                            message_ids: vec![message_id.clone()],
                                        })
                                    }
                                });

                            if let Some(data) = message.data {
                                if data.len() <= 8 {
                                    continue;
                                }
                                let mut slice = &data[8..];
                                match gossip::GossipNetMessageV2::binprot_read(&mut slice) {
                                    Ok(gossip::GossipNetMessageV2::NewState(block)) => {
                                        self.incoming_block = Some((*peer_id, block));
                                    }
                                    Ok(gossip::GossipNetMessageV2::TransactionPoolDiff {
                                        message, nonce,
                                    }) => {
                                        let nonce = nonce.as_u32();
                                        let txs = message.0.into_iter().map(|tx| (tx, nonce));
                                        self.incoming_transactions.extend(txs);
                                    }
                                    Ok(gossip::GossipNetMessageV2::SnarkPoolDiff {
                                        message,
                                        nonce,
                                    }) => {
                                        if let v2::NetworkPoolSnarkPoolDiffVersionedStableV2::AddSolvedWork(work) = message {
                                            self.incoming_snarks.push((work.1.into(), nonce.as_u32()));
                                        }
                                    }
                                    Err(err) => {
                                        dbg!(err);
                                    }
                                }
                            }
                        }
                        if let Some(control) = &v.control {
                            for graft in &control.graft {
                                if let Some(mesh_state) = self
                                    .topics
                                    .get_mut(graft.topic_id())
                                    .and_then(|m| m.get_mut(peer_id))
                                {
                                    mesh_state.mesh = P2pNetworkPubsubClientMeshAddingState::Added;
                                }
                            }
                            for prune in &control.prune {
                                if let Some(mesh_state) = self
                                    .topics
                                    .get_mut(prune.topic_id())
                                    .and_then(|m| m.get_mut(peer_id))
                                {
                                    mesh_state.mesh =
                                        P2pNetworkPubsubClientMeshAddingState::TheyRefused;
                                }
                            }
                            for iwant in &control.iwant {
                                for msg_id in &iwant.message_ids {
                                    if let Some(msg) = self.mcache.map.get(msg_id) {
                                        if let Some(client) = self.clients.get_mut(peer_id) {
                                            client.message.publish.push(msg.clone());
                                        }
                                    }
                                }
                            }
                            for ihave in &control.ihave {
                                let message_ids = ihave
                                    .message_ids
                                    .iter()
                                    .filter(|msg_id| !self.mcache.map.contains_key(*msg_id))
                                    .cloned()
                                    .collect();
                                if let Some(client) = self.clients.get_mut(peer_id) {
                                    let ctr =
                                        client.message.control.get_or_insert_with(Default::default);
                                    ctr.iwant.push(pb::ControlIWant { message_ids })
                                }
                            }
                        }
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
            // we want to add peer to our mesh
            P2pNetworkPubsubAction::Graft { peer_id, topic_id } => {
                let Some(state) = self
                    .topics
                    .get_mut(topic_id)
                    .and_then(|m| m.get_mut(peer_id))
                else {
                    return;
                };

                state.mesh = P2pNetworkPubsubClientMeshAddingState::Added;
            }
            P2pNetworkPubsubAction::Prune { peer_id, topic_id } => {
                let Some(state) = self
                    .topics
                    .get_mut(topic_id)
                    .and_then(|m| m.get_mut(peer_id))
                else {
                    return;
                };

                state.mesh = P2pNetworkPubsubClientMeshAddingState::WeRefused;
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

                let libp2p_peer_id =
                    libp2p_identity::PeerId::try_from(*author).expect("valid peer_id"); // This can't happen unless something is broken in the configuration
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
                        .iter_mut()
                        .for_each(|(_, state)| state.message.publish.push(message.clone()));
                }
            }
            P2pNetworkPubsubAction::OutgoingData { .. } => {}
        }
    }
}
