use serde::{Deserialize, Serialize};

use super::block_verify::SnarkBlockVerifyAction;
use super::work_verify::SnarkWorkVerifyAction;

pub type SnarkActionWithMeta = redux::ActionWithMeta<SnarkAction>;
pub type SnarkActionWithMetaRef<'a> = redux::ActionWithMeta<&'a SnarkAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum SnarkAction {
    BlockVerify(SnarkBlockVerifyAction),
    WorkVerify(SnarkWorkVerifyAction),
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkAction {
    fn is_enabled(&self, state: &crate::SnarkState, time: redux::Timestamp) -> bool {
        match self {
            SnarkAction::BlockVerify(action) => action.is_enabled(state, time),
            SnarkAction::WorkVerify(action) => action.is_enabled(state, time),
        }
    }
}
