use redux::ActionMeta;

use crate::{
    channels::rpc::{P2pChannelsRpcAction, P2pRpcRequest},
    connection::P2pConnectionService,
    P2pPeerStatus, P2pStore,
};

use super::P2pDiscoveryAction;

impl P2pDiscoveryAction {
    pub fn effects<S, Store>(self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: P2pStore<S>,
        Store::Service: P2pConnectionService,
    {
        match self {
            P2pDiscoveryAction::Init { peer_id } => {
                let Some(peer) = store.state().peers.get(&peer_id) else {
                    return;
                };
                let P2pPeerStatus::Ready(status) = &peer.status else {
                    return;
                };
                store.dispatch(P2pChannelsRpcAction::RequestSend {
                    peer_id,
                    id: status.channels.rpc.next_local_rpc_id(),
                    request: P2pRpcRequest::InitialPeers,
                });
            }
            P2pDiscoveryAction::Success { .. } => {}
            P2pDiscoveryAction::KademliaBootstrap => {
                #[cfg(feature = "p2p-libp2p")]
                {
                    // seed node doesn't have initial peers
                    // it will rely on incoming peers
                    let initial_peers = if !store.state().config.initial_peers.is_empty() {
                        store.state().config.initial_peers.clone()
                    } else if !store.state().kademlia.routes.is_empty() {
                        store
                            .state()
                            .kademlia
                            .routes
                            .values()
                            .flatten()
                            .cloned()
                            .collect()
                    } else {
                        vec![]
                    };

                    if !initial_peers.is_empty() {
                        store.service().start_discovery(initial_peers);
                    }
                }
            }
            P2pDiscoveryAction::KademliaInit => {
                #[cfg(feature = "p2p-libp2p")]
                store.service().find_random_peer();
            }
            P2pDiscoveryAction::KademliaAddRoute { .. } => {}
            P2pDiscoveryAction::KademliaSuccess { .. } => {}
            P2pDiscoveryAction::KademliaFailure { .. } => {}
        }
    }
}
