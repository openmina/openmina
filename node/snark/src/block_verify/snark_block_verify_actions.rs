use serde::{Deserialize, Serialize};

use mina_p2p_messages::v2::MinaBlockHeaderStableV2;

use super::{SnarkBlockVerifyError, SnarkBlockVerifyId};

pub type SnarkBlockVerifyActionWithMeta = redux::ActionWithMeta<SnarkBlockVerifyAction>;
pub type SnarkBlockVerifyActionWithMetaRef<'a> = redux::ActionWithMeta<&'a SnarkBlockVerifyAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum SnarkBlockVerifyAction {
    Init(SnarkBlockVerifyInitAction),
    Pending(SnarkBlockVerifyPendingAction),
    Error(SnarkBlockVerifyErrorAction),
    Success(SnarkBlockVerifySuccessAction),
    Finish(SnarkBlockVerifyFinishAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkBlockVerifyInitAction {
    pub req_id: SnarkBlockVerifyId,
    pub block: MinaBlockHeaderStableV2,
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkBlockVerifyInitAction {
    fn is_enabled(&self, state: &crate::SnarkState) -> bool {
        state.block_verify.jobs.next_req_id() == self.req_id
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkBlockVerifyPendingAction {
    pub req_id: SnarkBlockVerifyId,
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkBlockVerifyPendingAction {
    fn is_enabled(&self, state: &crate::SnarkState) -> bool {
        state
            .block_verify
            .jobs
            .get(self.req_id)
            .map_or(false, |v| v.is_init())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkBlockVerifyErrorAction {
    pub req_id: SnarkBlockVerifyId,
    pub error: SnarkBlockVerifyError,
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkBlockVerifyErrorAction {
    fn is_enabled(&self, state: &crate::SnarkState) -> bool {
        state
            .block_verify
            .jobs
            .get(self.req_id)
            .map_or(false, |v| v.is_pending())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkBlockVerifySuccessAction {
    pub req_id: SnarkBlockVerifyId,
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkBlockVerifySuccessAction {
    fn is_enabled(&self, state: &crate::SnarkState) -> bool {
        state
            .block_verify
            .jobs
            .get(self.req_id)
            .map_or(false, |v| v.is_pending())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkBlockVerifyFinishAction {
    pub req_id: SnarkBlockVerifyId,
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkBlockVerifyFinishAction {
    fn is_enabled(&self, state: &crate::SnarkState) -> bool {
        state
            .block_verify
            .jobs
            .get(self.req_id)
            .map_or(false, |v| v.is_finished())
    }
}

macro_rules! impl_into_snark_action {
    ($a:ty) => {
        impl From<$a> for crate::SnarkAction {
            fn from(value: $a) -> Self {
                Self::BlockVerify(value.into())
            }
        }
    };
}

impl_into_snark_action!(SnarkBlockVerifyInitAction);
impl_into_snark_action!(SnarkBlockVerifyPendingAction);
impl_into_snark_action!(SnarkBlockVerifyErrorAction);
impl_into_snark_action!(SnarkBlockVerifySuccessAction);
impl_into_snark_action!(SnarkBlockVerifyFinishAction);
