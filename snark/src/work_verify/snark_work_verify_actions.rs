use serde::{Deserialize, Serialize};

use openmina_core::{snark::Snark, ActionEvent};

use super::{SnarkWorkVerifyError, SnarkWorkVerifyId};

pub type SnarkWorkVerifyActionWithMeta = redux::ActionWithMeta<SnarkWorkVerifyAction>;
pub type SnarkWorkVerifyActionWithMetaRef<'a> = redux::ActionWithMeta<&'a SnarkWorkVerifyAction>;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = trace, fields(display(req_id), display(error)))]
pub enum SnarkWorkVerifyAction {
    #[action_event(level = info)]
    Init {
        req_id: SnarkWorkVerifyId,
        batch: Vec<Snark>,
        sender: String,
    },
    Pending {
        req_id: SnarkWorkVerifyId,
    },
    Error {
        req_id: SnarkWorkVerifyId,
        error: SnarkWorkVerifyError,
    },
    #[action_event(level = info)]
    Success {
        req_id: SnarkWorkVerifyId,
    },
    Finish {
        req_id: SnarkWorkVerifyId,
    },
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkWorkVerifyAction {
    fn is_enabled(&self, state: &crate::SnarkState, _time: redux::Timestamp) -> bool {
        match self {
            SnarkWorkVerifyAction::Init { req_id, batch, .. } => {
                !batch.is_empty() && state.work_verify.jobs.next_req_id() == *req_id
            }
            SnarkWorkVerifyAction::Pending { req_id } => state
                .work_verify
                .jobs
                .get(*req_id)
                .map_or(false, |v| v.is_init()),
            SnarkWorkVerifyAction::Error { req_id, .. } => state
                .work_verify
                .jobs
                .get(*req_id)
                .map_or(false, |v| v.is_pending()),
            SnarkWorkVerifyAction::Success { req_id } => state
                .work_verify
                .jobs
                .get(*req_id)
                .map_or(false, |v| v.is_pending()),
            SnarkWorkVerifyAction::Finish { req_id } => state
                .work_verify
                .jobs
                .get(*req_id)
                .map_or(false, |v| v.is_finished()),
        }
    }
}
