use std::time::Duration;

use node::ActionKind;

use crate::{
    hosts,
    node::RustNodeTestingConfig,
    scenarios::{ClusterRunner, RunCfg, RunCfgAdvanceTime},
};

/// Receive a message via meshsub
/// 1. Create a normal node with default devnet config, with devnet peers as initial peers
/// 2. Wait for 2 minutes
/// 3. Create a node with discovery disabled and first node as only peer
/// 4. Wait for first node to broadcast message to second one
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct P2pReceiveMessage;

impl P2pReceiveMessage {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        let config = RustNodeTestingConfig::devnet_default()
            .initial_peers(hosts::devnet())
            .initial_time(redux::Timestamp::global_now());

        let retransmitter_mina_rust_node = runner.add_rust_node(config);

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
            .initial_peers(vec![retransmitter_mina_rust_node.into()]);

        let receiver_mina_rust_node = runner.add_rust_node(config);

        runner
            .run(
                RunCfg::default()
                    .timeout(Duration::from_secs(60 * 30))
                    .advance_time(RunCfgAdvanceTime::Real)
                    .action_handler(move |node, _state, _, action| {
                        node == receiver_mina_rust_node
                            && matches!(
                                action.action().kind(),
                                ActionKind::P2pNetworkPubsubValidateIncomingMessage
                            )
                    }),
            )
            .await
            .expect("Test failed");
    }
}
