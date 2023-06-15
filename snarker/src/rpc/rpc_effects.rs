use p2p::connection::P2pConnectionResponse;

use crate::job_commitment::JobCommitmentCreateAction;
use crate::p2p::connection::incoming::P2pConnectionIncomingInitAction;
use crate::p2p::connection::outgoing::P2pConnectionOutgoingInitAction;
use crate::{Service, Store};

use super::{
    ActionStatsQuery, ActionStatsResponse, RpcAction, RpcActionWithMeta, RpcFinishAction,
    RpcP2pConnectionIncomingErrorAction, RpcP2pConnectionIncomingPendingAction,
    RpcP2pConnectionIncomingRespondAction, RpcP2pConnectionOutgoingPendingAction,
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
                    .map(|stats| ActionStatsResponse::SinceStart { stats });
                let _ = store.service.respond_action_stats_get(action.rpc_id, resp);
            }
            ActionStatsQuery::ForLatestBlock => {
                let resp = store
                    .service
                    .stats()
                    .and_then(|s| s.collect_action_stats_for_block_with_id(None))
                    .map(ActionStatsResponse::ForBlock);
                let _ = store.service.respond_action_stats_get(action.rpc_id, resp);
            }
            ActionStatsQuery::ForBlockWithId(id) => {
                let resp = store
                    .service
                    .stats()
                    .and_then(|s| s.collect_action_stats_for_block_with_id(Some(id)))
                    .map(ActionStatsResponse::ForBlock);
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
            let error = Err(format!("{:?}", action.error));
            let _ = store
                .service
                .respond_p2p_connection_outgoing(action.rpc_id, error);
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
            match store
                .state()
                .p2p
                .incoming_accept(action.opts.peer_id, &action.opts.offer)
            {
                Ok(_) => {
                    store.dispatch(P2pConnectionIncomingInitAction {
                        opts: action.opts,
                        rpc_id: Some(rpc_id),
                    });
                    store.dispatch(RpcP2pConnectionIncomingPendingAction { rpc_id });
                }
                Err(reason) => {
                    let response = P2pConnectionResponse::Rejected(reason);
                    store.dispatch(RpcP2pConnectionIncomingRespondAction { rpc_id, response });
                }
            }
        }
        RpcAction::P2pConnectionIncomingPending(_) => {}
        RpcAction::P2pConnectionIncomingRespond(action) => {
            let rpc_id = action.rpc_id;
            let error = match &action.response {
                P2pConnectionResponse::Accepted(_) => None,
                P2pConnectionResponse::InternalError => Some("RemoteInternalError".to_owned()),
                P2pConnectionResponse::Rejected(reason) => Some(format!("Rejected({:?})", reason)),
            };
            let _ = store
                .service
                .respond_p2p_connection_incoming_answer(rpc_id, action.response);
            if let Some(error) = error {
                store.dispatch(RpcP2pConnectionIncomingErrorAction { rpc_id, error });
            }
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
        RpcAction::SnarkerJobPickAndCommit(action) => {
            for job_id in action.available_jobs {
                if store
                    .state()
                    .job_commitments
                    .should_create_commitment(&job_id)
                {
                    if store
                        .service()
                        .respond_snarker_job_pick_and_commit(action.rpc_id, Some(job_id.clone()))
                        .is_err()
                    {
                        return;
                    }
                    store.dispatch(JobCommitmentCreateAction { job_id });
                    return;
                }
            }
            let _ = store
                .service()
                .respond_snarker_job_pick_and_commit(action.rpc_id, None);
        }
        RpcAction::Finish(_) => {}
    }
}
