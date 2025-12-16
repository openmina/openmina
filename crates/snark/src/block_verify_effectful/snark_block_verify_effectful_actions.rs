use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{block_verify::VerifiableBlockWithHash, BlockVerifier, VerifierSRS};

use super::SnarkBlockVerifyId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkBlockVerifyEffectfulAction {
    Init {
        req_id: SnarkBlockVerifyId,
        block: VerifiableBlockWithHash,
        verifier_index: BlockVerifier,
        verifier_srs: Arc<VerifierSRS>,
    },
}

impl redux::EnablingCondition<crate::SnarkState> for SnarkBlockVerifyEffectfulAction {
    fn is_enabled(&self, _state: &crate::SnarkState, _time: redux::Timestamp) -> bool {
        true
    }
}
