use std::time::Duration;

use p2p::{MioEvent, P2pEvent};
use p2p_testing::{
    cluster::ClusterBuilder,
    event::RustNodeEvent,
    futures::{StreamExt, TryStreamExt},
    predicates::{listener_is_ready, peer_is_connected},
    rust_node::RustNodeConfig,
    stream::ClusterStreamExt,
};

#[tokio::test]
async fn accept_connection() {
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
        .add_rust_node(RustNodeConfig::default())
        .expect("node2");

    let listener_is_ready = cluster
        .try_stream()
        .take_during(Duration::from_secs(2))
        .try_any(listener_is_ready(node1))
        .await
        .expect("unexpected error");
    assert!(listener_is_ready, "node1 should be ready");

    cluster.connect(node2, node1);
    let peer_id1 = cluster.rust_node(node1).state().my_id();

    let connected = cluster
        .try_stream()
        .take_during(Duration::from_secs(2))
        .try_any(peer_is_connected(node2, peer_id1))
        .await
        .expect("unexpected error");
    assert!(
        connected,
        "node should be able to connect to {peer_id1}: {connected:?}\nnode state: {:#?}",
        cluster.rust_node(node2).state().peers.get(&peer_id1)
    );

    {
        let mut cluster = cluster.take_during(Duration::from_secs(10));
        while let Some(event) = cluster.next().await {
            if let Some((id, event)) = event.rust() {
                if let RustNodeEvent::P2p {
                    event: P2pEvent::MioEvent(MioEvent::IncomingDataIsReady(_)),
                } = event
                {
                    let state = &cluster.rust_node(*id).state();
                    let id_state = &state.network.scheduler.identify_state;
                    println!("=== {id_state:?}");
                    for (peer, state) in &state.peers {
                        println!("=== {id:?} {peer}: {:?}", state.identify);
                    }
                }
            }
        }
    }
}
