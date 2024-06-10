use std::{sync::Arc, time::Duration};

use node::transition_frontier::genesis::GenesisConfig;
use serde::{Deserialize, Serialize};

use crate::{node::Recorder, scenarios::RunCfgAdvanceTime};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimulatorConfig {
    pub genesis: Arc<GenesisConfig>,
    pub seed_nodes: usize,
    pub normal_nodes: usize,
    pub snark_workers: usize,
    pub block_producers: usize,
    pub advance_time: RunCfgAdvanceTime,
    pub run_until: SimulatorRunUntil,
    pub run_until_timeout: Duration,
    pub recorder: Recorder,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SimulatorRunUntil {
    Forever,
    Epoch(u32),
    BlockchainLength(u32),
}
