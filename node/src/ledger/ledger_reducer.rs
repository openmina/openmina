use crate::Substate;

use super::{
    read::LedgerReadState, write::LedgerWriteState, LedgerAction, LedgerActionWithMetaRef,
    LedgerState,
};

impl LedgerState {
    pub fn reducer(state_context: Substate<Self>, action: LedgerActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            LedgerAction::Write(action) => LedgerWriteState::reducer(
                Substate::from_compatible_substate(state_context),
                meta.with_action(action),
            ),
            LedgerAction::Read(action) => LedgerReadState::reducer(
                Substate::from_compatible_substate(state_context),
                meta.with_action(action),
            ),
        }
    }
}
