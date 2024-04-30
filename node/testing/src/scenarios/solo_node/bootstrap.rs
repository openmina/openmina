use std::time::Duration;

use node::{
    p2p::connection::outgoing::P2pConnectionOutgoingInitOpts,
    transition_frontier::sync::TransitionFrontierSyncState,
};
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

        let mut timeout = TIMEOUT;
        let mut last_time = Instant::now();
        loop {
            if runner
                .wait_for_pending_events_with_timeout(Duration::from_secs(1))
                .await
            {
                let steps = runner
                    .pending_events(true)
                    .map(|(node_id, _, events)| {
                        events.map(move |(_, event)| ScenarioStep::Event {
                            node_id,
                            event: event.to_string(),
                        })
                    })
                    .flatten()
                    .collect::<Vec<_>>();

                for step in steps {
                    runner.exec_step(step).await.unwrap();
                }
            }

            runner
                .exec_step(ScenarioStep::CheckTimeouts { node_id })
                .await
                .unwrap();

            let new = Instant::now();
            let elapsed = new - last_time;
            last_time = new;

            match timeout.checked_sub(elapsed) {
                Some(new_timeout) => timeout = new_timeout,
                None => panic!("timeout"),
            }

            runner
                .exec_step(ScenarioStep::AdvanceNodeTime {
                    node_id,
                    by_nanos: elapsed.as_nanos() as _,
                })
                .await
                .unwrap();

            let this = runner.node(node_id).unwrap();
            let best_chain = &this.state().transition_frontier.best_chain;
            let sync_state = &this.state().transition_frontier.sync;
            if let Synced { time } = &sync_state {
                if best_chain.len() > 1 {
                    eprintln!(
                        "node: {node_id}, is synced at {time:?}, total elapsed {:?}",
                        TIMEOUT - timeout
                    );

                    break;
                }
            }
        }
    }
}
