use super::{
    PeerStagedLedgerPartsFetchState, StagedLedgerAuxAndPendingCoinbasesValidated,
    TransitionFrontierSyncLedgerStagedAction, TransitionFrontierSyncLedgerStagedActionWithMetaRef,
    TransitionFrontierSyncLedgerStagedPartsPeerInvalidAction,
    TransitionFrontierSyncLedgerStagedPartsPeerValidAction,
    TransitionFrontierSyncLedgerStagedState,
};

impl TransitionFrontierSyncLedgerStagedState {
    pub fn reducer(&mut self, action: TransitionFrontierSyncLedgerStagedActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            TransitionFrontierSyncLedgerStagedAction::PartsFetchPending(_) => {
                // handled in parent.
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit(_) => {}
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchPending(action) => {
                let Self::PartsFetchPending { attempts, .. } = self else {
                    return;
                };
                attempts.insert(
                    action.peer_id,
                    PeerStagedLedgerPartsFetchState::Pending {
                        time: meta.time(),
                        rpc_id: action.rpc_id,
                    },
                );
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchError(action) => {
                let Self::PartsFetchPending { attempts, .. } = self else {
                    return;
                };
                let Some(attempt) = attempts.get_mut(&action.peer_id) else {
                    return;
                };
                let PeerStagedLedgerPartsFetchState::Pending { rpc_id, .. } = &attempt else {
                    return;
                };
                *attempt = PeerStagedLedgerPartsFetchState::Error {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                    error: action.error.clone(),
                };
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchSuccess(action) => {
                let Self::PartsFetchPending {
                    block, attempts, ..
                } = self
                else {
                    return;
                };
                let Some(attempt) = attempts.get_mut(&action.peer_id) else {
                    return;
                };

                let expected_hash = block.staged_ledger_hashes();
                let validated = StagedLedgerAuxAndPendingCoinbasesValidated::validate(
                    &action.parts,
                    expected_hash,
                );

                *attempt = PeerStagedLedgerPartsFetchState::Success {
                    time: meta.time(),
                    parts: validated,
                };
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerInvalid(
                TransitionFrontierSyncLedgerStagedPartsPeerInvalidAction { sender, .. },
            )
            | TransitionFrontierSyncLedgerStagedAction::PartsPeerValid(
                TransitionFrontierSyncLedgerStagedPartsPeerValidAction { sender, .. },
            ) => {
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
            TransitionFrontierSyncLedgerStagedAction::PartsFetchSuccess(action) => {
                let Self::PartsFetchPending {
                    block, attempts, ..
                } = self
                else {
                    return;
                };
                let Some(attempt) = attempts.get_mut(&action.sender) else {
                    return;
                };
                let PeerStagedLedgerPartsFetchState::Valid { parts, .. } = attempt else {
                    return;
                };
                *self = Self::PartsFetchSuccess {
                    time: meta.time(),
                    block: block.clone(),
                    parts: parts.clone(),
                };
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructEmpty(_) => {
                // handled in parent.
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructInit(_) => {}
            TransitionFrontierSyncLedgerStagedAction::ReconstructPending(_) => {
                let Some((block, parts)) = self.block_with_parts() else {
                    return;
                };
                *self = Self::ReconstructPending {
                    time: meta.time(),
                    block: block.clone(),
                    parts: parts.cloned(),
                }
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructError(action) => {
                let Self::ReconstructPending { block, parts, .. } = self else {
                    return;
                };
                *self = Self::ReconstructError {
                    time: meta.time(),
                    block: block.clone(),
                    parts: parts.clone(),
                    error: action.error.clone(),
                };
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructSuccess(_) => {
                let Self::ReconstructPending { block, parts, .. } = self else {
                    return;
                };
                *self = Self::ReconstructSuccess {
                    time: meta.time(),
                    block: block.clone(),
                    parts: parts.clone(),
                };
            }
            TransitionFrontierSyncLedgerStagedAction::Success(_) => {
                let Self::ReconstructSuccess { block, parts, .. } = self else {
                    return;
                };

                *self = Self::Success {
                    time: meta.time(),
                    block: block.clone(),
                    needed_protocol_states: parts
                        .as_ref()
                        .map(|parts| &parts.needed_blocks[..])
                        .unwrap_or(&[])
                        .iter()
                        .map(|block| (block.hash(), block.clone()))
                        .collect(),
                };
            }
        }
    }
}
