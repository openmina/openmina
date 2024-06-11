use std::time::Duration;

use mina_p2p_messages::v2;
use node::transition_frontier::genesis::GenesisConfig;

use crate::{
    node::Recorder, scenarios::{ClusterRunner, RunCfgAdvanceTime}, simulator::{Simulator, SimulatorConfig, SimulatorRunUntil}
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
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        let initial_time = redux::Timestamp::global_now();
        let mut constants = v2::PROTOCOL_CONSTANTS.clone();
        constants.genesis_state_timestamp =
            v2::BlockTimeTimeStableV1((u64::from(initial_time) / 1_000_000).into());
        let genesis_cfg = GenesisConfig::Counts {
            whales: 1,
            fish: 2,
            constants,
        };
        let cfg = SimulatorConfig {
            genesis: genesis_cfg.into(),
            seed_nodes: 2,
            normal_nodes: 2,
            snark_workers: 1,
            block_producers: 3,
            advance_time: RunCfgAdvanceTime::Rand(1..=300),
            run_until: SimulatorRunUntil::Forever,
            run_until_timeout: Duration::MAX,
            recorder: Recorder::StateWithInputActions,
        };
        let mut simulator = Simulator::new(initial_time, cfg);
        simulator.run(&mut runner).await;
    }
}
