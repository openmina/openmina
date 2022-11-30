use serde::{Deserialize, Serialize};

use crate::VerifierIndex;

use super::block_verify::SnarkBlockVerifyState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkState {
    pub block_verify: SnarkBlockVerifyState,
}

impl SnarkState {
    pub fn new(verifier_index: VerifierIndex) -> Self {
        Self {
            block_verify: SnarkBlockVerifyState::new(verifier_index),
        }
    }
}
