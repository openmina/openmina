use serde::{Deserialize, Serialize};

use crate::block_verify_effectful::SnarkBlockVerifyEffectfulAction;
use crate::user_command_verify::SnarkUserCommandVerifyAction;
use crate::work_verify_effectful::SnarkWorkVerifyEffectfulAction;

use super::block_verify::SnarkBlockVerifyAction;
use super::work_verify::SnarkWorkVerifyAction;

pub type SnarkActionWithMeta = redux::ActionWithMeta<SnarkAction>;
pub type SnarkActionWithMetaRef<'a> = redux::ActionWithMeta<&'a SnarkAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum SnarkAction {
    BlockVerify(SnarkBlockVerifyAction),
    BlockVerifyEffect(SnarkBlockVerifyEffectfulAction),
    WorkVerify(SnarkWorkVerifyAction),
    WorkVerifyEffect(SnarkWorkVerifyEffectfulAction),
    UserCommandVerify(SnarkUserCommandVerifyAction),
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkAction {
    fn is_enabled(&self, state: &crate::SnarkState, time: redux::Timestamp) -> bool {
        match self {
            SnarkAction::BlockVerify(a) => a.is_enabled(state, time),
            SnarkAction::BlockVerifyEffect(a) => a.is_enabled(state, time),
            SnarkAction::WorkVerify(a) => a.is_enabled(state, time),
            SnarkAction::WorkVerifyEffect(a) => a.is_enabled(state, time),
            SnarkAction::UserCommandVerify(a) => a.is_enabled(state, time),
        }
    }
}

impl From<redux::AnyAction> for SnarkAction {
    fn from(action: redux::AnyAction) -> Self {
        *action.0.downcast::<Self>().expect("Downcast failed")
    }
}
