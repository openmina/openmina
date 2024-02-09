use crate::P2pKademliaState;

use super::{P2pDiscoveryAction, P2pDiscoveryActionWithMetaRef};

impl P2pKademliaState {
    pub fn reducer(&mut self, action: P2pDiscoveryActionWithMetaRef) {
        let (action, meta) = action.split();

        match action {
            P2pDiscoveryAction::KademliaBootstrap => {
                self.is_bootstrapping = true;
            }
            P2pDiscoveryAction::Init { .. } => {}
            P2pDiscoveryAction::Success { peers, peer_id } => {
                self.peer_timestamp.insert(*peer_id, meta.time());
                self.known_peers
                    .extend(peers.iter().cloned().map(|peer| (*peer.peer_id(), peer)));
            }
            P2pDiscoveryAction::KademliaInit => {
                self.outgoing_requests += 1;
            }
            P2pDiscoveryAction::KademliaAddRoute { peer_id, addresses } => {
                self.routes.insert(*peer_id, addresses.clone());
            }
            P2pDiscoveryAction::KademliaSuccess { peers } => {
                // TODO(vlad9486): handle failure, decrement the counter
                self.outgoing_requests -= 1;
                let len = self.known_peers.len();
                self.known_peers.extend(
                    peers
                        .iter()
                        .map(|peer_id| {
                            // TODO(vlad9486): use all
                            self.routes
                                .get(peer_id)
                                .and_then(|r| r.first())
                                .map(|opts| (opts.peer_id().clone(), opts.clone()))
                        })
                        .flatten(),
                );
                if self.known_peers.len() == len {
                    // this response doesn't yield new peers
                    self.saturated = Some(meta.time());
                } else {
                    self.saturated = None;
                }
            }
            P2pDiscoveryAction::KademliaFailure { .. } => {
                if !self.known_peers.is_empty() {
                    self.saturated = Some(meta.time());
                }
                self.outgoing_requests -= 1;
            }
        }
    }
}
