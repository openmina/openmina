use std::time::Duration;

use libp2p::Multiaddr;
use node::{
    event_source::Event,
    p2p::{connection::outgoing::P2pConnectionOutgoingInitOpts, P2pEvent},
};

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
        const STEPS: usize = 6_000;
        const STEP_DELAY: Duration = Duration::from_millis(200);

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
        eprintln!("set max peers per node: {MAX_PEERS_PER_NODE}");
        for seed in seeds {
            eprintln!("add initial peer: {seed}");
        }
        let config = RustNodeTestingConfig::berkeley_default()
            .ask_initial_peers_interval(Duration::from_secs(3600))
            .max_peers(MAX_PEERS_PER_NODE)
            .initial_peers(initial_peers)
            .randomize_peer_id();

        let node_id = runner.add_rust_node(config);

        let full_config = &runner.node(node_id).expect("must exist").state().p2p.config;
        let this = format!(
            "/ip4/127.0.0.1/tcp/{}/p2p/{}",
            full_config.libp2p_port.unwrap(),
            libp2p::PeerId::from(full_config.identity_pub_key.peer_id())
        );
        let this_maddr = this.parse::<Multiaddr>().unwrap();
        eprintln!("launch Openmina node, id: {node_id}, addr: {this}");

        let mut ocaml_node = None::<ocaml::Node>;

        for step in 0..STEPS {
            tokio::time::sleep(STEP_DELAY).await;

            let steps = runner
                .pending_events()
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

            let node = runner.node(node_id).expect("must exist");
            let ready_peers = node.state().p2p.ready_peers_iter().count();
            let known_peers = node.state().p2p.kademlia.known_peers.len();

            println!("step: {step}");
            println!("known peers: {known_peers}");
            println!("connected peers: {ready_peers}");

            // TODO: the threshold is too small, node cannot connect to many peer before the timeout
            if ready_peers >= KNOWN_PEERS && known_peers >= KNOWN_PEERS || step >= 1000 {
                eprintln!("step: {step}");
                eprintln!("known peers: {known_peers}");
                eprintln!("connected peers: {ready_peers}");

                let ocaml_node = ocaml_node.get_or_insert_with(|| {
                    let n = ocaml::Node::spawn_with_temp_dir(
                        8302,
                        3085,
                        8301,
                        [this_maddr.to_string()],
                    )
                    .expect("ocaml node");
                    eprintln!("launching OCaml node: {}", n.peer_id());
                    n
                });
                if let Some((peer_id, s)) = node
                    .state()
                    .p2p
                    .ready_peers_iter()
                    .find(|(peer_id, _)| *peer_id == &ocaml_node.peer_id)
                {
                    eprintln!("accept incoming connection from OCaml node: {peer_id}");
                    assert!(s.is_incoming);
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
