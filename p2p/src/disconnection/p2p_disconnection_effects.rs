use redux::ActionMeta;

use super::{P2pDisconnectionAction, P2pDisconnectionService};

impl P2pDisconnectionAction {
    pub fn effects<Store, S>(&self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pDisconnectionService,
    {
        match self {
            P2pDisconnectionAction::Init {
                peer_id,
                reason: _reason,
            } => {
                #[cfg(feature = "p2p-libp2p")]
                {
                    store.service().disconnect(*peer_id);
                    store.dispatch(P2pDisconnectionAction::Finish { peer_id: *peer_id });
                }
                #[cfg(not(feature = "p2p-libp2p"))]
                {
                    if let Some((addr, _)) = store
                        .state()
                        .network
                        .scheduler
                        .connections
                        .iter()
                        .find(|(_, conn_state)| conn_state.peer_id() == Some(peer_id))
                    {
                        store.dispatch(crate::P2pNetworkSchedulerAction::Disconnect {
                            addr: *addr,
                            reason: _reason.clone(),
                        });
                        store.dispatch(P2pDisconnectionAction::Finish { peer_id: *peer_id });
                    }
                }
            }
            P2pDisconnectionAction::Finish { .. } => {}
        }
    }
}
