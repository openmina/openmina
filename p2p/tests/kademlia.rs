use p2p::P2pNetworkKadBucket;
use p2p_testing::{
    cluster::{Cluster, ClusterBuilder, Listener},
    futures::TryStreamExt,
    predicates::kad_finished_bootstrap,
    rust_node::{RustNodeConfig, RustNodeId},
    stream::ClusterStreamExt,
};
use std::time::Duration;

#[tokio::test]
async fn kademlia() {
    std::env::set_var("OPENMINA_DISCOVERY_FILTER_ADDR", "false");

    let mut cluster = ClusterBuilder::new()
        .ports(11000..11200)
        .idle_duration(Duration::from_millis(100))
        .start()
        .await
        .expect("should build cluster");

    let node1 = cluster
        .add_rust_node(RustNodeConfig {
            discovery: true,
            ..Default::default()
        })
        .expect("node1");

    let config = RustNodeConfig {
        initial_peers: vec![Listener::Rust(node1)],
        discovery: true,
        ..Default::default()
    };

    let node2 = cluster.add_rust_node(config).expect("node2");

    let peer_id1 = cluster.rust_node(node1).state().my_id();
    let peer_id2 = cluster.rust_node(node2).state().my_id();

    let bootstrap_finished = cluster
        .try_stream()
        .take_during(Duration::from_secs(2))
        .try_any(kad_finished_bootstrap(node2))
        .await
        .expect("unexpected error");

    assert!(bootstrap_finished, "Bootstrap should have finished");

    let bucket = get_kad_bucket(&cluster, node2);

    assert_eq!(
        bucket.len(),
        2,
        "There must be 2 items in bucket self and seed node"
    );

    let self_peer = bucket.iter().find(|peer| peer.peer_id == peer_id2);
    assert!(self_peer.is_some(), "Self peer not found");

    let seed_peer = bucket.iter().find(|peer| peer.peer_id == peer_id1);
    assert!(seed_peer.is_some(), "Seed peer not found");

    let node3 = cluster
        .add_rust_node(RustNodeConfig {
            initial_peers: vec![Listener::Rust(node2)],
            discovery: true,
            ..Default::default()
        })
        .expect("Error creating node 3");

    let peer_id3 = cluster.rust_node(node3).state().my_id();

    let bootstrap_finished = cluster
        .try_stream()
        .take_during(Duration::from_secs(2))
        .try_any(kad_finished_bootstrap(node3))
        .await
        .expect("unexpected error");

    assert!(bootstrap_finished, "Bootstrap should have finished");
    let bucket = get_kad_bucket(&cluster, node3);

    assert_eq!(
        bucket.len(),
        3,
        "There must be 2 items in bucket self and seed node, and node1"
    );

    let self_peer = bucket.iter().find(|peer| peer.peer_id == peer_id3);
    assert!(self_peer.is_some(), "Self peer not found");

    let seed_peer = bucket.iter().find(|peer| peer.peer_id == peer_id2);
    assert!(seed_peer.is_some(), "Seed peer not found");

    let seed_peer = bucket.iter().find(|peer| peer.peer_id == peer_id1);
    assert!(seed_peer.is_some(), "Node1 peer not found");
}

#[tokio::test]
#[ignore]
async fn kademlia_incoming() {
    std::env::set_var("OPENMINA_DISCOVERY_FILTER_ADDR", "false");

    let mut cluster = ClusterBuilder::new()
        .ports(11000..11200)
        .idle_duration(Duration::from_millis(100))
        .start()
        .await
        .expect("should build cluster");

    let node1 = cluster
        .add_rust_node(RustNodeConfig::default())
        .expect("node1");

    let node2 = cluster
        .add_rust_node(RustNodeConfig {
            initial_peers: vec![Listener::Rust(node1)],
            ..Default::default()
        })
        .expect("node2");

    let peer_id1 = cluster.rust_node(node1).state().my_id();
    let peer_id2 = cluster.rust_node(node2).state().my_id();

    let bootstrap_finished = cluster
        .try_stream()
        .take_during(Duration::from_secs(2))
        .try_any(kad_finished_bootstrap(node2))
        .await
        .expect("unexpected error");

    assert!(bootstrap_finished, "Bootstrap should have finished");

    let bucket = get_kad_bucket(&cluster, node2);

    assert_eq!(
        bucket.len(),
        2,
        "There must be 2 items in bucket self and seed node"
    );

    let self_peer = bucket.iter().find(|peer| peer.peer_id == peer_id2);
    assert!(self_peer.is_some(), "Self peer not found");

    let seed_peer = bucket.iter().find(|peer| peer.peer_id == peer_id1);
    assert!(seed_peer.is_some(), "Seed peer not found");

    let bucket = get_kad_bucket(&cluster, node1);

    assert_eq!(
        bucket.len(),
        2,
        "There must be 2 items in bucket self and connecting node"
    );

    let self_peer = bucket.iter().find(|peer| peer.peer_id == peer_id1);
    assert!(self_peer.is_some(), "Self peer not found");

    let connecting_peer = bucket.iter().find(|peer| peer.peer_id == peer_id2);
    assert!(connecting_peer.is_some(), "Connecting peer not found");
}

/// Returns only first bucket from node
fn get_kad_bucket(cluster: &Cluster, node: RustNodeId) -> &P2pNetworkKadBucket<20> {
    cluster
        .rust_node(node)
        .state()
        .network
        .scheduler
        .discovery_state()
        .expect("Must be ready")
        .routing_table
        .buckets
        .first()
        .expect("Must have at least one bucket")
}
