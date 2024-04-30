// use std::{ops::Deref, pin::Pin, time::Duration};

// use futures::{TryStream, TryStreamExt};

// use crate::{
//     cluster::{Cluster, ClusterEvent, Listener, TimestampEvent, TimestampSource}, predicates::listener_is_ready, rust_node::RustNodeId, stream::ClusterStreamExt
// };

use std::time::Duration;

use futures::{StreamExt, TryStreamExt};

use crate::{
    cluster::{Cluster, ClusterEvent, Error, NodeId},
    event::RustNodeEvent,
    predicates::{
        all_listeners_are_ready, all_nodes_with_value, listeners_are_ready, nodes_peers_are_ready,
    },
    rust_node::{RustNodeConfig, RustNodeId},
    stream::ClusterStreamExt,
};

/// Creates an iterator that creates Rust nodes in the cluster using specified
/// configuration.
pub fn rust_nodes(
    cluster: &mut Cluster,
    config: RustNodeConfig,
) -> impl '_ + Iterator<Item = Result<RustNodeId, Error>> {
    std::iter::repeat_with(move || cluster.add_rust_node(config.clone()))
}

/// Maps an array of Rust node ids to an array of corresponding peer ids.
pub fn peer_ids<const N: usize>(
    cluster: &Cluster,
    rust_nodes: [RustNodeId; N],
) -> [p2p::PeerId; N] {
    rust_nodes.map(|id| cluster.rust_node(id).state().my_id())
}

/// Tries to create a Rust node for each configuration, returning an array of node ids, or an error.
pub fn rust_nodes_from_configs<const N: usize>(
    cluster: &mut Cluster,
    configs: [RustNodeConfig; N],
) -> Result<[RustNodeId; N], Error> {
    let vec: Vec<_> = configs
        .into_iter()
        .map(|config| cluster.add_rust_node(config))
        .collect::<Result<_, _>>()?;
    Ok(vec.try_into().expect("size should match"))
}

/// Tries to create as many Rust nodes using the specified configuration to fill the resulting array.
pub fn rust_nodes_from_config<const N: usize>(
    cluster: &mut Cluster,
    config: RustNodeConfig,
) -> Result<[RustNodeId; N], Error> {
    let vec: Vec<_> = rust_nodes(cluster, config)
        .take(N)
        .collect::<Result<_, _>>()?;
    Ok(vec.try_into().expect("size should match"))
}

/// Tries to create as many Rust nodes using the default configuration to fill the resulting array.
pub fn rust_nodes_from_default_config<const N: usize>(
    cluster: &mut Cluster,
) -> Result<[RustNodeId; N], Error> {
    rust_nodes_from_config(cluster, Default::default())
}

/// Runs the cluster for the specified period of `time`.
pub async fn run_cluster(cluster: &mut Cluster, time: Duration) {
    let mut stream = cluster.stream().take_during(time);
    while let Some(_) = stream.next().await {}
}

/// Tries to run the cluster for the specified period of `time`, returning `Err`
/// if error event happens.
pub async fn try_run_cluster(cluster: &mut Cluster, time: Duration) -> Result<(), ClusterEvent> {
    let mut stream = cluster.try_stream().take_during(time);
    while let Some(_) = stream.try_next().await? {}
    Ok(())
}

/// Runs the cluster for the specified period of `time`, returning early true if
/// the specified `nodes` are ready to accept connections.
pub async fn wait_for_nodes_to_listen<I>(cluster: &mut Cluster, nodes: I, time: Duration) -> bool
where
    I: IntoIterator<Item = RustNodeId>,
{
    cluster
        .stream()
        .take_during(time)
        .any(listeners_are_ready(nodes))
        .await
}

/// Tries to run the cluster for the specified period of `time`, returning early
/// true if the specified `nodes` are ready to accept connections.
pub async fn try_wait_for_nodes_to_listen<I>(
    cluster: &mut Cluster,
    nodes: I,
    time: Duration,
) -> Result<bool, ClusterEvent>
where
    I: IntoIterator<Item = RustNodeId>,
{
    cluster
        .try_stream()
        .take_during(time)
        .try_any(listeners_are_ready(nodes))
        .await
}

/// Tries to run the cluster for the specified period of `time`, returning early
/// true if the specified `nodes` are ready to accept connections.
pub async fn try_wait_for_all_nodes_to_listen<I>(
    cluster: &mut Cluster,
    nodes: I,
    time: Duration,
) -> Result<bool, ClusterEvent>
where
    I: IntoIterator<Item = NodeId>,
{
    cluster
        .try_stream()
        .take_during(time)
        .try_any(all_listeners_are_ready(nodes))
        .await
}

