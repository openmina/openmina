use ledger::scan_state::transaction_logic::valid;
use mina_p2p_messages::v2::{
    MinaBaseSignedCommandStableV2, MinaBaseZkappCommandTStableV1WireStableV1, NonZeroCurvePoint,
    TransactionSnarkWorkTStableV2,
};
use openmina_core::{
    block::AppliedBlock,
    bug_condition,
    requests::{RequestId, RpcId, RpcIdType},
    transaction::{TransactionPoolMessageSource, TransactionWithHash},
};
use p2p::{
    connection::{incoming::P2pConnectionIncomingAction, outgoing::P2pConnectionOutgoingAction},
    webrtc::P2pConnectionResponse,
    PeerId,
};
use redux::ActionWithMeta;

use crate::{
    ledger::read::{LedgerReadAction, LedgerReadInitCallback, LedgerReadRequest},
    p2p_ready,
    rpc::{GetBlockQuery, PooledCommandsQuery},
    rpc_effectful::RpcEffectfulAction,
    TransactionPoolAction,
};

use super::{
    ConsensusTimeQuery, PeerConnectionStatus, RpcAction, RpcPeerInfo, RpcRequest,
    RpcRequestExtraData, RpcRequestState, RpcRequestStatus, RpcScanStateSummaryGetQuery,
    RpcSnarkerConfig, RpcState,
};

