use serde::{Deserialize, Serialize};

use crate::SnarkConfig;

use super::block_verify::SnarkBlockVerifyState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkState {
    pub block_verify: SnarkBlockVerifyState,
}

impl SnarkState {
    pub fn new(config: SnarkConfig) -> Self {
        Self {
            block_verify: SnarkBlockVerifyState::new(
                config.block_verifier_index,
                config.block_verifier_srs,
            ),
        }
    }
}
