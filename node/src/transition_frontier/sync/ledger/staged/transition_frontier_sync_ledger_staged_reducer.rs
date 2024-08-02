use ledger::scan_state::protocol_state::MinaHash;
use mina_p2p_messages::{list::List, v2};
use p2p::channels::rpc::{P2pChannelsRpcAction, P2pRpcRequest};

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
                let Some(staged_ledger) =
                    None.or_else(|| global_state.transition_frontier.sync.ledger()?.staged())
                else {
                    return;
                };
                let Some(p2p) = global_state.p2p.ready() else {
                    return;
                };
                let block_hash = staged_ledger.target().staged.block_hash.clone();

                let ready_peers = staged_ledger
                    .filter_available_peers(p2p.ready_rpc_peers_iter())
                    .collect::<Vec<_>>();

                for (peer_id, rpc_id) in ready_peers {
                    // TODO(binier): maybe
                    // Enabling condition is true if the peer exists and is able to handle this request
                    if dispatcher.push_if_enabled(
                        P2pChannelsRpcAction::RequestSend {
                            peer_id,
                            id: rpc_id,
                            request: Box::new(
                                P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(
                                    block_hash.clone(),
                                ),
                            ),
                        },
                        global_state,
                        meta.time(),
                    ) {
                        dispatcher.push(
                            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchPending {
                                peer_id,
                                rpc_id,
                            },
                        );
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

                if dispatcher.push_if_enabled(
                    LedgerWriteAction::Init {
                        request: LedgerWriteRequest::StagedLedgerReconstruct {
                            snarked_ledger_hash,
                            parts,
                        },
                    },
                    global_state,
                    meta.time(),
                ) {
                    dispatcher.push(TransitionFrontierSyncLedgerStagedAction::ReconstructPending);
                }
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
                        .map(|block| (v2::StateHash::from_fp(MinaHash::hash(block)), block.clone()))
                        .collect(),
                };
            }
        }
    }
}
