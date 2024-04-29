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
            TransitionFrontierSyncLedgerAction::Init => {}
            TransitionFrontierSyncLedgerAction::Snarked(action) => {
                if let TransitionFrontierSyncLedgerSnarkedAction::Pending = action {
                    let Self::Init { target, .. } = self else {
                        return;
                    };
                    let s = TransitionFrontierSyncLedgerSnarkedState::pending(
                        meta.time(),
                        target.clone(),
                    );
                    *self = Self::Snarked(s);
                } else {
                    let Self::Snarked(state) = self else { return };
                    state.reducer(meta.with_action(action));
                }
            }
            TransitionFrontierSyncLedgerAction::Staged(
                TransitionFrontierSyncLedgerStagedAction::PartsFetchPending,
            ) => {
                let Self::Snarked(TransitionFrontierSyncLedgerSnarkedState::Success {
                    target, ..
                }) = self
                else {
                    return;
                };
                let s = TransitionFrontierSyncLedgerStagedState::pending(
                    meta.time(),
                    target.clone().with_staged().unwrap(),
                );
                *self = Self::Staged(s);
            }
            TransitionFrontierSyncLedgerAction::Staged(action) => match self {
                Self::Snarked(TransitionFrontierSyncLedgerSnarkedState::Success {
                    target, ..
                }) if matches!(
                    action,
                    TransitionFrontierSyncLedgerStagedAction::ReconstructEmpty
                ) =>
                {
                    let s = TransitionFrontierSyncLedgerStagedState::ReconstructEmpty {
                        time: meta.time(),
                        target: target.clone().with_staged().unwrap(),
                    };
                    *self = Self::Staged(s);
                }
                Self::Staged(state) => state.reducer(meta.with_action(action)),
                _ => return,
            },
            TransitionFrontierSyncLedgerAction::Success => {
                match self {
                    Self::Staged(TransitionFrontierSyncLedgerStagedState::Success {
                        target,
                        needed_protocol_states,
                        ..
                    }) => {
                        *self = Self::Success {
                            time: meta.time(),
                            target: target.clone().into(),
                            needed_protocol_states: std::mem::take(needed_protocol_states),
                        };
                    }
                    Self::Snarked(TransitionFrontierSyncLedgerSnarkedState::Success {
                        target,
                        ..
                    }) => {
                        *self = Self::Success {
                            time: meta.time(),
                            target: target.clone(),
                            // No additional protocol states needed for snarked ledger.
                            needed_protocol_states: Default::default(),
                        };
                    }
                    _ => {}
                }
            }
        }
    }
}
