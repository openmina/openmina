use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::genesis::TransitionFrontierGenesisConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierConfig {
    pub genesis: Arc<TransitionFrontierGenesisConfig>,
}

impl TransitionFrontierConfig {
    pub fn new(genesis: Arc<TransitionFrontierGenesisConfig>) -> Self {
        TransitionFrontierConfig { genesis }
    }
}
