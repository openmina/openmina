use std::collections::VecDeque;

use mina_p2p_messages::gossip::GossipNetMessageV2;
use mina_p2p_messages::v2::{MinaLedgerSyncLedgerAnswerStableV2, StateHash};
use p2p::disconnection::P2pDisconnectionInitAction;

use crate::consensus::{ConsensusBestTipHistoryUpdateAction, ConsensusBlockReceivedAction};
use crate::p2p::disconnection::P2pDisconnectionAction;
use crate::p2p::rpc::outgoing::{
    P2pRpcOutgoingStatus, P2pRpcRequestor, P2pRpcRequestorWatchedAccount,
};
use crate::p2p::rpc::P2pRpcResponse;
use crate::rpc::{RpcP2pConnectionOutgoingErrorAction, RpcP2pConnectionOutgoingSuccessAction};
use crate::watched_accounts::{
    WatchedAccountLedgerInitialState, WatchedAccountsBlockLedgerQuerySuccessAction,
    WatchedAccountsLedgerInitialStateGetError, WatchedAccountsLedgerInitialStateGetErrorAction,
    WatchedAccountsLedgerInitialStateGetSuccessAction,
};
use crate::{Service, Store};

use super::connection::outgoing::P2pConnectionOutgoingAction;
use super::connection::P2pConnectionAction;
use super::pubsub::P2pPubsubAction;
use super::rpc::outgoing::P2pRpcOutgoingAction;
use super::rpc::P2pRpcAction;
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
                P2pConnectionOutgoingAction::Pending(_) => {
                    // action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Error(action) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_outgoing_rpc_id(&action.peer_id) {
                        store.dispatch(RpcP2pConnectionOutgoingErrorAction {
                            rpc_id,
                            error: action.error.clone(),
                        });
                    }

                    // action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Success(action) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_outgoing_rpc_id(&action.peer_id) {
                        store.dispatch(RpcP2pConnectionOutgoingSuccessAction { rpc_id });
                    }
                    action.effects(&meta, store);
                }
            },
        },
        P2pAction::Disconnection(action) => {
            match action {
                P2pDisconnectionAction::Init(action) => action.effects(&meta, store),
                P2pDisconnectionAction::Finish(action) => {
                    let actions = store.state().watched_accounts.iter()
                    .filter_map(|(pub_key, a)| match &a.initial_state {
                        WatchedAccountLedgerInitialState::Pending { peer_id, .. } => {
                            if peer_id == &action.peer_id {
                                Some(WatchedAccountsLedgerInitialStateGetErrorAction {
                                    pub_key: pub_key.clone(),
                                    error: WatchedAccountsLedgerInitialStateGetError::PeerDisconnected,
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
            }
        }
        P2pAction::PeerReady(action) => {
            action.effects(&meta, store);
        }
        P2pAction::Pubsub(action) => match action {
            P2pPubsubAction::MessagePublish(action) => {
                action.effects(&meta, store);
            }
            P2pPubsubAction::BytesPublish(action) => {
                let rpc_id = action.rpc_id;
                action.effects(&meta, store);
                if let Some(rpc_id) = rpc_id {
                    // TODO(binier)
                    let _ = store.service.respond_p2p_publish(rpc_id, Ok(()));
                }
            }
            P2pPubsubAction::BytesReceived(action) => {
                action.effects(&meta, store);
            }
            P2pPubsubAction::MessageReceived(action) => match action.message {
                GossipNetMessageV2::NewState(block) => {
                    store.dispatch(ConsensusBlockReceivedAction {
                        hash: block.hash(),
                        block: block.into(),
                        history: None,
                    });
                }
                _ => {}
            },
        },
        P2pAction::Rpc(action) => match action {
            P2pRpcAction::Outgoing(action) => match action {
                P2pRpcOutgoingAction::Init(action) => {
                    action.effects(&meta, store);
                }
                P2pRpcOutgoingAction::Pending(_) => {}
                P2pRpcOutgoingAction::Received(action) => {
                    action.effects(&meta, store);
                }
                P2pRpcOutgoingAction::Error(action) => {
                    let action = store.state().watched_accounts.iter()
                        .find_map(|(pub_key, a)| match &a.initial_state {
                            WatchedAccountLedgerInitialState::Pending { peer_id, p2p_rpc_id, .. } => {
                                if peer_id == &action.peer_id && p2p_rpc_id == &action.rpc_id {
                                    Some(WatchedAccountsLedgerInitialStateGetErrorAction {
                                        pub_key: pub_key.clone(),
                                        error: WatchedAccountsLedgerInitialStateGetError::TransportError(action.error.clone()),
                                    })
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        });
                    if let Some(action) = action {
                        store.dispatch(action);
                    }
                }
                P2pRpcOutgoingAction::Success(action) => {
                    let Some(peer) = store.state().p2p.peers.get(&action.peer_id) else {
                        return;
                    };
                    let Some(peer) = peer.status.as_ready() else {
                        return;
                    };

                    let (_, resp, requestor) = match peer.rpc.outgoing.get(action.rpc_id) {
                        Some(P2pRpcOutgoingStatus::Success {
                            request,
                            response,
                            requestor,
                            ..
                        }) => (request, response, requestor),
                        _ => return,
                    };
                    match resp {
                        P2pRpcResponse::BestTipGet(None) => {
                            shared::log::warn!(
                                meta.time();
                                kind = "PeerNotSynced",
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                            );
                            store.dispatch(P2pDisconnectionInitAction {
                                peer_id: action.peer_id,
                            });
                        }
                        P2pRpcResponse::BestTipGet(Some(resp)) => {
                            let block = resp.data.clone();
                            // reconstruct hashes
                            let (body_hashes, oldest_block) = &resp.proof;
                            let history = {
                                let mut v = VecDeque::with_capacity(body_hashes.len());
                                v.push_front(oldest_block.hash());
                                v
                            };
                            let history = body_hashes
                                .iter()
                                .take(body_hashes.len().max(1) - 1)
                                .fold(history, |mut history, body_hash| {
                                    let pred_hash = history.front().unwrap();
                                    let hash = StateHash::from_hashes(pred_hash, body_hash);
                                    history.push_front(hash);
                                    history
                                });

                            let expected_hash = block.hash();
                            if let Some((pred_hash, body_hash)) =
                                history.front().zip(body_hashes.last())
                            {
                                let hash = StateHash::from_hashes(pred_hash, body_hash);
                                if hash != expected_hash {
                                    shared::warn!(meta.time();
                                        kind = "P2pRpcBestTipHashMismatch",
                                        response = serde_json::to_string(resp).ok(),
                                        expected_hash = expected_hash.to_string(),
                                        calculated_hash = hash.to_string(),
                                        calculated_history = serde_json::to_string(&history).ok());
                                    return;
                                }
                            }

                            if Some(&expected_hash) == store.state().consensus.best_tip.as_ref() {
                                if history.is_empty() {
                                    return;
                                }
                                store.dispatch(ConsensusBestTipHistoryUpdateAction {
                                    tip_hash: expected_hash,
                                    history: history.into(),
                                });
                            } else {
                                store.dispatch(ConsensusBlockReceivedAction {
                                    hash: expected_hash,
                                    block: block.into(),
                                    history: Some(history.into())
                                        .filter(|v: &Vec<_>| !v.is_empty()),
                                });
                            }
                        }
                        P2pRpcResponse::LedgerQuery(resp) => match &resp.0 {
                            Err(err) => {
                                let action = store.state().watched_accounts.iter()
                                    .find_map(|(pub_key, a)| match &a.initial_state {
                                        WatchedAccountLedgerInitialState::Pending { peer_id, p2p_rpc_id, .. } => {
                                            if peer_id == &action.peer_id && p2p_rpc_id == &action.rpc_id {
                                                Some(WatchedAccountsLedgerInitialStateGetErrorAction {
                                                    pub_key: pub_key.clone(),
                                                    error: WatchedAccountsLedgerInitialStateGetError::P2pRpcError(err.clone()),
                                                })
                                            } else {
                                                None
                                            }
                                        }
                                        _ => None,
                                    });
                                if let Some(action) = action {
                                    store.dispatch(action);
                                }
                            }
                            Ok(MinaLedgerSyncLedgerAnswerStableV2::AccountWithPath(result)) => {
                                match requestor.clone() {
                                    P2pRpcRequestor::WatchedAccount(
                                        P2pRpcRequestorWatchedAccount::BlockLedgerGet(
                                            pub_key,
                                            block_hash,
                                        ),
                                    ) => {
                                        if let Some((account, _)) = result {
                                            store.dispatch(
                                                WatchedAccountsBlockLedgerQuerySuccessAction {
                                                    pub_key,
                                                    block_hash,
                                                    ledger_account: account.clone(),
                                                },
                                            );
                                        }
                                    }
                                    P2pRpcRequestor::WatchedAccount(
                                        P2pRpcRequestorWatchedAccount::LedgerInitialGet(pub_key),
                                    ) => {
                                        store.dispatch(
                                            WatchedAccountsLedgerInitialStateGetSuccessAction {
                                                pub_key: pub_key.clone(),
                                                data: result.as_ref().map(|v| v.0.clone()),
                                            },
                                        );
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                    action.effects(&meta, store);
                }
                P2pRpcOutgoingAction::Finish(_) => {}
            },
        },
    }
}
