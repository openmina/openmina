use serde::{Deserialize, Serialize};

use crate::user_command_verify::SnarkUserCommandVerifyState;
use crate::SnarkConfig;

use super::block_verify::SnarkBlockVerifyState;
use super::work_verify::SnarkWorkVerifyState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkState {
    pub block_verify: SnarkBlockVerifyState,
    pub work_verify: SnarkWorkVerifyState,
    pub user_command_verify: SnarkUserCommandVerifyState,
}

impl SnarkState {
    pub fn new(config: SnarkConfig) -> Self {
        Self {
            block_verify: SnarkBlockVerifyState::new(
                config.block_verifier_index,
                config.block_verifier_srs,
            ),
            work_verify: SnarkWorkVerifyState::new(
                config.work_verifier_index.clone(),
                config.work_verifier_srs.clone(),
            ),
            user_command_verify: SnarkUserCommandVerifyState::new(
                config.work_verifier_index,
                config.work_verifier_srs,
            ),
        }
    }
}
