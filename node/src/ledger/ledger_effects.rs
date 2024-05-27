use mina_p2p_messages::v2;
use p2p::channels::rpc::P2pRpcRequest;

use crate::block_producer::vrf_evaluator::BlockProducerVrfEvaluatorAction;
use crate::p2p::channels::rpc::{P2pChannelsRpcAction, P2pRpcId, P2pRpcResponse};
use crate::p2p::PeerId;
use crate::transition_frontier::sync::ledger::staged::TransitionFrontierSyncLedgerStagedAction;
use crate::transition_frontier::sync::TransitionFrontierSyncAction;
use crate::{BlockProducerAction, RpcAction, Store};

use super::read::{
    LedgerReadAction, LedgerReadId, LedgerReadRequest, LedgerReadResponse,
    LedgerReadStagedLedgerAuxAndPendingCoinbases,
};
use super::write::{LedgerWriteAction, LedgerWriteResponse};
use super::{LedgerAction, LedgerActionWithMeta, LedgerAddress, LedgerService};

pub fn ledger_effects<S: LedgerService>(store: &mut Store<S>, action: LedgerActionWithMeta) {
    let (action, _) = action.split();

    match action {
        LedgerAction::Write(a) => match a {
            LedgerWriteAction::Init { request } => {
                store.service.write_init(request);
                store.dispatch(LedgerWriteAction::Pending);
            }
            LedgerWriteAction::Pending => {}
            LedgerWriteAction::Success { response } => {
                propagate_write_response(store, response);
                next_write_request_init(store);
            }
        },
        LedgerAction::Read(a) => match a {
            LedgerReadAction::FindTodos => {
                next_read_requests_init(store);
            }
            LedgerReadAction::Init { request } => {
                if store.state().ledger.read.has_same_request(&request) {
                    return;
                }
                let id = store.state().ledger.read.next_req_id();
                store.service.read_init(id, request.clone());
                store.dispatch(LedgerReadAction::Pending { id, request });
            }
            LedgerReadAction::Pending { .. } => {}
            LedgerReadAction::Success { id, response } => {
                propagate_read_response(store, id, response);
            }
            LedgerReadAction::Prune { .. } => {}
        },
    }
}

fn next_write_request_init<S: redux::Service>(store: &mut Store<S>) {
    if store.dispatch(BlockProducerAction::StagedLedgerDiffCreateInit) {
    } else if store.dispatch(TransitionFrontierSyncAction::BlocksNextApplyInit) {
    } else if store.dispatch(TransitionFrontierSyncAction::CommitInit) {
    } else if store.dispatch(TransitionFrontierSyncLedgerStagedAction::ReconstructInit) {
    }
}

fn propagate_write_response<S: redux::Service>(
    store: &mut Store<S>,
    response: LedgerWriteResponse,
) {
    let Some(request) = &store.state().ledger.write.request() else {
        return;
    };
    match (request, response) {
        (
            _,
            LedgerWriteResponse::StagedLedgerReconstruct {
                staged_ledger_hash,
                result,
            },
        ) => {
            let sync = &store.state().transition_frontier.sync;
            let expected_ledger = sync
                .ledger_target()
                .and_then(|target| target.staged)
                .map(|v| v.hashes.non_snark.ledger_hash);
            if expected_ledger.as_ref() == Some(&staged_ledger_hash) {
                match result {
                    Err(error) => {
                        store.dispatch(
                            TransitionFrontierSyncLedgerStagedAction::ReconstructError { error },
                        );
                    }
                    Ok(()) => {
                        store.dispatch(
                            TransitionFrontierSyncLedgerStagedAction::ReconstructSuccess {
                                ledger_hash: staged_ledger_hash,
                            },
                        );
                    }
                }
            }
        }
        (
            _,
            LedgerWriteResponse::StagedLedgerDiffCreate {
                pred_block_hash,
                global_slot_since_genesis,
                result,
            },
        ) => {
            let state = store.state.get();
            let Some((expected_pred_block_hash, expected_global_slot)) = None.or_else(|| {
                let pred_block = state.block_producer.current_parent_chain()?.last()?;
                let won_slot = state.block_producer.current_won_slot()?;
                let slot = won_slot.global_slot_since_genesis(pred_block.global_slot_diff());
                Some((pred_block.hash(), slot))
            }) else {
                return;
            };

            if &pred_block_hash == expected_pred_block_hash
                && global_slot_since_genesis == expected_global_slot
            {
                match result {
                    Err(err) => todo!("handle staged ledger diff creation err: {err}"),
                    Ok(output) => {
                        store.dispatch(BlockProducerAction::StagedLedgerDiffCreateSuccess {
                            output: *output,
                        });
                    }
                }
            }
        }
        (
            _,
            LedgerWriteResponse::BlockApply {
                block_hash: hash,
                result,
            },
        ) => match result {
            Err(err) => todo!("handle block({hash}) apply err: {err}"),
            Ok(_) => {
                store.dispatch(TransitionFrontierSyncAction::BlocksNextApplySuccess { hash });
            }
        },
        (
            _,
            LedgerWriteResponse::Commit {
                best_tip_hash,
                result,
            },
        ) => {
            let best_tip = store.state().transition_frontier.sync.best_tip();
            if best_tip.map_or(false, |tip| tip.hash() == &best_tip_hash) {
                store.dispatch(TransitionFrontierSyncAction::CommitSuccess { result });
            }
        }
    }
}

