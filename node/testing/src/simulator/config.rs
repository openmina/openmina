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
    #[serde(default)]
    pub advance_time: RunCfgAdvanceTime,
    #[serde(default)]
    pub run_until: SimulatorRunUntil,
    #[serde(default = "duration_max")]
    pub run_until_timeout: Duration,
    #[serde(default)]
    pub recorder: Recorder,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub enum SimulatorRunUntil {
    #[default]
    Forever,
    Epoch(u32),
    BlockchainLength(u32),
}

fn duration_max() -> Duration {
    Duration::MAX
}
