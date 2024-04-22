use serde::{Deserialize, Serialize};

use super::{SnarkBlockVerifyError, SnarkBlockVerifyId, VerifiableBlockWithHash};

pub type SnarkBlockVerifyActionWithMeta = redux::ActionWithMeta<SnarkBlockVerifyAction>;
pub type SnarkBlockVerifyActionWithMetaRef<'a> = redux::ActionWithMeta<&'a SnarkBlockVerifyAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkBlockVerifyAction {
    Init {
        req_id: SnarkBlockVerifyId,
        block: VerifiableBlockWithHash,
        verify_success_cb: redux::Callback,
    },
    Pending {
        req_id: SnarkBlockVerifyId,
        verify_success_cb: redux::Callback,
    },
    Error {
        req_id: SnarkBlockVerifyId,
        error: SnarkBlockVerifyError,
    },
    Success {
        req_id: SnarkBlockVerifyId,
    },
    Finish {
        req_id: SnarkBlockVerifyId,
    },
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkBlockVerifyAction {
    fn is_enabled(&self, state: &crate::SnarkState, _time: redux::Timestamp) -> bool {
        match self {
            SnarkBlockVerifyAction::Init { req_id, .. } => {
                state.block_verify.jobs.next_req_id() == *req_id
            }
            SnarkBlockVerifyAction::Pending { req_id, .. } => state
                .block_verify
                .jobs
                .get(*req_id)
                .map_or(false, |v| v.is_init()),
            SnarkBlockVerifyAction::Error { req_id, .. } => state
                .block_verify
                .jobs
                .get(*req_id)
                .map_or(false, |v| v.is_pending()),
            SnarkBlockVerifyAction::Success { req_id } => state
                .block_verify
                .jobs
                .get(*req_id)
                .map_or(false, |v| v.is_pending()),
            SnarkBlockVerifyAction::Finish { req_id } => state
                .block_verify
                .jobs
                .get(*req_id)
                .map_or(false, |v| v.is_finished()),
        }
    }
}
