use std::time::Duration;

use p2p::PeerId;
use p2p_testing::{
    cluster::{Cluster, ClusterBuilder, NodeId},
    libp2p_node::Libp2pNodeConfig,
    rust_node::{RustNodeConfig, RustNodeId},
    utils::{
        peer_ids, rust_nodes_from_default_config, try_wait_for_all_nodes_to_connect,
        try_wait_for_nodes_to_connect, wait_for_all_nodes_to_listen,
    },
};

/// Asserts that Rust node `id` has a peer `peer_id`, and it is in ready state.
fn assert_peer_is_ready(cluster: &Cluster, id: RustNodeId, peer_id: PeerId) {
    let state = cluster.rust_node(id).state();
    let peer = state.peers.get(&peer_id).expect("peer should exist");
    assert!(
        peer.status.as_ready().is_some(),
        "peer's status should be ready, but it {:#?}",
        peer.status
    )
}

/// Asserts that the Rust node `id` has only one connection, with specified `peer_id`.
fn assert_single_connection(cluster: &Cluster, id: RustNodeId, peer_id: PeerId) {
    let state = cluster.rust_node(id).state();
    let my_id = cluster.peer_id(id);
    let mut found = None;
    for (addr, conn_state) in &state.network.scheduler.connections {
        let conn_peer_id = conn_state.peer_id();
        assert!(
            conn_peer_id.is_some(),
            "{my_id}: non-initialized connection to {addr}"
        );
        if conn_peer_id == Some(&peer_id) {
            assert!(found.is_none(), "should be only one connection");
            found = Some(conn_state);
        }
    }
}

/// Tests that a Rust node can connect to another Rust node.
#[tokio::test]
async fn rust_to_rust() -> anyhow::Result<()> {
    let mut cluster = ClusterBuilder::default()
        .ports_with_len(10)
        .total_duration(Duration::from_secs(10))
        .start()
        .await?;

    let rust_node = cluster.add_rust_node(RustNodeConfig::default())?;
    let rust_node1 = cluster.add_rust_node(RustNodeConfig::default())?;
    let peer_id = cluster.peer_id(rust_node1);

    let listening =
        wait_for_all_nodes_to_listen(&mut cluster, [rust_node1], Duration::from_secs(2)).await;
    assert!(listening);

    cluster.connect(rust_node, rust_node1)?;

    let connected =
        try_wait_for_nodes_to_connect(&mut cluster, [(rust_node, peer_id)], Duration::from_secs(5))
            .await?;
    assert!(connected);

    assert_peer_is_ready(&cluster, rust_node, peer_id);

    Ok(())
}

/// Tests that a Rust node can connect to a libp2p client.
#[tokio::test]
async fn rust_to_libp2p() -> anyhow::Result<()> {
    let mut cluster = ClusterBuilder::default()
        .ports_with_len(10)
        .total_duration(Duration::from_secs(10))
        .start()
        .await?;

    let rust_node = cluster.add_rust_node(RustNodeConfig::default())?;
    let libp2p_node = cluster.add_libp2p_node(Libp2pNodeConfig::default())?;
    let peer_id = cluster.peer_id(libp2p_node);

    let listening =
        wait_for_all_nodes_to_listen(&mut cluster, [libp2p_node], Duration::from_secs(2)).await;
    assert!(listening);

    cluster.connect(rust_node, libp2p_node)?;

    let connected =
        try_wait_for_nodes_to_connect(&mut cluster, [(rust_node, peer_id)], Duration::from_secs(5))
            .await?;
    assert!(connected);

    assert_peer_is_ready(&cluster, rust_node, peer_id);

    Ok(())
}

/// Tests that a libp2p node can connect to a Rust node.
#[tokio::test]
async fn libp2p_to_rust() -> anyhow::Result<()> {
    let mut cluster = ClusterBuilder::default()
        .ports_with_len(10)
        .total_duration(Duration::from_secs(10))
        .start()
        .await?;

    let rust_node = cluster.add_rust_node(RustNodeConfig::default())?;
    let libp2p_node = cluster.add_libp2p_node(Libp2pNodeConfig::default())?;
    let peer_id = cluster.peer_id(libp2p_node);

    let listening =
        wait_for_all_nodes_to_listen(&mut cluster, [rust_node], Duration::from_secs(2)).await;
    assert!(listening);

    cluster.connect(libp2p_node, rust_node)?;

    let connected =
        try_wait_for_nodes_to_connect(&mut cluster, [(rust_node, peer_id)], Duration::from_secs(5))
            .await?;
    assert!(connected);

    assert_peer_is_ready(&cluster, rust_node, peer_id);

    Ok(())
}

/// Tests that a Rust node can connect to another Rust node.
#[tokio::test]
async fn mutual_rust_to_rust() -> anyhow::Result<()> {
    let mut cluster = ClusterBuilder::default()
        .ports_with_len(10)
        .total_duration(Duration::from_secs(10))
        .start()
        .await?;

    let [node1, node2] = rust_nodes_from_default_config(&mut cluster)?;
    let [peer_id1, peer_id2] = peer_ids(&cluster, [node1, node2]);

    let listening =
        wait_for_all_nodes_to_listen(&mut cluster, [node1, node2], Duration::from_secs(2)).await;
    assert!(listening);

    cluster.connect(node1, node2)?;
    cluster.connect(node2, node1)?;

    let connected = try_wait_for_nodes_to_connect(
        &mut cluster,
        [(node1, peer_id2), (node2, peer_id1)],
        Duration::from_secs(5),
    )
    .await?;
    assert!(connected);

    // try_run_cluster(&mut cluster, Duration::from_secs(2)).await?;

    assert_peer_is_ready(&cluster, node1, peer_id2);
    assert_peer_is_ready(&cluster, node2, peer_id1);

    assert_single_connection(&cluster, node1, peer_id2);
    assert_single_connection(&cluster, node2, peer_id1);
    Ok(())
}

