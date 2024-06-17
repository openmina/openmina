#![allow(warnings)]

use std::time::Duration;

use libp2p::Multiaddr;
use node::p2p::{connection::outgoing::P2pConnectionOutgoingInitOpts, PeerId};
use rand::Rng;

use crate::{
    node::{DaemonJson, OcamlNodeTestingConfig, RustNodeTestingConfig},
    scenario::{ListenerNode, ScenarioStep},
    scenarios::ClusterRunner,
};

/// Local test to ensure that the Openmina node can accept a connection from an existing OCaml node.
/// Launch an Openmina node and connect it to seed nodes of the public (or private) OCaml testnet.
/// Wait for the Openmina node to complete peer discovery.
/// Run a new OCaml node, specifying the Openmina node under testing as the initial peer.
/// Run the simulation until: OCaml node connects to Openmina node and Openmina node accepts the incoming connection.
/// Fail the test if the specified number of steps occur but the condition is not met.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct SoloNodeBasicConnectivityAcceptIncoming;

impl SoloNodeBasicConnectivityAcceptIncoming {
    pub async fn run(self, mut runner: ClusterRunner<'_>) {
        const MAX_PEERS_PER_NODE: usize = 100;
        const KNOWN_PEERS: usize = 7; // current berkeley network
        const STEPS: usize = 6_000;
        const STEP_DELAY: Duration = Duration::from_millis(200);

        let seeds_var = std::env::var("OPENMINA_SCENARIO_SEEDS");
        let seeds = seeds_var.as_ref().map_or_else(
            |_| node::p2p::BERKELEY_SEEDS.to_vec(),
            |val| val.split_whitespace().collect(),
        );

        let initial_peers = seeds
            .iter()
            .map(|s| s.parse::<Multiaddr>().unwrap())
            .map(|maddr| P2pConnectionOutgoingInitOpts::try_from(&maddr).unwrap())
            .map(ListenerNode::from)
            .collect::<Vec<_>>();
        eprintln!("set max peers per node: {MAX_PEERS_PER_NODE}");
        for seed in seeds {
            eprintln!("add initial peer: {seed}");
        }
        let config = RustNodeTestingConfig::devnet_default()
            .ask_initial_peers_interval(Duration::from_secs(3600))
            .max_peers(MAX_PEERS_PER_NODE)
            .initial_peers(initial_peers)
            .with_peer_id(rand::thread_rng().gen());

        let node_id = runner.add_rust_node(config);
        let node_addr = runner.node(node_id).unwrap().dial_addr();

        eprintln!("launch Openmina node, id: {node_id}, addr: {node_addr}");

        let mut ocaml_node = None::<PeerId>;

        for step in 0..STEPS {
            tokio::time::sleep(STEP_DELAY).await;

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

            let node = runner.node(node_id).expect("must exist");
            let ready_peers = node.state().p2p.ready_peers_iter().count();
            let my_id = node.state().p2p.my_id();
            let known_peers: usize = node
                .state()
                .p2p
                .ready()
                .and_then(|p2p| p2p.network.scheduler.discovery_state())
                .map_or(0, |discovery_state| {
                    discovery_state
                        .routing_table
                        .closest_peers(&my_id.into())
                        .count()
                });

            println!("step: {step}");
            println!("known peers: {known_peers}");
            println!("connected peers: {ready_peers}");

            // TODO: the threshold is too small, node cannot connect to many peer before the timeout
            if ready_peers >= KNOWN_PEERS && known_peers >= KNOWN_PEERS || step >= 1000 {
                eprintln!("step: {step}");
                eprintln!("known peers: {known_peers}");
                eprintln!("connected peers: {ready_peers}");

                let ocaml_peer_id = if let Some(peer_id) = ocaml_node.as_ref() {
                    *peer_id
                } else {
                    let node_id = runner.add_ocaml_node(OcamlNodeTestingConfig {
                        initial_peers: vec![node_addr.clone()],
                        daemon_json: DaemonJson::Custom(
                            "/var/lib/coda/config_dc6bf78b.json".to_owned(),
                        ),
                        block_producer: None,
                    });
                    let node = runner.ocaml_node(node_id).unwrap();
                    eprintln!("launching OCaml node {}", node.dial_addr());

                    ocaml_node = Some(node.peer_id());
                    node.peer_id()
                };
                let node = runner.node(node_id).expect("must exist");
                if let Some((peer_id, s)) = node
                    .state()
                    .p2p
                    .ready_peers_iter()
                    .find(|(peer_id, _)| *peer_id == &ocaml_peer_id)
                {
                    eprintln!("accept incoming connection from OCaml node: {peer_id}");
                    assert!(s.is_incoming);
                    return;
                }
            }
        }

        panic!("timeout");
    }
}
