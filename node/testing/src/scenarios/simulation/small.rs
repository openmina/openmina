use std::time::Duration;

use mina_p2p_messages::v2::{BlockTimeTimeStableV1, PROTOCOL_CONSTANTS};
use node::transition_frontier::genesis::GenesisConfig;

use crate::{
    scenarios::ClusterRunner,
    simulator::{Simulator, SimulatorConfig, SimulatorRunUntil},
};

/// Small simulation.
///
/// Run until `epoch_count < 8`.
///
/// - **whale** block producers: **2**.
/// - **fish** block producers: **4**.
/// - seed nodes: **2**.
/// - normal nodes: **2**.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct SimulationSmall;

impl SimulationSmall {
    pub async fn run(self, runner: ClusterRunner<'_>) {
        let initial_time = redux::Timestamp::global_now();
        let mut constants = PROTOCOL_CONSTANTS.clone();
        constants.genesis_state_timestamp =
            BlockTimeTimeStableV1((u64::from(initial_time) / 1_000_000).into());
        let genesis_cfg = GenesisConfig::Counts {
            whales: 2,
            fish: 4,
            constants,
        };
        let cfg = SimulatorConfig {
            genesis: genesis_cfg.into(),
            seed_nodes: 2,
            normal_nodes: 2,
            snark_workers: 1,
            block_producers: 6,
            run_until: SimulatorRunUntil::Epoch(3),
            run_until_timeout: Duration::from_secs(30 * 60),
        };
        let mut simulator = Simulator::new(initial_time, cfg);
        simulator.run(runner).await;
    }
}
