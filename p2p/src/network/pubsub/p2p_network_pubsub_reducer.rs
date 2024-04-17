use std::collections::BTreeSet;

use super::{P2pNetworkPubsubAction, P2pNetworkPubsubClientState, P2pNetworkPubsubState};

impl P2pNetworkPubsubState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkPubsubAction>) {
        match action.action() {
            P2pNetworkPubsubAction::NewStream {
                incoming: true,
                peer_id,
                protocol,
            } => drop(self.clients.insert(
                *peer_id,
                P2pNetworkPubsubClientState {
                    protocol: *protocol,
                    topics: BTreeSet::default(),
                },
            )),
            P2pNetworkPubsubAction::NewStream {
                incoming: false,
                peer_id,
                protocol,
            } => {
                let _ = (peer_id, protocol);
            }
            P2pNetworkPubsubAction::IncomingData { peer_id, data } => {
                let _ = (peer_id, data);
            }
            P2pNetworkPubsubAction::Broadcast { data, topic } => {
                let _ = (data, topic);
            }
        }
    }
}
