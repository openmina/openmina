use std::time::Duration;

use p2p_testing::{
    cluster::ClusterBuilder,
    futures::TryStreamExt,
    predicates::{listener_is_ready, peer_is_connected},
    rust_node::RustNodeConfig,
    stream::ClusterStreamExt,
};

#[tokio::test]
async fn accept_connection() {
    const NUM: u16 = 50;
    let mut cluster = ClusterBuilder::new()
        .ports_with_len(NUM * 2 + 2)
        .idle_duration(Duration::from_millis(100))
        .start()
        .await
        .expect("should build cluster");

    let node = cluster
        .add_rust_node(RustNodeConfig::default())
        .expect("node");

    for _ in 0..50 {
        let listener = cluster
            .add_rust_node(RustNodeConfig::default())
            .expect("listener node");
        let peer_id = cluster.rust_node(listener).state().my_id();

        let listener_is_ready = cluster
            .try_stream()
            .take_during(Duration::from_secs(2))
            .try_any(listener_is_ready(listener))
            .await
            .expect("unexpected error");
        assert!(listener_is_ready, "listener should be ready");

        cluster.connect(node, listener).expect("no errors");

        let connected = cluster
            .try_stream()
            .take_during(Duration::from_secs(2))
            .try_any(peer_is_connected(node, peer_id))
            .await
            .expect("unexpected error");
        assert!(
            connected,
            "node should be able to connect to {peer_id}: {connected:?}\nnode state: {:#?}",
            cluster.rust_node(node).state().peers.get(&peer_id)
        );
    }
}
