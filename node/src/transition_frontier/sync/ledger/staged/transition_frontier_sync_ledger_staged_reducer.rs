use mina_p2p_messages::{hash::MinaHash, list::List, v2};
use p2p::{
    channels::{
        rpc::{P2pChannelsRpcAction, P2pRpcId, P2pRpcRequest},
        streaming_rpc::{P2pChannelsStreamingRpcAction, P2pStreamingRpcRequest},
    },
    PeerId,
};
use rand::prelude::*;

use crate::ledger::write::{LedgerWriteAction, LedgerWriteRequest};

use super::{
    PeerStagedLedgerPartsFetchState, StagedLedgerAuxAndPendingCoinbasesValidated,
    TransitionFrontierSyncLedgerStagedAction, TransitionFrontierSyncLedgerStagedActionWithMetaRef,
    TransitionFrontierSyncLedgerStagedState,
};

impl TransitionFrontierSyncLedgerStagedState {
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: TransitionFrontierSyncLedgerStagedActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else {
            // TODO: log or propagate
            return;
        };
        let (action, meta) = action.split();

        match action {
            TransitionFrontierSyncLedgerStagedAction::PartsFetchPending => {
                // handled in parent. TODO(refactor) check this
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit);
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit => {
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let Some((p2p, target_best_tip, staged_ledger, fetch_attempts)) =
                    None.or_else(|| {
                        let staged = global_state.transition_frontier.sync.ledger()?.staged()?;
                        Some((
                            global_state.p2p.ready()?,
                            global_state.transition_frontier.sync.best_tip()?,
                            staged,
                            staged.fetch_attempts()?,
                        ))
                    })
                else {
                    return;
                };

                let mut peers =
                    p2p.ready_rpc_peers_iter()
                        .fold(Vec::new(), |mut list, (peer_id, state)| {
                            let mut order = 0u8;
                            // TODO(binier): maybe take into account fetch attempt time.
                            order = order.saturating_add(if fetch_attempts.contains_key(peer_id) {
                                5
                            } else {
                                0
                            });
                            order =
                                order.saturating_add(state.best_tip.as_ref().map_or(2, |tip| {
                                    let k = target_best_tip.constants().k.as_u32();
                                    if tip.height() > target_best_tip.height()
                                        || tip.height().abs_diff(target_best_tip.height()) > k
                                    {
                                        // can't have the block we need
                                        10
                                    } else {
                                        // has common block
                                        if IntoIterator::into_iter([
                                            tip.hash(),
                                            tip.pred_hash(),
                                            target_best_tip.hash(),
                                            target_best_tip.pred_hash(),
                                        ])
                                        .collect::<std::collections::BTreeSet<_>>()
                                        .len()
                                            < 4
                                        {
                                            0
                                        } else {
                                            1
                                        }
                                    }
                                }));

                            if list
                                .last()
                                .map_or(false, |(_, _, ord): &(_, _, u8)| *ord > order)
                            {
                                // remove less priority peers
                                list.clear();
                            }
                            list.push((peer_id, state.channels.next_local_rpc_id(), order));

                            list
                        });

                let block_hash = staged_ledger.target().staged.block_hash.clone();
                peers.shuffle(&mut global_state.pseudo_rng());

                for (&peer_id, rpc_id, _) in peers {
                    let enqueued = if p2p.is_libp2p_peer(&peer_id) {
                        // use old heavy rpc for libp2p peers.
                        dispatcher.push_if_enabled(
                            P2pChannelsRpcAction::RequestSend {
                                peer_id,
                                id: rpc_id,
                                request: Box::new(
                                    P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(
                                        block_hash.clone(),
                                    ),
                                ),
                                on_init: Some(redux::callback!(
                                    on_send_p2p_staged_ledger_parts_rpc_request(
                                        (peer_id: PeerId, rpc_id: P2pRpcId, _request: P2pRpcRequest)
                                    ) -> crate::Action {
                                        TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchPending {
                                            peer_id,
                                            rpc_id,
                                        }
                                    }
                                ))
                            },
                            global_state,
                            meta.time()
                        )
                    } else {
                        // use streaming rpc for webrtc peers.
                        dispatcher.push_if_enabled(
                            P2pChannelsStreamingRpcAction::RequestSend {
                                peer_id,
                                id: rpc_id,
                                request: Box::new(P2pStreamingRpcRequest::StagedLedgerParts(
                                    block_hash.clone(),
                                )),
                                on_init: Some(redux::callback!(
                                    on_send_streaming_p2p_staged_ledger_parts_rpc_request(
                                        (peer_id: PeerId, rpc_id: P2pRpcId, _request: P2pStreamingRpcRequest)
                                    ) -> crate::Action {
                                        TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchPending {
                                            peer_id,
                                            rpc_id,
                                        }
                                    }))
                            },
                            global_state,
                            meta.time()
                        )
                    };

                    // TODO: instead add an intermediary action for the Peer request with an enabling condition
                    // that will make sure that only one staged ledger part request
                    // is ongoing. So here we dispatch the action for all peers, but
                    // after one picks it up the rest will be filtered out.
                    if enqueued {
                        break;
                    }
                }
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchPending { peer_id, rpc_id } => {
                let Self::PartsFetchPending { attempts, .. } = state else {
                    return;
                };
                attempts.insert(
                    *peer_id,
                    PeerStagedLedgerPartsFetchState::Pending {
                        time: meta.time(),
                        rpc_id: *rpc_id,
                    },
                );
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchError {
                peer_id,
                error,
                ..
            } => {
                let Self::PartsFetchPending { attempts, .. } = state else {
                    return;
                };
                let Some(attempt) = attempts.get_mut(peer_id) else {
                    return;
                };
                let PeerStagedLedgerPartsFetchState::Pending { rpc_id, .. } = &attempt else {
                    return;
                };
                *attempt = PeerStagedLedgerPartsFetchState::Error {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                    error: error.clone(),
                };

                // Dispatch
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit);
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchSuccess {
                peer_id,
                parts,
                ..
            } => {
                let Self::PartsFetchPending {
                    target, attempts, ..
                } = state
                else {
                    return;
                };
                let Some(attempt) = attempts.get_mut(peer_id) else {
                    return;
                };

                let expected_hash = &target.staged.hashes;
                let validated =
                    StagedLedgerAuxAndPendingCoinbasesValidated::validate(parts, expected_hash);

                *attempt = PeerStagedLedgerPartsFetchState::Success {
                    time: meta.time(),
                    parts: validated,
                };

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                if !dispatcher.push_if_enabled(
                    TransitionFrontierSyncLedgerStagedAction::PartsPeerValid { sender: *peer_id },
                    global_state,
                    meta.time(),
                ) {
                    dispatcher.push(TransitionFrontierSyncLedgerStagedAction::PartsPeerInvalid {
                        sender: *peer_id,
                        parts: parts.clone(),
                    });
                }
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerInvalid { sender, .. }
            | TransitionFrontierSyncLedgerStagedAction::PartsPeerValid { sender, .. } => {
                let Self::PartsFetchPending { attempts, .. } = &mut *state else {
                    return;
                };
                let Some(attempt) = attempts.get_mut(sender) else {
                    return;
                };
                let PeerStagedLedgerPartsFetchState::Success { parts, .. } = attempt else {
                    return;
                };

                match parts {
                    StagedLedgerAuxAndPendingCoinbasesValidated::Invalid(_) => {
                        *attempt = PeerStagedLedgerPartsFetchState::Invalid { time: meta.time() };
                    }
                    StagedLedgerAuxAndPendingCoinbasesValidated::Valid(parts) => {
                        *attempt = PeerStagedLedgerPartsFetchState::Valid {
                            time: meta.time(),
                            parts: parts.clone(),
                        };
                    }
                }

                // Dispatch
                let dispatcher = state_context.into_dispatcher();
                if let TransitionFrontierSyncLedgerStagedAction::PartsPeerValid { .. } = action {
                    dispatcher.push(
                        TransitionFrontierSyncLedgerStagedAction::PartsFetchSuccess {
                            sender: *sender,
                        },
                    );
                } else {
                    dispatcher.push(TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit);
                }
            }
            TransitionFrontierSyncLedgerStagedAction::PartsFetchSuccess { sender } => {
                let Self::PartsFetchPending {
                    target, attempts, ..
                } = state
                else {
                    return;
                };
                let Some(attempt) = attempts.get_mut(sender) else {
                    return;
                };
                let PeerStagedLedgerPartsFetchState::Valid { parts, .. } = attempt else {
                    return;
                };
                *state = Self::PartsFetchSuccess {
                    time: meta.time(),
                    target: target.clone(),
                    parts: parts.clone(),
                };

                // Dispatch
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransitionFrontierSyncLedgerStagedAction::ReconstructInit);
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructEmpty => {
                // handled in parent. TODO(refactor): check this
                // Dispatch
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransitionFrontierSyncLedgerStagedAction::ReconstructInit);
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructInit => {
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let ledger_state = global_state.transition_frontier.sync.ledger();
                let Some((target, parts)) =
                    ledger_state.and_then(|s| s.staged()?.target_with_parts())
                else {
                    return;
                };
                let snarked_ledger_hash = target.snarked_ledger_hash.clone();
                let parts = parts.cloned();

                dispatcher.push(LedgerWriteAction::Init {
                    request: LedgerWriteRequest::StagedLedgerReconstruct {
                        snarked_ledger_hash,
                        parts,
                    },
                    on_init: redux::callback!(
                        on_staged_ledger_reconstruct_init(_request: LedgerWriteRequest) -> crate::Action {
                            TransitionFrontierSyncLedgerStagedAction::ReconstructPending
                        }
                    ),
                });
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructPending => {
                let Some((target, parts)) = state.target_with_parts() else {
                    return;
                };
                *state = Self::ReconstructPending {
                    time: meta.time(),
                    target: target.clone(),
                    parts: parts.cloned(),
                }
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructError { error } => {
                let Self::ReconstructPending { target, parts, .. } = state else {
                    return;
                };
                *state = Self::ReconstructError {
                    time: meta.time(),
                    target: target.clone(),
                    parts: parts.clone(),
                    error: error.clone(),
                };
                panic!("Staged ledger reconstruct failure {error}");
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructSuccess { .. } => {
                let Self::ReconstructPending { target, parts, .. } = state else {
                    return;
                };
                *state = Self::ReconstructSuccess {
                    time: meta.time(),
                    target: target.clone(),
                    parts: parts.clone(),
                };

                // Dispatch
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransitionFrontierSyncLedgerStagedAction::Success);
            }
            TransitionFrontierSyncLedgerStagedAction::Success => {
                let Self::ReconstructSuccess { target, parts, .. } = state else {
                    return;
                };

                *state = Self::Success {
                    time: meta.time(),
                    target: target.clone(),
                    needed_protocol_states: parts
                        .as_ref()
                        .map(|parts| &parts.needed_blocks)
                        .unwrap_or(&List::new())
                        .iter()
                        .filter_map(|block| {
                            Some((
                                v2::StateHash::from_fp(MinaHash::try_hash(block).ok()?),
                                block.clone(),
                            ))
                        })
                        .collect(),
                };
            }
        }
    }
}
