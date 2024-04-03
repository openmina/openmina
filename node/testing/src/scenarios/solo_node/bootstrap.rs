use std::time::Duration;

use node::{
    p2p::connection::outgoing::P2pConnectionOutgoingInitOpts,
    transition_frontier::sync::TransitionFrontierSyncState,
};
use redux::SystemTime;
use tokio::time::Instant;

use crate::{
    node::RustNodeTestingConfig,
    scenario::{ListenerNode, ScenarioStep},
    scenarios::ClusterRunner,
};

/// Set up single Rust node and bootstrap snarked ledger, bootstrap ledger and blocks.
///
/// 1. Node will connect to replayer.
/// 2. Observe that stacking ledger is synchronized.
/// 3. Observe that next epoch ledger is synchronized.
/// 4. Continue until transition frontier is synchronized.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct SoloNodeBootstrap;

impl SoloNodeBootstrap {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        use self::TransitionFrontierSyncState::*;

        const TIMEOUT: Duration = Duration::from_secs(60 * 40);

        const REPLAYER_1: &'static str =
            "/ip4/135.181.217.23/tcp/31968/p2p/12D3KooWPayQEdprqY2m3biReUUybA5LoULpJE7YWu6wetEKKELv";
        let replayer = (&REPLAYER_1.parse::<libp2p::Multiaddr>().unwrap())
            .try_into()
            .unwrap();

        let node_id = runner.add_rust_node(
            RustNodeTestingConfig::berkeley_default().initial_peers(vec![ListenerNode::Custom(
                P2pConnectionOutgoingInitOpts::LibP2P(replayer),
            )]),
        );
        eprintln!("launch Openmina node with default configuration, id: {node_id}");

        runner
            .run(
                TIMEOUT,
                |_, _, _| crate::scenarios::RunDecision::ContinueExec,
                |this_node_id, state, _, _| {
                    if let Synced { time } = &state.transition_frontier.sync {
                        eprintln!(
                            "node: {this_node_id}, is synced at {:?}",
                            SystemTime::from(*time)
                        );

                        if let Some(head) = state.transition_frontier.best_chain.first() {
                            if !head.is_genesis() {
                                return true;
                            }
                        }
                    }

                    false
                },
            )
            .await
            .unwrap();
    }
}
