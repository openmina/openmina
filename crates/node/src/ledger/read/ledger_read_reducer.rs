use mina_core::{bug_condition, requests::RequestId};
use mina_p2p_messages::v2;
use p2p::{
    channels::{
        rpc::{P2pChannelsRpcAction, P2pRpcId, P2pRpcRequest, P2pRpcResponse},
        streaming_rpc::{P2pChannelsStreamingRpcAction, P2pStreamingRpcRequest},
    },
    P2pAction, PeerId,
};
use redux::Dispatcher;

use crate::{
    block_producer::vrf_evaluator::BlockProducerVrfEvaluatorAction,
    ledger_effectful::LedgerEffectfulAction, Action, RpcAction, State, Substate,
};

use super::{
    LedgerAddress, LedgerReadAction, LedgerReadActionWithMetaRef, LedgerReadIdType,
    LedgerReadInitCallback, LedgerReadRequest, LedgerReadResponse,
    LedgerReadStagedLedgerAuxAndPendingCoinbases, LedgerReadState,
};

impl LedgerReadState {
    pub fn reducer(mut state_context: Substate<Self>, action: LedgerReadActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        let Ok(state) = state_context.get_substate_mut() else {
            return;
        };

        match action {
            LedgerReadAction::FindTodos => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                Self::next_read_requests_init(dispatcher, state);
            }
            LedgerReadAction::Init { request, callback } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                if state.ledger.read.has_same_request(request) {
                    return;
                }

                let id = state.ledger.read.next_req_id();
                dispatcher.push(LedgerEffectfulAction::ReadInit {
                    request: request.clone(),
                    callback: callback.clone(),
                    id,
                });
            }
            LedgerReadAction::Pending { request, .. } => {
                state.add(meta.time(), request.clone());
            }
            LedgerReadAction::Success { id, response } => {
                state.add_response(*id, meta.time(), response.clone());

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                Self::propagate_read_response(dispatcher, state, *id, response.clone());
                dispatcher.push(LedgerReadAction::Prune { id: *id });
            }
            LedgerReadAction::Prune { id } => {
                state.remove(*id);
            }
        }
    }

    fn propagate_read_response(
        dispatcher: &mut Dispatcher<Action, State>,
        state: &State,
        id: RequestId<LedgerReadIdType>,
        response: LedgerReadResponse,
    ) {
        let Some(request) = state.ledger.read.get(id) else {
            bug_condition!("Request with id: {} not found", id);
            return;
        };

        match (request.request(), response) {
            (
                LedgerReadRequest::DelegatorTable(ledger_hash, pub_key),
                LedgerReadResponse::DelegatorTable(table),
            ) => {
                let expected = state.block_producer.vrf_delegator_table_inputs();
                if !expected.is_some_and(|(expected_hash, producer)| {
                    ledger_hash == expected_hash && pub_key == producer
                }) {
                    bug_condition!("delegator table unexpected");
                    return;
                }
                match table {
                    None => {
                        // TODO(tizoc): Revise this, may be better to dispatch a different action here
                        // and avoid running the VRF evaluator altogether when we know that the
                        // table is empty.
                        dispatcher.push(
                            BlockProducerVrfEvaluatorAction::FinalizeDelegatorTableConstruction {
                                delegator_table: Default::default(),
                            },
                        );
                    }
                    Some(table) => {
                        dispatcher.push(
                            BlockProducerVrfEvaluatorAction::FinalizeDelegatorTableConstruction {
                                delegator_table: table.into(),
                            },
                        );
                    }
                }
            }
            (_, LedgerReadResponse::DelegatorTable(..)) => unreachable!(),
            (req, LedgerReadResponse::GetNumAccounts(resp)) => {
                for (peer_id, id, _) in find_peers_with_ledger_rpc(state, req) {
                    dispatcher.push(P2pChannelsRpcAction::ResponseSend {
                        peer_id,
                        id,
                        response: resp.as_ref().map(|(num_accounts, hash)| {
                            Box::new(P2pRpcResponse::LedgerQuery(
                                v2::MinaLedgerSyncLedgerAnswerStableV2::NumAccounts(
                                    (*num_accounts).into(),
                                    hash.clone(),
                                ),
                            ))
                        }),
                    });
                }
            }
            (req, LedgerReadResponse::GetChildHashesAtAddr(resp)) => {
                for (peer_id, id, _) in find_peers_with_ledger_rpc(state, req) {
                    dispatcher.push(P2pChannelsRpcAction::ResponseSend {
                        peer_id,
                        id,
                        response: resp.as_ref().map(|(left, right)| {
                            Box::new(P2pRpcResponse::LedgerQuery(
                                v2::MinaLedgerSyncLedgerAnswerStableV2::ChildHashesAre(
                                    left.clone(),
                                    right.clone(),
                                ),
                            ))
                        }),
                    });
                }
            }
            (req, LedgerReadResponse::GetChildAccountsAtAddr(resp)) => {
                for (peer_id, id, _) in find_peers_with_ledger_rpc(state, req) {
                    dispatcher.push(P2pChannelsRpcAction::ResponseSend {
                        peer_id,
                        id,
                        response: resp.as_ref().map(|accounts| {
                            Box::new(P2pRpcResponse::LedgerQuery(
                                v2::MinaLedgerSyncLedgerAnswerStableV2::ContentsAre(
                                    accounts.iter().cloned().collect(),
                                ),
                            ))
                        }),
                    });
                }
            }
            (req, LedgerReadResponse::GetStagedLedgerAuxAndPendingCoinbases(resp)) => {
                for (peer_id, id, is_streaming) in find_peers_with_ledger_rpc(state, req) {
                    if is_streaming {
                        dispatcher.push(P2pChannelsStreamingRpcAction::ResponseSendInit {
                            peer_id,
                            id,
                            response: resp.clone().map(Into::into),
                        });
                    } else {
                        dispatcher.push(P2pChannelsRpcAction::ResponseSend {
                            peer_id,
                            id,
                            response: resp.clone().map(|data| {
                                Box::new(P2pRpcResponse::StagedLedgerAuxAndPendingCoinbasesAtBlock(
                                    data,
                                ))
                            }),
                        });
                    }
                }
            }
            (
                LedgerReadRequest::ScanStateSummary(ledger_hash),
                LedgerReadResponse::ScanStateSummary(scan_state),
            ) => {
                for rpc_id in state
                    .rpc
                    .scan_state_summary_rpc_ids()
                    .filter(|(_, hash, _)| *hash == ledger_hash)
                    .map(|(id, ..)| id)
                    .collect::<Vec<_>>()
                {
                    dispatcher.push(RpcAction::ScanStateSummaryGetSuccess {
                        rpc_id,
                        scan_state: scan_state.clone(),
                    });
                }
            }
            (_, LedgerReadResponse::ScanStateSummary(..)) => unreachable!(),
            (_req, LedgerReadResponse::GetAccounts(..)) => todo!(),
            (_, LedgerReadResponse::AccountsForRpc(rpc_id, accounts, account_query)) => {
                dispatcher.push(RpcAction::LedgerAccountsGetSuccess {
                    rpc_id,
                    accounts,
                    account_query,
                });
            }
            (_, LedgerReadResponse::GetLedgerStatus(rpc_id, resp)) => {
                dispatcher.push(RpcAction::LedgerStatusGetSuccess {
                    rpc_id,
                    response: resp.clone(),
                });
            }
            (_, LedgerReadResponse::GetAccountDelegators(rpc_id, resp)) => {
                dispatcher.push(RpcAction::LedgerAccountDelegatorsGetSuccess {
                    rpc_id,
                    response: resp.clone(),
                });
            }
        }
    }

    fn next_read_requests_init(dispatcher: &mut Dispatcher<Action, State>, state: &State) {
        // fetching delegator table, this is required because delegator table construction requires reading from ledger.
        // It could be that ledger read quota was reached when vrf tried to initiate that read, so we need to "retry" it if that's the case
        dispatcher.push(BlockProducerVrfEvaluatorAction::BeginDelegatorTableConstruction);

        // p2p rpcs
        let mut peers = state
            .p2p
            .ready_peers_iter()
            .filter(|(_, peer)| {
                peer.channels
                    .rpc
                    .remote_todo_requests_iter()
                    .next()
                    .is_some()
                    || peer.channels.streaming_rpc.remote_todo_request().is_some()
            })
            .map(|(peer_id, peer)| (*peer_id, peer.channels.rpc_remote_last_responded()))
            .collect::<Vec<_>>();
        peers.sort_by_key(|(_, last_responded)| *last_responded);
        for (peer_id, _) in peers {
            let Some((id, request, is_streaming)) = None.or_else(|| {
                let peer = state.p2p.ready()?.get_ready_peer(&peer_id)?;
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
                            build_staged_ledger_parts_request(state, block_hash)?
                        }
                        _ => return None,
                    };

                    Some((req.id, ledger_request, false))
                })
                .or_else(|| {
                    let (id, req) = peer.channels.streaming_rpc.remote_todo_request()?;
                    let ledger_request = match req {
                        P2pStreamingRpcRequest::StagedLedgerParts(block_hash) => {
                            build_staged_ledger_parts_request(state, block_hash)?
                        }
                    };
                    Some((id, ledger_request, true))
                })
            }) else {
                continue;
            };

            dispatcher.push(LedgerReadAction::Init {
            request,
            callback: LedgerReadInitCallback::P2pChannelsResponsePending
         {      callback: redux::callback!(on_ledger_read_init_p2p_channels_response_pending((is_streaming: bool, id: P2pRpcId, peer_id: PeerId)) -> crate::Action{
                    if is_streaming {
                        P2pAction::from(P2pChannelsStreamingRpcAction::ResponsePending {
                            peer_id,
                            id,
                        })
                    } else {
                        P2pAction::from(P2pChannelsRpcAction::ResponsePending {
                            peer_id,
                            id,
                        })
                    }
                }),
                args:(is_streaming, id, peer_id)
        }
        });

            if !state.ledger.read.is_total_cost_under_limit() {
                return;
            }
        }

        // rpcs
        let rpcs = state
            .rpc
            .scan_state_summary_rpc_ids()
            .filter(|(.., status)| status.is_init())
            .map(|(id, ..)| id)
            .collect::<Vec<_>>();

        for rpc_id in rpcs {
            dispatcher.push(RpcAction::ScanStateSummaryLedgerGetInit { rpc_id });
            if !state.ledger.read.is_total_cost_under_limit() {
                return;
            }
        }

        let ledger_account_rpc = state
            .rpc
            .accounts_request_rpc_ids()
            .filter(|(.., status)| status.is_init())
            .map(|(id, req, _)| (id, req))
            .collect::<Vec<_>>();

        for (rpc_id, req) in ledger_account_rpc {
            dispatcher.push(RpcAction::LedgerAccountsGetInit {
                rpc_id,
                account_query: req,
            });
            if !state.ledger.read.is_total_cost_under_limit() {
                return;
            }
        }
    }
}

