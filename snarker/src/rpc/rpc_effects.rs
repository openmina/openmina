use crate::p2p::connection::incoming::P2pConnectionIncomingInitAction;
use crate::p2p::connection::outgoing::P2pConnectionOutgoingInitAction;
use crate::{Service, Store};

use super::{
    ActionStatsQuery, ActionStatsResponse, RpcAction, RpcActionWithMeta, RpcFinishAction,
    RpcP2pConnectionIncomingPendingAction, RpcP2pConnectionOutgoingPendingAction,
};

pub fn rpc_effects<S: Service>(store: &mut Store<S>, action: RpcActionWithMeta) {
    let (action, _) = action.split();

    match action {
        RpcAction::GlobalStateGet(action) => {
            let _ = store
                .service
                .respond_state_get(action.rpc_id, store.state.get());
        }
        RpcAction::ActionStatsGet(action) => match action.query {
            ActionStatsQuery::SinceStart => {
                let resp = store
                    .service
                    .stats()
                    .map(|s| s.collect_action_stats_since_start())
                    .map(|s| ActionStatsResponse::SinceStart(s));
                let _ = store.service.respond_action_stats_get(action.rpc_id, resp);
            }
        },
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
        RpcAction::P2pConnectionIncomingInit(action) => {
            let rpc_id = action.rpc_id;
            store.dispatch(P2pConnectionIncomingInitAction {
                peer_id: action.opts.peer_id,
                signaling: action.opts.signaling,
                offer: action.opts.offer,
                rpc_id: Some(rpc_id),
            });
            store.dispatch(RpcP2pConnectionIncomingPendingAction { rpc_id });
        }
        RpcAction::P2pConnectionIncomingPending(_) => {}
        RpcAction::P2pConnectionIncomingAnswerSet(action) => {
            let _ = store
                .service
                .respond_p2p_connection_incoming_answer(action.rpc_id, Ok(action.answer));
        }
        RpcAction::P2pConnectionIncomingError(action) => {
            let _ = store
                .service
                .respond_p2p_connection_incoming(action.rpc_id, Err(action.error));
            store.dispatch(RpcFinishAction {
                rpc_id: action.rpc_id,
            });
        }
        RpcAction::P2pConnectionIncomingSuccess(action) => {
            let _ = store
                .service
                .respond_p2p_connection_incoming(action.rpc_id, Ok(()));
            store.dispatch(RpcFinishAction {
                rpc_id: action.rpc_id,
            });
        }
        RpcAction::Finish(_) => {}
    }
}