impl RpcState {
    pub fn reducer(mut state_context: crate::Substate<Self>, action: ActionWithMeta<&RpcAction>) {
        let Ok(state) = state_context.get_substate_mut() else {
            return;
        };

        let (action, meta) = action.split();
        match action {
            RpcAction::GlobalStateGet { rpc_id, filter } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::GlobalStateGet {
                    rpc_id: *rpc_id,
                    filter: filter.clone(),
                });
            }
            RpcAction::StatusGet { rpc_id } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::StatusGet { rpc_id: *rpc_id });
            }
            RpcAction::HeartbeatGet { rpc_id } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::HeartbeatGet { rpc_id: *rpc_id });
            }
            RpcAction::ActionStatsGet { rpc_id, query } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::ActionStatsGet {
                    rpc_id: *rpc_id,
                    query: *query,
                });
            }
            RpcAction::SyncStatsGet { rpc_id, query } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::SyncStatsGet {
                    rpc_id: *rpc_id,
                    query: *query,
                });
            }
            RpcAction::BlockProducerStatsGet { rpc_id } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::BlockProducerStatsGet { rpc_id: *rpc_id });
            }
            RpcAction::MessageProgressGet { rpc_id } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::MessageProgressGet { rpc_id: *rpc_id });
            }
            RpcAction::PeersGet { rpc_id } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let peers = collect_rpc_peers_info(state);
                dispatcher.push(RpcEffectfulAction::PeersGet {
                    rpc_id: *rpc_id,
                    peers,
                });
            }
            RpcAction::P2pConnectionOutgoingInit { rpc_id, opts } => {
                let rpc_state = RpcRequestState {
                    req: RpcRequest::P2pConnectionOutgoing(opts.clone()),
                    status: RpcRequestStatus::Init { time: meta.time() },
                    data: Default::default(),
                };
                state.requests.insert(*rpc_id, rpc_state);

                let dispatcher = state_context.into_dispatcher();

                dispatcher.push(P2pConnectionOutgoingAction::Init {
                    opts: opts.clone(),
                    rpc_id: Some(*rpc_id),
                    on_success: Some(redux::callback!(
                        on_p2p_connection_outgoing_rpc_connection_success((peer_id: PeerId, rpc_id: Option<RpcId>)) -> crate::Action {
                            let Some(rpc_id) = rpc_id else {
                                unreachable!("RPC ID not provided");
                            };

                            RpcAction::P2pConnectionOutgoingPending{ rpc_id }
                        }
                    )),
                });
            }
            RpcAction::P2pConnectionOutgoingPending { rpc_id } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    bug_condition!(
                        "Rpc state not found for RpcAction::P2pConnectionOutgoingPending({})",
                        rpc_id
                    );
                    return;
                };
                rpc.status = RpcRequestStatus::Pending { time: meta.time() };
            }
            RpcAction::P2pConnectionOutgoingError { rpc_id, error } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    bug_condition!(
                        "Rpc state not found for RpcAction::P2pConnectionOutgoingError({})",
                        rpc_id
                    );
                    return;
                };
                rpc.status = RpcRequestStatus::Error {
                    time: meta.time(),
                    error: format!("{:?}", error),
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::P2pConnectionOutgoingError {
                    rpc_id: *rpc_id,
                    error: format!("{:?}", error),
                });
            }
            RpcAction::P2pConnectionOutgoingSuccess { rpc_id } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    bug_condition!(
                        "Rpc state not found for RpcAction::P2pConnectionOutgoingSuccess({})",
                        rpc_id
                    );
                    return;
                };
                rpc.status = RpcRequestStatus::Success { time: meta.time() };
                let dispatcher = state_context.into_dispatcher();
                dispatcher
                    .push(RpcEffectfulAction::P2pConnectionOutgoingSuccess { rpc_id: *rpc_id });
            }
            RpcAction::P2pConnectionIncomingInit { rpc_id, opts } => {
                let rpc_state = RpcRequestState {
                    req: RpcRequest::P2pConnectionIncoming(opts.clone()),
                    status: RpcRequestStatus::Init { time: meta.time() },
                    data: Default::default(),
                };
                state.requests.insert(*rpc_id, rpc_state);

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p = p2p_ready!(state.p2p, meta.time());

                match p2p.incoming_accept(opts.peer_id, &opts.offer) {
                    Ok(_) => {
                        dispatcher.push(P2pConnectionIncomingAction::Init {
                            opts: opts.clone(),
                            rpc_id: Some(*rpc_id),
                        });
                        dispatcher
                            .push(RpcAction::P2pConnectionIncomingPending { rpc_id: *rpc_id });
                    }
                    Err(reason) => {
                        let response = P2pConnectionResponse::Rejected(reason);
                        dispatcher.push(RpcAction::P2pConnectionIncomingRespond {
                            rpc_id: *rpc_id,
                            response,
                        });
                    }
                }
            }
            RpcAction::P2pConnectionIncomingPending { rpc_id } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    bug_condition!(
                        "Rpc state not found for RpcAction::P2pConnectionIncomingPending({})",
                        rpc_id
                    );
                    return;
                };
                rpc.status = RpcRequestStatus::Pending { time: meta.time() };
            }
            RpcAction::P2pConnectionIncomingRespond { rpc_id, response } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::P2pConnectionIncomingRespond {
                    rpc_id: *rpc_id,
                    response: response.clone(),
                });
            }
            RpcAction::P2pConnectionIncomingError { rpc_id, error } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    bug_condition!(
                        "Rpc state not found for RpcAction::P2pConnectionIncomingError({})",
                        rpc_id
                    );
                    return;
                };
                rpc.status = RpcRequestStatus::Error {
                    time: meta.time(),
                    error: format!("{:?}", error),
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::P2pConnectionIncomingError {
                    rpc_id: *rpc_id,
                    error: error.to_owned(),
                });
            }
            RpcAction::P2pConnectionIncomingSuccess { rpc_id } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    bug_condition!(
                        "Rpc state not found for RpcAction::P2pConnectionIncomingSuccess({})",
                        rpc_id
                    );
                    return;
                };
                rpc.status = RpcRequestStatus::Success { time: meta.time() };
                let dispatcher = state_context.into_dispatcher();
                dispatcher
                    .push(RpcEffectfulAction::P2pConnectionIncomingSuccess { rpc_id: *rpc_id });
            }
            RpcAction::ScanStateSummaryGetInit { rpc_id, query } => {
                let rpc_state = RpcRequestState {
                    req: RpcRequest::ScanStateSummaryGet(query.clone()),
                    status: RpcRequestStatus::Init { time: meta.time() },
                    data: Default::default(),
                };
                state.requests.insert(*rpc_id, rpc_state);

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcAction::ScanStateSummaryLedgerGetInit { rpc_id: *rpc_id });
            }
            RpcAction::ScanStateSummaryLedgerGetInit { rpc_id } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let transition_frontier = &state.transition_frontier;

                let Some(query) = None.or_else(|| {
                    let req = state.rpc.requests.get(rpc_id)?;
                    match &req.req {
                        RpcRequest::ScanStateSummaryGet(query) => Some(query),
                        _ => None,
                    }
                }) else {
                    return;
                };

                let block = match query {
                    RpcScanStateSummaryGetQuery::ForBestTip => {
                        transition_frontier.best_tip_breadcrumb()
                    }
                    RpcScanStateSummaryGetQuery::ForBlockWithHash(hash) => transition_frontier
                        .best_chain
                        .iter()
                        .rev()
                        .find(|b| b.hash() == hash),
                    RpcScanStateSummaryGetQuery::ForBlockWithHeight(height) => transition_frontier
                        .best_chain
                        .iter()
                        .rev()
                        .find(|b| b.height() == *height),
                };
                let block = match block {
                    Some(v) => v.clone(),
                    None => {
                        dispatcher.push(RpcAction::ScanStateSummaryGetPending {
                            rpc_id: *rpc_id,
                            block: None,
                        });
                        dispatcher.push(RpcAction::ScanStateSummaryGetSuccess {
                            rpc_id: *rpc_id,
                            scan_state: Ok(Vec::new()),
                        });
                        return;
                    }
                };

                dispatcher.push(LedgerReadAction::Init {
                    request: LedgerReadRequest::ScanStateSummary(
                        block.staged_ledger_hashes().clone(),
                    ),
                    callback: LedgerReadInitCallback::RpcScanStateSummaryGetPending {
                        callback: redux::callback!(
                            on_ledger_read_init_rpc_scan_state_summary_get_pending((rpc_id: RequestId<RpcIdType>, block: AppliedBlock)) -> crate::Action{
                                RpcAction::ScanStateSummaryGetPending { rpc_id, block: Some(block) }
                            }
                        ),
                        args: (*rpc_id, block),
                    },
                });
            }
            RpcAction::ScanStateSummaryGetPending { rpc_id, block } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    bug_condition!(
                        "Rpc state not found for RpcAction::ScanStateSummaryGetPending({})",
                        rpc_id
                    );
                    return;
                };
                rpc.status = RpcRequestStatus::Pending { time: meta.time() };
                rpc.data = RpcRequestExtraData::FullBlockOpt(block.clone());
            }
            RpcAction::ScanStateSummaryGetSuccess { rpc_id, scan_state } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    bug_condition!(
                        "Rpc state not found for RpcAction::ScanStateSummaryGetSuccess({})",
                        rpc_id
                    );
                    return;
                };
                rpc.status = RpcRequestStatus::Success { time: meta.time() };
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::ScanStateSummaryGetSuccess {
                    rpc_id: *rpc_id,
                    scan_state: scan_state.clone(),
                });
            }
            RpcAction::SnarkPoolAvailableJobsGet { rpc_id } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::SnarkPoolAvailableJobsGet { rpc_id: *rpc_id });
            }
            RpcAction::SnarkPoolJobGet { rpc_id, job_id } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::SnarkPoolJobGet {
                    rpc_id: *rpc_id,
                    job_id: job_id.clone(),
                });
            }
            RpcAction::SnarkPoolCompletedJobsGet { rpc_id } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                let jobs = state
                    .snark_pool
                    .completed_snarks_iter()
                    .map(|s| TransactionSnarkWorkTStableV2::from(s.clone()))
                    .collect::<Vec<_>>();

                dispatcher.push(RpcEffectfulAction::SnarkPoolCompletedJobsGet {
                    rpc_id: *rpc_id,
                    jobs,
                });
            }
            RpcAction::SnarkPoolPendingJobsGet { rpc_id } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                let jobs = state
                    .snark_pool
                    .available_jobs_iter()
                    .cloned()
                    .collect::<Vec<_>>();

                dispatcher.push(RpcEffectfulAction::SnarkPoolPendingJobsGet {
                    rpc_id: *rpc_id,
                    jobs,
                })
            }
            RpcAction::SnarkerConfigGet { rpc_id } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                let config = state
                    .config
                    .snarker
                    .as_ref()
                    .map(|config| RpcSnarkerConfig {
                        public_key: config.public_key.as_ref().clone(),
                        fee: config.fee.clone(),
                    });

                dispatcher.push(RpcEffectfulAction::SnarkerConfigGet {
                    rpc_id: *rpc_id,
                    config,
                });
            }
            RpcAction::SnarkerJobCommit { rpc_id, job_id } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::SnarkerJobCommit {
                    rpc_id: *rpc_id,
                    job_id: job_id.clone(),
                });
            }
            RpcAction::SnarkerJobSpec { rpc_id, job_id } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::SnarkerJobSpec {
                    rpc_id: *rpc_id,
                    job_id: job_id.clone(),
                });
            }
            RpcAction::SnarkerWorkersGet { rpc_id } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let snark_worker = state.external_snark_worker.0.clone();
                dispatcher.push(RpcEffectfulAction::SnarkerWorkersGet {
                    rpc_id: *rpc_id,
                    snark_worker,
                });
            }
            RpcAction::HealthCheck { rpc_id } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                let has_peers = state.p2p.ready_peers_iter().map(|(peer_id, _peer)| {
                    openmina_core::log::debug!(meta.time(); "found ready peer: {peer_id}")
                })
                .next()
                .ok_or_else(|| {
                    openmina_core::log::warn!(meta.time(); "no ready peers");
                    String::from("no ready peers")
                });

                dispatcher.push(RpcEffectfulAction::HealthCheck {
                    rpc_id: *rpc_id,
                    has_peers,
                });
            }
            RpcAction::ReadinessCheck { rpc_id } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::ReadinessCheck { rpc_id: *rpc_id });
            }
            RpcAction::DiscoveryRoutingTable { rpc_id } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                let response = state
                    .p2p
                    .ready()
                    .and_then(|p2p| p2p.network.scheduler.discovery_state())
                    .and_then(|discovery_state| {
                        match (&discovery_state.routing_table).try_into() {
                            Ok(resp) => Some(resp),
                            Err(err) => {
                                bug_condition!(
                                    "{:?} error converting routing table into response: {:?}",
                                    err,
                                    action
                                );
                                None
                            }
                        }
                    });

                dispatcher.push(RpcEffectfulAction::DiscoveryRoutingTable {
                    rpc_id: *rpc_id,
                    response,
                });
            }
            RpcAction::DiscoveryBoostrapStats { rpc_id } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                let response = state
                    .p2p
                    .ready()
                    .and_then(|p2p| p2p.network.scheduler.discovery_state())
                    .and_then(|discovery_state: &p2p::P2pNetworkKadState| {
                        discovery_state.bootstrap_stats().cloned()
                    });

                dispatcher.push(RpcEffectfulAction::DiscoveryBoostrapStats {
                    rpc_id: *rpc_id,
                    response,
                });
            }
            RpcAction::Finish { rpc_id } => {
                state.requests.remove(rpc_id);
            }
            RpcAction::TransactionPool { rpc_id } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let response = state.transaction_pool.get_all_transactions();
                dispatcher.push(RpcEffectfulAction::TransactionPool {
                    rpc_id: *rpc_id,
                    response,
                });
            }
            RpcAction::LedgerAccountsGetInit {
                rpc_id,
                account_query,
            } => {
                let rpc_state = RpcRequestState {
                    req: RpcRequest::LedgerAccountsGet(account_query.clone()),
                    status: RpcRequestStatus::Init { time: meta.time() },
                    data: Default::default(),
                };
                state.requests.insert(*rpc_id, rpc_state);

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let ledger_hash = if let Some(best_tip) = state.transition_frontier.best_tip() {
                    best_tip.merkle_root_hash()
                } else {
                    return;
                };

                dispatcher.push(LedgerReadAction::Init {
                    request: LedgerReadRequest::AccountsForRpc(
                        *rpc_id,
                        ledger_hash.clone(),
                        account_query.clone(),
                    ),
                    callback: LedgerReadInitCallback::RpcLedgerAccountsGetPending {
                        callback: redux::callback!(
                            on_ledger_read_init_rpc_actions_get_init(rpc_id: RequestId<RpcIdType>) -> crate::Action{
                                RpcAction::LedgerAccountsGetPending { rpc_id }
                            }
                        ),
                        args: *rpc_id,
                    },
                })
            }
            RpcAction::LedgerAccountsGetPending { rpc_id } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Pending { time: meta.time() };
            }
            RpcAction::LedgerAccountsGetSuccess {
                rpc_id,
                account_query,
                accounts,
            } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Success { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::LedgerAccountsGetSuccess {
                    rpc_id: *rpc_id,
                    account_query: account_query.clone(),
                    accounts: accounts.clone(),
                });
            }
            RpcAction::TransactionInjectInit { rpc_id, commands } => {
                let rpc_state = RpcRequestState {
                    req: RpcRequest::TransactionInject(commands.clone()),
                    status: RpcRequestStatus::Init { time: meta.time() },
                    data: Default::default(),
                };
                state.requests.insert(*rpc_id, rpc_state);

                let commands_with_hash = commands
                    .clone()
                    .into_iter()
                    // TODO: do something it it cannot be hashed?
                    .filter_map(|cmd| TransactionWithHash::try_new(cmd).ok())
                    .collect();

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcAction::TransactionInjectPending { rpc_id: *rpc_id });
                dispatcher.push(TransactionPoolAction::StartVerify {
                    commands: commands_with_hash,
                    from_source: TransactionPoolMessageSource::rpc(*rpc_id),
                });
            }
            RpcAction::TransactionInjectPending { rpc_id } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Pending { time: meta.time() };
            }
            RpcAction::TransactionInjectSuccess { rpc_id, response } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Success { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                let response = response.clone().into_iter().map(|cmd| cmd.data).collect();
                dispatcher.push(RpcEffectfulAction::TransactionInjectSuccess {
                    rpc_id: *rpc_id,
                    response,
                });
            }
            RpcAction::TransactionInjectRejected { rpc_id, response } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Success { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                let response = response
                    .clone()
                    .into_iter()
                    .map(|(cmd, failure)| (cmd.data, failure))
                    .collect();

                dispatcher.push(RpcEffectfulAction::TransactionInjectRejected {
                    rpc_id: *rpc_id,
                    response,
                });
            }
            RpcAction::TransactionInjectFailure { rpc_id, errors } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Error {
                    time: meta.time(),
                    error: "Transaction injection failed".to_string(),
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::TransactionInjectFailure {
                    rpc_id: *rpc_id,
                    errors: errors.clone(),
                });
            }
            RpcAction::TransitionFrontierUserCommandsGet { rpc_id } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                let commands = state
                    .transition_frontier
                    .best_chain
                    .iter()
                    .flat_map(|block| block.body().commands_iter().map(|v| v.data.clone()))
                    .collect::<Vec<_>>();

                dispatcher.push(RpcEffectfulAction::TransitionFrontierUserCommandsGet {
                    rpc_id: *rpc_id,
                    commands,
                });
            }
            RpcAction::BestChain { rpc_id, max_length } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                let best_chain = state
                    .transition_frontier
                    .best_chain
                    .iter()
                    .rev()
                    .take(*max_length as usize)
                    .cloned()
                    .rev()
                    .collect();

                dispatcher.push(RpcEffectfulAction::BestChain {
                    rpc_id: *rpc_id,
                    best_chain,
                });
            }
            RpcAction::ConsensusConstantsGet { rpc_id } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let response = state.config.consensus_constants.clone();
                dispatcher.push(RpcEffectfulAction::ConsensusConstantsGet {
                    rpc_id: *rpc_id,
                    response,
                });
            }
            RpcAction::TransactionStatusGet { rpc_id, tx } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::TransactionStatusGet {
                    rpc_id: *rpc_id,
                    tx: tx.clone(),
                });
            }
            RpcAction::BlockGet { rpc_id, query } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                let find_fn = |block: &&AppliedBlock| match query {
                    GetBlockQuery::Hash(hash) => block.hash() == hash,
                    GetBlockQuery::Height(height) => block.height() == *height,
                };

                let block = state
                    .transition_frontier
                    .best_chain
                    .iter()
                    .find(find_fn)
                    .cloned();

                dispatcher.push(RpcEffectfulAction::BlockGet {
                    rpc_id: *rpc_id,
                    block,
                });
            }
            RpcAction::P2pConnectionIncomingAnswerReady {
                rpc_id,
                answer,
                peer_id,
            } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcAction::P2pConnectionIncomingRespond {
                    rpc_id: *rpc_id,
                    response: answer.clone(),
                });
                dispatcher
                    .push(P2pConnectionIncomingAction::AnswerSendSuccess { peer_id: *peer_id });
            }
            RpcAction::PooledUserCommands { rpc_id, query } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                let PooledCommandsQuery {
                    public_key,
                    hashes,
                    ids,
                } = query;

                let all_transactions = state.transaction_pool.get_all_transactions();

                let mut user_commands: Vec<_> = all_transactions
                    .into_iter()
                    .filter_map(|tx| match tx.data {
                        valid::UserCommand::SignedCommand(signed_command) => Some((
                            tx.hash,
                            MinaBaseSignedCommandStableV2::from(*signed_command),
                        )),
                        valid::UserCommand::ZkAppCommand(_) => None,
                    })
                    .collect();

                if let Some(pk) = public_key {
                    let pk = NonZeroCurvePoint::from(pk.clone());
                    user_commands.retain(|(_, tx)| tx.signer == pk)
                }

                if let Some(hashes) = hashes {
                    user_commands.retain(|(hash, _)| hashes.contains(hash))
                }

                if let Some(ids) = ids {
                    user_commands.retain(|(_, tx)| ids.contains(tx))
                }

                dispatcher.push(RpcEffectfulAction::PooledUserCommands {
                    rpc_id: *rpc_id,
                    user_commands: user_commands.into_iter().map(|(_, tx)| tx).collect(),
                });
            }
            RpcAction::GenesisBlock { rpc_id } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let genesis_block = state.genesis_block();
                dispatcher.push(RpcEffectfulAction::GenesisBlock {
                    rpc_id: *rpc_id,
                    genesis_block,
                });
            }
            RpcAction::PooledZkappCommands { rpc_id, query } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                let PooledCommandsQuery {
                    public_key,
                    hashes,
                    ids,
                } = query;

                let all_transactions = state.transaction_pool.get_all_transactions();

                let mut zkapp_commands: Vec<_> = all_transactions
                    .into_iter()
                    .filter_map(|tx| match tx.data {
                        valid::UserCommand::SignedCommand(_) => None,
                        valid::UserCommand::ZkAppCommand(zkapp) => Some((
                            tx.hash,
                            MinaBaseZkappCommandTStableV1WireStableV1::from(&zkapp.zkapp_command),
                        )),
                    })
                    .collect();

                if let Some(pk) = public_key {
                    let pk = NonZeroCurvePoint::from(pk.clone());
                    zkapp_commands.retain(|(_, tx)| tx.fee_payer.body.public_key == pk);
                }

                if let Some(hashes) = hashes {
                    zkapp_commands.retain(|(hash, _)| hashes.contains(hash));
                }

                if let Some(ids) = ids {
                    zkapp_commands.retain(|(_, tx)| ids.contains(tx));
                }

                dispatcher.push(RpcEffectfulAction::PooledZkappCommands {
                    rpc_id: *rpc_id,
                    zkapp_commands: zkapp_commands.into_iter().map(|(_, tx)| tx).collect(),
                });
            }
            RpcAction::ConsensusTimeGet { rpc_id, query } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let consensus_time = match query {
                    ConsensusTimeQuery::Now => state.consensus_time_now(),
                    ConsensusTimeQuery::BestTip => state.consensus_time_best_tip(),
                };
                println!("consensus_time: {:?}", consensus_time);
                dispatcher.push(RpcEffectfulAction::ConsensusTimeGet {
                    rpc_id: *rpc_id,
                    consensus_time,
                });
            }
            RpcAction::LedgerStatusGetInit {
                rpc_id,
                ledger_hash,
            } => {
                let rpc_state = RpcRequestState {
                    req: RpcRequest::LedgerStatusGet(ledger_hash.clone()),
                    status: RpcRequestStatus::Init { time: meta.time() },
                    data: Default::default(),
                };
                state.requests.insert(*rpc_id, rpc_state);

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let ledger_hash = if let Some(best_tip) = state.transition_frontier.best_tip() {
                    best_tip.merkle_root_hash()
                } else {
                    return;
                };

                dispatcher.push(LedgerReadAction::Init {
                    request: LedgerReadRequest::GetLedgerStatus(*rpc_id, ledger_hash.clone()),
                    callback: LedgerReadInitCallback::RpcLedgerStatusGetPending {
                        callback: redux::callback!(
                            on_ledger_read_init_rpc_actions_get_init(rpc_id: RequestId<RpcIdType>) -> crate::Action{
                                RpcAction::LedgerStatusGetPending { rpc_id }
                            }
                        ),
                        args: *rpc_id,
                    },
                })
            }
            RpcAction::LedgerStatusGetPending { rpc_id } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Pending { time: meta.time() };
            }
            RpcAction::LedgerStatusGetSuccess { rpc_id, response } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Success { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::LedgerStatusGetSuccess {
                    rpc_id: *rpc_id,
                    response: response.clone(),
                });
            }
            RpcAction::LedgerAccountDelegatorsGetInit {
                rpc_id,
                ledger_hash,
                account_id,
            } => {
                let rpc_state = RpcRequestState {
                    req: RpcRequest::LedgerAccountDelegatorsGet(
                        ledger_hash.clone(),
                        account_id.clone(),
                    ),
                    status: RpcRequestStatus::Init { time: meta.time() },
                    data: Default::default(),
                };
                state.requests.insert(*rpc_id, rpc_state);

                let dispatcher = state_context.into_dispatcher();

                dispatcher.push(LedgerReadAction::Init {
                    request: LedgerReadRequest::GetAccountDelegators(*rpc_id, ledger_hash.clone(), account_id.clone()),
                    callback: LedgerReadInitCallback::RpcLedgerAccountDelegatorsGetPending {
                        callback: redux::callback!(
                            on_ledger_read_init_rpc_actions_get_init(rpc_id: RequestId<RpcIdType>) -> crate::Action{
                                RpcAction::LedgerAccountDelegatorsGetPending { rpc_id }
                            }
                        ),
                        args: *rpc_id,
                    },
                })
            }
            RpcAction::LedgerAccountDelegatorsGetPending { rpc_id } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Pending { time: meta.time() };
            }
            RpcAction::LedgerAccountDelegatorsGetSuccess { rpc_id, response } => {
                let Some(rpc) = state.requests.get_mut(rpc_id) else {
                    return;
                };
                rpc.status = RpcRequestStatus::Success { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(RpcEffectfulAction::LedgerAccountDelegatorsGetSuccess {
                    rpc_id: *rpc_id,
                    response: response.clone(),
                });
            }
        }
    }
}

