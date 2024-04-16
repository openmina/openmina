use ledger::scan_state::transaction_logic::{verifiable, WithStatus};
use serde::{Deserialize, Serialize};

use openmina_core::ActionEvent;

use super::{SnarkUserCommandVerifyError, SnarkUserCommandVerifyId};

pub type SnarkUserCommandVerifyActionWithMeta = redux::ActionWithMeta<SnarkUserCommandVerifyAction>;
pub type SnarkUserCommandVerifyActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a SnarkUserCommandVerifyAction>;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = trace, fields(display(req_id), display(error)))]
pub enum SnarkUserCommandVerifyAction {
    #[action_event(level = info)]
    Init {
        req_id: SnarkUserCommandVerifyId,
        commands: Vec<WithStatus<verifiable::UserCommand>>,
        sender: String,
    },
    Pending {
        req_id: SnarkUserCommandVerifyId,
    },
    Error {
        req_id: SnarkUserCommandVerifyId,
        error: SnarkUserCommandVerifyError,
    },
    #[action_event(level = info)]
    Success {
        req_id: SnarkUserCommandVerifyId,
    },
    Finish {
        req_id: SnarkUserCommandVerifyId,
    },
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkUserCommandVerifyAction {
    fn is_enabled(&self, state: &crate::SnarkState, _time: redux::Timestamp) -> bool {
        match self {
            SnarkUserCommandVerifyAction::Init {
                req_id, commands, ..
            } => !commands.is_empty() && state.user_command_verify.jobs.next_req_id() == *req_id,
            SnarkUserCommandVerifyAction::Pending { req_id } => state
                .user_command_verify
                .jobs
                .get(*req_id)
                .map_or(false, |v| v.is_init()),
            SnarkUserCommandVerifyAction::Error { req_id, .. } => state
                .user_command_verify
                .jobs
                .get(*req_id)
                .map_or(false, |v| v.is_pending()),
            SnarkUserCommandVerifyAction::Success { req_id } => state
                .user_command_verify
                .jobs
                .get(*req_id)
                .map_or(false, |v| v.is_pending()),
            SnarkUserCommandVerifyAction::Finish { req_id } => state
                .user_command_verify
                .jobs
                .get(*req_id)
                .map_or(false, |v| v.is_finished()),
        }
    }
}