fn next_read_requests_init<S: redux::Service>(store: &mut Store<S>) {
    if !store.state().ledger.read.is_total_cost_under_limit() {
        return;
    }

    // fetching delegator table
    store.dispatch(BlockProducerVrfEvaluatorAction::BeginDelegatorTableConstruction);

    // p2p rpcs
    let mut peers = store
        .state()
        .p2p
        .ready_peers_iter()
        .filter(|(_, peer)| {
            peer.channels
                .rpc
                .remote_todo_requests_iter()
                .next()
                .is_some()
        })
        .map(|(peer_id, peer)| (*peer_id, peer.channels.rpc.remote_last_responded()))
        .collect::<Vec<_>>();
    peers.sort_by_key(|(_, last_responded)| *last_responded);
    for (peer_id, _) in peers {
        let Some((id, request)) = None.or_else(|| {
            let peer = store.state().p2p.ready()?.get_ready_peer(&peer_id)?;
            let mut reqs = peer.channels.rpc.remote_todo_requests_iter();
            reqs.find_map(|req| {
                let ledger_request = match &req.request {
                    P2pRpcRequest::LedgerQuery(hash, query) => match query {
                        v2::MinaLedgerSyncLedgerQueryStableV1::NumAccounts => {
                            LedgerReadRequest::GetNumAccounts(hash.clone())
                        }
                        v2::MinaLedgerSyncLedgerQueryStableV1::WhatChildHashes(addr) => {
                            LedgerReadRequest::GetChildHashesAtAddr(hash.clone(), addr.into())
                        }
                        v2::MinaLedgerSyncLedgerQueryStableV1::WhatContents(addr) => {
                            LedgerReadRequest::GetChildAccountsAtAddr(hash.clone(), addr.into())
                        }
                    },
                    P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(block_hash) => {
                        let tf = &store.state().transition_frontier;
                        let ledger_hash = tf
                            .best_chain
                            .iter()
                            .find(|b| &b.hash == block_hash)
                            .map(|b| b.staged_ledger_hash().clone())?;
                        let protocol_states = tf
                            .needed_protocol_states
                            .iter()
                            .map(|(hash, b)| (hash.clone(), b.clone()))
                            .chain(
                                tf.best_chain
                                    .iter()
                                    .take_while(|b| b.hash() != block_hash)
                                    .map(|b| (b.hash().clone(), b.header().protocol_state.clone())),
                            )
                            .collect();

                        LedgerReadRequest::GetStagedLedgerAuxAndPendingCoinbases(
                            LedgerReadStagedLedgerAuxAndPendingCoinbases {
                                ledger_hash,
                                protocol_states,
                            },
                        )
                    }
                    _ => return None,
                };

                Some((req.id, ledger_request))
            })
        }) else {
            continue;
        };
        if store.dispatch(LedgerReadAction::Init { request }) {
            store.dispatch(P2pChannelsRpcAction::ResponsePending { peer_id, id });
        }
        if !store.state().ledger.read.is_total_cost_under_limit() {
            return;
        }
    }

    // rpcs
    let rpcs = store
        .state()
        .rpc
        .scan_state_summary_rpc_ids()
        .filter(|(.., status)| status.is_init())
        .map(|(id, ..)| id)
        .collect::<Vec<_>>();

    for rpc_id in rpcs {
        store.dispatch(RpcAction::ScanStateSummaryLedgerGetInit { rpc_id });
        if !store.state().ledger.read.is_total_cost_under_limit() {
            return;
        }
    }
}

fn find_peers_with_ledger_rpc(
    state: &crate::State,
    req: &LedgerReadRequest,
) -> Vec<(PeerId, P2pRpcId)> {
    let Some(p2p) = state.p2p.ready() else {
        return Vec::new();
    };
    p2p.ready_peers_iter()
        .flat_map(|(peer_id, state)| {
            state
                .channels
                .rpc
                .remote_pending_requests_iter()
                .map(move |req| (peer_id, req.id, &req.request))
        })
        .filter(|(_, _, peer_req)| match (req, peer_req) {
            (
                LedgerReadRequest::GetNumAccounts(h1),
                P2pRpcRequest::LedgerQuery(h2, v2::MinaLedgerSyncLedgerQueryStableV1::NumAccounts),
            ) => h1 == h2,
            (
                LedgerReadRequest::GetChildHashesAtAddr(h1, addr1),
                P2pRpcRequest::LedgerQuery(
                    h2,
                    v2::MinaLedgerSyncLedgerQueryStableV1::WhatChildHashes(addr2),
                ),
            ) => h1 == h2 && addr1 == &LedgerAddress::from(addr2),
            (
                LedgerReadRequest::GetChildAccountsAtAddr(h1, addr1),
                P2pRpcRequest::LedgerQuery(
                    h2,
                    v2::MinaLedgerSyncLedgerQueryStableV1::WhatContents(addr2),
                ),
            ) => h1 == h2 && addr1 == &LedgerAddress::from(addr2),
            (
                LedgerReadRequest::GetStagedLedgerAuxAndPendingCoinbases(data),
                P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(block_hash),
            ) => state
                .transition_frontier
                .get_state_body(block_hash)
                .map_or(false, |b| {
                    b.blockchain_state.staged_ledger_hash.non_snark.ledger_hash == data.ledger_hash
                }),
            _ => false,
        })
        .map(|(peer_id, rpc_id, _)| (*peer_id, rpc_id))
        .collect()
}

