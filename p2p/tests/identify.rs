use std::{collections::BTreeSet, time::Duration};

use p2p::{MioEvent, P2pEvent};
use p2p_testing::{
    cluster::ClusterBuilder,
    event::RustNodeEvent,
    futures::TryStreamExt,
    predicates::{listener_is_ready, peer_is_connected},
    rust_node::RustNodeConfig,
    stream::ClusterStreamExt,
};

#[tokio::test]
async fn rust_node_to_rust_node() {
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

    let peer_id1 = cluster.rust_node(node1).state().my_id();
    let peer_id2 = cluster.rust_node(node2).state().my_id();

    // wait for node1 to be ready to accept incoming conections
    let listener_is_ready = cluster
        .try_stream()
        .take_during(Duration::from_secs(2))
        .try_any(listener_is_ready(node1))
        .await
        .expect("unexpected error");
    assert!(listener_is_ready, "node1 should be ready");

    // connect node2 to node1
    cluster.connect(node2, node1).expect("no error");

    // wait for node2 to have peer_id1 (node1) as its peer in `ready` state`
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
        let mut not_identified = BTreeSet::from_iter([(node1, peer_id2), (node2, peer_id1)]);
        let mut addrs = Vec::new();
        let take_during = cluster.try_stream().take_during(Duration::from_secs(10));

        // run the cluster until both nodes have identify data about each other
        let found = take_during
            .try_any_with_rust(|id, event, state| {
                if let RustNodeEvent::P2p {
                    event: P2pEvent::MioEvent(MioEvent::IncomingDataIsReady(_)),
                } = event
                {
                    for (peer_id, state) in &state.peers {
                        if state.identify.is_some() && not_identified.remove(&(id, *peer_id)) {
                            if let Some(identify) = state.identify.as_ref() {
                                // collect address and peer id to test later
                                addrs.extend(
                                    identify
                                        .listen_addrs
                                        .iter()
                                        .map(|addr| (addr.clone(), *peer_id)),
                                );
                            }
                        }
                    }
                }
                // done when both nodes identify themselves
                not_identified.is_empty()
            })
            .await
            .expect("no errors");
        assert!(
            found,
            "nodes should be identified, but these are not: {not_identified:?}"
        );

        // for each address provided by identify, create a node and ensure it
        // can connect to that address
        for (addr, peer_id) in addrs {
            let node = cluster
                .add_rust_node(RustNodeConfig::default())
                .expect("no error");
            cluster
                .connect(node, addr.with_p2p(peer_id.into()).expect("no error"))
                .expect("no error");
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
}
