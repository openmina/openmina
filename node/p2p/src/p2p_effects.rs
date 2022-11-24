use redux::ActionMeta;

use crate::rpc::outgoing::P2pRpcOutgoingInitAction;
use crate::rpc::P2pRpcRequest;
use crate::P2pPeerReadyAction;

impl P2pPeerReadyAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pRpcOutgoingInitAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pRpcOutgoingInitAction {
            peer_id: self.peer_id,
            rpc_id: match store.state().get_ready_peer(&self.peer_id) {
                Some(p) => p.rpc.outgoing.next_req_id(),
                None => return,
            },
            request: P2pRpcRequest::MenuGet(()),
        });
        store.dispatch(P2pRpcOutgoingInitAction {
            peer_id: self.peer_id,
            rpc_id: match store.state().get_ready_peer(&self.peer_id) {
                Some(p) => p.rpc.outgoing.next_req_id(),
                None => return,
            },
            request: P2pRpcRequest::TransitionKnowledgeGet(()),
        });
    }
}
