use serde::{Deserialize, Serialize};

use super::{LedgerReadId, LedgerReadRequest, LedgerReadRequestState, LedgerReadResponse};

pub type LedgerReadActionWithMeta = redux::ActionWithMeta<LedgerReadAction>;
pub type LedgerReadActionWithMetaRef<'a> = redux::ActionWithMeta<&'a LedgerReadAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LedgerReadAction {
    FindTodos,
    Init {
        request: LedgerReadRequest,
    },
    Pending {
        id: LedgerReadId,
        request: LedgerReadRequest,
    },
    Success {
        id: LedgerReadId,
        response: LedgerReadResponse,
    },
    Prune {
        id: LedgerReadId,
    },
}

impl redux::EnablingCondition<crate::State> for LedgerReadAction {
    fn is_enabled(&self, state: &crate::State, _time: redux::Timestamp) -> bool {
        match self {
            LedgerReadAction::FindTodos => state.ledger.read.is_total_cost_under_limit(),
            LedgerReadAction::Init { .. } => state.ledger.read.is_total_cost_under_limit(),
            LedgerReadAction::Pending { id, .. } => {
                state.ledger.read.is_total_cost_under_limit()
                    && !state.ledger.read.contains(*id)
                    && state.ledger.read.next_req_id() == *id
            }
            LedgerReadAction::Success { id, response } => match &state.ledger.read.get(*id) {
                Some(LedgerReadRequestState::Pending { request, .. }) => {
                    request.kind() == response.kind()
                }
                _ => false,
            },
            LedgerReadAction::Prune { id } => matches!(
                state.ledger.read.get(*id),
                Some(LedgerReadRequestState::Success { .. })
            ),
        }
    }
}

impl From<LedgerReadAction> for crate::Action {
    fn from(value: LedgerReadAction) -> Self {
        Self::Ledger(value.into())
    }
}
