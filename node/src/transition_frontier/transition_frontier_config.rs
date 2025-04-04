//! Configuration for the transition frontier, including genesis settings
//! that determine the initial state of the blockchain.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::genesis::TransitionFrontierGenesisConfig;

/// Configuration for the transition frontier component.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierConfig {
    /// Genesis configuration containing initial blockchain parameters
    pub genesis: Arc<TransitionFrontierGenesisConfig>,
}

impl TransitionFrontierConfig {
    pub fn new(genesis: Arc<TransitionFrontierGenesisConfig>) -> Self {
        TransitionFrontierConfig { genesis }
    }
}
