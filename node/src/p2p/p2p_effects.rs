use mina_p2p_messages::v2::{MinaLedgerSyncLedgerAnswerStableV2, StateHash};
use openmina_core::block::BlockWithHash;

use crate::consensus::{ConsensusBlockChainProofUpdateAction, ConsensusBlockReceivedAction};
use crate::rpc::{
    RpcP2pConnectionIncomingErrorAction, RpcP2pConnectionIncomingRespondAction,
    RpcP2pConnectionIncomingSuccessAction, RpcP2pConnectionOutgoingErrorAction,
    RpcP2pConnectionOutgoingSuccessAction,
};
use crate::snark_pool::candidate::{
    SnarkPoolCandidateInfoReceivedAction, SnarkPoolCandidatePeerPruneAction,
    SnarkPoolCandidateWorkReceivedAction,
};
use crate::snark_pool::SnarkPoolJobCommitmentAddAction;
use crate::transition_frontier::sync::ledger::snarked::{
    PeerLedgerQueryError, PeerLedgerQueryResponse,
    TransitionFrontierSyncLedgerSnarkedPeerQueryErrorAction,
    TransitionFrontierSyncLedgerSnarkedPeerQuerySuccessAction,
    TransitionFrontierSyncLedgerSnarkedPeersQueryAction,
};
use crate::transition_frontier::sync::ledger::staged::{
    PeerStagedLedgerPartsFetchError, TransitionFrontierSyncLedgerStagedPartsPeerFetchErrorAction,
    TransitionFrontierSyncLedgerStagedPartsPeerFetchInitAction,
    TransitionFrontierSyncLedgerStagedPartsPeerFetchSuccessAction,
};
use crate::transition_frontier::sync::{
    PeerBlockFetchError, TransitionFrontierSyncBlocksPeerQueryErrorAction,
    TransitionFrontierSyncBlocksPeerQuerySuccessAction,
    TransitionFrontierSyncBlocksPeersQueryAction,
};
use crate::watched_accounts::{
    WatchedAccountLedgerInitialState, WatchedAccountsLedgerInitialStateGetError,
    WatchedAccountsLedgerInitialStateGetErrorAction,
};
use crate::{Service, Store};

use super::channels::best_tip::{P2pChannelsBestTipAction, P2pChannelsBestTipResponseSendAction};
use super::channels::rpc::{
    BestTipWithProof, P2pChannelsRpcAction, P2pChannelsRpcRequestSendAction,
    P2pChannelsRpcResponseSendAction, P2pRpcRequest, P2pRpcResponse,
};
use super::channels::snark::P2pChannelsSnarkAction;
use super::channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentAction;
use super::channels::P2pChannelsAction;
use super::connection::incoming::{
    P2pConnectionIncomingAction, P2pConnectionIncomingAnswerSendSuccessAction,
};
use super::connection::outgoing::P2pConnectionOutgoingAction;
use super::connection::{P2pConnectionAction, P2pConnectionResponse};
use super::disconnection::{
    P2pDisconnectionAction, P2pDisconnectionInitAction, P2pDisconnectionReason,
};
use super::discovery::{P2pDiscoveryAction, P2pDiscoveryInitAction, P2pDiscoverySuccessAction};
use super::peer::P2pPeerAction;
use super::{P2pAction, P2pActionWithMeta};

use p2p::P2pPeerStatus;

