use crate::p2p::pubsub::P2pPubsubMessagePublishAction;
use crate::Store;
use crate::{
    p2p::connection::outgoing::P2pConnectionOutgoingInitAction,
    watched_accounts::WatchedAccountsAddAction,
};

use super::{
    RpcAction, RpcActionWithMeta, RpcFinishAction, RpcP2pConnectionOutgoingPendingAction,
    RpcService,
};

pub fn rpc_effects<S: RpcService>(store: &mut Store<S>, action: RpcActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        RpcAction::GlobalStateGet(action) => {
            let _ = store
                .service
                .respond_state_get(action.rpc_id, store.state.get());
        }
        RpcAction::P2pConnectionOutgoingInit(action) => {
            let (rpc_id, opts) = (action.rpc_id, action.opts);
            store.dispatch(P2pConnectionOutgoingInitAction {
                opts,
                rpc_id: Some(rpc_id),
            });
            store.dispatch(RpcP2pConnectionOutgoingPendingAction { rpc_id });
        }
        RpcAction::P2pConnectionOutgoingPending(_) => {}
        RpcAction::P2pConnectionOutgoingError(action) => {
            let _ = store
                .service
                .respond_p2p_connection_outgoing(action.rpc_id, Err(action.error));
            store.dispatch(RpcFinishAction {
                rpc_id: action.rpc_id,
            });
        }
        RpcAction::P2pConnectionOutgoingSuccess(action) => {
            let _ = store
                .service
                .respond_p2p_connection_outgoing(action.rpc_id, Ok(()));
            store.dispatch(RpcFinishAction {
                rpc_id: action.rpc_id,
            });
        }
        RpcAction::WatchedAccountsAdd(action) => {
            let enabled = store.dispatch(WatchedAccountsAddAction {
                pub_key: action.pub_key.clone(),
            });
            let _ = store
                .service
                .respond_watched_accounts_add(action.rpc_id, enabled);
        }
        RpcAction::P2pPubsubMessagePublish(action) => {
            store.dispatch(P2pPubsubMessagePublishAction {
                topic: action.topic,
                message: action.message,
                rpc_id: Some(action.rpc_id),
            });
        }
        RpcAction::Finish(_) => {}
    }
}
