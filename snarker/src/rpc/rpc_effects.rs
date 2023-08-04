use mina_p2p_messages::v2::{StateBodyHash, UnsignedExtendedUInt64Int64ForVersionTagsStableV1};
use p2p::connection::P2pConnectionResponse;

use crate::external_snark_worker::available_job_to_snark_worker_spec;
use crate::p2p::connection::incoming::P2pConnectionIncomingInitAction;
use crate::p2p::connection::outgoing::P2pConnectionOutgoingInitAction;
use crate::snark_pool::SnarkPoolCommitmentCreateAction;
use crate::{Service, Store};

use super::{
    ActionStatsQuery, ActionStatsResponse, RpcAction, RpcActionWithMeta, RpcFinishAction,
    RpcP2pConnectionIncomingErrorAction, RpcP2pConnectionIncomingPendingAction,
    RpcP2pConnectionIncomingRespondAction, RpcP2pConnectionOutgoingPendingAction,
    RpcSnarkPoolJobFull, RpcSnarkPoolJobSnarkWork, RpcSnarkPoolJobSummary,
    RpcSnarkerJobCommitResponse, RpcSnarkerJobSpecResponse,
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
        RpcAction::SyncStatsGet(action) => {
            let resp = store
                .service
                .stats()
                .map(|s| s.collect_sync_stats(action.query.limit));
            let _ = store.service.respond_sync_stats_get(action.rpc_id, resp);
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
        RpcAction::SnarkPoolAvailableJobsGet(action) => {
            let resp = store
                .state()
                .snark_pool
                .range(..)
                .map(|(_, job)| RpcSnarkPoolJobSummary {
                    time: job.time,
                    id: job.id.clone(),
                    commitment: job.commitment.clone(),
                    snark: job.snark.as_ref().map(|snark| RpcSnarkPoolJobSnarkWork {
                        prover: snark.work.prover.clone(),
                        fee: snark.work.fee.clone(),
                        received_t: snark.received_t,
                        sender: snark.sender,
                    }),
                })
                .collect::<Vec<_>>();
            let _ = store.service().respond_snark_pool_get(action.rpc_id, resp);
        }
        RpcAction::SnarkPoolJobGet(action) => {
            let resp = store.state().snark_pool.range(..).find_map(|(_, job)| {
                if &job.id == &action.job_id {
                    Some(RpcSnarkPoolJobFull {
                        time: job.time,
                        id: job.id.clone(),
                        job: job.job.clone(),
                        commitment: job.commitment.clone(),
                        snark: job.snark.as_ref().map(|snark| RpcSnarkPoolJobSnarkWork {
                            prover: snark.work.prover.clone(),
                            fee: snark.work.fee.clone(),
                            received_t: snark.received_t,
                            sender: snark.sender,
                        }),
                    })
                } else {
                    None
                }
            });
            let _ = store
                .service()
                .respond_snark_pool_job_get(action.rpc_id, resp);
        }
        RpcAction::SnarkerJobCommit(action) => {
            let job_id = action.job_id;
            if !store.state().snark_pool.should_create_commitment(&job_id) {
                let _ = store.service().respond_snarker_job_commit(
                    action.rpc_id,
                    RpcSnarkerJobCommitResponse::JobNotFound,
                );
                // TODO(binier): differentiate between job not found and job already taken.
                return;
            }
            if !store.state().external_snark_worker.has_idle() {
                let _ = store.service().respond_snarker_job_commit(
                    action.rpc_id,
                    RpcSnarkerJobCommitResponse::SnarkerBusy,
                );
                return;
            }
            if store
                .service()
                .respond_snarker_job_commit(action.rpc_id, RpcSnarkerJobCommitResponse::Ok)
                .is_err()
            {
                return;
            }
            store.dispatch(SnarkPoolCommitmentCreateAction { job_id });
        }
        RpcAction::SnarkerJobSpec(action) => {
            let job_id = action.job_id;
            let pub_key = store.state().config.public_key.clone();
            let Some(job) = store.state().snark_pool.get(&job_id) else {
                if store
                    .service()
                    .respond_snarker_job_spec(action.rpc_id, RpcSnarkerJobSpecResponse::JobNotFound)
                    .is_err()
                {
                    return;
                }
                return;
            };
            let protocol_state_body = |block_hash: StateBodyHash| {
                store
                    .state()
                    .transition_frontier
                    .best_chain
                    .iter()
                    .find_map(|block_with_hash| {
                        if block_with_hash.block.header.protocol_state.body.hash() == *block_hash {
                            Some(block_with_hash.block.header.protocol_state.body.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            };
            let input = available_job_to_snark_worker_spec(
                pub_key.into(),
                mina_p2p_messages::v2::CurrencyFeeStableV1(
                    UnsignedExtendedUInt64Int64ForVersionTagsStableV1(1_000_000_000_u64.into()),
                ),
                job.job.clone(),
                &protocol_state_body,
            );
            if store
                .service()
                .respond_snarker_job_spec(action.rpc_id, RpcSnarkerJobSpecResponse::Ok(input))
                .is_err()
            {
                return;
            }
        }
        RpcAction::SnarkerWorkersGet(action) => {
            let the_only = store.state().external_snark_worker.clone();
            if store
                .service()
                .respond_snarker_workers(action.rpc_id, vec![the_only.into()])
                .is_err()
            {
                return;
            }
        }
        RpcAction::Finish(_) => {}
    }
}
