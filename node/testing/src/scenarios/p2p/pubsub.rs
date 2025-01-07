use std::time::Duration;

use crate::{
    hosts,
    node::RustNodeTestingConfig,
    scenarios::{ClusterRunner, RunCfg, RunCfgAdvanceTime},
};

/// Receive a block via meshsub
/// 1. Create a normal node with default devnet config, with devnet peers as initial peers
/// 2. Wait for 2 minutes
/// 3. Create a node with discovery disabled and first node as only peer
/// 4. Wait for first node to broadcast block to second one
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct P2pReceiveBlock;

impl P2pReceiveBlock {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        let config = RustNodeTestingConfig::devnet_default().initial_peers(hosts::devnet());

        let retransmitter_openmina_node = runner.add_rust_node(config);
        let retransmitter_peer_id = runner
            .node(retransmitter_openmina_node)
            .unwrap()
            .state()
            .p2p
            .my_id();

        let _ = runner
            .run(
                RunCfg::default()
                    .timeout(Duration::from_secs(120))
                    .advance_time(RunCfgAdvanceTime::Real)
                    .action_handler(|_, _, _, _| false),
            )
            .await;

        let config = RustNodeTestingConfig::devnet_default()
            // Make sure it doesn't connect to any more peers
            .with_no_peer_discovery()
            .initial_peers(vec![retransmitter_openmina_node.into()]);

        let receiver_openmina_node = runner.add_rust_node(config);

        runner
            .run(
                RunCfg::default()
                    .timeout(Duration::from_secs(60 * 30))
                    .advance_time(RunCfgAdvanceTime::Real)
                    .action_handler(move |node, state, _, _| {
                        let Some(p2p) = state.p2p.ready() else {
                            return false;
                        };

                        node == receiver_openmina_node
                            && p2p
                                .network
                                .scheduler
                                .broadcast_state
                                .incoming_block
                                .as_ref()
                                .map_or(false, |(peer_id, _)| peer_id.eq(&retransmitter_peer_id))
                    }),
            )
            .await
            .expect("Failed to receive block");
    }
}
