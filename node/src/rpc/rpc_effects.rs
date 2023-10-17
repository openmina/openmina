use std::time::Duration;

use mina_p2p_messages::v2::MinaBaseTransactionStatusStableV2;

use crate::external_snark_worker::available_job_to_snark_worker_spec;
use crate::p2p::connection::incoming::P2pConnectionIncomingInitAction;
use crate::p2p::connection::outgoing::P2pConnectionOutgoingInitAction;
use crate::p2p::connection::P2pConnectionResponse;
use crate::snark_pool::SnarkPoolCommitmentCreateAction;
use crate::{Service, Store};

use super::{
    ActionStatsQuery, ActionStatsResponse, RpcAction, RpcActionWithMeta, RpcFinishAction,
    RpcP2pConnectionIncomingErrorAction, RpcP2pConnectionIncomingPendingAction,
    RpcP2pConnectionIncomingRespondAction, RpcP2pConnectionOutgoingPendingAction,
    RpcScanStateSummary, RpcScanStateSummaryBlock, RpcScanStateSummaryBlockTransaction,
    RpcScanStateSummaryBlockTransactionKind, RpcScanStateSummaryGetQuery,
    RpcScanStateSummaryScanStateJob, RpcSnarkPoolJobFull, RpcSnarkPoolJobSnarkWork,
    RpcSnarkPoolJobSummary, RpcSnarkerJobCommitResponse, RpcSnarkerJobSpecResponse,
};

macro_rules! respond_or_log {
    ($e:expr, $t:expr) => {
        if let Err(err) = $e {
            openmina_core::log::warn!($t; "Failed to respond: {err}");
        }
    };
}

