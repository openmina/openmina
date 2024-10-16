use ark_ff::fields::arithmetic::InvalidBigInt;
use mina_p2p_messages::v2::{MinaLedgerSyncLedgerAnswerStableV2, StateHash};
use openmina_core::{block::BlockWithHash, bug_condition};
use p2p::{
    channels::{
        best_tip::P2pChannelsBestTipAction,
        rpc::{BestTipWithProof, P2pChannelsRpcAction, P2pRpcRequest, P2pRpcResponse},
        streaming_rpc::P2pStreamingRpcResponseFull,
    },
    disconnection::{P2pDisconnectionAction, P2pDisconnectionReason},
    PeerId,
};
use redux::{ActionMeta, ActionWithMeta, Dispatcher};

use crate::{
    p2p_ready,
    snark_pool::candidate::SnarkPoolCandidateAction,
    transition_frontier::sync::{
        ledger::{
            snarked::{
                PeerLedgerQueryError, PeerLedgerQueryResponse,
                TransitionFrontierSyncLedgerSnarkedAction,
            },
            staged::{PeerStagedLedgerPartsFetchError, TransitionFrontierSyncLedgerStagedAction},
        },
        PeerBlockFetchError, TransitionFrontierSyncAction,
    },
    watched_accounts::{
        WatchedAccountLedgerInitialState, WatchedAccountsLedgerInitialStateGetError,
    },
    Action, ConsensusAction, State, WatchedAccountsAction,
};

use super::P2pCallbacksAction;

