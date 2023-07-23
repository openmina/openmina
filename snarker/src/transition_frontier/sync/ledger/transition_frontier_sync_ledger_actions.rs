use serde::{Deserialize, Serialize};

use super::snarked::TransitionFrontierSyncLedgerSnarkedAction;
use super::staged::{
    TransitionFrontierSyncLedgerStagedAction, TransitionFrontierSyncLedgerStagedState,
};
use super::TransitionFrontierSyncLedgerState;

pub type TransitionFrontierSyncLedgerActionWithMeta =
    redux::ActionWithMeta<TransitionFrontierSyncLedgerAction>;
pub type TransitionFrontierSyncLedgerActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierSyncLedgerAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncLedgerAction {
    Init(TransitionFrontierSyncLedgerInitAction),
    Snarked(TransitionFrontierSyncLedgerSnarkedAction),
    Staged(TransitionFrontierSyncLedgerStagedAction),
    Success(TransitionFrontierSyncLedgerSuccessAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerInitAction {}

impl redux::EnablingCondition<crate::State> for TransitionFrontierSyncLedgerInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .map_or(false, |s| {
                matches!(s, TransitionFrontierSyncLedgerState::Init { .. })
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSuccessAction {}

impl redux::EnablingCondition<crate::State> for TransitionFrontierSyncLedgerSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .and_then(|s| s.staged())
            .map_or(false, |s| s.is_success())
    }
}

use crate::transition_frontier::{sync::TransitionFrontierSyncAction, TransitionFrontierAction};

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::TransitionFrontier(TransitionFrontierAction::Sync(
                    TransitionFrontierSyncAction::Ledger(value.into()),
                ))
            }
        }
    };
}

impl_into_global_action!(TransitionFrontierSyncLedgerInitAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSuccessAction);
