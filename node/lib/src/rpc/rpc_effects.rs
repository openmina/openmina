use crate::p2p::connection::outgoing::P2pConnectionOutgoingInitAction;
use crate::Store;

use super::{
    RpcAction, RpcActionWithMeta, RpcFinishAction, RpcP2pConnectionOutgoingPendingAction,
    RpcService,
};

pub fn rpc_effects<S: RpcService>(store: &mut Store<S>, action: RpcActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        RpcAction::GlobalStateGet(action) => {
            store
                .service
                .respond_state_get(action.rpc_id, store.state.get());
        }
        RpcAction::P2pConnectionOutgoingInit(content) => {
            let (rpc_id, opts) = (content.rpc_id, content.opts);
            store.dispatch(P2pConnectionOutgoingInitAction {
                opts,
                rpc_id: Some(rpc_id),
            });
            store.dispatch(RpcP2pConnectionOutgoingPendingAction { rpc_id });
        }
        RpcAction::P2pConnectionOutgoingPending(_) => {}
        RpcAction::P2pConnectionOutgoingError(content) => {
            store
                .service
                .respond_p2p_connection_outgoing(content.rpc_id, Err(content.error));
            store.dispatch(RpcFinishAction {
                rpc_id: content.rpc_id,
            });
        }
        RpcAction::P2pConnectionOutgoingSuccess(content) => {
            store
                .service
                .respond_p2p_connection_outgoing(content.rpc_id, Ok(()));
            store.dispatch(RpcFinishAction {
                rpc_id: content.rpc_id,
            });
        }
        RpcAction::Finish(_) => {}
    }
}