fn find_peers_with_ledger_rpc(
    state: &crate::State,
    req: &LedgerReadRequest,
) -> Vec<(PeerId, P2pRpcId, bool)> {
    let Some(p2p) = state.p2p.ready() else {
        return Vec::new();
    };
    p2p.ready_peers_iter()
        .flat_map(|(peer_id, peer)| {
            let rpcs = peer
                .channels
                .rpc
                .remote_pending_requests_iter()
                .map(move |req| (peer_id, req.id, &req.request))
                .filter(|(_, _, peer_req)| match (req, peer_req) {
                    (
                        LedgerReadRequest::GetNumAccounts(h1),
                        P2pRpcRequest::LedgerQuery(
                            h2,
                            v2::MinaLedgerSyncLedgerQueryStableV1::NumAccounts,
                        ),
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
                        .is_some_and(|b| b.blockchain_state.staged_ledger_hash == data.ledger_hash),
                    _ => false,
                })
                .map(|(peer_id, rpc_id, _)| (*peer_id, rpc_id, false));
            let streaming_rpcs = peer
                .channels
                .streaming_rpc
                .remote_pending_request()
                .into_iter()
                .filter(|(_, peer_req)| match (req, peer_req) {
                    (
                        LedgerReadRequest::GetStagedLedgerAuxAndPendingCoinbases(data),
                        P2pStreamingRpcRequest::StagedLedgerParts(block_hash),
                    ) => state
                        .transition_frontier
                        .get_state_body(block_hash)
                        .is_some_and(|b| b.blockchain_state.staged_ledger_hash == data.ledger_hash),
                    _ => false,
                })
                .map(|(rpc_id, _)| (*peer_id, rpc_id, true));
            rpcs.chain(streaming_rpcs)
        })
        .collect()
}

fn build_staged_ledger_parts_request(
    state: &crate::State,
    block_hash: &v2::StateHash,
) -> Option<LedgerReadRequest> {
    let tf = &state.transition_frontier;
    let ledger_hash = tf
        .best_chain
        .iter()
        .find(|b| b.hash() == block_hash)
        .map(|b| b.staged_ledger_hashes().clone())?;
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

    Some(LedgerReadRequest::GetStagedLedgerAuxAndPendingCoinbases(
        LedgerReadStagedLedgerAuxAndPendingCoinbases {
            ledger_hash,
            protocol_states,
        },
    ))
}
