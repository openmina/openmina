use std::time::Duration;

use mina_p2p_messages::v2;
use node::transition_frontier::genesis::{GenesisConfig, NonStakers};
use openmina_node_native::replay_state_with_input_actions;

use crate::{
    node::Recorder,
    scenarios::{ClusterRunner, RunCfgAdvanceTime},
    simulator::{Simulator, SimulatorConfig, SimulatorRunUntil},
};

/// Makes sure we can successfully record and replay multiple nodes
/// in the cluster + block production.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct RecordReplayBlockProduction;

impl RecordReplayBlockProduction {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        let initial_time = redux::Timestamp::global_now();
        let mut constants = v2::PROTOCOL_CONSTANTS.clone();
        constants.genesis_state_timestamp =
            v2::BlockTimeTimeStableV1((u64::from(initial_time) / 1_000_000).into());
        let genesis_cfg = GenesisConfig::Counts {
            whales: 1,
            fish: 2,
            non_stakers: NonStakers::None,
            constants,
        };
        let cfg = SimulatorConfig {
            genesis: genesis_cfg.into(),
            seed_nodes: 1,
            normal_nodes: 1,
            snark_workers: 1,
            block_producers: 3,
            advance_time: RunCfgAdvanceTime::Rand(1..=200),
            run_until: SimulatorRunUntil::BlockchainLength(10),
            run_until_timeout: Duration::from_secs(10 * 60),
            recorder: Recorder::StateWithInputActions,
        };
        let mut simulator = Simulator::new(initial_time, cfg);
        simulator.run(&mut runner).await;

        // flush the recorded data.
        node::recorder::Recorder::graceful_shutdown();

        for (id, node) in runner.nodes_iter() {
            let recording_dir = node.work_dir().child("recorder");
            eprintln!("replaying node: {id} from {recording_dir:?}");
            let replayed_node = replay_state_with_input_actions(
                recording_dir.as_os_str().to_str().unwrap(),
                None,
                |_, _| Ok(()),
            )
            .expect("replay failed");

            assert_eq!(
                node.state().last_action(),
                replayed_node.store().state().last_action()
            );
        }
    }
}