pub fn rpc_effects<S: Service>(store: &mut Store<S>, action: RpcActionWithMeta) {
    let (action, meta) = action.split();

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
        RpcAction::ScanStateSummaryGet(action) => {
            let state = store.state.get();
            let transition_frontier = &state.transition_frontier;
            let snark_pool = &state.snark_pool;

            let service = &store.service;
            let res = None.or_else(|| {
                let block = match action.query {
                    RpcScanStateSummaryGetQuery::ForBestTip => transition_frontier.best_tip(),
                    RpcScanStateSummaryGetQuery::ForBlockWithHash(hash) => transition_frontier
                        .best_chain
                        .iter()
                        .rev()
                        .find(|b| b.hash == hash),
                    RpcScanStateSummaryGetQuery::ForBlockWithHeight(height) => transition_frontier
                        .best_chain
                        .iter()
                        .rev()
                        .find(|b| b.height() == height),
                }?;
                let coinbases =
                    block
                        .coinbases_iter()
                        .map(|_| RpcScanStateSummaryBlockTransaction {
                            hash: None,
                            kind: RpcScanStateSummaryBlockTransactionKind::Coinbase,
                            status: MinaBaseTransactionStatusStableV2::Applied,
                        });
                let block_summary = RpcScanStateSummaryBlock {
                    hash: block.hash().clone(),
                    height: block.height(),
                    global_slot: block.global_slot(),
                    transactions: block
                        .commands_iter()
                        .map(|tx| RpcScanStateSummaryBlockTransaction {
                            hash: tx.data.hash().ok(),
                            kind: (&tx.data).into(),
                            status: tx.status.clone(),
                        })
                        .chain(coinbases)
                        .collect(),
                    completed_works: block
                        .completed_works_iter()
                        .map(|work| (&work.proofs).into())
                        .collect(),
                };

                let mut scan_state = service.scan_state_summary(block.staged_ledger_hash().clone());
                scan_state.iter_mut().flatten().for_each(|job| match job {
                    RpcScanStateSummaryScanStateJob::Todo {
                        job_id,
                        bundle_job_id,
                        job: kind,
                        seq_no,
                    } => {
                        let Some(data) = snark_pool.get(bundle_job_id) else {
                            return;
                        };
                        let commitment = data.commitment.clone();
                        let snark = data.snark.as_ref().map(|snark| RpcSnarkPoolJobSnarkWork {
                            snarker: snark.work.snarker.clone(),
                            fee: snark.work.fee.clone(),
                            received_t: snark.received_t,
                            sender: snark.sender,
                        });

                        if commitment.is_none() && snark.is_none() {
                            return;
                        }
                        *job = RpcScanStateSummaryScanStateJob::Pending {
                            job_id: job_id.clone(),
                            bundle_job_id: bundle_job_id.clone(),
                            job: kind.clone(),
                            seq_no: *seq_no,
                            commitment,
                            snark,
                        };
                    }
                    _ => {}
                });
                Some(RpcScanStateSummary {
                    block: block_summary,
                    scan_state,
                })
            });
            let _ = store
                .service
                .respond_scan_state_summary_get(action.rpc_id, res);
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
                        snarker: snark.work.snarker.clone(),
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
                            snarker: snark.work.snarker.clone(),
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
        RpcAction::SnarkerConfigGet(action) => {
            let config =
                store
                    .state
                    .get()
                    .config
                    .snarker
                    .as_ref()
                    .map(|config| super::RpcSnarkerConfig {
                        public_key: config.public_key.as_ref().clone(),
                        fee: config.fee.clone(),
                    });
            let _ = store
                .service()
                .respond_snarker_config_get(action.rpc_id, config);
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
            let input = available_job_to_snark_worker_spec(
                job.job.clone(),
                &store.state().transition_frontier,
            );
            // TODO(binier): maybe don't require snarker to be enabled here.
            let Some(config) = store.state.get().config.snarker.as_ref() else {
                return;
            };
            let public_key = config.public_key.clone().into();
            let fee = config.fee.clone();
            let input = match input {
                Ok(instances) => RpcSnarkerJobSpecResponse::Ok(
                    mina_p2p_messages::v2::SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse(Some((
                        mina_p2p_messages::v2::SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0 {
                            instances,
                            fee,
                        },
                        public_key,
                    )))
                ),
                Err(err) => RpcSnarkerJobSpecResponse::Err(err),
            };
            if store
                .service()
                .respond_snarker_job_spec(action.rpc_id, input)
                .is_err()
            {
                return;
            }
        }
        RpcAction::SnarkerWorkersGet(action) => {
            let the_only = store.state().external_snark_worker.0.clone();
            if store
                .service()
                .respond_snarker_workers(action.rpc_id, vec![the_only.into()])
                .is_err()
            {
                return;
            }
        }
        RpcAction::HealthCheck(action) => {
            let some_peers = store
                .state()
                .p2p
                .ready_peers_iter()
                .map(|(peer_id, _peer)| {
                    openmina_core::log::debug!(openmina_core::log::system_time(); "found ready peer: {peer_id}")
                })
                .next()
                .ok_or_else(|| {
                    openmina_core::log::warn!(openmina_core::log::system_time(); "no ready peers");
                    String::from("no ready peers") });
            respond_or_log!(
                store
                    .service()
                    .respond_health_check(action.rpc_id, some_peers),
                meta.time()
            );
        }
        RpcAction::ReadinessCheck(action) => {
            let synced = store
                .service()
                .stats()
                .and_then(|stats| stats.get_sync_time())
                .ok_or_else(|| String::from("Not synced"))
                .and_then(|t| {
                    meta.time().checked_sub(t).ok_or_else(|| {
                        format!("Cannot get duration between {t:?} and {:?}", meta.time())
                    })
                })
                .and_then(|dur| {
                    const THRESH: Duration = Duration::from_secs(60 * 3 * 10);
                    if dur <= THRESH {
                        Ok(())
                    } else {
                        Err(format!(
                            "Synced {:?} ago, which is more than the threshold {:?}",
                            dur, THRESH
                        ))
                    }
                });
            openmina_core::log::debug!(meta.time(); summary = "readiness check", result = format!("{synced:?}"));
            respond_or_log!(
                store.service().respond_health_check(action.rpc_id, synced),
                meta.time()
            );
        }
        RpcAction::Finish(_) => {}
    }
}
