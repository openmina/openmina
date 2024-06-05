use p2p::{identity::SecretKey, P2pNetworkKadBucket, PeerId};
use p2p_testing::{
    cluster::{Cluster, ClusterBuilder, ClusterEvent, Listener},
    event::{allow_disconnections, RustNodeEvent},
    futures::TryStreamExt,
    predicates::kad_finished_bootstrap,
    rust_node::{RustNodeConfig, RustNodeId},
    stream::ClusterStreamExt,
    utils::{
        peer_ids, try_wait_for_all_nodes_with_value, try_wait_for_nodes_to_connect,
        try_wait_for_nodes_to_listen,
    },
};
use std::{future::ready, net::Ipv4Addr, time::Duration};

#[tokio::test]
async fn kademlia_routing_table() {
    std::env::set_var("OPENMINA_DISCOVERY_FILTER_ADDR", "false");

    let mut cluster = ClusterBuilder::new()
        .ports_with_len(10)
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
async fn kademlia_incoming_routing_table() {
    std::env::set_var("OPENMINA_DISCOVERY_FILTER_ADDR", "false");

    let mut cluster = ClusterBuilder::new()
        .ports_with_len(10)
        .idle_duration(Duration::from_millis(100))
        .start()
        .await
        .expect("should build cluster");

    let node1 = cluster.add_rust_node(rust_config()).expect("node1");

    let node2 = cluster
        .add_rust_node(RustNodeConfig {
            initial_peers: vec![Listener::Rust(node1)],
            ..rust_config()
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

fn rust_config() -> RustNodeConfig {
    RustNodeConfig::default().with_discovery(true)
}

#[tokio::test]
async fn bootstrap_no_peers() -> anyhow::Result<()> {
    let mut cluster = ClusterBuilder::new()
        .ports_with_len(3)
        .idle_duration(Duration::from_millis(100))
        .is_error(allow_disconnections)
        .start()
        .await?;

    // node will not initiate Kademlia bootstrap unless there is at least one peer in the table (possibly offline)
    let random_peer = Listener::SocketPeerId(
        (Ipv4Addr::LOCALHOST, cluster.next_port()?).into(),
        SecretKey::rand().public_key().peer_id(),
    );
    let node = cluster.add_rust_node(rust_config().with_initial_peers([random_peer]))?;

    let bootstrapped = cluster
        .try_stream()
        .take_during(Duration::from_secs(2))
        .try_any(|event| {
            ready(matches!(
                event,
                ClusterEvent::Rust {
                    id,
                    event: RustNodeEvent::KadBootstrapFinished,
                }
                if id == node
            ))
        })
        .await?;

    assert!(bootstrapped);

    Ok(())
}

/// Tests simple discovery use-case.
///
/// A node should be able to discover and connect a node connected to the seed node.
#[tokio::test]
async fn discovery_seed_single_peer() -> anyhow::Result<()> {
    std::env::set_var("OPENMINA_DISCOVERY_FILTER_ADDR", false.to_string());
    let mut cluster = ClusterBuilder::new()
        .ports_with_len(6)
        .idle_duration(Duration::from_millis(100))
        .start()
        .await?;

    let [seed, peer, node_ut] =
        p2p_testing::utils::rust_nodes_from_config(&mut cluster, rust_config())?;
    let peer_id = cluster.peer_id(peer);

    assert!(
        try_wait_for_nodes_to_listen(&mut cluster, [seed], Duration::from_secs(2)).await?,
        "seed should listen"
    );
    cluster.connect(peer, seed)?;

    // wait for all peers to be identified by the seed
    wait_for_identify(&mut cluster, [(seed, peer_id)], Duration::from_secs(10)).await;

    println!("========= connect node under test");
    cluster.connect(node_ut, seed)?;

    assert!(
        try_wait_for_nodes_to_connect(&mut cluster, [(node_ut, peer_id)], Duration::from_secs(30))
            .await?,
        "peer should be connected"
    );

    Ok(())
}

#[tokio::test]
async fn discovery_seed_multiple_peers() -> anyhow::Result<()> {
    std::env::set_var("OPENMINA_DISCOVERY_FILTER_ADDR", false.to_string());
    const PEERS: usize = 15;
    let mut cluster = ClusterBuilder::new()
        .ports_with_len(PEERS as u16 * 2 + 4)
        .idle_duration(Duration::from_millis(100))
        .is_error(allow_disconnections)
        .start()
        .await?;

    let [seed, nodes @ ..]: [_; PEERS + 1] =
        p2p_testing::utils::rust_nodes_from_config(&mut cluster, rust_config())?;
    let peer_ids = peer_ids(&cluster, nodes);

    assert!(
        try_wait_for_nodes_to_listen(&mut cluster, [seed], Duration::from_secs(2)).await?,
        "should listen"
    );
    for node in nodes {
        cluster.connect(node, seed)?;
    }

    // wait for all peers to be identified by the seed
    wait_for_identify(
        &mut cluster,
        peer_ids.map(|peer_id| (seed, peer_id)),
        Duration::from_secs(10),
    )
    .await;

    println!("========= add new node");
    let node = cluster.add_rust_node(rust_config())?;
    cluster.connect(node, seed)?;

    // time to reconnect to all newly discovered peers + some margin
    let dur = Duration::from_secs(PEERS as u64 + 5);
    assert!(
        try_wait_for_nodes_to_connect(&mut cluster, peer_ids.map(|peer_id| (node, peer_id)), dur)
            .await?,
        "all peers should be connected\n{}",
        serde_json::to_string_pretty(&cluster.rust_node(node).state()).unwrap()
    );

    Ok(())
}

async fn wait_for_identify<I>(cluster: &mut Cluster, nodes_peers: I, time: Duration)
where
    I: IntoIterator<Item = (RustNodeId, PeerId)>,
{
    let identified = try_wait_for_all_nodes_with_value(cluster, nodes_peers, time, |event| {
        if let RustNodeEvent::Identify { peer_id, .. } = event {
            Some(peer_id)
        } else {
            None
        }
    })
    .await
    .expect("no errors");
    assert!(identified, "all peers should be identified");
}
