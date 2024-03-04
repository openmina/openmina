use std::time::Duration;

use node::event_source::Event;
use node::p2p::{P2pEvent, PeerId};

use crate::cluster::ClusterOcamlNodeId;
use crate::node::{DaemonJson, OcamlNodeTestingConfig, OcamlStep};
use crate::scenarios::cluster_runner::ClusterRunner;
use crate::{node::RustNodeTestingConfig, scenario::ScenarioStep};

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
        const PAUSE_UNTIL_OCAML_NODES_READY: Duration = Duration::from_secs(30 * 60);

        let ocaml_seed_config = OcamlNodeTestingConfig {
            initial_peers: Vec::new(),
            daemon_json: DaemonJson::Custom("/var/lib/coda/berkeley.json".to_owned()),
        };

        let seed_a = runner.add_ocaml_node(ocaml_seed_config.clone());
        let seed_a_dial_addr = runner.ocaml_node(seed_a).unwrap().dial_addr();

        eprintln!("launching OCaml seed node: {seed_a_dial_addr}");

        let ocaml_node_config = OcamlNodeTestingConfig {
            initial_peers: vec![seed_a_dial_addr],
            ..ocaml_seed_config
        };

        tokio::time::sleep(Duration::from_secs(60)).await;

        let nodes = (1..TOTAL_OCAML_NODES)
            .map(|_| runner.add_ocaml_node(ocaml_node_config.clone()))
            .collect::<Vec<_>>();

        // wait for ocaml nodes to be ready
        for node in &nodes {
            runner
                .exec_step(ScenarioStep::Ocaml {
                    node_id: *node,
                    step: OcamlStep::WaitReady {
                        timeout: PAUSE_UNTIL_OCAML_NODES_READY,
                    },
                })
                .await
                .expect("OCaml node should be ready");
        }
        eprintln!("OCaml nodes should be ready now");

        let config = RustNodeTestingConfig::berkeley_default()
            .ask_initial_peers_interval(Duration::from_secs(3600))
            .max_peers(100)
            .initial_peers(
                nodes
                    .iter()
                    .chain(std::iter::once(&seed_a))
                    .map(|node_id| (*node_id).into())
                    .collect(),
            );
        let node_id = runner.add_rust_node(config);
        eprintln!("launching Openmina node {node_id}");

        let mut additional_ocaml_node = None::<(ClusterOcamlNodeId, PeerId)>;

        let mut timeout = STEPS;
        loop {
            if timeout == 0 {
                break;
            }
            timeout -= 1;

            tokio::time::sleep(STEP_DELAY).await;

            let steps = runner
                .pending_events(true)
                .map(|(node_id, _, events)| {
                    events.map(move |(_, event)| {
                        match event {
                            Event::P2p(P2pEvent::Discovery(event)) => {
                                eprintln!("event: {event}");
                            }
                            _ => {}
                        }
                        ScenarioStep::Event {
                            node_id,
                            event: event.to_string(),
                        }
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
                if additional_ocaml_node.is_none() {
                    eprintln!("the Openmina node finished peer discovery",);
                    eprintln!(
                        "connected peers: {:?}",
                        this.state().p2p.peers.keys().collect::<Vec<_>>()
                    );
                    let node_id = runner.add_ocaml_node(ocaml_node_config.clone());
                    let node = runner.ocaml_node(node_id).unwrap();
                    eprintln!("launching additional OCaml node {}", node.dial_addr());

                    additional_ocaml_node = Some((node_id, node.peer_id()));
                }
            }

            if let Some((_, additional_ocaml_node_peer_id)) = &additional_ocaml_node {
                let peer_id = additional_ocaml_node_peer_id;

                if runner
                    .node(node_id)
                    .unwrap()
                    .state()
                    .p2p
                    .peers
                    .iter()
                    .filter(|(id, n)| n.is_libp2p && id == &peer_id)
                    .filter_map(|(_, n)| n.status.as_ready())
                    .find(|n| n.is_incoming)
                    .is_some()
                {
                    eprintln!("the additional OCaml node connected to Openmina node");
                    eprintln!("success");

                    break;
                }
            }
        }

        if timeout == 0 {
            panic!("timeout");
        }
    }
}