impl crate::State {
    pub fn p2p_callback_reducer(
        state_context: crate::Substate<Self>,
        action: ActionWithMeta<&P2pCallbacksAction>,
    ) {
        let (action, meta) = action.split();
        let (dispatcher, state) = state_context.into_dispatcher_and_state();

        match action {
            P2pCallbacksAction::P2pChannelsRpcReady { peer_id } => {
                let peer_id = *peer_id;

                if state.p2p.get_peer(&peer_id).map_or(false, |p| p.is_libp2p) {
                    // for webrtc peers, we don't need to send this rpc, as we
                    // will receive current best tip in best tip channel anyways.
                    dispatcher.push(P2pChannelsRpcAction::RequestSend {
                        peer_id,
                        id: 0,
                        request: Box::new(P2pRpcRequest::BestTipWithProof),
                        on_init: None,
                    });
                }

                dispatcher.push(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
                dispatcher.push(TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit);
                dispatcher.push(TransitionFrontierSyncAction::BlocksPeersQuery);
            }
            P2pCallbacksAction::P2pChannelsRpcTimeout { peer_id, id } => {
                let peer_id = *peer_id;
                let rpc_id = *id;
                let Some(peer) = state.p2p.get_ready_peer(&peer_id) else {
                    bug_condition!("get_ready_peer({:?}) returned None", peer_id);
                    return;
                };

                let Some(rpc_kind) = peer.channels.rpc.pending_local_rpc_kind() else {
                    bug_condition!("peer: {:?} pending_local_rpc_kind() returned None", peer_id);
                    return;
                };

                dispatcher.push(
                    TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressError {
                        peer_id,
                        rpc_id,
                        error: PeerLedgerQueryError::Timeout,
                    },
                );
                dispatcher.push(
                    TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchError {
                        peer_id,
                        rpc_id,
                        error: PeerStagedLedgerPartsFetchError::Timeout,
                    },
                );
                dispatcher.push(TransitionFrontierSyncAction::BlocksPeerQueryError {
                    peer_id,
                    rpc_id,
                    error: PeerBlockFetchError::Timeout,
                });
                dispatcher.push(P2pDisconnectionAction::Init {
                    peer_id,
                    reason: P2pDisconnectionReason::TransitionFrontierRpcTimeout(rpc_kind),
                });
            }
            P2pCallbacksAction::P2pChannelsRpcResponseReceived {
                peer_id,
                id,
                response,
            } => {
                State::handle_rpc_channels_response(dispatcher, meta, *id, *peer_id, response);
                dispatcher.push(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
                dispatcher.push(TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit);
                dispatcher.push(TransitionFrontierSyncAction::BlocksPeersQuery);
            }
            P2pCallbacksAction::P2pChannelsRpcRequestReceived {
                peer_id,
                id,
                request,
            } => {
                State::handle_rpc_channels_request(
                    dispatcher,
                    state,
                    meta,
                    *request.clone(),
                    *peer_id,
                    *id,
                );
            }
            P2pCallbacksAction::P2pChannelsStreamingRpcReady => {
                dispatcher.push(TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit);
            }
            P2pCallbacksAction::P2pChannelsStreamingRpcTimeout { peer_id, id } => {
                let peer_id = *peer_id;
                let rpc_id = *id;

                let Some(peer) = state.p2p.get_ready_peer(&peer_id) else {
                    bug_condition!("get_ready_peer({:?}) returned None", peer_id);
                    return;
                };
                let Some(rpc_kind) = peer.channels.streaming_rpc.pending_local_rpc_kind() else {
                    bug_condition!("peer: {:?} pending_local_rpc_kind() returned None", peer_id);
                    return;
                };
                dispatcher.push(
                    TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchError {
                        peer_id,
                        rpc_id,
                        error: PeerStagedLedgerPartsFetchError::Timeout,
                    },
                );
                dispatcher.push(P2pDisconnectionAction::Init {
                    peer_id,
                    reason: P2pDisconnectionReason::TransitionFrontierStreamingRpcTimeout(rpc_kind),
                });
            }
            P2pCallbacksAction::P2pChannelsStreamingRpcResponseReceived {
                peer_id,
                id,
                response,
            } => {
                let peer_id = *peer_id;
                let rpc_id = *id;

                match response {
                    None => {
                        dispatcher.push(
                            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchError {
                                peer_id,
                                rpc_id,
                                error: PeerStagedLedgerPartsFetchError::DataUnavailable,
                            },
                        );
                    }
                    Some(P2pStreamingRpcResponseFull::StagedLedgerParts(parts)) => {
                        dispatcher.push(
                            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchSuccess {
                                peer_id,
                                rpc_id,
                                parts: parts.clone(),
                            },
                        );
                    }
                }
                dispatcher.push(TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit);
            }
            P2pCallbacksAction::P2pDisconnection { peer_id } => {
                let peer_id = *peer_id;

                if let Some(s) = state.transition_frontier.sync.ledger() {
                    s.snarked()
                        .map(|s| {
                            s.peer_address_query_pending_rpc_ids(&peer_id)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                        .into_iter()
                        .for_each(|rpc_id| {
                            dispatcher.push(
                                TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressError {
                                    peer_id,
                                    rpc_id,
                                    error: PeerLedgerQueryError::Disconnected,
                                },
                            );
                        });

                    if let Some(rpc_id) = s
                        .snarked()
                        .and_then(|s| s.peer_num_accounts_rpc_id(&peer_id))
                    {
                        dispatcher.push(
                            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsError {
                                peer_id,
                                rpc_id,
                                error: PeerLedgerQueryError::Disconnected,
                            },
                        );
                    }

                    if let Some(rpc_id) = s.staged().and_then(|s| s.parts_fetch_rpc_id(&peer_id)) {
                        dispatcher.push(
                            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchError {
                                peer_id,
                                rpc_id,
                                error: PeerStagedLedgerPartsFetchError::Disconnected,
                            },
                        )
                    }
                }

                state
                    .transition_frontier
                    .sync
                    .blocks_fetch_from_peer_pending_rpc_ids(&peer_id)
                    .for_each(|rpc_id| {
                        dispatcher.push(TransitionFrontierSyncAction::BlocksPeerQueryError {
                            peer_id,
                            rpc_id,
                            error: PeerBlockFetchError::Disconnected,
                        });
                    });

                state
                    .watched_accounts
                    .iter()
                    .filter_map(|(pub_key, a)| match &a.initial_state {
                        WatchedAccountLedgerInitialState::Pending {
                            peer_id: account_peer_id,
                            ..
                        } => {
                            if account_peer_id == &peer_id {
                                Some(WatchedAccountsAction::LedgerInitialStateGetError {
                                    pub_key: pub_key.clone(),
                                    error:
                                        WatchedAccountsLedgerInitialStateGetError::PeerDisconnected,
                                })
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .for_each(|action| dispatcher.push(action));

                dispatcher.push(SnarkPoolCandidateAction::PeerPrune { peer_id });
            }
            P2pCallbacksAction::RpcRespondBestTip { peer_id } => {
                let Some(best_tip) = state.transition_frontier.best_tip() else {
                    bug_condition!("Best tip not found");
                    return;
                };

                dispatcher.push(P2pChannelsBestTipAction::ResponseSend {
                    peer_id: *peer_id,
                    best_tip: best_tip.clone(),
                });
            }
        }
    }

    fn handle_rpc_channels_request(
        dispatcher: &mut Dispatcher<Action, State>,
        state: &State,
        meta: ActionMeta,
        request: P2pRpcRequest,
        peer_id: PeerId,
        id: u64,
    ) {
        match request {
            P2pRpcRequest::BestTipWithProof => {
                let best_chain = &state.transition_frontier.best_chain;
                let response = None.or_else(|| {
                    let best_tip = best_chain.last()?;
                    let mut chain_iter = best_chain.iter();
                    let root_block = chain_iter.next();
                    // when our best tip is genesis block.
                    let root_block = root_block.unwrap_or(best_tip);
                    // TODO(binier): cache body hashes
                    let Ok(body_hashes) = chain_iter
                        .map(|b| b.header().protocol_state.body.try_hash())
                        .collect::<Result<_, _>>()
                    else {
                        openmina_core::error!(meta.time(); "P2pRpcRequest::BestTipWithProof: invalid protocol state");
                        return None;
                    };

                    Some(BestTipWithProof {
                        best_tip: best_tip.block().clone(),
                        proof: (body_hashes, root_block.block().clone()),
                    })
                });
                let response = response.map(P2pRpcResponse::BestTipWithProof).map(Box::new);
                dispatcher.push(P2pChannelsRpcAction::ResponseSend {
                    peer_id,
                    id,
                    response,
                });
            }
            P2pRpcRequest::Block(hash) => {
                let best_chain = &state.transition_frontier.best_chain;
                let response = best_chain
                    .iter()
                    .rev()
                    .find(|b| b.hash() == &hash)
                    .map(|b| b.block().clone())
                    .map(P2pRpcResponse::Block)
                    .map(Box::new);
                dispatcher.push(P2pChannelsRpcAction::ResponseSend {
                    peer_id,
                    id,
                    response,
                });
            }
            P2pRpcRequest::LedgerQuery(..) => {
                // async ledger request will be triggered
                // by `LedgerReadAction::FindTodos`.
            }
            P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(..) => {
                // async ledger request will be triggered
                // by `LedgerReadAction::FindTodos`.
            }
            P2pRpcRequest::Snark(job_id) => {
                let job = state.snark_pool.get(&job_id);
                let response = job
                    .and_then(|job| job.snark.as_ref())
                    .map(|snark| snark.work.clone())
                    .map(P2pRpcResponse::Snark)
                    .map(Box::new);

                dispatcher.push(P2pChannelsRpcAction::ResponseSend {
                    peer_id,
                    id,
                    response,
                });
            }
            P2pRpcRequest::InitialPeers => {
                let p2p = p2p_ready!(state.p2p, meta.time());
                let peers = p2p
                    .peers
                    .iter()
                    .filter_map(|(_, v)| v.dial_opts.clone())
                    .collect();
                let response = Some(Box::new(P2pRpcResponse::InitialPeers(peers)));

                dispatcher.push(P2pChannelsRpcAction::ResponseSend {
                    peer_id,
                    id,
                    response,
                });
            }
        }
    }

    fn handle_rpc_channels_response(
        dispatcher: &mut Dispatcher<Action, State>,
        meta: ActionMeta,
        id: u64,
        peer_id: PeerId,
        response: &Option<Box<P2pRpcResponse>>,
    ) {
        match response.as_deref() {
            None => {
                dispatcher.push(
                    TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressError {
                        peer_id,
                        rpc_id: id,
                        error: PeerLedgerQueryError::DataUnavailable,
                    },
                );
                dispatcher.push(
                    TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchError {
                        peer_id,
                        rpc_id: id,
                        error: PeerStagedLedgerPartsFetchError::DataUnavailable,
                    },
                );
                dispatcher.push(TransitionFrontierSyncAction::BlocksPeerQueryError {
                    peer_id,
                    rpc_id: id,
                    error: PeerBlockFetchError::DataUnavailable,
                });
            }
            Some(P2pRpcResponse::BestTipWithProof(resp)) => {
                let (body_hashes, root_block) = &resp.proof;

                let (Ok(best_tip), Ok(root_block)) = (
                    BlockWithHash::try_new(resp.best_tip.clone()),
                    BlockWithHash::try_new(root_block.clone()),
                ) else {
                    openmina_core::error!(meta.time(); "P2pRpcResponse::BestTipWithProof: invalid blocks");
                    return;
                };

                // reconstruct hashes
                let Ok(hashes) = body_hashes
                    .iter()
                    .take(body_hashes.len().saturating_sub(1))
                    .scan(root_block.hash.clone(), |pred_hash, body_hash| {
                        *pred_hash = match StateHash::try_from_hashes(pred_hash, body_hash) {
                            Ok(hash) => hash,
                            Err(_) => return Some(Err(InvalidBigInt)),
                        };
                        Some(Ok(pred_hash.clone()))
                    })
                    .collect::<Result<Vec<_>, _>>()
                else {
                    openmina_core::error!(meta.time(); "P2pRpcResponse::BestTipWithProof: invalid hashes");
                    return;
                };

                if let Some(pred_hash) = hashes.last() {
                    let expected_hash = &best_tip.block.header.protocol_state.previous_state_hash;

                    if pred_hash != expected_hash {
                        openmina_core::warn!(meta.time();
                        kind = "P2pRpcBestTipHashMismatch",
                        response = serde_json::to_string(&resp).ok(),
                        expected_hash = expected_hash.to_string(),
                        calculated_hash = pred_hash.to_string());
                        return;
                    }
                }
                dispatcher.push(ConsensusAction::BlockChainProofUpdate {
                    hash: best_tip.hash,
                    chain_proof: (hashes, root_block),
                });
            }
            Some(P2pRpcResponse::LedgerQuery(answer)) => match answer {
                MinaLedgerSyncLedgerAnswerStableV2::ChildHashesAre(left, right) => {
                    dispatcher.push(
                        TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressSuccess {
                            peer_id,
                            rpc_id: id,
                            response: PeerLedgerQueryResponse::ChildHashes(
                                left.clone(),
                                right.clone(),
                            ),
                        },
                    );
                }
                MinaLedgerSyncLedgerAnswerStableV2::ContentsAre(accounts) => {
                    dispatcher.push(
                        TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressSuccess {
                            peer_id,
                            rpc_id: id,
                            response: PeerLedgerQueryResponse::ChildAccounts(
                                accounts.iter().cloned().collect(),
                            ),
                        },
                    );
                }
                MinaLedgerSyncLedgerAnswerStableV2::NumAccounts(count, contents_hash) => {
                    dispatcher.push(
                        TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsSuccess {
                            peer_id,
                            rpc_id: id,
                            response: PeerLedgerQueryResponse::NumAccounts(
                                count.as_u64(),
                                contents_hash.clone(),
                            ),
                        },
                    );
                }
            },
            Some(P2pRpcResponse::StagedLedgerAuxAndPendingCoinbasesAtBlock(parts)) => {
                dispatcher.push(
                    TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchSuccess {
                        peer_id,
                        rpc_id: id,
                        parts: parts.clone(),
                    },
                );
            }
            Some(P2pRpcResponse::Block(block)) => {
                let Ok(block) = BlockWithHash::try_new(block.clone()) else {
                    openmina_core::error!(meta.time(); "P2pRpcResponse::Block: invalid block");
                    return;
                };
                dispatcher.push(TransitionFrontierSyncAction::BlocksPeerQuerySuccess {
                    peer_id,
                    rpc_id: id,
                    response: block,
                });
            }
            Some(P2pRpcResponse::Snark(snark)) => {
                dispatcher.push(SnarkPoolCandidateAction::WorkReceived {
                    peer_id,
                    work: snark.clone(),
                });
            }
            Some(P2pRpcResponse::InitialPeers(_)) => {}
        }
    }
}
