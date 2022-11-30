use serde::{Deserialize, Serialize};

use super::block_verify::SnarkBlockVerifyState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkState {
    pub block_verify: SnarkBlockVerifyState,
}

impl SnarkState {
    pub fn new() -> Self {
        Self {
            block_verify: SnarkBlockVerifyState::new(),
        }
    }
}
