use std::{sync::Arc, time::Duration};

use node::transition_frontier::genesis::GenesisConfig;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimulatorConfig {
    pub genesis: Arc<GenesisConfig>,
    pub seed_nodes: usize,
    pub normal_nodes: usize,
    pub snark_workers: usize,
    pub block_producers: usize,
    pub run_until: SimulatorRunUntil,
    pub run_until_timeout: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SimulatorRunUntil {
    Epoch(u32),
}
