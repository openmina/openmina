use std::{
    collections::{BTreeSet, HashSet},
    future::{ready, Ready},
};

use libp2p::swarm::SwarmEvent;
use p2p::PeerId;

use crate::{
    cluster::{ClusterEvent, NodeId},
    event::RustNodeEvent,
    libp2p_node::Libp2pEvent,
    rust_node::RustNodeId,
};

/// Wraps plain function over a cluster event into an async one.
pub fn async_fn<T, F>(mut f: F) -> impl FnMut(ClusterEvent) -> Ready<T>
where
    F: FnMut(ClusterEvent) -> T,
{
    move |event| ready(f(event))
}

/// Predicate returning true for a cluster event corresponging to the specified node started listening.
pub fn listener_is_ready(id: RustNodeId) -> impl FnMut(ClusterEvent) -> Ready<bool> {
    move |event| {
        ready(
            matches!(event.rust(), Some((event_id, RustNodeEvent::ListenerReady { .. })) if *event_id == id),
        )
    }
}

/// Predicate if kademlia has finished bootstrap
pub fn kad_finished_bootstrap(id: RustNodeId) -> impl FnMut(ClusterEvent) -> Ready<bool> {
    move |event| {
        ready(matches!(
            event.rust(),
            Some((event_id, RustNodeEvent::KadBootstrapFinished)) if *event_id == id
        ))
    }
}

/// Predicate returning true for a cluster event corresponging to the specified node started listening.
pub fn listeners_are_ready<I>(ids: I) -> impl FnMut(ClusterEvent) -> Ready<bool>
where
    I: IntoIterator<Item = RustNodeId>,
{
    let mut ids: HashSet<RustNodeId> = HashSet::from_iter(ids);
    move |event| {
        ready(
            if let Some((event_id, RustNodeEvent::ListenerReady { .. })) = event.rust() {
                ids.remove(event_id) && ids.is_empty()
            } else {
                false
            },
        )
    }
}

/// Predicate returning true for a cluster event corresponging to the specified node started listening.
pub fn all_listeners_are_ready<T, I>(ids: I) -> impl FnMut(ClusterEvent) -> Ready<bool>
where
    I: IntoIterator<Item = T>,
    T: Into<NodeId>,
{
    let mut ids: HashSet<NodeId> = HashSet::from_iter(ids.into_iter().map(Into::into));
    move |event| {
        ready(
            match event {
                ClusterEvent::Rust {
                    id,
                    event: RustNodeEvent::ListenerReady { .. },
                } => ids.remove(&NodeId::Rust(id)),
                ClusterEvent::Libp2p {
                    id,
                    event: SwarmEvent::NewListenAddr { address, .. },
                } => {
                    println!("{id:?}: new listen addr: {address}");
                    ids.remove(&NodeId::Libp2p(id))
                }
                _ => false,
            } && ids.is_empty(),
        )
    }
}

pub fn nodes_peers_are_ready<I>(nodes_peers: I) -> impl FnMut(ClusterEvent) -> Ready<bool>
where
    I: IntoIterator<Item = (RustNodeId, PeerId)>,
{
    let mut nodes_peers = BTreeSet::from_iter(nodes_peers);
    move |event| {
        ready(
            if let ClusterEvent::Rust {
                id,
                event: RustNodeEvent::PeerConnected { peer_id, .. },
            } = event
            {
                nodes_peers.remove(&(id, peer_id)) && nodes_peers.is_empty()
            } else {
                false
            },
        )
    }
}

/// Returns a predicate over cluster events that returns `true` once it is
/// called at least once for events that represent established connection
/// between a node and a peer from the `nodes_peers`.
pub fn all_nodes_peers_are_ready<I>(nodes_peers: I) -> impl FnMut(ClusterEvent) -> Ready<bool>
where
    I: IntoIterator<Item = (NodeId, PeerId)>,
{
    let mut nodes_peers = BTreeSet::from_iter(nodes_peers);
    move |event| {
        ready(match event {
            ClusterEvent::Rust {
                id,
                event: RustNodeEvent::PeerConnected { peer_id, .. },
            } => nodes_peers.remove(&(id.into(), peer_id)) && nodes_peers.is_empty(),
            ClusterEvent::Libp2p {
                id,
                event: Libp2pEvent::ConnectionEstablished { peer_id, .. },
            } => nodes_peers.remove(&(id.into(), peer_id.into())) && nodes_peers.is_empty(),
            _ => false,
        })
    }
}

/// Predicate returning true when encountered an event signalling that the peer `peer_id` is connected to the node `id`.
pub fn peer_is_connected(
    id: RustNodeId,
    peer_id: PeerId,
) -> impl FnMut(ClusterEvent) -> Ready<bool> {
    move |event| {
        ready(
            matches!(event.rust(), Some((event_id, RustNodeEvent::PeerConnected { peer_id: pid, .. })) if *event_id == id && pid == &peer_id),
        )
    }
}

/// Function that wraps a cluster event into a `Result` using default sense of erroneous event.
pub fn default_errors(event: &ClusterEvent) -> bool {
    match &event {
        ClusterEvent::Rust { event: e, .. } => match e {
            RustNodeEvent::ListenerError { .. } => true,
            RustNodeEvent::PeerConnectionError { .. } => true,
            RustNodeEvent::PeerDisconnected { .. } => true,
            RustNodeEvent::P2p { event: e } => match e {
                p2p::P2pEvent::Connection(_) => false,
                p2p::P2pEvent::Channel(e) => matches!(
                    e,
                    p2p::P2pChannelEvent::Opened(_, _, Err(_))
                        | p2p::P2pChannelEvent::Sent(_, _, _, Err(_))
                        | p2p::P2pChannelEvent::Received(_, Err(_))
                ),
                p2p::P2pEvent::MioEvent(e) => matches!(
                    e,
                    p2p::MioEvent::ListenerError { .. }
                        | p2p::MioEvent::IncomingConnectionDidAccept(_, Err(_))
                        | p2p::MioEvent::IncomingDataDidReceive(_, Err(_))
                        | p2p::MioEvent::OutgoingConnectionDidConnect(_, Err(_))
                        | p2p::MioEvent::OutgoingDataDidSend(_, Err(_))
                        | p2p::MioEvent::ConnectionDidClose(_, Err(_))
                ),
            },
            _ => false,
        },
        _ => false,
    }
}

/// For an event for a rust node _id_, that `f` maps to `Some(v)`,
/// removes the pair `(id, v)` from the `nodes_items`, returning `true` if it
/// runs out.
pub fn all_nodes_with_value<T, I, F>(
    nodes_items: I,
    mut f: F,
) -> impl FnMut(ClusterEvent) -> Ready<bool>
where
    T: PartialEq + Eq,
    I: IntoIterator<Item = (RustNodeId, T)>,
    F: FnMut(RustNodeEvent) -> Option<T>,
{
    let mut nodes_items = Vec::from_iter(nodes_items);
    move |event| {
        ready(if let ClusterEvent::Rust { id, event } = event {
            f(event)
                .and_then(|v| {
                    nodes_items
                        .iter()
                        .position(|(_id, _v)| _id == &id && _v == &v)
                })
                .map_or(false, |i| {
                    nodes_items.swap_remove(i);
                    nodes_items.is_empty()
                })
        } else {
            false
        })
    }
}
