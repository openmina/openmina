use std::future::{ready, Ready};

use p2p::PeerId;

use crate::{cluster::ClusterEvent, event::RustNodeEvent, rust_node::RustNodeId};

/// Wraps plain function over a cluster event into an async one.
pub fn async_fn<T, F>(mut f: F) -> impl FnMut(&ClusterEvent) -> Ready<T>
where
    F: FnMut(&ClusterEvent) -> T,
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