/// Runs the cluster for specified period of `time`, returning `true` early if
/// for each specified pair of Rust node id and peer id there was a succesfull
/// connection.
pub async fn wait_for_nodes_to_connect<I>(
    cluster: &mut Cluster,
    nodes_peers: I,
    time: Duration,
) -> bool
where
    I: IntoIterator<Item = (RustNodeId, p2p::PeerId)>,
{
    cluster
        .stream()
        .take_during(time)
        .any(nodes_peers_are_ready(nodes_peers))
        .await
}

/// Tries to run the cluster for specified period of `time`, returning `true`
/// early if for each specified pair of Rust node id and peer id there was a
/// succesfull connection.
pub async fn try_wait_for_nodes_to_connect<I>(
    cluster: &mut Cluster,
    nodes_peers: I,
    time: Duration,
) -> Result<bool, ClusterEvent>
where
    I: IntoIterator<Item = (RustNodeId, p2p::PeerId)>,
{
    cluster
        .try_stream()
        .take_during(time)
        .try_any(nodes_peers_are_ready(nodes_peers))
        .await
}

/// Tries to wait for particular event to happen for all specified pairs of (node, peer_id).
///
/// Function `f` extract peer_id from a Rust event.
///
/// See [`super::predicates::all_nodes_with_items`].
pub async fn try_wait_for_all_node_peer<I, F>(
    cluster: &mut Cluster,
    nodes_peers: I,
    time: Duration,
    f: F,
) -> Result<bool, ClusterEvent>
where
    I: IntoIterator<Item = (RustNodeId, p2p::PeerId)>,
    F: FnMut(RustNodeEvent) -> Option<p2p::PeerId>,
{
    cluster
        .try_stream()
        .take_during(time)
        .try_any(all_nodes_with_value(nodes_peers, f))
        .await
}

/// Tries to wait for particular event to happen at least once for all specified pairs of (node, v).
///
/// Function `f` extract value `v` from a Rust event.
///
/// See [`super::predicates::all_nodes_with_items`].
pub async fn try_wait_for_all_nodes_with_value<T, I, F>(
    cluster: &mut Cluster,
    nodes_peers: I,
    time: Duration,
    f: F,
) -> Result<bool, ClusterEvent>
where
    T: Eq,
    I: IntoIterator<Item = (RustNodeId, T)>,
    F: FnMut(RustNodeEvent) -> Option<T>,
{
    cluster
        .try_stream()
        .take_during(time)
        .try_any(all_nodes_with_value(nodes_peers, f))
        .await
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        cluster::ClusterBuilder,
        rust_node::RustNodeConfig,
        utils::{peer_ids, wait_for_nodes_to_connect, wait_for_nodes_to_listen},
    };

    #[tokio::test]
    async fn test() {
        let mut cluster = ClusterBuilder::new()
            .ports(11000..11200)
            .start()
            .await
            .expect("should build cluster");

        let _nodes = super::rust_nodes(&mut cluster, RustNodeConfig::default())
            .take(3)
            .collect::<Vec<_>>();

        let [_node1, _node2] =
            super::rust_nodes_from_config(&mut cluster, RustNodeConfig::default()).unwrap();
        let [node1, node2, node3] = super::rust_nodes_from_default_config(&mut cluster).unwrap();

        let ready =
            wait_for_nodes_to_listen(&mut cluster, [node1, node2, node3], Duration::from_secs(10))
                .await;
        assert!(ready);

        let [peer_id2, peer_id3] = peer_ids(&cluster, [node2, node3]);

        cluster.connect(node1, node2).expect("no error");
        cluster.connect(node1, node3).expect("no error");

        let ready = wait_for_nodes_to_connect(
            &mut cluster,
            [(node1, peer_id2), (node1, peer_id3)],
            Duration::from_secs(10),
        )
        .await;
        assert!(ready);
    }
}

// async fn try_listener_is_ready<T, L>(cluster: &mut T, node: RustNodeId, duration: Duration) -> Result<bool, T::Error>
// where
//     T: Unpin + Deref<Target = Cluster> + TryStream<Ok = ClusterEvent> + TimestampSource,
//     T::Item: TimestampEvent,
//     L: Into<Listener>,
// {

//     ClusterStreamExt::take_during(Pin::new(cluster), duration).try_any(listener_is_ready(node)).await
// }

// async fn try_connect_and_ready<T, L>(cluster: &mut T, node: RustNodeId, peer: L) -> bool
// where
//     T: Deref<Cluster> + TryStream<Ok = ClusterEvent>,
//     L: Into<Listener>,
// {
//     cluster.
// }
