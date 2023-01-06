use redux::ActionMeta;

use super::{P2pDisconnectionFinishAction, P2pDisconnectionInitAction, P2pDisconnectionService};

impl P2pDisconnectionInitAction {
    pub fn effects<Store, S>(&self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pDisconnectionService,
        P2pDisconnectionFinishAction: redux::EnablingCondition<S>,
    {
        store.service().disconnect(self.peer_id);
        store.dispatch(P2pDisconnectionFinishAction {
            peer_id: self.peer_id,
        });
    }
}
