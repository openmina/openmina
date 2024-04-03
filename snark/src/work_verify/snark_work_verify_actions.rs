use serde::{Deserialize, Serialize};

use openmina_core::{action_info, action_trace, action_warn, log::ActionEvent, snark::Snark};

use super::{SnarkWorkVerifyError, SnarkWorkVerifyId};

pub type SnarkWorkVerifyActionWithMeta = redux::ActionWithMeta<SnarkWorkVerifyAction>;
pub type SnarkWorkVerifyActionWithMetaRef<'a> = redux::ActionWithMeta<&'a SnarkWorkVerifyAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkWorkVerifyAction {
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

impl ActionEvent for SnarkWorkVerifyAction {
    fn action_event<T>(&self, context: &T)
    where
        T: openmina_core::log::EventContext,
    {
        match self {
            SnarkWorkVerifyAction::Init { req_id, batch, .. } => action_info!(
                context,
                req_id = display(req_id),
                trace_batch =
                    serde_json::to_string(&batch.iter().map(|v| v.job_id()).collect::<Vec<_>>())
                        .ok()
            ),
            SnarkWorkVerifyAction::Pending { req_id } => {
                action_trace!(context, req_id = display(req_id))
            }
            SnarkWorkVerifyAction::Error { req_id, error } => {
                action_warn!(context, req_id = display(req_id), error = display(error))
            }
            SnarkWorkVerifyAction::Success { req_id } => {
                action_info!(context, req_id = display(req_id))
            }
            SnarkWorkVerifyAction::Finish { req_id } => {
                action_trace!(context, req_id = display(req_id))
            }
        }
    }
}
