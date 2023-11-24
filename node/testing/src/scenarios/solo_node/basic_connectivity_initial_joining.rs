use std::time::Duration;

use libp2p::Multiaddr;

use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;

use crate::{
    node::RustNodeTestingConfig, scenario::ScenarioStep, scenarios::cluster_runner::ClusterRunner,
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
        const KNOWN_PEERS: usize = 5; // current berkeley network
        const STEPS: usize = 3_000;
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
        let config = RustNodeTestingConfig::berkeley_default()
            .ask_initial_peers_interval(Duration::from_secs(3600))
            .max_peers(MAX_PEERS_PER_NODE)
            .initial_peers(initial_peers);

        let node_id = runner.add_rust_node(config);

        dbg!(libp2p::PeerId::from(
            runner
                .node(node_id)
                .expect("must exist")
                .state()
                .p2p
                .config
                .identity_pub_key
                .peer_id()
        ));

        for step in 0..STEPS {
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

            let node = runner.node(node_id).expect("must exist");
            let ready_peers = node.state().p2p.ready_peers_iter().count();
            let known_peers = node.state().p2p.kademlia.known_peers.len();

            println!("step: {step}");
            println!("known peers: {known_peers}");
            println!("connected peers: {ready_peers}");

            // TODO: the threshold is too small, node cannot connect to many peer before the timeout
            if ready_peers >= KNOWN_PEERS && known_peers >= KNOWN_PEERS {
                return;
            }
        }

        panic!("timeout");
    }
}
