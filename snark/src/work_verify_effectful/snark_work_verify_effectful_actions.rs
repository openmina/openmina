use std::sync::Arc;

use openmina_core::snark::Snark;
use serde::{Deserialize, Serialize};

use crate::{TransactionVerifier, VerifierSRS};

use super::SnarkWorkVerifyId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkWorkVerifyEffectfulAction {
    Init {
        req_id: SnarkWorkVerifyId,
        batch: Vec<Snark>,
        sender: String,
        verifier_index: TransactionVerifier,
        verifier_srs: Arc<VerifierSRS>,
    },
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkWorkVerifyEffectfulAction {
    fn is_enabled(&self, _state: &crate::SnarkState, _time: redux::Timestamp) -> bool {
        true
    }
}
