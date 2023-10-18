use std::time::Duration;

use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;

use crate::{
    cluster::ClusterConfig,
    node::RustNodeTestingConfig,
    scenario::{ListenerNode, ScenarioStep},
    Cluster,
};

pub fn run() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    let _guard = rt.enter();

    rt.block_on(run_inner())
}

fn init_opts_from_multiaddr(maddr: libp2p::Multiaddr) -> P2pConnectionOutgoingInitOpts {
    let peer_id = maddr
        .iter()
        .find_map(|p| match p {
            libp2p::multiaddr::Protocol::P2p(v) => Some(libp2p::PeerId::from_multihash(v).unwrap()),
            _ => None,
        })
        .unwrap();

    P2pConnectionOutgoingInitOpts::LibP2P {
        peer_id: peer_id.into(),
        maddr,
    }
}

async fn run_inner() {
    const MAX_PEERS_PER_NODE: usize = 12;
    const STEPS: usize = 40;

    let mut cluster = Cluster::new(ClusterConfig::default());
    let mut nodes = vec![];

    let seed_maddr =
        "/dns4/seed-1.berkeley.o1test.net/tcp/10000/p2p/12D3KooWAdgYL6hv18M3iDBdaK1dRygPivSfAfBNDzie6YqydVbs"
            .parse()
            .unwrap();

    let node = cluster
        .add_rust_node(RustNodeTestingConfig::berkeley_default().max_peers(MAX_PEERS_PER_NODE));
    cluster
        .exec_step(ScenarioStep::ConnectNodes {
            dialer: node,
            listener: ListenerNode::Custom(init_opts_from_multiaddr(seed_maddr)),
        })
        .await
        .unwrap();
    nodes.push(node);

    for step in 0..STEPS {
        tokio::time::sleep(Duration::from_millis(1000)).await;

        let steps = cluster
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
            cluster.exec_step(step).await.unwrap();
        }
        for &node_id in &nodes {
            cluster
                .exec_step(ScenarioStep::CheckTimeouts { node_id })
                .await
                .unwrap();

            let node = cluster.node(node_id).expect("must exist");
            let ready_peers = node.state().p2p.ready_peers_iter().count();
            let known_peers = node.state().p2p.known_peers.len();

            println!("step: {step}");
            println!("known peers: {known_peers}");
            println!("connected peers: {ready_peers}");

            if ready_peers >= 3 && known_peers >= MAX_PEERS_PER_NODE {
                return;
            }
        }
    }

    panic!("timeout");
}
