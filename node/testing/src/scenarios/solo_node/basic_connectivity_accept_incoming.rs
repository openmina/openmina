use std::time::Duration;

use libp2p::Multiaddr;
use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;

use crate::{
    node::RustNodeTestingConfig, ocaml, scenario::ScenarioStep,
    scenarios::cluster_runner::ClusterRunner,
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
        const STEPS: usize = 1_000;

        let seeds = [
            "/dns4/seed-1.berkeley.o1test.net/tcp/10000/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs",
            "/dns4/seed-2.berkeley.o1test.net/tcp/10001/p2p/12D3KooWLjs54xHzVmMmGYb7W5RVibqbwD1co7M2ZMfPgPm7iAag",
            "/dns4/seed-3.berkeley.o1test.net/tcp/10002/p2p/12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv",
        ];

        let initial_peers = seeds
            .into_iter()
            .map(|s| s.parse::<Multiaddr>().unwrap())
            .map(|maddr| P2pConnectionOutgoingInitOpts::try_from(&maddr).unwrap())
            .collect::<Vec<_>>();
        let config = RustNodeTestingConfig::berkeley_default()
            .ask_initial_peers_interval(Duration::from_secs(3600))
            .max_peers(MAX_PEERS_PER_NODE)
            .initial_peers(initial_peers);

        let node_id = runner.add_rust_node(config);

        let mut ocaml_node = None::<ocaml::Node>;

        let full_config = &runner.node(node_id).expect("must exist").state().p2p.config;
        let this = format!(
            "/ip4/127.0.0.1/tcp/{}/p2p/{}",
            full_config.libp2p_port.unwrap(),
            libp2p::PeerId::from(full_config.identity_pub_key.peer_id())
        );
        dbg!(&this);
        let this_maddr = this.parse::<Multiaddr>().unwrap();

        for step in 0..STEPS {
            tokio::time::sleep(Duration::from_millis(400)).await;

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
            if ready_peers >= KNOWN_PEERS && known_peers >= KNOWN_PEERS {
                let ocaml_node = ocaml_node.get_or_insert_with(|| {
                    ocaml::Node::spawn_berkeley(8302, 3085, Some(&[&this_maddr]))
                });
                if node
                    .state()
                    .p2p
                    .ready_peers_iter()
                    .find(|(peer_id, _)| **peer_id == ocaml_node.peer_id.into())
                    .is_some()
                {
                    return;
                }
            }
        }

        if let Some(mut ocaml_node) = ocaml_node.take() {
            ocaml_node.kill();
        }

        panic!("timeout");
    }
}
