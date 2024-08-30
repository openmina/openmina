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
                    id: status.channels.next_local_rpc_id(),
                    request: Box::new(P2pRpcRequest::InitialPeers),
                });
            }
            P2pDiscoveryAction::Success { .. } => {}
        }
    }
}
