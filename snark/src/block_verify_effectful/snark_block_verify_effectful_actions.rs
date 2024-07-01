use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::{block_verify::VerifiableBlockWithHash, VerifierIndex, VerifierSRS};

use super::SnarkBlockVerifyId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkBlockVerifyEffectfulAction {
    Init {
        req_id: SnarkBlockVerifyId,
        block: VerifiableBlockWithHash,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<Mutex<VerifierSRS>>,
    },
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkBlockVerifyEffectfulAction {
    fn is_enabled(&self, _state: &crate::SnarkState, _time: redux::Timestamp) -> bool {
        true
    }
}
