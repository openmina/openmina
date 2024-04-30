use serde::{Deserialize, Serialize};

use super::{read::LedgerReadAction, write::LedgerWriteAction};

pub type LedgerActionWithMeta = redux::ActionWithMeta<LedgerAction>;
pub type LedgerActionWithMetaRef<'a> = redux::ActionWithMeta<&'a LedgerAction>;

#[derive(Serialize, Deserialize, derive_more::From, Debug, Clone)]
pub enum LedgerAction {
    Write(LedgerWriteAction),
    Read(LedgerReadAction),
}

impl redux::EnablingCondition<crate::State> for LedgerAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        match self {
            LedgerAction::Write(action) => action.is_enabled(state, time),
            LedgerAction::Read(action) => action.is_enabled(state, time),
        }
    }
}
