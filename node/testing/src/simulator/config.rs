use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::node::DaemonJsonGenConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimulatorConfig {
    pub daemon_json: DaemonJsonGenConfig,
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
