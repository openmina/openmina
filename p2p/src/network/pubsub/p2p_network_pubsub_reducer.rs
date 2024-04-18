use std::collections::BTreeSet;

use super::{pb, P2pNetworkPubsubAction, P2pNetworkPubsubClientState, P2pNetworkPubsubState};

impl P2pNetworkPubsubState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkPubsubAction>) {
        match action.action() {
            P2pNetworkPubsubAction::NewStream {
                incoming: true,
                peer_id,
                addr,
                stream_id,
                protocol,
                ..
            } => drop(self.clients.insert(
                *peer_id,
                P2pNetworkPubsubClientState {
                    protocol: *protocol,
                    addr: *addr,
                    stream_id: *stream_id,
                    topics: BTreeSet::default(),
                    message: pb::Rpc {
                        subscriptions: vec![],
                        publish: vec![],
                        control: None,
                    },
                },
            )),
            P2pNetworkPubsubAction::NewStream {
                incoming: false,
                peer_id,
                ..
            } => drop(self.servers.insert(*peer_id, ())),
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
            P2pNetworkPubsubAction::Broadcast { data, topic, key } => {
                // TODO: set seqno and add signature
                let message = pb::Message {
                    from: None,
                    data: Some(data.0.clone().into_vec()),
                    seqno: None,
                    topic: topic.clone(),
                    signature: None,
                    key: key.as_ref().map(|v| v.0.clone().into_vec()),
                };
                self.clients
                    .values_mut()
                    .filter(|state| state.topics.contains(&message.topic))
                    .for_each(|state| state.message.publish.push(message.clone()));
            }
            P2pNetworkPubsubAction::OutgoingData { .. } => {}
        }
    }
}
