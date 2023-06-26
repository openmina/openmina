use serde::{Deserialize, Serialize};

use crate::ProtocolConstants;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierConfig {
    pub protocol_constants: ProtocolConstants,
}