pub fn p2p_effects<S: Service>(store: &mut Store<S>, action: P2pActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        P2pAction::Connection(action) => match action {
            P2pConnectionAction::Outgoing(action) => match action {
                P2pConnectionOutgoingAction::RandomInit(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Init(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Reconnect(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::OfferSdpCreatePending(_) => {}
                P2pConnectionOutgoingAction::OfferSdpCreateError(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::OfferSdpCreateSuccess(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::OfferReady(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::OfferSendSuccess(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::AnswerRecvPending(_) => {}
                P2pConnectionOutgoingAction::AnswerRecvError(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::AnswerRecvSuccess(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::FinalizePending(_) => {}
                P2pConnectionOutgoingAction::FinalizeError(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::FinalizeSuccess(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Timeout(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Error(action) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_rpc_id(&action.peer_id) {
                        store.dispatch(RpcP2pConnectionOutgoingErrorAction {
                            rpc_id,
                            error: action.error.clone(),
                        });
                    }
                    // action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Success(action) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_rpc_id(&action.peer_id) {
                        store.dispatch(RpcP2pConnectionOutgoingSuccessAction { rpc_id });
                    }
                    action.effects(&meta, store);
                }
            },
            P2pConnectionAction::Incoming(action) => match action {
                P2pConnectionIncomingAction::Init(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::AnswerSdpCreatePending(_) => {}
                P2pConnectionIncomingAction::AnswerSdpCreateError(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::AnswerSdpCreateSuccess(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::AnswerReady(action) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_rpc_id(&action.peer_id) {
                        store.dispatch(RpcP2pConnectionIncomingRespondAction {
                            rpc_id,
                            response: P2pConnectionResponse::Accepted(action.answer.clone()),
                        });
                        store.dispatch(P2pConnectionIncomingAnswerSendSuccessAction {
                            peer_id: action.peer_id,
                        });
                    }
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::AnswerSendSuccess(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::FinalizePending(_) => {}
                P2pConnectionIncomingAction::FinalizeError(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::FinalizeSuccess(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::Timeout(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::Error(action) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_rpc_id(&action.peer_id) {
                        store.dispatch(RpcP2pConnectionIncomingErrorAction {
                            rpc_id,
                            error: format!("{:?}", action.error),
                        });
                    }
                    // action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::Success(action) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_rpc_id(&action.peer_id) {
                        store.dispatch(RpcP2pConnectionIncomingSuccessAction { rpc_id });
                    }
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::Libp2pReceived(action) => {
                    action.effects(&meta, store);
                }
            },
        },
        P2pAction::Disconnection(action) => match action {
            P2pDisconnectionAction::Init(action) => action.effects(&meta, store),
            P2pDisconnectionAction::Finish(action) => {
                if let Some(s) = store.state().transition_frontier.sync.root_ledger() {
                    let rpc_ids = s
                        .snarked()
                        .map(|s| s.peer_query_pending_rpc_ids(&action.peer_id).collect())
                        .unwrap_or(vec![]);
                    let staged_ledger_parts_fetch_rpc_id = s
                        .staged()
                        .and_then(|s| s.parts_fetch_rpc_id(&action.peer_id));

                    for rpc_id in rpc_ids {
                        store.dispatch(TransitionFrontierSyncLedgerSnarkedPeerQueryErrorAction {
                            peer_id: action.peer_id,
                            rpc_id,
                            error: PeerLedgerQueryError::Disconnected,
                        });
                    }

                    if let Some(rpc_id) = staged_ledger_parts_fetch_rpc_id {
                        store.dispatch(
                            TransitionFrontierSyncLedgerStagedPartsPeerFetchErrorAction {
                                peer_id: action.peer_id,
                                rpc_id,
                                error: PeerStagedLedgerPartsFetchError::Disconnected,
                            },
                        );
                    }
                }

                let blocks_fetch_rpc_ids = store
                    .state()
                    .transition_frontier
                    .sync
                    .blocks_fetch_from_peer_pending_rpc_ids(&action.peer_id)
                    .collect::<Vec<_>>();

                for rpc_id in blocks_fetch_rpc_ids {
                    store.dispatch(TransitionFrontierSyncBlocksPeerQueryErrorAction {
                        peer_id: action.peer_id,
                        rpc_id,
                        error: PeerBlockFetchError::Disconnected,
                    });
                }

                let actions = store
                    .state()
                    .watched_accounts
                    .iter()
                    .filter_map(|(pub_key, a)| match &a.initial_state {
                        WatchedAccountLedgerInitialState::Pending { peer_id, .. } => {
                            if peer_id == &action.peer_id {
                                Some(WatchedAccountsLedgerInitialStateGetErrorAction {
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
                    .collect::<Vec<_>>();

                for action in actions {
                    store.dispatch(action);
                }

                store.dispatch(SnarkPoolCandidatePeerPruneAction {
                    peer_id: action.peer_id,
                });
            }
        },
        P2pAction::Discovery(action) => match action {
            P2pDiscoveryAction::Init(P2pDiscoveryInitAction { peer_id }) => {
                let Some(peer) = store.state().p2p.peers.get(&peer_id) else {
                    return;
                };
                let P2pPeerStatus::Ready(status) = &peer.status else {
                    return;
                };
                store.dispatch(P2pChannelsRpcRequestSendAction {
                    peer_id,
                    id: status.channels.rpc.next_local_rpc_id(),
                    request: P2pRpcRequest::InitialPeers,
                });
            }
            P2pDiscoveryAction::Success(_) => {}
            P2pDiscoveryAction::KademliaInit(action) => {
                // dbg!(action);
                store.service().find_random_peer();
            }
            P2pDiscoveryAction::KademliaSuccess(action) => {
                // dbg!(action);
            }
        },
        P2pAction::Channels(action) => match action {
            P2pChannelsAction::MessageReceived(action) => {
                action.effects(&meta, store);
            }
            P2pChannelsAction::BestTip(action) => match action {
                P2pChannelsBestTipAction::Init(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsBestTipAction::Pending(_) => {}
                P2pChannelsBestTipAction::Ready(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsBestTipAction::RequestSend(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsBestTipAction::Received(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsBestTipAction::RequestReceived(action) => {
                    if let Some(best_tip) = store.state().transition_frontier.best_tip() {
                        store.dispatch(P2pChannelsBestTipResponseSendAction {
                            peer_id: action.peer_id,
                            best_tip: best_tip.clone(),
                        });
                    }
                }
                P2pChannelsBestTipAction::ResponseSend(action) => {
                    action.effects(&meta, store);
                }
            },
            P2pChannelsAction::Snark(action) => match action {
                P2pChannelsSnarkAction::Init(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsSnarkAction::Pending(_) => {}
                P2pChannelsSnarkAction::Ready(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsSnarkAction::RequestSend(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsSnarkAction::PromiseReceived(_) => {}
                P2pChannelsSnarkAction::Received(action) => {
                    action.effects(&meta, store);
                    store.dispatch(SnarkPoolCandidateInfoReceivedAction {
                        peer_id: action.peer_id,
                        info: action.snark,
                    });
                }
                P2pChannelsSnarkAction::RequestReceived(_) => {}
                P2pChannelsSnarkAction::ResponseSend(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsSnarkAction::Libp2pReceived(action) => {
                    store.dispatch(SnarkPoolCandidateWorkReceivedAction {
                        peer_id: action.peer_id,
                        work: action.snark,
                    });
                }
                P2pChannelsSnarkAction::Libp2pBroadcast(action) => {
                    action.effects(&meta, store);
                }
            },
            P2pChannelsAction::SnarkJobCommitment(action) => match action {
                P2pChannelsSnarkJobCommitmentAction::Init(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsSnarkJobCommitmentAction::Pending(_) => {}
                P2pChannelsSnarkJobCommitmentAction::Ready(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsSnarkJobCommitmentAction::RequestSend(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsSnarkJobCommitmentAction::PromiseReceived(_) => {}
                P2pChannelsSnarkJobCommitmentAction::Received(action) => {
                    action.effects(&meta, store);
                    store.dispatch(SnarkPoolJobCommitmentAddAction {
                        commitment: action.commitment,
                        sender: action.peer_id,
                    });
                }
                P2pChannelsSnarkJobCommitmentAction::RequestReceived(_) => {}
                P2pChannelsSnarkJobCommitmentAction::ResponseSend(action) => {
                    action.effects(&meta, store);
                }
            },
            P2pChannelsAction::Rpc(action) => match action {
                P2pChannelsRpcAction::Init(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsRpcAction::Pending(_) => {}
                P2pChannelsRpcAction::Ready(a) => {
                    store.dispatch(P2pChannelsRpcRequestSendAction {
                        peer_id: a.peer_id,
                        id: 0,
                        request: P2pRpcRequest::BestTipWithProof,
                    });

                    store.dispatch(TransitionFrontierSyncLedgerSnarkedPeersQueryAction {});
                    store.dispatch(TransitionFrontierSyncLedgerStagedPartsPeerFetchInitAction {});
                    store.dispatch(TransitionFrontierSyncBlocksPeersQueryAction {});
                }
                P2pChannelsRpcAction::RequestSend(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsRpcAction::Timeout(action) => {
                    store.dispatch(TransitionFrontierSyncLedgerSnarkedPeerQueryErrorAction {
                        peer_id: action.peer_id,
                        rpc_id: action.id,
                        error: PeerLedgerQueryError::Timeout,
                    });
                    store.dispatch(
                        TransitionFrontierSyncLedgerStagedPartsPeerFetchErrorAction {
                            peer_id: action.peer_id,
                            rpc_id: action.id,
                            error: PeerStagedLedgerPartsFetchError::Timeout,
                        },
                    );
                    store.dispatch(TransitionFrontierSyncBlocksPeerQueryErrorAction {
                        peer_id: action.peer_id,
                        rpc_id: action.id,
                        error: PeerBlockFetchError::Timeout,
                    });
                    store.dispatch(P2pDisconnectionInitAction {
                        peer_id: action.peer_id,
                        reason: P2pDisconnectionReason::TransitionFrontierRpcTimeout,
                    });
                }
                P2pChannelsRpcAction::ResponseReceived(action) => {
                    action.effects(&meta, store);
                    match action.response.as_ref() {
                        None => {
                            store.dispatch(
                                TransitionFrontierSyncLedgerSnarkedPeerQueryErrorAction {
                                    peer_id: action.peer_id,
                                    rpc_id: action.id,
                                    error: PeerLedgerQueryError::DataUnavailable,
                                },
                            );
                            store.dispatch(
                                TransitionFrontierSyncLedgerStagedPartsPeerFetchErrorAction {
                                    peer_id: action.peer_id,
                                    rpc_id: action.id,
                                    error: PeerStagedLedgerPartsFetchError::DataUnavailable,
                                },
                            );
                            store.dispatch(TransitionFrontierSyncBlocksPeerQueryErrorAction {
                                peer_id: action.peer_id,
                                rpc_id: action.id,
                                error: PeerBlockFetchError::DataUnavailable,
                            });
                        }
                        Some(P2pRpcResponse::BestTipWithProof(resp)) => {
                            let (body_hashes, root_block) = &resp.proof;
                            let best_tip = BlockWithHash::new(resp.best_tip.clone());
                            let root_block = BlockWithHash::new(root_block.clone());

                            // reconstruct hashes
                            let hashes = body_hashes
                                .iter()
                                .take(body_hashes.len().saturating_sub(1))
                                .scan(root_block.hash.clone(), |pred_hash, body_hash| {
                                    *pred_hash = StateHash::from_hashes(pred_hash, body_hash);
                                    Some(pred_hash.clone())
                                })
                                .collect::<Vec<_>>();

                            if let Some(pred_hash) = hashes.last() {
                                let expected_hash =
                                    &best_tip.block.header.protocol_state.previous_state_hash;
                                if pred_hash != expected_hash {
                                    openmina_core::warn!(meta.time();
                                        kind = "P2pRpcBestTipHashMismatch",
                                        response = serde_json::to_string(resp).ok(),
                                        expected_hash = expected_hash.to_string(),
                                        calculated_hash = pred_hash.to_string());
                                    return;
                                }
                            }
                            store.dispatch(ConsensusBlockChainProofUpdateAction {
                                hash: best_tip.hash,
                                chain_proof: (hashes, root_block),
                            });
                        }
                        Some(P2pRpcResponse::LedgerQuery(answer)) => match answer {
                            MinaLedgerSyncLedgerAnswerStableV2::ChildHashesAre(left, right) => {
                                store.dispatch(
                                    TransitionFrontierSyncLedgerSnarkedPeerQuerySuccessAction {
                                        peer_id: action.peer_id,
                                        rpc_id: action.id,
                                        response: PeerLedgerQueryResponse::ChildHashes(
                                            left.clone(),
                                            right.clone(),
                                        ),
                                    },
                                );
                            }
                            MinaLedgerSyncLedgerAnswerStableV2::ContentsAre(accounts) => {
                                store.dispatch(
                                    TransitionFrontierSyncLedgerSnarkedPeerQuerySuccessAction {
                                        peer_id: action.peer_id,
                                        rpc_id: action.id,
                                        response: PeerLedgerQueryResponse::ChildAccounts(
                                            accounts.clone(),
                                        ),
                                    },
                                );
                            }
                            _ => {}
                        },
                        Some(P2pRpcResponse::StagedLedgerAuxAndPendingCoinbasesAtBlock(parts)) => {
                            store.dispatch(
                                TransitionFrontierSyncLedgerStagedPartsPeerFetchSuccessAction {
                                    peer_id: action.peer_id,
                                    rpc_id: action.id,
                                    parts: parts.clone(),
                                },
                            );
                        }
                        Some(P2pRpcResponse::Block(block)) => {
                            let block = BlockWithHash::new(block.clone());
                            store.dispatch(TransitionFrontierSyncBlocksPeerQuerySuccessAction {
                                peer_id: action.peer_id,
                                rpc_id: action.id,
                                response: block,
                            });
                        }
                        Some(P2pRpcResponse::Snark(snark)) => {
                            store.dispatch(SnarkPoolCandidateWorkReceivedAction {
                                peer_id: action.peer_id,
                                work: snark.clone(),
                            });
                        }
                        Some(P2pRpcResponse::InitialPeers(peers)) => {
                            store.dispatch(P2pDiscoverySuccessAction {
                                peer_id: action.peer_id,
                                peers: peers.clone(),
                            });
                        }
                    }
                    store.dispatch(TransitionFrontierSyncLedgerSnarkedPeersQueryAction {});
                    store.dispatch(TransitionFrontierSyncLedgerStagedPartsPeerFetchInitAction {});
                    store.dispatch(TransitionFrontierSyncBlocksPeersQueryAction {});
                }
                P2pChannelsRpcAction::RequestReceived(action) => {
                    match action.request {
                        P2pRpcRequest::BestTipWithProof => {
                            let best_chain = &store.state().transition_frontier.best_chain;
                            let response = None.or_else(|| {
                                let best_tip = best_chain.last()?;
                                let mut chain_iter = best_chain.iter();
                                let root_block = chain_iter.next()?;
                                // TODO(binier): cache body hashes
                                let body_hashes = chain_iter
                                    .map(|b| b.block.header.protocol_state.body.hash())
                                    .collect();

                                Some(BestTipWithProof {
                                    best_tip: best_tip.block.clone(),
                                    proof: (body_hashes, root_block.block.clone()),
                                })
                            });
                            let response = response.map(P2pRpcResponse::BestTipWithProof);
                            store.dispatch(P2pChannelsRpcResponseSendAction {
                                peer_id: action.peer_id,
                                id: action.id,
                                response,
                            });
                        }
                        P2pRpcRequest::Block(hash) => {
                            let best_chain = &store.state().transition_frontier.best_chain;
                            let response = best_chain
                                .iter()
                                .rev()
                                .find(|block| block.hash == hash)
                                .map(|block| block.block.clone())
                                .map(P2pRpcResponse::Block);
                            store.dispatch(P2pChannelsRpcResponseSendAction {
                                peer_id: action.peer_id,
                                id: action.id,
                                response,
                            });
                        }
                        P2pRpcRequest::LedgerQuery(ledger_hash, query) => {
                            let response = store
                                .service
                                .answer_ledger_query(ledger_hash, query)
                                .map(P2pRpcResponse::LedgerQuery);

                            store.dispatch(P2pChannelsRpcResponseSendAction {
                                peer_id: action.peer_id,
                                id: action.id,
                                response,
                            });
                        }
                        P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(block_hash) => {
                            let transition_frontier = &store.state.get().transition_frontier;
                            let best_chain = &transition_frontier.best_chain;

                            let response = best_chain
                                .iter()
                                .find(|b| b.hash == block_hash)
                                .map(|b| b.staged_ledger_hash().clone())
                                .and_then(|ledger_hash| {
                                    let protocol_states = transition_frontier
                                        .needed_protocol_states
                                        .iter()
                                        .map(|(hash, b)| (hash.clone(), b.clone()))
                                        .chain(
                                            best_chain
                                                .iter()
                                                .take_while(|b| b.hash() != &block_hash)
                                                .map(|b| {
                                                    (
                                                        b.hash().clone(),
                                                        b.header().protocol_state.clone(),
                                                    )
                                                }),
                                        )
                                        .collect();

                                    store.service.staged_ledger_aux_and_pending_coinbase(
                                        ledger_hash,
                                        protocol_states,
                                    )
                                })
                                .map(P2pRpcResponse::StagedLedgerAuxAndPendingCoinbasesAtBlock);

                            store.dispatch(P2pChannelsRpcResponseSendAction {
                                peer_id: action.peer_id,
                                id: action.id,
                                response,
                            });
                        }
                        P2pRpcRequest::Snark(job_id) => {
                            let job = store.state().snark_pool.get(&job_id);
                            let response = job
                                .and_then(|job| job.snark.as_ref())
                                .map(|snark| snark.work.clone())
                                .map(P2pRpcResponse::Snark);

                            store.dispatch(P2pChannelsRpcResponseSendAction {
                                peer_id: action.peer_id,
                                id: action.id,
                                response,
                            });
                        }
                        P2pRpcRequest::InitialPeers => {
                            let peers = store
                                .state()
                                .p2p
                                .peers
                                .iter()
                                .filter_map(|(_, state)| {
                                    state.dial_opts.as_ref()?.try_into_mina_rpc()
                                })
                                .collect();
                            let response = Some(P2pRpcResponse::InitialPeers(peers));

                            store.dispatch(P2pChannelsRpcResponseSendAction {
                                peer_id: action.peer_id,
                                id: action.id,
                                response,
                            });
                        }
                    }
                }
                P2pChannelsRpcAction::ResponseSend(action) => {
                    action.effects(&meta, store);
                }
            },
        },
        P2pAction::Peer(action) => match action {
            P2pPeerAction::Ready(action) => {
                action.effects(&meta, store);
            }
            P2pPeerAction::BestTipUpdate(action) => {
                store.dispatch(ConsensusBlockReceivedAction {
                    hash: action.best_tip.hash,
                    block: action.best_tip.block,
                    chain_proof: None,
                });
                store.dispatch(TransitionFrontierSyncLedgerSnarkedPeersQueryAction {});
                store.dispatch(TransitionFrontierSyncLedgerStagedPartsPeerFetchInitAction {});
                store.dispatch(TransitionFrontierSyncBlocksPeersQueryAction {});
            }
        },
    }
}
