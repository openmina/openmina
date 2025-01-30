use crate::Substate;

use super::{
    read::LedgerReadState,
    write::{LedgerWriteAction, LedgerWriteResponse, LedgerWriteState},
    LedgerAction, LedgerActionWithMetaRef, LedgerState,
};

impl LedgerState {
    pub fn reducer(mut state_context: Substate<Self>, action: LedgerActionWithMetaRef<'_>) {
        let (action, meta) = action.split();

        match action {
            LedgerAction::Write(action) => {
                if let LedgerWriteAction::Success {
                    response: LedgerWriteResponse::Commit { result, .. },
                } = action
                {
                    if let Ok(state) = state_context.get_substate_mut() {
                        state.alive_masks = result.alive_masks;
                    }
                }
                LedgerWriteState::reducer(
                    Substate::from_compatible_substate(state_context),
                    meta.with_action(action),
                )
            }
            LedgerAction::Read(action) => LedgerReadState::reducer(
                Substate::from_compatible_substate(state_context),
                meta.with_action(action),
            ),
        }
    }
}
