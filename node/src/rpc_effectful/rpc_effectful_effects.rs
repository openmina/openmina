use super::RpcEffectfulAction;
use crate::{
    block_producer::BlockProducerWonSlot,
    external_snark_worker::available_job_to_snark_worker_spec,
    p2p::connection::P2pConnectionResponse,
    p2p_ready,
    rpc::{
        AccountQuery, AccountSlim, ActionStatsQuery, ActionStatsResponse, CurrentMessageProgress,
        LedgerSyncProgress, MessagesStats, PeerConnectionStatus, RpcAction, RpcBlockProducerStats,
        RpcMessageProgressResponse, RpcNodeStatus, RpcNodeStatusTransactionPool,
        RpcNodeStatusTransitionFrontier, RpcNodeStatusTransitionFrontierBlockSummary,
        RpcNodeStatusTransitionFrontierSync, RpcPeerInfo, RpcRequestExtraData, RpcScanStateSummary,
        RpcScanStateSummaryBlock, RpcScanStateSummaryBlockTransaction,
        RpcScanStateSummaryBlockTransactionKind, RpcScanStateSummaryScanStateJob,
        RpcSnarkPoolJobFull, RpcSnarkPoolJobSnarkWork, RpcSnarkPoolJobSummary,
        RpcSnarkerJobCommitResponse, RpcSnarkerJobSpecResponse, RpcTransactionInjectResponse,
        TransactionStatus,
    },
    snark_pool::SnarkPoolAction,
    transition_frontier::sync::{
        ledger::TransitionFrontierSyncLedgerState, TransitionFrontierSyncState,
    },
    Service, Store,
};
use ledger::{
    scan_state::currency::{Balance, Magnitude},
    Account,
};
use mina_p2p_messages::{
    rpc_kernel::QueryHeader,
    v2::{MinaBaseTransactionStatusStableV2, TransactionHash},
};
use mina_signer::CompressedPubKey;
use openmina_core::block::ArcBlockWithHash;
use redux::ActionWithMeta;
use std::{collections::BTreeMap, time::Duration};

macro_rules! respond_or_log {
    ($e:expr, $t:expr) => {
        if let Err(err) = $e {
            openmina_core::log::warn!($t; "Failed to respond: {err}");
        }
    };
}

