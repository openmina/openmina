use serde::{Deserialize, Serialize};

use super::sync::{TransitionFrontierSyncAction, TransitionFrontierSyncState};

pub type TransitionFrontierActionWithMeta = redux::ActionWithMeta<TransitionFrontierAction>;
pub type TransitionFrontierActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierAction {
    Sync(TransitionFrontierSyncAction),
    Synced(TransitionFrontierSyncedAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncedAction {}

impl redux::EnablingCondition<crate::State> for TransitionFrontierSyncedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        matches!(
            state.transition_frontier.sync,
            TransitionFrontierSyncState::BlocksSuccess { .. }
        )
    }
}

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::TransitionFrontier(value.into())
            }
        }
    };
}

impl_into_global_action!(TransitionFrontierSyncedAction);
