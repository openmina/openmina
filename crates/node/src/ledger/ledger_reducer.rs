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
                        if result.alive_masks > 294 {
                            // TODO(binier): should be a bug condition, but can't be
                            // because we get false positive during testing, since
                            // multiple nodes/ledger run in the same process.
                            mina_core::log::warn!(
                                meta.time();
                                "ledger mask leak: more than 294 ledger masks ({}) detected!",
                                result.alive_masks
                            );
                        }
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
