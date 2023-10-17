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
    let mut cluster = Cluster::new(ClusterConfig::default());
    let seed_node = cluster.add_rust_node(RustNodeTestingConfig::berkeley_default());
    let mut nodes = vec![seed_node];

    for _ in 0..1000 {
        tokio::time::sleep(Duration::from_millis(100)).await;

        if nodes.len() < 100 {
            let node = cluster.add_rust_node(RustNodeTestingConfig::berkeley_default());
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
        println!("{} {}", p2p.known_peers.len(), p2p.peers.len());
    }
}