/// Tests that a many Rust nodes can connect to each other at the same time.
#[tokio::test]
async fn mutual_rust_to_rust_many() -> anyhow::Result<()> {
    const NUM: u16 = 16;
    let mut cluster = ClusterBuilder::default()
        .ports_with_len(NUM * 2)
        .total_duration(Duration::from_secs(60))
        .start()
        .await?;

    let nodes = rust_nodes_from_default_config::<{ NUM as usize }>(&mut cluster)?;

    let node_to_node = nodes.into_iter().flat_map(|node| {
        nodes
            .into_iter()
            .filter_map(move |other_node| (node != other_node).then(|| (node, other_node)))
    });

    let listening = wait_for_all_nodes_to_listen(&mut cluster, nodes, Duration::from_secs(2)).await;
    assert!(listening);

    node_to_node
        .clone()
        .map(|(n1, n2)| cluster.connect(n1, n2))
        .collect::<Result<_, _>>()?;
    let node_to_peer = node_to_node
        .map(|(n1, n2)| (n1, cluster.peer_id(n2)))
        .collect::<Vec<_>>();

    let connected =
        try_wait_for_nodes_to_connect(&mut cluster, node_to_peer.clone(), Duration::from_secs(10))
            .await?;
    assert!(connected);

    for (node, peer_id) in node_to_peer {
        assert_peer_is_ready(&cluster, node, peer_id);
        assert_single_connection(&cluster, node, peer_id);
    }

    Ok(())
}

/// Tests that a Rust node can resolve mutual connection between itself and a libp2p-based node.
#[tokio::test]
async fn mutual_rust_to_libp2p() -> anyhow::Result<()> {
    let mut cluster = ClusterBuilder::default()
        .ports_with_len(10)
        .total_duration(Duration::from_secs(10))
        .start()
        .await?;

    let rust_node = cluster.add_rust_node(RustNodeConfig::default())?;
    let libp2p_node = cluster.add_libp2p_node(Libp2pNodeConfig::default())?;
    let [peer_id1, peer_id2] = peer_ids(
        &cluster,
        [NodeId::from(rust_node), NodeId::from(libp2p_node)],
    );

    let listening = wait_for_all_nodes_to_listen(
        &mut cluster,
        [NodeId::from(rust_node), NodeId::from(libp2p_node)],
        Duration::from_secs(2),
    )
    .await;
    assert!(listening);

    cluster.connect(rust_node, libp2p_node)?;
    cluster.connect(libp2p_node, rust_node)?;

    let connected = try_wait_for_all_nodes_to_connect(
        &mut cluster,
        [(rust_node.into(), peer_id2), (libp2p_node.into(), peer_id1)],
        Duration::from_secs(5),
    )
    .await?;
    assert!(connected);

    // try_run_cluster(&mut cluster, Duration::from_secs(2)).await?;

    assert_peer_is_ready(&cluster, rust_node, peer_id2);

    assert_single_connection(&cluster, rust_node, peer_id2);
    Ok(())
}

/// Tests that a Rust node can resolve mutual connection between itself and a libp2p-based node.
#[tokio::test]
#[ignore = "TODO: see https://github.com/openmina/openmina/issues/399"]
async fn mutual_rust_to_libp2p_port_reuse() -> anyhow::Result<()> {
    let mut cluster = ClusterBuilder::default()
        .ports_with_len(10)
        .total_duration(Duration::from_secs(10))
        .start()
        .await?;

    let rust_node = cluster.add_rust_node(RustNodeConfig::default())?;
    let libp2p_node = cluster.add_libp2p_node(Libp2pNodeConfig {
        port_reuse: true,
        ..Default::default()
    })?;
    let [peer_id1, peer_id2] = peer_ids(
        &cluster,
        [NodeId::from(rust_node), NodeId::from(libp2p_node)],
    );

    let listening = wait_for_all_nodes_to_listen(
        &mut cluster,
        [NodeId::from(rust_node), NodeId::from(libp2p_node)],
        Duration::from_secs(2),
    )
    .await;
    assert!(listening);

    cluster.connect(rust_node, libp2p_node)?;
    cluster.connect(libp2p_node, rust_node)?;

    let connected = try_wait_for_all_nodes_to_connect(
        &mut cluster,
        [(rust_node.into(), peer_id2), (libp2p_node.into(), peer_id1)],
        Duration::from_secs(5),
    )
    .await?;
    assert!(connected);

    // try_run_cluster(&mut cluster, Duration::from_secs(2)).await?;

    assert_peer_is_ready(&cluster, rust_node, peer_id2);

    assert_single_connection(&cluster, rust_node, peer_id2);
    Ok(())
}
