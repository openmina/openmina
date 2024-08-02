use std::sync::{Arc, Mutex};

use mina_p2p_messages::{list::List, v2};
use serde::{Deserialize, Serialize};

use crate::{VerifierIndex, VerifierSRS};

use super::SnarkUserCommandVerifyId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkUserCommandVerifyEffectfulAction {
    Init {
        req_id: SnarkUserCommandVerifyId,
        commands: List<v2::MinaBaseUserCommandStableV2>,
        // commands: Vec<WithStatus<verifiable::UserCommand>>,
        // sender: String,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<Mutex<VerifierSRS>>,
    },
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkUserCommandVerifyEffectfulAction {
    fn is_enabled(&self, _state: &crate::SnarkState, _time: redux::Timestamp) -> bool {
        true
    }
}