fn propagate_read_response<S: redux::Service>(
    store: &mut Store<S>,
    id: LedgerReadId,
    response: LedgerReadResponse,
) {
    let request = match store.state().ledger.read.get(id) {
        None => return,
        Some(v) => v.request(),
    };
    match (request, response) {
        (
            LedgerReadRequest::DelegatorTable(ledger_hash, pub_key),
            LedgerReadResponse::DelegatorTable(table),
        ) => {
            let expected = store.state().block_producer.vrf_delegator_table_inputs();
            if !expected.map_or(false, |(expected_hash, producer)| {
                ledger_hash == expected_hash && pub_key == producer
            }) {
                eprintln!("delegator table unexpected");
                return;
            }
            match table {
                None => todo!("delegator table construction error handling"),
                Some(table) => {
                    store.dispatch(
                        BlockProducerVrfEvaluatorAction::FinalizeDelegatorTableConstruction {
                            delegator_table: table.into(),
                        },
                    );
                }
            }
        }
        (_, LedgerReadResponse::DelegatorTable(..)) => unreachable!(),
        (req, LedgerReadResponse::GetNumAccounts(resp)) => {
            for (peer_id, id) in find_peers_with_ledger_rpc(store.state(), req) {
                store.dispatch(P2pChannelsRpcAction::ResponseSend {
                    peer_id,
                    id,
                    response: resp.as_ref().map(|(num_accounts, hash)| {
                        P2pRpcResponse::LedgerQuery(
                            v2::MinaLedgerSyncLedgerAnswerStableV2::NumAccounts(
                                (*num_accounts).into(),
                                hash.clone(),
                            ),
                        )
                    }),
                });
            }
        }
        (req, LedgerReadResponse::GetChildHashesAtAddr(resp)) => {
            for (peer_id, id) in find_peers_with_ledger_rpc(store.state(), req) {
                store.dispatch(P2pChannelsRpcAction::ResponseSend {
                    peer_id,
                    id,
                    response: resp.as_ref().map(|(left, right)| {
                        P2pRpcResponse::LedgerQuery(
                            v2::MinaLedgerSyncLedgerAnswerStableV2::ChildHashesAre(
                                left.clone(),
                                right.clone(),
                            ),
                        )
                    }),
                });
            }
        }
        (req, LedgerReadResponse::GetChildAccountsAtAddr(resp)) => {
            for (peer_id, id) in find_peers_with_ledger_rpc(store.state(), req) {
                store.dispatch(P2pChannelsRpcAction::ResponseSend {
                    peer_id,
                    id,
                    response: resp.as_ref().map(|accounts| {
                        P2pRpcResponse::LedgerQuery(
                            v2::MinaLedgerSyncLedgerAnswerStableV2::ContentsAre(
                                accounts.iter().cloned().collect(),
                            ),
                        )
                    }),
                });
            }
        }
        (req, LedgerReadResponse::GetStagedLedgerAuxAndPendingCoinbases(resp)) => {
            for (peer_id, id) in find_peers_with_ledger_rpc(store.state(), req) {
                store.dispatch(P2pChannelsRpcAction::ResponseSend {
                    peer_id,
                    id,
                    response: resp.clone().map(|data| {
                        P2pRpcResponse::StagedLedgerAuxAndPendingCoinbasesAtBlock(data)
                    }),
                });
            }
        }
        (
            LedgerReadRequest::ScanStateSummary(ledger_hash),
            LedgerReadResponse::ScanStateSummary(scan_state),
        ) => {
            for rpc_id in store
                .state()
                .rpc
                .scan_state_summary_rpc_ids()
                .filter(|(_, hash, _)| *hash == ledger_hash)
                .map(|(id, ..)| id)
                .collect::<Vec<_>>()
            {
                store.dispatch(RpcAction::ScanStateSummaryGetSuccess {
                    rpc_id,
                    scan_state: scan_state.clone(),
                });
            }
        }
        (_, LedgerReadResponse::ScanStateSummary(..)) => unreachable!(),
    }
}
