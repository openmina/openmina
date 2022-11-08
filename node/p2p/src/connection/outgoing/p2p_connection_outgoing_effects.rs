use redux::ActionMeta;

use crate::connection::P2pConnectionService;

use super::{P2pConnectionOutgoingInitAction, P2pConnectionOutgoingPendingAction};

impl P2pConnectionOutgoingInitAction {
    pub fn effects<Store, S>(self, _: ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionOutgoingPendingAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.opts.peer_id;
        store.service().outgoing_init(self.opts);
        store.dispatch(P2pConnectionOutgoingPendingAction { peer_id });
    }
}
