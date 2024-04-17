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
                },
            )),
            P2pNetworkPubsubAction::NewStream { .. } => {}
            P2pNetworkPubsubAction::IncomingData { peer_id, data } => {
                let Some(state) = self.clients.get_mut(peer_id) else {
                    return;
                };
                match <pb::Rpc as prost::Message>::decode_length_delimited(&**data) {
                    Ok(v) => {
                        for subscription in v.subscriptions {
                            dbg!(&subscription);
                            if subscription.subscribe() {
                                state.topics.insert(subscription.topic_id().to_owned());
                            } else {
                                state.topics.remove(subscription.topic_id());
                            }
                        }
                    }
                    Err(err) => {
                        dbg!(err);
                    }
                }
            }
            P2pNetworkPubsubAction::Broadcast { data, topic } => {
                let _ = (data, topic);
            }
            P2pNetworkPubsubAction::OutgoingData { .. } => {}
        }
    }
}
