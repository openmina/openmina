use std::{collections::BTreeSet, time::Duration};

use multiaddr::Multiaddr;
use p2p::PeerId;
use p2p_testing::{
    cluster::{Cluster, ClusterBuilder, ClusterEvent},
    event::RustNodeEvent,
    futures::TryStreamExt,
    predicates::{async_fn, listener_is_ready, peer_is_connected},
    rust_node::{RustNodeConfig, RustNodeId},
    stream::ClusterStreamExt,
};

#[tokio::test]
async fn rust_node_to_rust_node() -> anyhow::Result<()> {
    let mut cluster = ClusterBuilder::new()
        .ports_with_len(10)
        .idle_duration(Duration::from_millis(100))
        .start()
        .await?;

    let node1 = cluster.add_rust_node(RustNodeConfig::default())?;

    let node2 = cluster.add_rust_node(RustNodeConfig::default())?;

    let peer_id1 = cluster.rust_node(node1).state().my_id();
    let peer_id2 = cluster.rust_node(node2).state().my_id();

    // wait for node1 to be ready to accept incoming conections
    let listener_is_ready = cluster
        .try_stream()
        .take_during(Duration::from_secs(2))
        .try_any(listener_is_ready(node1))
        .await?;
    assert!(listener_is_ready, "node1 should be ready");

    // connect node2 to node1
    cluster.connect(node2, node1)?;

    // wait for node2 to have peer_id1 (node1) as its peer in `ready` state`
    let connected = cluster
        .try_stream()
        .take_during(Duration::from_secs(2))
        .try_any(peer_is_connected(node2, peer_id1))
        .await?;
    assert!(
        connected,
        "node should be able to connect to {peer_id1}: {connected:?}\nnode state: {:#?}",
        cluster.rust_node(node2).state().peers.get(&peer_id1)
    );

    {
        let mut not_identified = BTreeSet::from_iter([(node1, peer_id2), (node2, peer_id1)]);

        // run the cluster until both nodes have identify data about each other
        let addrs =
            wait_for_identify(&mut cluster, &mut not_identified, Duration::from_secs(10)).await?;

        // for each address provided by identify, create a node and ensure it
        // can connect to that address
        for (peer_id, addr) in addrs {
            let node = cluster
                .add_rust_node(RustNodeConfig::default())
                .expect("no error");
            cluster
                .connect(
                    node,
                    addr.clone().with_p2p(peer_id.into()).expect("no error"),
                )
                .expect("no error");
            let connected = cluster
                .try_stream()
                .take_during(Duration::from_secs(5))
                .try_any(peer_is_connected(node, peer_id))
                .await
                .expect("unexpected error");
            assert!(
                connected,
                "node {} should be able to connect to {peer_id} via {addr}: {connected:?}\nnode state: {:#?}", cluster.peer_id(node),
                cluster.rust_node(node).state().peers.get(&peer_id)
            );
        }
    }

    Ok(())
}

async fn wait_for_identify(
    cluster: &mut Cluster,
    nodes_peers: &mut BTreeSet<(RustNodeId, PeerId)>,
    time: Duration,
) -> anyhow::Result<Vec<(PeerId, Multiaddr)>> {
    let mut addrs = Vec::new();
    let pred = |event: ClusterEvent| {
        if let ClusterEvent::Rust {
            id,
            event: RustNodeEvent::Identify { peer_id, info },
        } = event
        {
            if nodes_peers.remove(&(id, peer_id)) {
                addrs.extend(info.listen_addrs.iter().map(|addr| (peer_id, addr.clone())));
                return nodes_peers.is_empty();
            }
        }
        false
    };
    let identified = cluster
        .try_stream()
        .take_during(time)
        .try_any(async_fn(pred))
        .await?;
    assert!(identified, "all peers should be identified");
    Ok(addrs)
}
