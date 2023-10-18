use std::time::Duration;

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

async fn run_inner() {
    const TOTAL_PEERS: usize = 20;
    const STEPS_PER_PEER: usize = 10;
    const EXTRA_STEPS: usize = 400;
    const MAX_PEERS_PER_NODE: usize = 12;

    let mut cluster = Cluster::new(ClusterConfig::default());
    let seed_node =
        cluster.add_rust_node(RustNodeTestingConfig::berkeley_default().max_peers(TOTAL_PEERS));
    let mut nodes = vec![seed_node];

    for step in 0..(TOTAL_PEERS * STEPS_PER_PEER + EXTRA_STEPS) {
        tokio::time::sleep(Duration::from_millis(100)).await;

        if step % STEPS_PER_PEER == 0 && nodes.len() < TOTAL_PEERS {
            let node = cluster.add_rust_node(
                RustNodeTestingConfig::berkeley_default().max_peers(MAX_PEERS_PER_NODE),
            );
            cluster
                .exec_step(ScenarioStep::ConnectNodes {
                    dialer: node,
                    listener: ListenerNode::Rust(seed_node),
                })
                .await
                .unwrap();
            nodes.push(node);
        }

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
        }
    }

    for node_id in nodes {
        let node = cluster.node(node_id).expect("node must exist");
        let p2p = &node.state().p2p;
        let ready_peers = p2p.ready_peers_iter().count();
        // each node know all nodes
        assert_eq!(p2p.known_peers.len(), TOTAL_PEERS);
        // each node connected to some peers
        assert!(ready_peers >= 3);
        // maximum is not exceeded
        let max_peers = if node_id == seed_node {
            TOTAL_PEERS
        } else {
            MAX_PEERS_PER_NODE
        };
        assert!(ready_peers <= max_peers);
    }
}
