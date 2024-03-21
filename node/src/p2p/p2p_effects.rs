use mina_p2p_messages::v2::{MinaLedgerSyncLedgerAnswerStableV2, StateHash};
use openmina_core::block::BlockWithHash;

use crate::consensus::ConsensusAction;
use crate::rpc::RpcAction;
use crate::snark_pool::candidate::SnarkPoolCandidateAction;
use crate::snark_pool::SnarkPoolAction;
use crate::transition_frontier::sync::ledger::snarked::{
    PeerLedgerQueryError, PeerLedgerQueryResponse, TransitionFrontierSyncLedgerSnarkedAction,
};
use crate::transition_frontier::sync::ledger::staged::{
    PeerStagedLedgerPartsFetchError, TransitionFrontierSyncLedgerStagedAction,
};
use crate::transition_frontier::sync::{PeerBlockFetchError, TransitionFrontierSyncAction};
use crate::watched_accounts::{
    WatchedAccountLedgerInitialState, WatchedAccountsAction,
    WatchedAccountsLedgerInitialStateGetError,
};
use crate::{Service, Store};

use super::channels::best_tip::P2pChannelsBestTipAction;
use super::channels::rpc::{BestTipWithProof, P2pChannelsRpcAction, P2pRpcRequest, P2pRpcResponse};
use super::channels::snark::P2pChannelsSnarkAction;
use super::channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentAction;
use super::channels::P2pChannelsAction;
use super::connection::incoming::P2pConnectionIncomingAction;
use super::connection::outgoing::P2pConnectionOutgoingAction;
use super::connection::{P2pConnectionAction, P2pConnectionResponse};
use super::disconnection::{P2pDisconnectionAction, P2pDisconnectionReason};
use super::discovery::P2pDiscoveryAction;
use super::peer::P2pPeerAction;
use super::{P2pAction, P2pActionWithMeta};

