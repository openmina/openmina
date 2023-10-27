use std::time::Duration;

use crate::{
    node::RustNodeTestingConfig,
    scenario::{ListenerNode, ScenarioStep},
    scenarios::cluster_runner::ClusterRunner,
};

/// Global test that aims to be deterministic.
/// Launch `TOTAL_PEERS` number of nodes with `MAX_PEERS_PER_NODE` is est as the maximum number of peers.
/// Launch a seed node where `TOTAL_PEERS` is set as the maximum number of peers.
/// Run the simulation until the following condition is satisfied:
/// * Each node is connected to a number of peers determined by the `P2pState::min_peers` method.
/// Fail the test if any node exceeds the maximum number of connections.
/// Fail the test if the specified number of steps occur but the condition is not met.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct MultiNodeBasicConnectivityInitialJoining;

impl MultiNodeBasicConnectivityInitialJoining {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        const TOTAL_PEERS: usize = 20;
        const STEPS_PER_PEER: usize = 10;
        const EXTRA_STEPS: usize = 300;
        const MAX_PEERS_PER_NODE: usize = 12;

        let seed_node =
            runner.add_rust_node(RustNodeTestingConfig::berkeley_default().max_peers(TOTAL_PEERS));
        let mut nodes = vec![seed_node];

        for step in 0..(TOTAL_PEERS * STEPS_PER_PEER + EXTRA_STEPS) {
            tokio::time::sleep(Duration::from_millis(100)).await;

            if step % STEPS_PER_PEER == 0 && nodes.len() < TOTAL_PEERS {
                let node = runner.add_rust_node(
                    RustNodeTestingConfig::berkeley_default()
                        .max_peers(MAX_PEERS_PER_NODE)
                        .ask_initial_peers_interval(Duration::ZERO),
                );
                runner
                    .exec_step(ScenarioStep::ConnectNodes {
                        dialer: node,
                        listener: ListenerNode::Rust(seed_node),
                    })
                    .await
                    .unwrap();
                nodes.push(node);
            }

            let steps = runner
                .pending_events()
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

            let mut conditions_met = true;
            for &node_id in &nodes {
                runner
                    .exec_step(ScenarioStep::CheckTimeouts { node_id })
                    .await
                    .unwrap();

                let node = runner.node(node_id).expect("node must exist");
                let p2p: &node::p2p::P2pState = &node.state().p2p;
                let ready_peers = p2p.ready_peers_iter().count();

                // each node connected to some peers
                conditions_met &= ready_peers >= node.state().p2p.min_peers();

                // maximum is not exceeded
                let max_peers = if node_id == seed_node {
                    TOTAL_PEERS
                } else {
                    MAX_PEERS_PER_NODE
                };
                assert!(ready_peers <= max_peers);
            }

            if conditions_met {
                return;
            }
        }

        for node_id in &nodes {
            let node = runner.node(*node_id).expect("node must exist");
            let p2p: &node::p2p::P2pState = &node.state().p2p;
            let ready_peers = p2p.ready_peers_iter().count();
            // each node connected to some peers
            println!(
                "must hold {ready_peers} >= {}",
                node.state().p2p.min_peers()
            );
        }

        for node_id in nodes {
            let node = runner.node(node_id).expect("node must exist");
            eprintln!(
                "{node_id:?} - {} - p2p state: {:#?}",
                &node.state().p2p.my_id(),
                &node.state().p2p.peers
            );
        }

        assert!(false);
    }
}
