use redux::ActionMeta;

use crate::connection::P2pConnectionService;
use crate::P2pPeerReadyAction;

use super::{
    P2pConnectionOutgoingInitAction, P2pConnectionOutgoingPendingAction,
    P2pConnectionOutgoingRandomInitAction, P2pConnectionOutgoingReconnectAction,
    P2pConnectionOutgoingSuccessAction,
};

impl P2pConnectionOutgoingRandomInitAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionOutgoingInitAction: redux::EnablingCondition<S>,
    {
        let peers = store.state().initial_unused_peers();
        let picked_peer = store.service().random_pick(&peers);
        store.dispatch(P2pConnectionOutgoingInitAction {
            opts: picked_peer,
            rpc_id: None,
        });
    }
}

impl P2pConnectionOutgoingInitAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
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

impl P2pConnectionOutgoingReconnectAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
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

impl P2pConnectionOutgoingSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pPeerReadyAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        store.dispatch(P2pPeerReadyAction { peer_id });
    }
}
