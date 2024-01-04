use std::time::Duration;

use crate::{
    node::DaemonJsonGenConfig,
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
        let mut simulator = Simulator::new(SimulatorConfig {
            daemon_json: DaemonJsonGenConfig::Counts { whales: 2, fish: 4 },
            seed_nodes: 2,
            normal_nodes: 2,
            run_until: SimulatorRunUntil::Epoch(8),
            run_until_timeout: Duration::from_secs(30 * 60),
        });
        simulator.run(runner).await;
    }
}
