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

impl Default for TransitionFrontierConfig {
    fn default() -> Self {
        // TODO(binier): better way.
        Self {
            protocol_constants: serde_json::from_value(serde_json::json!({
                "k": "290",
                "slots_per_epoch": "7140",
                "slots_per_sub_window": "7",
                "delta": "0",
                // TODO(binier): fix wrong timestamp
                "genesis_state_timestamp": "0",
            }))
            .unwrap(),
        }
    }
}
