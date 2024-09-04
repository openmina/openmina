use serde::{Deserialize, Serialize};

use super::{LedgerWriteRequest, LedgerWriteResponse, LedgerWriteState};

pub type LedgerWriteActionWithMeta = redux::ActionWithMeta<LedgerWriteAction>;
pub type LedgerWriteActionWithMetaRef<'a> = redux::ActionWithMeta<&'a LedgerWriteAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LedgerWriteAction {
    Init {
        request: LedgerWriteRequest,
        on_init: redux::Callback<LedgerWriteRequest>,
    },
    Pending,
    Success {
        response: LedgerWriteResponse,
    },
}

impl redux::EnablingCondition<crate::State> for LedgerWriteAction {
    fn is_enabled(&self, state: &crate::State, _time: redux::Timestamp) -> bool {
        match self {
            LedgerWriteAction::Init { .. } => matches!(
                &state.ledger.write,
                LedgerWriteState::Idle { .. } | LedgerWriteState::Success { .. }
            ),
            LedgerWriteAction::Pending { .. } => {
                matches!(&state.ledger.write, LedgerWriteState::Init { .. })
            }
            LedgerWriteAction::Success { response } => match &state.ledger.write {
                LedgerWriteState::Pending { request, .. } => request.kind() == response.kind(),
                _ => false,
            },
        }
    }
}

impl From<LedgerWriteAction> for crate::Action {
    fn from(value: LedgerWriteAction) -> Self {
        Self::Ledger(value.into())
    }
}
