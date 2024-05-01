use std::collections::BTreeSet;

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
                        });
                state.outgoing_stream_id = Some(*stream_id);
                state.protocol = *protocol;
                state.addr = *addr;

                self.servers.insert(*peer_id, ());
            }
            P2pNetworkPubsubAction::IncomingData { peer_id, data, .. } => {
                let Some(state) = self.clients.get_mut(peer_id) else {
                    return;
                };
                match <pb::Rpc as prost::Message>::decode_length_delimited(&**data) {
                    Ok(v) => {
                        for subscription in v.subscriptions {
                            if subscription.subscribe() {
                                state.topics.insert(subscription.topic_id().to_owned());
                            } else {
                                state.topics.remove(subscription.topic_id());
                            }
                        }
                        for message in v.publish {
                            self.clients
                                .iter_mut()
                                .filter(|(c, state)| {
                                    // don't send back to who sent this
                                    **c != *peer_id && state.topics.contains(&message.topic)
                                })
                                .for_each(|(_, state)| state.message.publish.push(message.clone()));
                        }
                        // TODO: handle control messages
                    }
                    Err(err) => {
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

                let libp2p_peer_id = libp2p_identity::PeerId::from(author.clone());
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
