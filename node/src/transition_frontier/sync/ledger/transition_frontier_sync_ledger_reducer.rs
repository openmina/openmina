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
        mut state_context: crate::Substate<Self>,
        action: TransitionFrontierSyncLedgerActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else {
            // TODO: log or propagate
            return;
        };
        let (action, meta) = action.split();

        match action {
            TransitionFrontierSyncLedgerAction::Init => {}
            TransitionFrontierSyncLedgerAction::Snarked(action) => {
                if let TransitionFrontierSyncLedgerSnarkedAction::Pending = action {
                    let Self::Init { target, .. } = state else {
                        return;
                    };
                    let s = TransitionFrontierSyncLedgerSnarkedState::pending(
                        meta.time(),
                        target.clone(),
                    );
                    *state = Self::Snarked(s);
                } else {
                    if state.snarked().is_none() {
                        return;
                    };
                    TransitionFrontierSyncLedgerSnarkedState::reducer(
                        Substate::from_compatible_substate(state_context),
                        meta.with_action(action),
                    );
                }
            }
            TransitionFrontierSyncLedgerAction::Staged(
                TransitionFrontierSyncLedgerStagedAction::PartsFetchPending,
            ) => {
                let Self::Snarked(TransitionFrontierSyncLedgerSnarkedState::Success {
                    target, ..
                }) = state
                else {
                    return;
                };
                let s = TransitionFrontierSyncLedgerStagedState::pending(
                    meta.time(),
                    target.clone().with_staged().unwrap(),
                );
                *state = Self::Staged(s);
            }
            TransitionFrontierSyncLedgerAction::Staged(action) => match state {
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
                    *state = Self::Staged(s);
                }
                Self::Staged(_) => TransitionFrontierSyncLedgerStagedState::reducer(
                    Substate::from_compatible_substate(state_context),
                    meta.with_action(action),
                ),
                _ => (),
            },
            TransitionFrontierSyncLedgerAction::Success => {
                match state {
                    Self::Staged(TransitionFrontierSyncLedgerStagedState::Success {
                        target,
                        needed_protocol_states,
                        ..
                    }) => {
                        *state = Self::Success {
                            time: meta.time(),
                            target: target.clone().into(),
                            needed_protocol_states: std::mem::take(needed_protocol_states),
                        };
                    }
                    Self::Snarked(TransitionFrontierSyncLedgerSnarkedState::Success {
                        target,
                        ..
                    }) => {
                        *state = Self::Success {
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
