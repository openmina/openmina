use std::time::Duration;

use crate::{
    node::RustNodeTestingConfig,
    scenario::{ListenerNode, ScenarioStep},
    scenarios::cluster_runner::ClusterRunner,
};

/// Local test to ensure that the Openmina node can connect to an existing OCaml testnet.
/// Launch an Openmina node and connect it to seed nodes of the public (or private) OCaml testnet.
/// Run the simulation until:
/// * Number of known peers is greater than or equal to the maximum number of peers.
/// * Number of connected peers is greater than or equal to some threshold.
/// Fail the test if the specified number of steps occur but the condition is not met.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct SoloNodeBasicConnectivityInitialJoining;

impl SoloNodeBasicConnectivityInitialJoining {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        const MAX_PEERS_PER_NODE: usize = 100;
        const KNOWN_PEERS: usize = 7; // current berkeley network
        const STEPS: usize = 4_000;

        let mut nodes = vec![];

        let node = runner
            .add_rust_node(RustNodeTestingConfig::berkeley_default().max_peers(MAX_PEERS_PER_NODE));

        let seeds = [
            // "/dns4/seed-1.berkeley.o1test.net/tcp/10000/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs",
            "/dns4/seed-2.berkeley.o1test.net/tcp/10001/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag",
            // "/dns4/seed-3.berkeley.o1test.net/tcp/10002/p2p/12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv",
        ];
        for seed in seeds {
            runner
                .exec_step(ScenarioStep::ConnectNodes {
                    dialer: node,
                    listener: ListenerNode::Custom(seed.parse().unwrap()),
                })
                .await
                .unwrap();
        }
        nodes.push(node);

        for step in 0..STEPS {
            tokio::time::sleep(Duration::from_millis(100)).await;

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
            for &node_id in &nodes {
                runner
                    .exec_step(ScenarioStep::AdvanceNodeTime {
                        node_id,
                        by_nanos: 100_000_000,
                    })
                    .await
                    .unwrap();

                runner
                    .exec_step(ScenarioStep::CheckTimeouts { node_id })
                    .await
                    .unwrap();

                let node = runner.node(node_id).expect("must exist");
                let ready_peers = node.state().p2p.ready_peers_iter().count();
                let known_peers = node.state().p2p.kademlia.known_peers.len();

                println!("step: {step}");
                println!("known peers: {known_peers}");
                println!("connected peers: {ready_peers}");

                // TODO: the threshold is too small, node cannot connect to many peer before the timeout
                if ready_peers >= 3 && known_peers >= KNOWN_PEERS {
                    return;
                }
            }
        }

        panic!("timeout");
    }
}
