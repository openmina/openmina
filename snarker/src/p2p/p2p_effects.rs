use mina_p2p_messages::v2::{MinaLedgerSyncLedgerAnswerStableV2, StateHash};
use shared::block::BlockWithHash;

use crate::consensus::ConsensusBlockReceivedAction;
use crate::job_commitment::JobCommitmentAddAction;
use crate::p2p::channels::rpc::{P2pChannelsRpcRequestSendAction, P2pRpcRequest};
use crate::p2p::disconnection::P2pDisconnectionInitAction;
use crate::p2p::peer::P2pPeerAction;
use crate::rpc::{
    RpcP2pConnectionIncomingErrorAction, RpcP2pConnectionIncomingRespondAction,
    RpcP2pConnectionIncomingSuccessAction, RpcP2pConnectionOutgoingErrorAction,
    RpcP2pConnectionOutgoingSuccessAction,
};
use crate::transition_frontier::sync::ledger::{
    PeerLedgerQueryError, PeerLedgerQueryResponse,
    TransitionFrontierSyncLedgerSnarkedPeerQueryErrorAction,
    TransitionFrontierSyncLedgerSnarkedPeerQuerySuccessAction,
    TransitionFrontierSyncLedgerSnarkedPeersQueryAction,
    TransitionFrontierSyncLedgerStagedPartsFetchErrorAction,
    TransitionFrontierSyncLedgerStagedPartsFetchInitAction,
    TransitionFrontierSyncLedgerStagedPartsFetchSuccessAction,
};
use crate::transition_frontier::{
    TransitionFrontierSyncBestTipUpdateAction, TransitionFrontierSyncBlocksPeerQueryErrorAction,
    TransitionFrontierSyncBlocksPeerQuerySuccessAction,
    TransitionFrontierSyncBlocksPeersQueryAction, TransitionFrontierSyncInitAction,
};
use crate::watched_accounts::{
    WatchedAccountLedgerInitialState, WatchedAccountsLedgerInitialStateGetError,
    WatchedAccountsLedgerInitialStateGetErrorAction,
};
use crate::{Service, Store};

use super::channels::best_tip::{P2pChannelsBestTipAction, P2pChannelsBestTipResponseSendAction};
use super::channels::rpc::{P2pChannelsRpcAction, P2pRpcResponse};
use super::channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentAction;
use super::channels::P2pChannelsAction;
use super::connection::incoming::{
    P2pConnectionIncomingAction, P2pConnectionIncomingAnswerSendSuccessAction,
};
use super::connection::outgoing::P2pConnectionOutgoingAction;
use super::connection::{P2pConnectionAction, P2pConnectionResponse};
use super::disconnection::P2pDisconnectionAction;
use super::{P2pAction, P2pActionWithMeta};

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
            },
        },
        P2pAction::Disconnection(action) => match action {
            P2pDisconnectionAction::Init(action) => action.effects(&meta, store),
            P2pDisconnectionAction::Finish(action) => {
                if let Some(s) = store.state().transition_frontier.sync.root_ledger() {
                    let rpc_ids = s
                        .snarked_ledger_peer_query_pending_rpc_ids(&action.peer_id)
                        .collect::<Vec<_>>();
                    let staged_ledger_parts_fetch_rpc_id =
                        s.staged_ledger_parts_fetch_rpc_id(&action.peer_id);

                    for rpc_id in rpc_ids {
                        store.dispatch(TransitionFrontierSyncLedgerSnarkedPeerQueryErrorAction {
                            peer_id: action.peer_id,
                            rpc_id,
                            error: PeerLedgerQueryError::Disconnected,
                        });
                    }

                    if let Some(rpc_id) = staged_ledger_parts_fetch_rpc_id {
                        store.dispatch(TransitionFrontierSyncLedgerStagedPartsFetchErrorAction {
                            peer_id: action.peer_id,
                            rpc_id,
                            error: PeerLedgerQueryError::Disconnected,
                        });
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
                        error: PeerLedgerQueryError::Disconnected,
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
                    store.dispatch(ConsensusBlockReceivedAction {
                        hash: action.best_tip.hash,
                        block: action.best_tip.block,
                        history: None,
                    });
                }
                P2pChannelsBestTipAction::RequestReceived(action) => {
                    if let Some(best_tip) = store.state().consensus.best_tip_block_with_hash() {
                        store.dispatch(P2pChannelsBestTipResponseSendAction {
                            peer_id: action.peer_id,
                            best_tip,
                        });
                    }
                }
                P2pChannelsBestTipAction::ResponseSend(action) => {
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
                    store.dispatch(JobCommitmentAddAction {
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
                    store.dispatch(TransitionFrontierSyncLedgerStagedPartsFetchInitAction {});
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
                    store.dispatch(TransitionFrontierSyncLedgerStagedPartsFetchErrorAction {
                        peer_id: action.peer_id,
                        rpc_id: action.id,
                        error: PeerLedgerQueryError::Timeout,
                    });
                    store.dispatch(TransitionFrontierSyncBlocksPeerQueryErrorAction {
                        peer_id: action.peer_id,
                        rpc_id: action.id,
                        error: PeerLedgerQueryError::Timeout,
                    });
                    store.dispatch(P2pDisconnectionInitAction {
                        peer_id: action.peer_id,
                    });
                }
                P2pChannelsRpcAction::ResponseReceived(action) => {
                    action.effects(&meta, store);
                    match action.response.as_ref() {
                        None => {
                            // TODO(binier): better handling
                            store.dispatch(P2pDisconnectionInitAction {
                                peer_id: action.peer_id,
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
                                    shared::warn!(meta.time();
                                        kind = "P2pRpcBestTipHashMismatch",
                                        response = serde_json::to_string(resp).ok(),
                                        expected_hash = expected_hash.to_string(),
                                        calculated_hash = pred_hash.to_string());
                                    return;
                                }
                            }
                            if !store.state().transition_frontier.sync.is_pending()
                                && !store.state().transition_frontier.sync.is_synced()
                            {
                                store.dispatch(TransitionFrontierSyncInitAction {
                                    best_tip,
                                    root_block,
                                    blocks_inbetween: hashes,
                                });
                            } else {
                                store.dispatch(TransitionFrontierSyncBestTipUpdateAction {
                                    best_tip,
                                    root_block,
                                    blocks_inbetween: hashes,
                                });
                            }
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
                                TransitionFrontierSyncLedgerStagedPartsFetchSuccessAction {
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
                    }
                    store.dispatch(TransitionFrontierSyncLedgerSnarkedPeersQueryAction {});
                    store.dispatch(TransitionFrontierSyncLedgerStagedPartsFetchInitAction {});
                    store.dispatch(TransitionFrontierSyncBlocksPeersQueryAction {});
                }
                P2pChannelsRpcAction::RequestReceived(_action) => {
                    // TODO(binier): handle incoming rpc requests.
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
                    history: None,
                });
                store.dispatch(TransitionFrontierSyncLedgerSnarkedPeersQueryAction {});
                store.dispatch(TransitionFrontierSyncLedgerStagedPartsFetchInitAction {});
                store.dispatch(TransitionFrontierSyncBlocksPeersQueryAction {});
            }
        },
    }
}
