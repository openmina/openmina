use serde::{Deserialize, Serialize};

use shared::snark::Snark;

use super::{SnarkWorkVerifyError, SnarkWorkVerifyId};

pub type SnarkWorkVerifyActionWithMeta = redux::ActionWithMeta<SnarkWorkVerifyAction>;
pub type SnarkWorkVerifyActionWithMetaRef<'a> = redux::ActionWithMeta<&'a SnarkWorkVerifyAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum SnarkWorkVerifyAction {
    Init(SnarkWorkVerifyInitAction),
    Pending(SnarkWorkVerifyPendingAction),
    Error(SnarkWorkVerifyErrorAction),
    Success(SnarkWorkVerifySuccessAction),
    Finish(SnarkWorkVerifyFinishAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkWorkVerifyInitAction {
    pub req_id: SnarkWorkVerifyId,
    pub batch: Vec<Snark>,
    pub sender: String,
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkWorkVerifyInitAction {
    fn is_enabled(&self, state: &crate::SnarkState) -> bool {
        !self.batch.is_empty() && state.work_verify.jobs.next_req_id() == self.req_id
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkWorkVerifyPendingAction {
    pub req_id: SnarkWorkVerifyId,
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkWorkVerifyPendingAction {
    fn is_enabled(&self, state: &crate::SnarkState) -> bool {
        state
            .work_verify
            .jobs
            .get(self.req_id)
            .map_or(false, |v| v.is_init())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkWorkVerifyErrorAction {
    pub req_id: SnarkWorkVerifyId,
    pub error: SnarkWorkVerifyError,
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkWorkVerifyErrorAction {
    fn is_enabled(&self, state: &crate::SnarkState) -> bool {
        state
            .work_verify
            .jobs
            .get(self.req_id)
            .map_or(false, |v| v.is_pending())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkWorkVerifySuccessAction {
    pub req_id: SnarkWorkVerifyId,
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkWorkVerifySuccessAction {
    fn is_enabled(&self, state: &crate::SnarkState) -> bool {
        state
            .work_verify
            .jobs
            .get(self.req_id)
            .map_or(false, |v| v.is_pending())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkWorkVerifyFinishAction {
    pub req_id: SnarkWorkVerifyId,
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkWorkVerifyFinishAction {
    fn is_enabled(&self, state: &crate::SnarkState) -> bool {
        state
            .work_verify
            .jobs
            .get(self.req_id)
            .map_or(false, |v| v.is_finished())
    }
}

macro_rules! impl_into_snark_action {
    ($a:ty) => {
        impl From<$a> for crate::SnarkAction {
            fn from(value: $a) -> Self {
                Self::WorkVerify(value.into())
            }
        }
    };
}

impl_into_snark_action!(SnarkWorkVerifyInitAction);
impl_into_snark_action!(SnarkWorkVerifyPendingAction);
impl_into_snark_action!(SnarkWorkVerifyErrorAction);
impl_into_snark_action!(SnarkWorkVerifySuccessAction);
impl_into_snark_action!(SnarkWorkVerifyFinishAction);
