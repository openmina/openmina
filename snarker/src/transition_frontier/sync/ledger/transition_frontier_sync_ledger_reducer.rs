use super::snarked::{
    TransitionFrontierSyncLedgerSnarkedAction, TransitionFrontierSyncLedgerSnarkedState,
};
use super::staged::{
    TransitionFrontierSyncLedgerStagedAction, TransitionFrontierSyncLedgerStagedState,
};
use super::{
    TransitionFrontierSyncLedgerAction, TransitionFrontierSyncLedgerActionWithMetaRef,
    TransitionFrontierSyncLedgerState,
};

impl TransitionFrontierSyncLedgerState {
    pub fn reducer(&mut self, action: TransitionFrontierSyncLedgerActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            TransitionFrontierSyncLedgerAction::Init(_) => {}
            TransitionFrontierSyncLedgerAction::Snarked(action) => {
                if let TransitionFrontierSyncLedgerSnarkedAction::Pending(_) = action {
                    let Self::Init { block, .. } = self else { return };
                    let s = TransitionFrontierSyncLedgerSnarkedState::pending(
                        meta.time(),
                        block.clone(),
                    );
                    *self = Self::Snarked(s);
                } else {
                    let Self::Snarked(state) = self else { return };
                    state.reducer(meta.with_action(action));
                }
            }
            TransitionFrontierSyncLedgerAction::Staged(action) => {
                if let TransitionFrontierSyncLedgerStagedAction::PartsFetchPending(_) = action {
                    let Self::Snarked(TransitionFrontierSyncLedgerSnarkedState::Success { block, .. }) = self else { return };
                    let s = TransitionFrontierSyncLedgerStagedState::pending(
                        meta.time(),
                        block.clone(),
                    );
                    *self = Self::Staged(s);
                } else {
                    let Self::Staged(state) = self else { return };
                    state.reducer(meta.with_action(action));
                }
            }
            TransitionFrontierSyncLedgerAction::Success(_) => {
                let Self::Staged(TransitionFrontierSyncLedgerStagedState::Success { block, .. }) = self else { return };

                *self = Self::Success {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
        }
    }
}
