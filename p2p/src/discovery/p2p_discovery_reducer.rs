use openmina_core::warn;

use crate::{P2pKademliaState, common::peer_addrs_iter};

use super::{
    P2pDiscoveryAction, P2pDiscoveryActionWithMetaRef, P2pDiscoveryInitAction,
    P2pDiscoveryKademliaAddRouteAction, P2pDiscoverySuccessAction,
};

impl P2pKademliaState {
    pub fn reducer(&mut self, action: P2pDiscoveryActionWithMetaRef) {
        let (action, meta) = action.split();

        match action {
            P2pDiscoveryAction::KademliaBootstrap(_) => {
                self.is_bootstrapping = true;
            }
            P2pDiscoveryAction::Init(P2pDiscoveryInitAction { .. }) => {}
            P2pDiscoveryAction::Success(P2pDiscoverySuccessAction { peer_id, peers }) => {
                self.peer_timestamp.insert(*peer_id, meta.time());
                let peers = peers.iter().filter_map(|peer| match peer.clone().try_into() {
                    Ok(v) => Some(v),
                    Err(e) => {
                        warn!(meta.time(); "error converting network peer {peer:?}: {e}");
                        None
                    }
                });
                self.known_peers.extend(peer_addrs_iter(peers).filter_map(|v| match v {
                    Ok(v) => Some(v),
                    Err(e) => {
                        warn!(meta.time(); "error collecting addr for peer {e}");
                        None
                    },
                }));
            }
            P2pDiscoveryAction::KademliaInit(..) => {
                self.outgoing_requests += 1;
            }
            P2pDiscoveryAction::KademliaAddRoute(P2pDiscoveryKademliaAddRouteAction {
                peer_id,
                addresses,
            }) => {
                self.routes.insert(*peer_id, addresses.clone());
            }
            P2pDiscoveryAction::KademliaSuccess(action) => {
                // TODO(vlad9486): handle failure, decrement the counter
                self.outgoing_requests -= 1;
                let len = self.known_peers.len();
                self.known_peers.extend(
                    action
                        .peers
                        .iter()
                        .map(|peer_id| {
                            self.routes
                                .get(peer_id)
                                .map(|addrs| (peer_id.clone(), addrs.clone()))
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
            P2pDiscoveryAction::KademliaFailure(_) => {
                if !self.known_peers.is_empty() {
                    self.saturated = Some(meta.time());
                }
                self.outgoing_requests -= 1;
            }
        }
    }
}