pub fn rpc_effects<S: Service>(store: &mut Store<S>, action: ActionWithMeta<RpcEffectfulAction>) {
    let (action, meta) = action.split();

    match action {
        RpcEffectfulAction::GlobalStateGet { rpc_id, filter } => {
            let _ = store
                .service
                .respond_state_get(rpc_id, (store.state.get(), filter.as_deref()));
        }
        RpcEffectfulAction::StatusGet { rpc_id } => {
            let state = store.state.get();
            let chain_id = state.p2p.ready().map(|p2p| p2p.chain_id.to_hex());
            let block_summary =
                |b: &ArcBlockWithHash| RpcNodeStatusTransitionFrontierBlockSummary {
                    hash: b.hash().clone(),
                    height: b.height(),
                    global_slot: b.global_slot(),
                };
            let status = RpcNodeStatus {
                chain_id,
                transition_frontier: RpcNodeStatusTransitionFrontier {
                    best_tip: state.transition_frontier.best_tip().map(block_summary),
                    sync: RpcNodeStatusTransitionFrontierSync {
                        time: state.transition_frontier.sync.time(),
                        status: state.transition_frontier.sync.to_string(),
                        phase: state.transition_frontier.sync.sync_phase().to_string(),
                        target: state.transition_frontier.sync.best_tip().map(block_summary),
                    },
                },
                peers: collect_rpc_peers_info(state),
                snark_pool: state.snark_pool.jobs_iter().fold(
                    Default::default(),
                    |mut acc, job| {
                        acc.snarks += job.snark.is_some() as usize;
                        acc.total_jobs += 1;
                        acc
                    },
                ),
                transaction_pool: RpcNodeStatusTransactionPool {
                    transactions: state.transaction_pool.size(),
                },
            };
            let _ = store.service.respond_status_get(rpc_id, Some(status));
        }
        RpcEffectfulAction::ActionStatsGet { rpc_id, query } => match query {
            ActionStatsQuery::SinceStart => {
                let resp = store
                    .service
                    .stats()
                    .map(|s| s.collect_action_stats_since_start())
                    .map(|stats| ActionStatsResponse::SinceStart { stats });
                let _ = store.service.respond_action_stats_get(rpc_id, resp);
            }
            ActionStatsQuery::ForLatestBlock => {
                let resp = store
                    .service
                    .stats()
                    .and_then(|s| s.collect_action_stats_for_block_with_id(None))
                    .map(ActionStatsResponse::ForBlock);
                let _ = store.service.respond_action_stats_get(rpc_id, resp);
            }
            ActionStatsQuery::ForBlockWithId(id) => {
                let resp = store
                    .service
                    .stats()
                    .and_then(|s| s.collect_action_stats_for_block_with_id(Some(id)))
                    .map(ActionStatsResponse::ForBlock);
                let _ = store.service.respond_action_stats_get(rpc_id, resp);
            }
        },
        RpcEffectfulAction::SyncStatsGet { rpc_id, query } => {
            let resp = store
                .service
                .stats()
                .map(|s| s.collect_sync_stats(query.limit));
            let _ = store.service.respond_sync_stats_get(rpc_id, resp);
        }
        RpcEffectfulAction::BlockProducerStatsGet { rpc_id } => {
            let mut create_response = || {
                let state = store.state.get();
                let best_tip = state.transition_frontier.best_tip()?;
                let won_slots = &state.block_producer.vrf_evaluator()?.won_slots;

                let stats = store.service.stats()?;
                let attempts = stats.block_producer().collect_attempts();
                let future_slot = attempts.last().map_or(0, |v| v.won_slot.global_slot + 1);

                let cur_global_slot = state.cur_global_slot();
                let slots_per_epoch = best_tip.constants().slots_per_epoch.as_u32();
                let epoch_start =
                    cur_global_slot.map(|slot| (slot / slots_per_epoch) * slots_per_epoch);

                Some(RpcBlockProducerStats {
                    current_time: meta.time(),
                    current_global_slot: cur_global_slot,
                    epoch_start,
                    epoch_end: epoch_start.map(|slot| slot + slots_per_epoch),
                    attempts,
                    future_won_slots: won_slots
                        .range(future_slot..)
                        .map(|(_, won_slot)| {
                            let won_slot = BlockProducerWonSlot::from_vrf_won_slot(
                                won_slot,
                                best_tip.genesis_timestamp(),
                            );
                            (&won_slot).into()
                        })
                        .collect(),
                })
            };
            let response = create_response();
            let _ = store
                .service
                .respond_block_producer_stats_get(rpc_id, response);
        }
        RpcEffectfulAction::MessageProgressGet { rpc_id } => {
            // TODO: move to stats
            let p2p = p2p_ready!(store.state().p2p, meta.time());
            let messages_stats = p2p
                .network
                .scheduler
                .rpc_outgoing_streams
                .iter()
                .filter_map(|(peer_id, streams)| {
                    let (_, rpc_state) = streams.first_key_value()?;
                    let QueryHeader { tag: name, .. } = rpc_state.pending.clone()?;
                    let name = name.to_string();
                    let buffer = &rpc_state.buffer;
                    let current_request = if buffer.len() < 8 {
                        None
                    } else {
                        let received_bytes = buffer.len() - 8;
                        let total_bytes = u64::from_le_bytes(
                            buffer[..8].try_into().expect("cannot fail checked above"),
                        ) as usize;
                        Some(CurrentMessageProgress {
                            name,
                            received_bytes,
                            total_bytes,
                        })
                    };

                    Some((
                        *peer_id,
                        MessagesStats {
                            current_request,
                            responses: rpc_state
                                .total_stats
                                .iter()
                                .map(|((name, _), count)| (name.to_string(), *count))
                                .collect(),
                        },
                    ))
                })
                .collect();

            let mut response = RpcMessageProgressResponse {
                messages_stats,
                staking_ledger_sync: None,
                next_epoch_ledger_sync: None,
                root_ledger_sync: None,
            };

            match &store.state().transition_frontier.sync {
                TransitionFrontierSyncState::StakingLedgerPending(state) => {
                    if let TransitionFrontierSyncLedgerState::Snarked(state) = &state.ledger {
                        response.staking_ledger_sync = state.estimation()
                    }
                }
                TransitionFrontierSyncState::NextEpochLedgerPending(state) => {
                    if let TransitionFrontierSyncLedgerState::Snarked(state) = &state.ledger {
                        response.next_epoch_ledger_sync = state.estimation()
                    }
                }
                TransitionFrontierSyncState::RootLedgerPending(state) => match &state.ledger {
                    TransitionFrontierSyncLedgerState::Snarked(state) => {
                        response.root_ledger_sync = state.estimation()
                    }
                    TransitionFrontierSyncLedgerState::Staged(_) => {
                        // We want to answer with a result that will serve as a 100% complete process for the
                        // frontend while it is still waiting for the staged ledger to complete. Could be cleaner.
                        response.root_ledger_sync = Some(LedgerSyncProgress {
                            fetched: 1,
                            estimation: 1,
                        });
                    }
                    _ => {}
                },
                _ => {}
            }

            let _ = store
                .service
                .respond_message_progress_stats_get(rpc_id, response);
        }
        RpcEffectfulAction::PeersGet { rpc_id, peers } => {
            respond_or_log!(
                store.service().respond_peers_get(rpc_id, peers),
                meta.time()
            );
        }
        RpcEffectfulAction::P2pConnectionOutgoingError { rpc_id, error } => {
            let _ = store
                .service
                .respond_p2p_connection_outgoing(rpc_id, Err(error));

            store.dispatch(RpcAction::Finish { rpc_id });
        }
        RpcEffectfulAction::P2pConnectionOutgoingSuccess { rpc_id } => {
            let _ = store
                .service
                .respond_p2p_connection_outgoing(rpc_id, Ok(()));
            store.dispatch(RpcAction::Finish { rpc_id });
        }
        RpcEffectfulAction::P2pConnectionIncomingRespond { rpc_id, response } => {
            let error = match &response {
                P2pConnectionResponse::Accepted(_) => None,
                P2pConnectionResponse::Rejected(reason) => Some(format!("Rejected({:?})", reason)),
                P2pConnectionResponse::SignalDecryptionFailed => {
                    Some("RemoteSignalDecryptionFailed".to_owned())
                }
                P2pConnectionResponse::InternalError => Some("RemoteInternalError".to_owned()),
            };
            let _ = store
                .service
                .respond_p2p_connection_incoming_answer(rpc_id, response);
            if let Some(error) = error {
                store.dispatch(RpcAction::P2pConnectionIncomingError { rpc_id, error });
            }
        }
        RpcEffectfulAction::P2pConnectionIncomingError { rpc_id, error } => {
            let _ = store
                .service
                .respond_p2p_connection_incoming(rpc_id, Err(error));
            store.dispatch(RpcAction::Finish { rpc_id });
        }
        RpcEffectfulAction::P2pConnectionIncomingSuccess { rpc_id } => {
            let _ = store
                .service
                .respond_p2p_connection_incoming(rpc_id, Ok(()));
            store.dispatch(RpcAction::Finish { rpc_id });
        }
        RpcEffectfulAction::ScanStateSummaryGetSuccess {
            rpc_id,
            mut scan_state,
        } => {
            let req = store.state().rpc.requests.get(&rpc_id);
            let Some(block) = req.and_then(|req| match &req.data {
                RpcRequestExtraData::FullBlockOpt(opt) => opt.as_ref(),
                _ => None,
            }) else {
                let _ = store.service.respond_scan_state_summary_get(
                    rpc_id,
                    Err("target block not found".to_string()),
                );
                return;
            };
            let coinbases = block
                .coinbases_iter()
                .map(|_| RpcScanStateSummaryBlockTransaction {
                    hash: None,
                    kind: RpcScanStateSummaryBlockTransactionKind::Coinbase,
                    status: MinaBaseTransactionStatusStableV2::Applied,
                });
            let block_summary = RpcScanStateSummaryBlock {
                hash: block.hash().clone(),
                height: block.height(),
                global_slot: block.global_slot_since_genesis(),
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

            let snark_pool = &store.state().snark_pool;
            scan_state.iter_mut().flatten().flatten().for_each(|job| {
                if let RpcScanStateSummaryScanStateJob::Todo {
                    job_id,
                    bundle_job_id,
                    job: kind,
                    seq_no,
                } = job
                {
                    let Some(data) = snark_pool.get(bundle_job_id) else {
                        return;
                    };
                    let commitment = data.commitment.clone().map(Box::new);
                    let snark = data
                        .snark
                        .as_ref()
                        .map(|snark| RpcSnarkPoolJobSnarkWork {
                            snarker: snark.work.snarker.clone(),
                            fee: snark.work.fee.clone(),
                            received_t: snark.received_t,
                            sender: snark.sender,
                        })
                        .map(Box::new);

                    if commitment.is_none() && snark.is_none() {
                        return;
                    }
                    *job = RpcScanStateSummaryScanStateJob::Pending {
                        job_id: job_id.clone(),
                        bundle_job_id: bundle_job_id.clone(),
                        job: Box::new(kind.clone()),
                        seq_no: *seq_no,
                        commitment,
                        snark,
                    };
                }
            });
            let res = scan_state.map(|scan_state| RpcScanStateSummary {
                block: block_summary,
                scan_state,
            });
            let _ = store.service.respond_scan_state_summary_get(rpc_id, res);
        }
        RpcEffectfulAction::SnarkPoolAvailableJobsGet { rpc_id } => {
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
            let _ = store.service().respond_snark_pool_get(rpc_id, resp);
        }
        RpcEffectfulAction::SnarkPoolJobGet { job_id, rpc_id } => {
            let resp = store.state().snark_pool.range(..).find_map(|(_, job)| {
                if job.id == job_id {
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
            let _ = store.service().respond_snark_pool_job_get(rpc_id, resp);
        }
        RpcEffectfulAction::SnarkerConfigGet { rpc_id, config } => {
            let _ = store.service().respond_snarker_config_get(rpc_id, config);
        }
        RpcEffectfulAction::SnarkerJobCommit { rpc_id, job_id } => {
            if !store.state().snark_pool.should_create_commitment(&job_id) {
                let _ = store
                    .service()
                    .respond_snarker_job_commit(rpc_id, RpcSnarkerJobCommitResponse::JobNotFound);
                // TODO(binier): differentiate between job not found and job already taken.
                return;
            }
            if !store.state().external_snark_worker.has_idle() {
                let _ = store
                    .service()
                    .respond_snarker_job_commit(rpc_id, RpcSnarkerJobCommitResponse::SnarkerBusy);
                return;
            }
            if store
                .service()
                .respond_snarker_job_commit(rpc_id, RpcSnarkerJobCommitResponse::Ok)
                .is_err()
            {
                return;
            }
            store.dispatch(SnarkPoolAction::CommitmentCreate { job_id });
        }
        RpcEffectfulAction::SnarkerJobSpec { rpc_id, job_id } => {
            let Some(job) = store.state().snark_pool.get(&job_id) else {
                if store
                    .service()
                    .respond_snarker_job_spec(rpc_id, RpcSnarkerJobSpecResponse::JobNotFound)
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

            // TODO: handle potential errors
            let _ = store.service().respond_snarker_job_spec(rpc_id, input);
        }
        RpcEffectfulAction::SnarkerWorkersGet {
            rpc_id,
            snark_worker,
        } => {
            // TODO: handle potential errors
            let _ = store
                .service()
                .respond_snarker_workers(rpc_id, vec![snark_worker.into()]);
        }
        RpcEffectfulAction::HealthCheck { rpc_id, has_peers } => {
            respond_or_log!(
                store.service().respond_health_check(rpc_id, has_peers),
                meta.time()
            );
        }
        RpcEffectfulAction::ReadinessCheck { rpc_id } => {
            const THRESH: Duration = Duration::from_secs(60 * 3 * 10);
            let synced = match store.state().transition_frontier.sync {
                TransitionFrontierSyncState::Synced { time }
                    if meta.time().checked_sub(time) <= Some(THRESH) =>
                {
                    Ok(())
                }
                TransitionFrontierSyncState::Synced { time } => Err(format!(
                    "Synced {:?} ago, which is more than the threshold {:?}",
                    meta.time().checked_sub(time),
                    THRESH
                )),
                _ => Err("not synced".to_owned()),
            };
            // let synced = store
            //     .service()
            //     .stats()
            //     .and_then(|stats| stats.get_sync_time())
            //     .ok_or_else(|| String::from("Not synced"))
            //     .and_then(|t| {
            //         meta.time().checked_sub(t).ok_or_else(|| {
            //             format!("Cannot get duration between {t:?} and {:?}", meta.time())
            //         })
            //     })
            //     .and_then(|dur| {
            //         const THRESH: Duration = Duration::from_secs(60 * 3 * 10);
            //         if dur <= THRESH {
            //             Ok(())
            //         } else {
            //             Err(format!(
            //                 "Synced {:?} ago, which is more than the threshold {:?}",
            //                 dur, THRESH
            //             ))
            //         }
            //     });
            // openmina_core::log::debug!(meta.time(); summary = "readiness check", result = format!("{synced:?}"));
            respond_or_log!(
                store.service().respond_readiness_check(rpc_id, synced),
                meta.time()
            );
        }
        RpcEffectfulAction::DiscoveryRoutingTable { rpc_id, response } => {
            respond_or_log!(
                store
                    .service()
                    .respond_discovery_routing_table(rpc_id, response),
                meta.time()
            );
        }
        RpcEffectfulAction::DiscoveryBoostrapStats { rpc_id, response } => {
            respond_or_log!(
                store
                    .service()
                    .respond_discovery_bootstrap_stats(rpc_id, response),
                meta.time()
            );
        }
        RpcEffectfulAction::TransactionPool { rpc_id, response } => {
            respond_or_log!(
                store.service().respond_transaction_pool(rpc_id, response),
                meta.time()
            )
        }
        RpcEffectfulAction::LedgerAccountsGetSuccess {
            rpc_id,
            accounts,
            account_query,
        } => {
            // TODO(adonagy): maybe something more effective?
            match account_query {
                AccountQuery::SinglePublicKey(_pk) => todo!(),
                // all the accounts for the FE in Slim form
                AccountQuery::All => {
                    let mut accounts: BTreeMap<CompressedPubKey, Account> = accounts
                        .into_iter()
                        .map(|acc| (acc.public_key.clone(), acc))
                        .collect();
                    let nonces_and_amount = store
                        .state()
                        .transaction_pool
                        .get_pending_amount_and_nonce();

                    nonces_and_amount
                        .iter()
                        .for_each(|(account_id, (nonce, amount))| {
                            if let Some(account) = accounts.get_mut(&account_id.public_key) {
                                if let Some(nonce) = nonce {
                                    if nonce >= &account.nonce {
                                        // increment the last nonce in the pool
                                        account.nonce = nonce.incr();
                                    }
                                }
                                account.balance = account
                                    .balance
                                    .sub_amount(*amount)
                                    .unwrap_or(Balance::zero());
                            }
                        });

                    let accounts = accounts
                        .into_values()
                        .map(|v| v.into())
                        .collect::<Vec<AccountSlim>>();

                    respond_or_log!(
                        store
                            .service()
                            .respond_ledger_slim_accounts(rpc_id, accounts),
                        meta.time()
                    )
                }
                // for the graphql endpoint
                AccountQuery::PubKeyWithTokenId(..) => {
                    respond_or_log!(
                        store.service().respond_ledger_accounts(rpc_id, accounts),
                        meta.time()
                    )
                }
            }
        }
        RpcEffectfulAction::TransactionInjectSuccess { rpc_id, response } => {
            respond_or_log!(
                store.service().respond_transaction_inject(
                    rpc_id,
                    RpcTransactionInjectResponse::Success(response)
                ),
                meta.time()
            )
        }
        RpcEffectfulAction::TransactionInjectRejected { rpc_id, response } => {
            respond_or_log!(
                store.service().respond_transaction_inject(
                    rpc_id,
                    RpcTransactionInjectResponse::Rejected(response)
                ),
                meta.time()
            )
        }
        RpcEffectfulAction::TransactionInjectFailure { rpc_id, errors } => {
            respond_or_log!(
                store.service().respond_transaction_inject(
                    rpc_id,
                    RpcTransactionInjectResponse::Failure(errors)
                ),
                meta.time()
            )
        }
        RpcEffectfulAction::TransitionFrontierUserCommandsGet { rpc_id, commands } => {
            respond_or_log!(
                store
                    .service()
                    .respond_transition_frontier_commands(rpc_id, commands),
                meta.time()
            )
        }
        RpcEffectfulAction::BestChain { rpc_id, best_chain } => {
            respond_or_log!(
                store.service().respond_best_chain(rpc_id, best_chain),
                meta.time()
            )
        }
        RpcEffectfulAction::ConsensusConstantsGet { rpc_id, response } => {
            respond_or_log!(
                store
                    .service()
                    .respond_consensus_constants(rpc_id, response),
                meta.time()
            )
        }
        RpcEffectfulAction::TransactionStatusGet { rpc_id, tx } => {
            // Check if the transaction is in the pool, if it is, return PENDING
            let tx_hash = tx.hash().ok();

            let in_tx_pool = store
                .state()
                .transaction_pool
                .get_all_transactions()
                .iter()
                .any(|tx_with_hash| {
                    Some(TransactionHash::from(tx_with_hash.hash.as_ref())) == tx_hash
                });

            if in_tx_pool {
                respond_or_log!(
                    store
                        .service()
                        .respond_transaction_status(rpc_id, TransactionStatus::Pending),
                    meta.time()
                );
                return;
            }

            let in_transition_frontier = if let Some(hash) = tx_hash {
                store
                    .state()
                    .transition_frontier
                    .contains_transaction(&hash)
            } else {
                false
            };

            // Check whether the transaction is in the transition frontier, if it is, return INCLUDED
            if in_transition_frontier {
                respond_or_log!(
                    store
                        .service()
                        .respond_transaction_status(rpc_id, TransactionStatus::Included),
                    meta.time()
                )
            // Otherwise, return UNKNOWN
            } else {
                respond_or_log!(
                    store
                        .service()
                        .respond_transaction_status(rpc_id, TransactionStatus::Unknown),
                    meta.time()
                )
            }
        }
    }
}

fn collect_rpc_peers_info(state: &crate::State) -> Vec<RpcPeerInfo> {
    state.p2p.ready().map_or_else(Vec::new, |p2p| {
        p2p.peers
            .iter()
            .map(|(peer_id, state)| {
                let best_tip = state.status.as_ready().and_then(|r| r.best_tip.as_ref());
                let (connection_status, time) = match &state.status {
                    p2p::P2pPeerStatus::Connecting(c) => match c {
                        p2p::connection::P2pConnectionState::Outgoing(o) => {
                            (PeerConnectionStatus::Connecting, o.time().into())
                        }
                        p2p::connection::P2pConnectionState::Incoming(i) => {
                            (PeerConnectionStatus::Connecting, i.time().into())
                        }
                    },
                    p2p::P2pPeerStatus::Disconnecting { time } => {
                        (PeerConnectionStatus::Disconnected, (*time).into())
                    }
                    p2p::P2pPeerStatus::Disconnected { time } => {
                        (PeerConnectionStatus::Disconnected, (*time).into())
                    }
                    p2p::P2pPeerStatus::Ready(r) => {
                        (PeerConnectionStatus::Connected, r.connected_since.into())
                    }
                };
                RpcPeerInfo {
                    peer_id: *peer_id,
                    connection_status,
                    address: state.dial_opts.as_ref().map(|opts| opts.to_string()),
                    best_tip: best_tip.map(|bt| bt.hash.clone()),
                    best_tip_height: best_tip.map(|bt| bt.height()),
                    best_tip_global_slot: best_tip.map(|bt| bt.global_slot_since_genesis()),
                    best_tip_timestamp: best_tip.map(|bt| bt.timestamp().into()),
                    time,
                }
            })
            .collect()
    })
}