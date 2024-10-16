use std::time::Duration;

use node::ActionKind;
use openmina_node_native::replay_state_with_input_actions;

use crate::{
    hosts,
    node::{Recorder, RustNodeTestingConfig, TestPeerId},
    scenarios::{ClusterRunner, RunCfg, RunCfgAdvanceTime},
};

/// Bootstrap a rust node while recorder of state and input actions is
/// enabled and make sure we can successfully replay it.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct RecordReplayBootstrap;

impl RecordReplayBootstrap {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        let initial_peers = hosts::devnet();

        let node_id = runner.add_rust_node(RustNodeTestingConfig {
            initial_time: redux::Timestamp::global_now(),
            initial_peers,
            peer_id: TestPeerId::Bytes(rand::random()),
            recorder: Recorder::StateWithInputActions,
            ..RustNodeTestingConfig::devnet_default()
        });

        // bootstrap the node.
        runner
            .run(
                RunCfg::default()
                    .timeout(Duration::from_secs(40 * 60))
                    .advance_time(RunCfgAdvanceTime::Real)
                    .action_handler(|_, state, _, a| {
                        a.action().kind() == ActionKind::TransitionFrontierSynced
                            && state
                                .transition_frontier
                                .best_tip()
                                .map_or(false, |tip| !tip.is_genesis())
                    }),
            )
            .await
            .expect("node failed to bootstrap");
        // flush the recorded data.
        node::recorder::Recorder::graceful_shutdown();

        let node = runner.node(node_id).unwrap();

        let recording_dir = node.work_dir().child("recorder");
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