pub fn collect_rpc_peers_info(state: &crate::State) -> Vec<RpcPeerInfo> {
    state.p2p.ready().map_or_else(Vec::new, |p2p| {
        p2p.peers
            .iter()
            .map(|(peer_id, state)| {
                let best_tip = state.status.as_ready().and_then(|r| r.best_tip.as_ref());
                let (connection_status, time, incoming, connecting_details) = match &state.status {
                    p2p::P2pPeerStatus::Connecting(c) => match c {
                        p2p::connection::P2pConnectionState::Outgoing(o) => (
                            PeerConnectionStatus::Connecting,
                            o.time().into(),
                            false,
                            Some(format!("{o:?}")),
                        ),
                        p2p::connection::P2pConnectionState::Incoming(i) => (
                            PeerConnectionStatus::Connecting,
                            i.time().into(),
                            true,
                            Some(format!("{i:?}")),
                        ),
                    },
                    p2p::P2pPeerStatus::Disconnecting { time } => (
                        PeerConnectionStatus::Disconnecting,
                        (*time).into(),
                        false,
                        None,
                    ),
                    p2p::P2pPeerStatus::Disconnected { time } => (
                        PeerConnectionStatus::Disconnected,
                        (*time).into(),
                        false,
                        None,
                    ),
                    p2p::P2pPeerStatus::Ready(r) => (
                        PeerConnectionStatus::Connected,
                        r.connected_since.into(),
                        r.is_incoming,
                        None,
                    ),
                };
                RpcPeerInfo {
                    peer_id: *peer_id,
                    connection_status,
                    connecting_details,
                    address: state.dial_opts.as_ref().map(|opts| opts.to_string()),
                    is_libp2p: state.is_libp2p,
                    incoming,
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
