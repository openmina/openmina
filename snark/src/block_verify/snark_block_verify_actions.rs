use openmina_core::{block::BlockHash, ActionEvent, SubstateAccess};
use serde::{Deserialize, Serialize};

use super::{SnarkBlockVerifyError, SnarkBlockVerifyId, VerifiableBlockWithHash};

pub type SnarkBlockVerifyActionWithMeta = redux::ActionWithMeta<SnarkBlockVerifyAction>;
pub type SnarkBlockVerifyActionWithMetaRef<'a> = redux::ActionWithMeta<&'a SnarkBlockVerifyAction>;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum SnarkBlockVerifyAction {
    Init {
        block: VerifiableBlockWithHash,

        on_init: redux::Callback<(BlockHash, SnarkBlockVerifyId)>,
        on_success: redux::Callback<BlockHash>,
        on_error: redux::Callback<(BlockHash, SnarkBlockVerifyError)>,
    },
    Pending {
        req_id: SnarkBlockVerifyId,
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
            SnarkBlockVerifyAction::Init { .. } => true,
            SnarkBlockVerifyAction::Pending { req_id } => state
                .block_verify
                .jobs
                .get(*req_id)
                .is_some_and(|v| v.is_init()),
            SnarkBlockVerifyAction::Error { req_id, .. } => state
                .block_verify
                .jobs
                .get(*req_id)
                .is_some_and(|v| v.is_pending()),
            SnarkBlockVerifyAction::Success { req_id } => state
                .block_verify
                .jobs
                .get(*req_id)
                .is_some_and(|v| v.is_pending()),
            SnarkBlockVerifyAction::Finish { req_id } => state
                .block_verify
                .jobs
                .get(*req_id)
                .is_some_and(|v| v.is_finished()),
        }
    }
}

impl<T> redux::EnablingCondition<T> for SnarkBlockVerifyAction
where
    T: SubstateAccess<crate::SnarkState>,
{
    fn is_enabled(&self, state: &T, time: redux::Timestamp) -> bool {
        state
            .substate()
            .is_ok_and(|state| self.is_enabled(state, time))
    }
}
