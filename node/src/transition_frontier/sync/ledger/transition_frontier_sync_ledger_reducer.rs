use crate::Substate;

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
    pub fn reducer(
        mut state: crate::Substate<Self>,
        action: TransitionFrontierSyncLedgerActionWithMetaRef<'_>,
    ) {
        let (action, meta) = action.split();
        let state_mut = &mut *state;
        match action {
            TransitionFrontierSyncLedgerAction::Init => {}
            TransitionFrontierSyncLedgerAction::Snarked(action) => {
                if let TransitionFrontierSyncLedgerSnarkedAction::Pending = action {
                    let Self::Init { target, .. } = state_mut else {
                        return;
                    };
                    let s = TransitionFrontierSyncLedgerSnarkedState::pending(
                        meta.time(),
                        target.clone(),
                    );
                    *state_mut = Self::Snarked(s);
                } else {
                    if state_mut.snarked().is_none() {
                        return;
                    };
                    TransitionFrontierSyncLedgerSnarkedState::reducer(
                        Substate::from_compatible_substate(state),
                        meta.with_action(action),
                    );
                }
            }
            TransitionFrontierSyncLedgerAction::Staged(
                TransitionFrontierSyncLedgerStagedAction::PartsFetchPending,
            ) => {
                let Self::Snarked(TransitionFrontierSyncLedgerSnarkedState::Success {
                    target, ..
                }) = state_mut
                else {
                    return;
                };
                let s = TransitionFrontierSyncLedgerStagedState::pending(
                    meta.time(),
                    target.clone().with_staged().unwrap(),
                );
                *state_mut = Self::Staged(s);
            }
            TransitionFrontierSyncLedgerAction::Staged(action) => match state_mut {
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
                    *state_mut = Self::Staged(s);
                }
                Self::Staged(_) => TransitionFrontierSyncLedgerStagedState::reducer(
                    Substate::from_compatible_substate(state),
                    meta.with_action(action),
                ),
                _ => (),
            },
            TransitionFrontierSyncLedgerAction::Success => {
                match state_mut {
                    Self::Staged(TransitionFrontierSyncLedgerStagedState::Success {
                        target,
                        needed_protocol_states,
                        ..
                    }) => {
                        *state_mut = Self::Success {
                            time: meta.time(),
                            target: target.clone().into(),
                            needed_protocol_states: std::mem::take(needed_protocol_states),
                        };
                    }
                    Self::Snarked(TransitionFrontierSyncLedgerSnarkedState::Success {
                        target,
                        ..
                    }) => {
                        *state_mut = Self::Success {
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
