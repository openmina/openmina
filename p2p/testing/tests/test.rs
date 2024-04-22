use std::time::Duration;

use futures::TryStreamExt;
use testing::{
    cluster::ClusterBuilder,
    predicates::{default_errors, listener_is_ready, peer_is_connected},
    rust_node::RustNodeConfig,
    stream::ClusterStreamExt,
};

#[tokio::test]
async fn test() {
    let mut cluster = ClusterBuilder::new()
        .ports(11000..11200)
        .idle_duration(Duration::from_millis(100))
        .start()
        .await
        .expect("should build cluster");

    let node = cluster
        .add_rust_node(RustNodeConfig::default())
        .expect("node");

    let mut cluster = cluster
        .map_errors(default_errors)
        .take_during(Duration::from_secs(20));

    for _ in 0..50 {
        let listener = cluster
            .add_rust_node(RustNodeConfig::default())
            .expect("listener node");
        let peer_id = cluster.rust_node(listener).state().my_id();

        let listener_is_ready = (&mut cluster)
            .try_any(listener_is_ready(listener))
            .await
            .expect("unexpected error");
        assert!(listener_is_ready, "listener should be ready");

        cluster.connect(node, listener);

        let connected = (&mut cluster)
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
