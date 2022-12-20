use redux::ActionMeta;

use crate::rpc::{P2pRpcOutgoingError, P2pRpcService};

use super::{
    P2pRpcOutgoingErrorAction, P2pRpcOutgoingFinishAction, P2pRpcOutgoingInitAction,
    P2pRpcOutgoingPendingAction, P2pRpcOutgoingReceivedAction, P2pRpcOutgoingStatus,
    P2pRpcOutgoingSuccessAction,
};

impl P2pRpcOutgoingInitAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pRpcService,
        P2pRpcOutgoingPendingAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        let rpc_id = self.rpc_id;
        store.service().outgoing_init(peer_id, rpc_id, self.request);
        store.dispatch(P2pRpcOutgoingPendingAction { peer_id, rpc_id });
    }
}

impl P2pRpcOutgoingReceivedAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pRpcService,
        P2pRpcOutgoingErrorAction: redux::EnablingCondition<S>,
        P2pRpcOutgoingSuccessAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        let rpc_id = self.rpc_id;

        let Some(peer) = store.state().peers.get(&peer_id) else { return };
        let Some(peer) = peer.status.as_ready() else { return };

        match peer.rpc.outgoing.get(rpc_id) {
            Some(P2pRpcOutgoingStatus::Received {
                request, response, ..
            }) => match request.validate_response(response) {
                Ok(_) => {
                    store.dispatch(P2pRpcOutgoingSuccessAction { peer_id, rpc_id });
                }
                Err(err) => {
                    store.dispatch(P2pRpcOutgoingErrorAction {
                        peer_id,
                        rpc_id,
                        error: P2pRpcOutgoingError::ResponseInvalid(err),
                    });
                }
            },
            _ => {}
        }
    }
}

impl P2pRpcOutgoingErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pRpcService,
        P2pRpcOutgoingFinishAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        let rpc_id = self.rpc_id;
        store.dispatch(P2pRpcOutgoingFinishAction { peer_id, rpc_id });
    }
}

impl P2pRpcOutgoingSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pRpcService,
        P2pRpcOutgoingFinishAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        let rpc_id = self.rpc_id;
        store.dispatch(P2pRpcOutgoingFinishAction { peer_id, rpc_id });
    }
}
