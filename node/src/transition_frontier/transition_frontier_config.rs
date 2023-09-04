use serde::{Deserialize, Serialize};

use crate::ProtocolConstants;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierConfig {
    pub protocol_constants: ProtocolConstants,
}

impl TransitionFrontierConfig {
    pub fn k(&self) -> usize {
        self.protocol_constants.k.0.as_u32() as usize
    }
}
