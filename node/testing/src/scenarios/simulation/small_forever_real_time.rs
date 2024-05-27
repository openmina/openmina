use std::time::Duration;

use node::transition_frontier::genesis::GenesisConfig;

use crate::{
    scenarios::{ClusterRunner, RunCfgAdvanceTime},
    simulator::{Simulator, SimulatorConfig, SimulatorRunUntil},
};

/// Small never-ending simulation.
///
/// Node's use real time.
///
/// - **whale** block producers: **1**.
/// - **fish** block producers: **2**.
/// - seed nodes: **2**.
/// - normal nodes: **2**.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct SimulationSmallForeverRealTime;

impl SimulationSmallForeverRealTime {
    pub async fn run(self, runner: ClusterRunner<'_>) {
        let initial_time = redux::Timestamp::global_now();
        let genesis_cfg = GenesisConfig::Counts {
            whales: 1,
            fish: 2,
            constants: GenesisConfig::default_constants(u64::from(initial_time) / 1_000_000),
        };
        let cfg = SimulatorConfig {
            genesis: genesis_cfg.into(),
            seed_nodes: 2,
            normal_nodes: 2,
            snark_workers: 1,
            block_producers: 3,
            advance_time: RunCfgAdvanceTime::Real,
            run_until: SimulatorRunUntil::Forever,
            run_until_timeout: Duration::MAX,
        };
        let mut simulator = Simulator::new(initial_time, cfg);
        simulator.run(runner).await;
    }
}
