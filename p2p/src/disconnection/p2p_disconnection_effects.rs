use redux::ActionMeta;

use super::{P2pDisconnectionAction, P2pDisconnectionService};

impl P2pDisconnectionAction {
    pub fn effects<Store, S>(&self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pDisconnectionService,
        P2pDisconnectionAction: redux::EnablingCondition<S>,
    {
        match self {
            P2pDisconnectionAction::Init { peer_id, .. } => {
                store.service().disconnect(*peer_id);
                store.dispatch(P2pDisconnectionAction::Finish {
                    peer_id: *peer_id,
                });
            }
            P2pDisconnectionAction::Finish { .. } => {}
        }
    }
}
