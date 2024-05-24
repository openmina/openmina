use ledger::scan_state::protocol_state::MinaHash;
use mina_p2p_messages::{list::List, v2};

use super::{
    PeerStagedLedgerPartsFetchState, StagedLedgerAuxAndPendingCoinbasesValidated,
    TransitionFrontierSyncLedgerStagedAction, TransitionFrontierSyncLedgerStagedActionWithMetaRef,
    TransitionFrontierSyncLedgerStagedState,
};

impl TransitionFrontierSyncLedgerStagedState {
    pub fn reducer(&mut self, action: TransitionFrontierSyncLedgerStagedActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            TransitionFrontierSyncLedgerStagedAction::PartsFetchPending => {
                // handled in parent.
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit => {}
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchPending { peer_id, rpc_id } => {
                let Self::PartsFetchPending { attempts, .. } = self else {
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
                let Self::PartsFetchPending { attempts, .. } = self else {
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
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchSuccess {
                peer_id,
                parts,
                ..
            } => {
                let Self::PartsFetchPending {
                    target, attempts, ..
                } = self
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
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerInvalid { sender, .. }
            | TransitionFrontierSyncLedgerStagedAction::PartsPeerValid { sender, .. } => {
                let Self::PartsFetchPending { attempts, .. } = self else {
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
            }
            TransitionFrontierSyncLedgerStagedAction::PartsFetchSuccess { sender } => {
                let Self::PartsFetchPending {
                    target, attempts, ..
                } = self
                else {
                    return;
                };
                let Some(attempt) = attempts.get_mut(sender) else {
                    return;
                };
                let PeerStagedLedgerPartsFetchState::Valid { parts, .. } = attempt else {
                    return;
                };
                *self = Self::PartsFetchSuccess {
                    time: meta.time(),
                    target: target.clone(),
                    parts: parts.clone(),
                };
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructEmpty => {
                // handled in parent.
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructInit => {}
            TransitionFrontierSyncLedgerStagedAction::ReconstructPending => {
                let Some((target, parts)) = self.target_with_parts() else {
                    return;
                };
                *self = Self::ReconstructPending {
                    time: meta.time(),
                    target: target.clone(),
                    parts: parts.cloned(),
                }
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructError { error } => {
                let Self::ReconstructPending { target, parts, .. } = self else {
                    return;
                };
                *self = Self::ReconstructError {
                    time: meta.time(),
                    target: target.clone(),
                    parts: parts.clone(),
                    error: error.clone(),
                };
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructSuccess { .. } => {
                let Self::ReconstructPending { target, parts, .. } = self else {
                    return;
                };
                *self = Self::ReconstructSuccess {
                    time: meta.time(),
                    target: target.clone(),
                    parts: parts.clone(),
                };
            }
            TransitionFrontierSyncLedgerStagedAction::Success => {
                let Self::ReconstructSuccess { target, parts, .. } = self else {
                    return;
                };

                *self = Self::Success {
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
