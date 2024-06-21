use std::sync::{Arc, Mutex};

use openmina_core::snark::Snark;
use serde::{Deserialize, Serialize};

use crate::{VerifierIndex, VerifierSRS};

use super::SnarkWorkVerifyId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkWorkVerifyEffectfulAction {
    Init {
        req_id: SnarkWorkVerifyId,
        batch: Vec<Snark>,
        sender: String,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<Mutex<VerifierSRS>>,
    },
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkWorkVerifyEffectfulAction {
    fn is_enabled(&self, _state: &crate::SnarkState, _time: redux::Timestamp) -> bool {
        true
    }
}
