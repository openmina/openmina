use std::time::Duration;

use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;

use crate::scenarios::cluster_runner::ClusterRunner;
use crate::{node::RustNodeTestingConfig, scenario::ScenarioStep};

use crate::ocaml::{Node, NodeKey};

/// Global test with OCaml nodes.
/// Run an OCaml node as a seed node. Run three normal OCaml nodes connecting only to the seed node.
/// Wait 3 minutes for the OCaml nodes to start.
/// Run the Openmina node (application under test).
/// Wait for the Openmina node to complete peer discovery and connect to all OCaml nodes.
/// (This step ensures that the Openmina node can discover OCaml nodes).
/// Run another OCaml node that connects only to the seed node.
/// So this additional OCaml node only knows the address of the seed node.
/// Wait for the additional OCaml node to initiate a connection to the Openmina node.
/// (This step ensures that the OCaml node can discover the Openmina node).
/// Fail the test on the timeout.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct MultiNodeBasicConnectivityPeerDiscovery;

impl MultiNodeBasicConnectivityPeerDiscovery {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        const STEPS: usize = 4_000;
        const STEP_DELAY: Duration = Duration::from_millis(200);
        const TOTAL_OCAML_NODES: u16 = 4;
        const PAUSE_UNTIL_OCAML_NODES_READY: Duration = Duration::from_secs(180);

        let seed_a_key = NodeKey::generate();
        let mut seed_a = Node::spawn_with_key(seed_a_key, 8302, 3085, 8301, true, Some(&[]));

        tokio::time::sleep(Duration::from_secs(60)).await;
        let nodes = (1..TOTAL_OCAML_NODES)
            .map(|i| {
                Node::spawn(
                    8302 + i * 10,
                    3085 + i * 10,
                    8301 + i * 10,
                    Some(&[&seed_a.local_addr()]),
                )
            })
            .collect::<Vec<_>>();

        tokio::time::sleep(PAUSE_UNTIL_OCAML_NODES_READY).await;

        let config = RustNodeTestingConfig::berkeley_default()
            .ask_initial_peers_interval(Duration::from_secs(3600))
            .max_peers(100)
            .libp2p_port(10000)
            .initial_peers(
                nodes
                    .iter()
                    .chain(std::iter::once(&seed_a))
                    .map(|n| P2pConnectionOutgoingInitOpts::try_from(&n.local_addr()).unwrap())
                    .collect(),
            );
        let node_id = runner.add_rust_node(config);

        let mut additional_ocaml_node = None::<Node>;

        let mut timeout = STEPS;
        loop {
            if timeout == 0 {
                break;
            }
            timeout -= 1;

            tokio::time::sleep(STEP_DELAY).await;

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

            runner
                .exec_step(ScenarioStep::AdvanceNodeTime {
                    node_id,
                    by_nanos: STEP_DELAY.as_nanos() as _,
                })
                .await
                .unwrap();

            runner
                .exec_step(ScenarioStep::CheckTimeouts { node_id })
                .await
                .unwrap();

            let this = runner.node(node_id).unwrap();
            // finish discovering
            if this.state().p2p.kademlia.saturated.is_some() {
                // the node must find all already running OCaml nodes
                // assert_eq!(this.state().p2p.peers.len(), TOTAL_OCAML_NODES as usize);
                additional_ocaml_node.get_or_insert_with(|| {
                    Node::spawn(9000, 4000, 9001, Some(&[&seed_a.local_addr()]))
                });
            }

            if let Some(additional_ocaml_node) = &additional_ocaml_node {
                let peer_id = additional_ocaml_node.peer_id();

                if runner
                    .node(node_id)
                    .unwrap()
                    .state()
                    .p2p
                    .peers
                    .iter()
                    .filter(|(id, n)| n.is_libp2p && **id == peer_id.into())
                    .filter_map(|(_, n)| n.status.as_ready())
                    .find(|n| n.is_incoming)
                    .is_some()
                {
                    break;
                }
            }
        }

        for mut node in nodes {
            node.kill();
        }
        seed_a.kill();
        additional_ocaml_node.as_mut().map(Node::kill);

        if timeout == 0 {
            panic!("timeout");
        }
    }
}
