use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::transition_frontier::sync::TransitionFrontierSyncAction;
use crate::TransitionFrontierAction;

use super::snarked::TransitionFrontierSyncLedgerSnarkedAction;
use super::staged::TransitionFrontierSyncLedgerStagedAction;
use super::TransitionFrontierSyncLedgerState;

pub type TransitionFrontierSyncLedgerActionWithMeta =
    redux::ActionWithMeta<TransitionFrontierSyncLedgerAction>;
pub type TransitionFrontierSyncLedgerActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierSyncLedgerAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum TransitionFrontierSyncLedgerAction {
    Init,
    Snarked(TransitionFrontierSyncLedgerSnarkedAction),
    Staged(TransitionFrontierSyncLedgerStagedAction),
    Success,
}

impl redux::EnablingCondition<crate::State> for TransitionFrontierSyncLedgerAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        match self {
            TransitionFrontierSyncLedgerAction::Init => {
                state.transition_frontier.sync.ledger().map_or(false, |s| {
                    matches!(s, TransitionFrontierSyncLedgerState::Init { .. })
                })
            }
            TransitionFrontierSyncLedgerAction::Snarked(a) => a.is_enabled(state, time),
            TransitionFrontierSyncLedgerAction::Staged(a) => a.is_enabled(state, time),
            TransitionFrontierSyncLedgerAction::Success => {
                state.transition_frontier.sync.is_ledger_sync_complete()
            }
        }
    }
}

impl From<TransitionFrontierSyncLedgerAction> for crate::Action {
    fn from(value: TransitionFrontierSyncLedgerAction) -> Self {
        Self::TransitionFrontier(TransitionFrontierAction::Sync(
            TransitionFrontierSyncAction::Ledger(value),
        ))
    }
}
