use std::time::Duration;

use mina_p2p_messages::v2;
use node::transition_frontier::genesis::{GenesisConfig, NonStakers};

use crate::{
    node::Recorder,
    scenarios::{ClusterRunner, RunCfgAdvanceTime},
    simulator::{Simulator, SimulatorConfig, SimulatorRunUntil},
};

/// Create and Sync up 50 nodes, one amoung them is block producer.
///
/// 1. Create the nodes.
/// 2. Connect them to each other.
/// 3. Wait kademlia bootstrap is done, observe the connection graph.
/// 4. Wait pubsub mesh construction is done, observe the mesh.
/// 5. Wait block is produced and observe the propagation.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct MultiNodePubsubPropagateBlock;

impl MultiNodePubsubPropagateBlock {
    const WORKERS: usize = 20;

    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        let initial_time = redux::Timestamp::global_now();
        let mut constants = v2::PROTOCOL_CONSTANTS.clone();
        constants.genesis_state_timestamp =
            v2::BlockTimeTimeStableV1((u64::from(initial_time) / 1_000_000).into());
        let genesis_cfg = GenesisConfig::Counts {
            whales: 1,
            fish: 0,
            non_stakers: NonStakers::None,
            constants,
        };
        let config = SimulatorConfig {
            genesis: genesis_cfg.into(),
            seed_nodes: 1,
            normal_nodes: Self::WORKERS,
            snark_workers: 1,
            block_producers: 1,
            advance_time: RunCfgAdvanceTime::Rand(1..=200),
            run_until: SimulatorRunUntil::BlockchainLength(3),
            run_until_timeout: Duration::from_secs(10 * 60),
            recorder: Recorder::StateWithInputActions,
        };
        let mut simulator = Simulator::new(initial_time, config);
        simulator.run(&mut runner).await;
    }
}
