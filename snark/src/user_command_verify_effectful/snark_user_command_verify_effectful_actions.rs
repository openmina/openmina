use ledger::scan_state::transaction_logic::{verifiable, WithStatus};
use serde::{Deserialize, Serialize};

use super::SnarkUserCommandVerifyId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkUserCommandVerifyEffectfulAction {
    Init {
        req_id: SnarkUserCommandVerifyId,
        commands: Vec<WithStatus<verifiable::UserCommand>>,
    },
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkUserCommandVerifyEffectfulAction {
    fn is_enabled(&self, _state: &crate::SnarkState, _time: redux::Timestamp) -> bool {
        true
    }
}