pub fn node_p2p_effects<S: Service>(store: &mut Store<S>, action: P2pActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        P2pAction::Listen(action) => {
            action.effects(&meta, store);
        }
        P2pAction::Connection(action) => match action {
            P2pConnectionAction::Outgoing(action) => {
                match action {
                    P2pConnectionOutgoingAction::Error {
                        ref peer_id,
                        ref error,
                    } => {
                        let p2p = &store.state().p2p;
                        if let Some(rpc_id) = p2p.peer_connection_rpc_id(peer_id) {
                            store.dispatch(RpcAction::P2pConnectionOutgoingError {
                                rpc_id,
                                error: error.clone(),
                            });
                        }
                    }
                    P2pConnectionOutgoingAction::Success { ref peer_id } => {
                        let p2p = &store.state().p2p;
                        if let Some(rpc_id) = p2p.peer_connection_rpc_id(peer_id) {
                            store.dispatch(RpcAction::P2pConnectionOutgoingSuccess { rpc_id });
                        }
                    }
                    _ => {}
                }
                action.effects(&meta, store);
            }
            P2pConnectionAction::Incoming(action) => {
                match &action {
                    P2pConnectionIncomingAction::AnswerReady { peer_id, answer } => {
                        let p2p = &store.state().p2p;
                        if let Some(rpc_id) = p2p.peer_connection_rpc_id(peer_id) {
                            store.dispatch(RpcAction::P2pConnectionIncomingRespond {
                                rpc_id,
                                response: P2pConnectionResponse::Accepted(answer.clone()),
                            });
                            store.dispatch(P2pConnectionIncomingAction::AnswerSendSuccess {
                                peer_id: *peer_id,
                            });
                        }
                    }
                    P2pConnectionIncomingAction::Error { peer_id, error } => {
                        let p2p = &store.state().p2p;
                        if let Some(rpc_id) = p2p.peer_connection_rpc_id(peer_id) {
                            store.dispatch(RpcAction::P2pConnectionIncomingError {
                                rpc_id,
                                error: format!("{:?}", error),
                            });
                        }
                    }
                    P2pConnectionIncomingAction::Success { peer_id } => {
                        let p2p = &store.state().p2p;
                        if let Some(rpc_id) = p2p.peer_connection_rpc_id(peer_id) {
                            store.dispatch(RpcAction::P2pConnectionIncomingSuccess { rpc_id });
                        }
                    }
                    _ => {}
                }
                action.effects(&meta, store);
            }
        },
        P2pAction::Disconnection(action) => {
            action.effects(&meta, store);

            match action {
                P2pDisconnectionAction::Init { .. } => {}
                P2pDisconnectionAction::Finish { peer_id } => {
                    if let Some(s) = store.state().transition_frontier.sync.ledger() {
                        let snarked_ledger_num_accounts_rpc_id = s
                            .snarked()
                            .and_then(|s| s.peer_num_accounts_rpc_id(&peer_id));
                        let snarked_ledger_address_rpc_ids = s
                            .snarked()
                            .map(|s| s.peer_address_query_pending_rpc_ids(&peer_id).collect())
                            .unwrap_or(vec![]);
                        let staged_ledger_parts_fetch_rpc_id =
                            s.staged().and_then(|s| s.parts_fetch_rpc_id(&peer_id));

                        for rpc_id in snarked_ledger_address_rpc_ids {
                            store.dispatch(
                                TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressError {
                                    peer_id,
                                    rpc_id,
                                    error: PeerLedgerQueryError::Disconnected,
                                },
                            );
                        }

                        if let Some(rpc_id) = snarked_ledger_num_accounts_rpc_id {
                            store.dispatch(
                                TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsError {
                                    peer_id,
                                    rpc_id,
                                    error: PeerLedgerQueryError::Disconnected,
                                },
                            );
                        }

                        if let Some(rpc_id) = staged_ledger_parts_fetch_rpc_id {
                            store.dispatch(
                                TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchError {
                                    peer_id,
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
                        .blocks_fetch_from_peer_pending_rpc_ids(&peer_id)
                        .collect::<Vec<_>>();

                    for rpc_id in blocks_fetch_rpc_ids {
                        store.dispatch(TransitionFrontierSyncAction::BlocksPeerQueryError {
                            peer_id,
                            rpc_id,
                            error: PeerBlockFetchError::Disconnected,
                        });
                    }

                    let actions = store
                        .state()
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
                        .collect::<Vec<_>>();

                    for action in actions {
                        store.dispatch(action);
                    }

                    store.dispatch(SnarkPoolCandidateAction::PeerPrune { peer_id });
                }
            }
        }
        P2pAction::Discovery(action) => action.effects(&meta, store),
        P2pAction::Channels(action) => match action {
            P2pChannelsAction::MessageReceived(action) => {
                action.effects(&meta, store);
            }
            P2pChannelsAction::BestTip(action) => {
                if let P2pChannelsBestTipAction::RequestReceived { peer_id } = action {
                    if let Some(best_tip) = store.state().transition_frontier.best_tip() {
                        store.dispatch(P2pChannelsBestTipAction::ResponseSend {
                            peer_id,
                            best_tip: best_tip.clone(),
                        });
                    }
                }
                action.effects(&meta, store);
            }
            P2pChannelsAction::Snark(action) => {
                // TODO: does the order matter here? if not this clone can be removed
                action.clone().effects(&meta, store);
                match action {
                    P2pChannelsSnarkAction::Received { peer_id, snark } => {
                        store.dispatch(SnarkPoolCandidateAction::InfoReceived {
                            peer_id,
                            info: snark,
                        });
                    }
                    P2pChannelsSnarkAction::Libp2pReceived { peer_id, snark, .. } => {
                        store.dispatch(SnarkPoolCandidateAction::WorkReceived {
                            peer_id,
                            work: snark,
                        });
                    }
                    _ => {}
                }
            }
            P2pChannelsAction::SnarkJobCommitment(action) => {
                // TODO: does the order matter here? if not this clone can be removed
                action.clone().effects(&meta, store);
                if let P2pChannelsSnarkJobCommitmentAction::Received {
                    peer_id,
                    commitment,
                } = action
                {
                    store.dispatch(SnarkPoolAction::CommitmentAdd {
                        commitment,
                        sender: peer_id,
                    });
                }
            }
            P2pChannelsAction::Rpc(action) => {
                // TODO: does the order matter here? if not this clone can be removed
                action.clone().effects(&meta, store);
                match action {
                    P2pChannelsRpcAction::Ready { peer_id } => {
                        store.dispatch(P2pChannelsRpcAction::RequestSend {
                            peer_id,
                            id: 0,
                            request: P2pRpcRequest::BestTipWithProof,
                        });

                        store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
                        store
                            .dispatch(TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit);
                        store.dispatch(TransitionFrontierSyncAction::BlocksPeersQuery);
                    }
                    P2pChannelsRpcAction::Timeout { peer_id, id } => {
                        store.dispatch(
                            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressError {
                                peer_id,
                                rpc_id: id,
                                error: PeerLedgerQueryError::Timeout,
                            },
                        );
                        store.dispatch(
                            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchError {
                                peer_id,
                                rpc_id: id,
                                error: PeerStagedLedgerPartsFetchError::Timeout,
                            },
                        );
                        store.dispatch(TransitionFrontierSyncAction::BlocksPeerQueryError {
                            peer_id,
                            rpc_id: id,
                            error: PeerBlockFetchError::Timeout,
                        });
                        store.dispatch(P2pDisconnectionAction::Init {
                            peer_id,
                            reason: P2pDisconnectionReason::TransitionFrontierRpcTimeout,
                        });
                    }
                    P2pChannelsRpcAction::ResponseReceived {
                        peer_id,
                        id,
                        response,
                    } => {
                        match response.as_ref() {
                            None => {
                                store.dispatch(
                                    TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressError {
                                        peer_id,
                                        rpc_id: id,
                                        error: PeerLedgerQueryError::DataUnavailable,
                                    },
                                );
                                store.dispatch(
                                    TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchError {
                                        peer_id,
                                        rpc_id: id,
                                        error: PeerStagedLedgerPartsFetchError::DataUnavailable,
                                    },
                                );
                                store.dispatch(
                                    TransitionFrontierSyncAction::BlocksPeerQueryError {
                                        peer_id,
                                        rpc_id: id,
                                        error: PeerBlockFetchError::DataUnavailable,
                                    },
                                );
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
                                        response = serde_json::to_string(&resp).ok(),
                                        expected_hash = expected_hash.to_string(),
                                        calculated_hash = pred_hash.to_string());
                                        return;
                                    }
                                }
                                store.dispatch(ConsensusAction::BlockChainProofUpdate {
                                    hash: best_tip.hash,
                                    chain_proof: (hashes, root_block),
                                });
                            }
                            Some(P2pRpcResponse::LedgerQuery(answer)) => match answer {
                                MinaLedgerSyncLedgerAnswerStableV2::ChildHashesAre(left, right) => {
                                    store.dispatch(
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
                                    store.dispatch(
                                        TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressSuccess {
                                            peer_id,
                                            rpc_id: id,
                                            response: PeerLedgerQueryResponse::ChildAccounts(
                                                accounts.iter().cloned().collect(),
                                            ),
                                        },
                                    );
                                }
                                MinaLedgerSyncLedgerAnswerStableV2::NumAccounts(
                                    count,
                                    contents_hash,
                                ) => {
                                    store.dispatch(
                                        TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsSuccess {
                                            peer_id,
                                            rpc_id: id,
                                            response: PeerLedgerQueryResponse::NumAccounts(
                                                count.as_u64(), contents_hash.clone()
                                            ),
                                        },
                                    );
                                }
                            },
                            Some(P2pRpcResponse::StagedLedgerAuxAndPendingCoinbasesAtBlock(
                                parts,
                            )) => {
                                store.dispatch(
                                    TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchSuccess {
                                        peer_id,
                                        rpc_id: id,
                                        parts: parts.clone(),
                                    },
                                );
                            }
                            Some(P2pRpcResponse::Block(block)) => {
                                let block = BlockWithHash::new(block.clone());
                                store.dispatch(
                                    TransitionFrontierSyncAction::BlocksPeerQuerySuccess {
                                        peer_id,
                                        rpc_id: id,
                                        response: block,
                                    },
                                );
                            }
                            Some(P2pRpcResponse::Snark(snark)) => {
                                store.dispatch(SnarkPoolCandidateAction::WorkReceived {
                                    peer_id,
                                    work: snark.clone(),
                                });
                            }
                            Some(P2pRpcResponse::InitialPeers(peers)) => {
                                store.dispatch(P2pDiscoveryAction::Success {
                                    peer_id,
                                    peers: peers.clone(),
                                });
                            }
                        }
                        store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
                        store.dispatch(
                            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit {},
                        );
                        store.dispatch(TransitionFrontierSyncAction::BlocksPeersQuery);
                    }
                    P2pChannelsRpcAction::RequestReceived {
                        peer_id,
                        id,
                        request,
                    } => {
                        match request {
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
                                store.dispatch(P2pChannelsRpcAction::ResponseSend {
                                    peer_id,
                                    id,
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
                                store.dispatch(P2pChannelsRpcAction::ResponseSend {
                                    peer_id,
                                    id,
                                    response,
                                });
                            }
                            P2pRpcRequest::LedgerQuery(ledger_hash, query) => {
                                let response = store
                                    .service
                                    .answer_ledger_query(ledger_hash, query)
                                    .map(P2pRpcResponse::LedgerQuery);

                                store.dispatch(P2pChannelsRpcAction::ResponseSend {
                                    peer_id,
                                    id,
                                    response,
                                });
                            }
                            P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(
                                block_hash,
                            ) => {
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

                                store.dispatch(P2pChannelsRpcAction::ResponseSend {
                                    peer_id,
                                    id,
                                    response,
                                });
                            }
                            P2pRpcRequest::Snark(job_id) => {
                                let job = store.state().snark_pool.get(&job_id);
                                let response = job
                                    .and_then(|job| job.snark.as_ref())
                                    .map(|snark| snark.work.clone())
                                    .map(P2pRpcResponse::Snark);

                                store.dispatch(P2pChannelsRpcAction::ResponseSend {
                                    peer_id,
                                    id,
                                    response,
                                });
                            }
                            P2pRpcRequest::InitialPeers => {
                                let peers = store
                                    .state()
                                    .p2p
                                    .peers
                                    .iter()
                                    .filter_map(|(_, v)| v.dial_opts.clone())
                                    .collect();
                                let response = Some(P2pRpcResponse::InitialPeers(peers));

                                store.dispatch(P2pChannelsRpcAction::ResponseSend {
                                    peer_id,
                                    id,
                                    response,
                                });
                            }
                        }
                    }
                    P2pChannelsRpcAction::Init { .. } => {}
                    P2pChannelsRpcAction::Pending { .. } => {}
                    P2pChannelsRpcAction::RequestSend { .. } => {}
                    P2pChannelsRpcAction::ResponseSend { .. } => {}
                }
            }
        },
        P2pAction::Peer(action) => match action {
            P2pPeerAction::Ready { .. } => {
                action.effects(&meta, store);
            }
            P2pPeerAction::BestTipUpdate { best_tip, .. } => {
                store.dispatch(ConsensusAction::BlockReceived {
                    hash: best_tip.hash,
                    block: best_tip.block,
                    chain_proof: None,
                });
                store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
                store.dispatch(TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit);
                store.dispatch(TransitionFrontierSyncAction::BlocksPeersQuery);
            }
        },
        P2pAction::Network(action) => action.effects(&meta, store),
    }
}
