use std::net::{IpAddr, SocketAddr};

use p2p::{
    connection::{incoming::P2pConnectionIncomingAction, outgoing::P2pConnectionOutgoingAction},
    disconnection::P2pDisconnectionAction,
    P2pAction, P2pEvent, PeerId,
};

#[derive(Debug)]
pub enum RustNodeEvent {
    Interface {
        addr: IpAddr,
    },
    ListenerReady {
        addr: SocketAddr,
    },
    ListenerError {
        addr: SocketAddr,
        error: String,
    },
    PeerConnected {
        peer_id: PeerId,
        incoming: bool,
    },
    PeerConnectionError {
        peer_id: Option<PeerId>,
        incoming: bool,
        error: String,
    },
    PeerDisconnected {
        peer_id: PeerId,
        reason: String,
    },
    P2p {
        event: P2pEvent,
    },
}

pub(super) trait RustNodeEventStore {
    fn store_event(&mut self, event: RustNodeEvent);
}

pub(super) fn event_mapper_effect(store: &mut super::redux::Store, action: P2pAction) {
    let store_event = |store: &mut super::redux::Store, event| store.service().store_event(event);
    match action {
        P2pAction::Connection(action) => match action {
            p2p::connection::P2pConnectionAction::Outgoing(action) => match action {
                P2pConnectionOutgoingAction::Success { peer_id } => store_event(
                    store,
                    RustNodeEvent::PeerConnected {
                        peer_id,
                        incoming: false,
                    },
                ),
                P2pConnectionOutgoingAction::Error { peer_id, error } => store_event(
                    store,
                    RustNodeEvent::PeerConnectionError {
                        peer_id: Some(peer_id),
                        incoming: false,
                        error: error.to_string(),
                    },
                ),
                _ => {}
            },
            p2p::connection::P2pConnectionAction::Incoming(action) => match action {
                P2pConnectionIncomingAction::Success { peer_id } => store_event(
                    store,
                    RustNodeEvent::PeerConnected {
                        peer_id,
                        incoming: true,
                    },
                ),
                P2pConnectionIncomingAction::Libp2pReceived { peer_id } => store_event(
                    store,
                    RustNodeEvent::PeerConnected {
                        peer_id,
                        incoming: true,
                    },
                ),
                P2pConnectionIncomingAction::Error { peer_id, error } => store_event(
                    store,
                    RustNodeEvent::PeerConnectionError {
                        peer_id: Some(peer_id),
                        incoming: true,
                        error: error.to_string(),
                    },
                ),
                _ => {}
            },
        },
        P2pAction::Disconnection(P2pDisconnectionAction::Init { peer_id, reason }) => store_event(
            store,
            RustNodeEvent::PeerDisconnected {
                peer_id,
                reason: reason.to_string(),
            },
        ),

        P2pAction::Network(p2p::P2pNetworkAction::Scheduler(action)) => match action {
            p2p::P2pNetworkSchedulerAction::InterfaceDetected { ip } => {
                store_event(store, RustNodeEvent::Interface { addr: ip })
            }
            p2p::P2pNetworkSchedulerAction::ListenerReady { listener } => {
                store_event(store, RustNodeEvent::ListenerReady { addr: listener })
            }
            p2p::P2pNetworkSchedulerAction::ListenerError { listener, error } => store_event(
                store,
                RustNodeEvent::ListenerError {
                    addr: listener,
                    error,
                },
            ),
            p2p::P2pNetworkSchedulerAction::Error { addr, error } => {
                if let Some(conn_state) = store.state().0.network.scheduler.connections.get(&addr) {
                    if conn_state.peer_id().is_none() {
                        let error = error.to_string();
                        let incoming = conn_state.incoming;
                        store_event(
                            store,
                            RustNodeEvent::PeerConnectionError {
                                peer_id: None,
                                incoming,
                                error,
                            },
                        );
                    }
                }
            }
            _ => {}
        },
        _ => {}
    }
}
