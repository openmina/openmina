use std::net::{IpAddr, SocketAddr};

use p2p::{
    channels::{rpc::P2pChannelsRpcAction, P2pChannelsAction},
    connection::{incoming::P2pConnectionIncomingAction, outgoing::P2pConnectionOutgoingAction},
    disconnection::P2pDisconnectionAction,
    identify::P2pIdentifyAction,
    network::identify::P2pNetworkIdentify,
    peer::P2pPeerAction,
    MioEvent, P2pAction, P2pEvent, PeerId,
};

use crate::cluster::ClusterEvent;

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
    RpcChannelReady {
        peer_id: PeerId,
    },
    RpcChannelRequestReceived {
        peer_id: PeerId,
        id: p2p::channels::rpc::P2pRpcId,
        request: p2p::channels::rpc::P2pRpcRequest,
    },
    RpcChannelResponseReceived {
        peer_id: PeerId,
        id: p2p::channels::rpc::P2pRpcId,
        response: Option<p2p::channels::rpc::P2pRpcResponse>,
    },
    Identify {
        peer_id: PeerId,
        info: Box<P2pNetworkIdentify>,
    },
    KadBootstrapFinished,
    KadUpdateFindNodeRequest {
        peer_id: PeerId,
        closest_peers: Vec<p2p::network::kad::P2pNetworkKadEntry>,
    },
    /// Other non-specific p2p event.
    P2p {
        event: P2pEvent,
    },
    /// Timeout event with no specific outcome.
    Idle,
}

pub(super) trait RustNodeEventStore {
    fn store_event(&mut self, event: RustNodeEvent);
}

pub(super) fn event_mapper_effect(store: &mut super::redux::Store, action: P2pAction) {
    let store_event = |store: &mut super::redux::Store, event| store.service().store_event(event);
    match action {
        P2pAction::Peer(P2pPeerAction::Ready { peer_id, incoming }) => {
            store_event(store, RustNodeEvent::PeerConnected { peer_id, incoming })
        }
        P2pAction::Connection(action) => match action {
            p2p::connection::P2pConnectionAction::Outgoing(
                P2pConnectionOutgoingAction::Error { peer_id, error },
            ) => store_event(
                store,
                RustNodeEvent::PeerConnectionError {
                    peer_id: Some(peer_id),
                    incoming: false,
                    error: error.to_string(),
                },
            ),
            p2p::connection::P2pConnectionAction::Incoming(
                P2pConnectionIncomingAction::Error { peer_id, error },
            ) => store_event(
                store,
                RustNodeEvent::PeerConnectionError {
                    peer_id: Some(peer_id),
                    incoming: true,
                    error: error.to_string(),
                },
            ),
            _ => {}
        },

        P2pAction::Disconnection(P2pDisconnectionAction::Init { peer_id, reason }) => store_event(
            store,
            RustNodeEvent::PeerDisconnected {
                peer_id,
                reason: reason.to_string(),
            },
        ),
        P2pAction::Network(p2p::P2pNetworkAction::Kad(p2p::P2pNetworkKadAction::System(
            p2p::P2pNetworkKademliaAction::BootstrapFinished,
        ))) => {
            store_event(store, RustNodeEvent::KadBootstrapFinished);
        }
        P2pAction::Network(p2p::P2pNetworkAction::Kad(p2p::P2pNetworkKadAction::System(
            p2p::P2pNetworkKademliaAction::UpdateFindNodeRequest {
                peer_id,
                closest_peers,
                ..
            },
        ))) => {
            store_event(
                store,
                RustNodeEvent::KadUpdateFindNodeRequest {
                    peer_id,
                    closest_peers,
                },
            );
        }
        P2pAction::Channels(P2pChannelsAction::Rpc(action)) => match action {
            P2pChannelsRpcAction::Ready { peer_id } => {
                store_event(store, RustNodeEvent::RpcChannelReady { peer_id })
            }
            P2pChannelsRpcAction::RequestReceived {
                peer_id,
                id,
                request,
            } => {
                // if matches!(store.service.peek_rust_node_event(), Some(RustNodeEvent::RpcChannelReady { peer_id: pid }) if pid == &peer_id )
                // {
                //     store.service.rust_node_event();
                // }
                store_event(
                    store,
                    RustNodeEvent::RpcChannelRequestReceived {
                        peer_id,
                        id,
                        request,
                    },
                )
            }
            P2pChannelsRpcAction::ResponseReceived {
                peer_id,
                id,
                response,
            } => store_event(
                store,
                RustNodeEvent::RpcChannelResponseReceived {
                    peer_id,
                    id,
                    response,
                },
            ),
            _ => {}
        },
        P2pAction::Identify(P2pIdentifyAction::UpdatePeerInformation { peer_id, info }) => {
            store_event(store, RustNodeEvent::Identify { peer_id, info })
        }

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

pub fn is_error(event: &ClusterEvent) -> bool {
    let ClusterEvent::Rust { event, .. } = event else {
        return false;
    };
    match event {
        RustNodeEvent::ListenerError { .. } => true,
        RustNodeEvent::PeerConnectionError { .. } => true,
        RustNodeEvent::PeerDisconnected { .. } => true,
        RustNodeEvent::P2p { event } => match event {
            P2pEvent::Connection(_event) => false, // TODO
            P2pEvent::Channel(_event) => false,    // TODO
            P2pEvent::MioEvent(event) => matches!(
                event,
                MioEvent::ListenerError { .. }
                    | MioEvent::IncomingConnectionDidAccept(_, Err(_))
                    | MioEvent::IncomingDataDidReceive(_, Err(_))
                    | MioEvent::OutgoingConnectionDidConnect(_, Err(_))
                    | MioEvent::OutgoingDataDidSend(_, Err(_))
                    | MioEvent::ConnectionDidClose(_, Err(_))
            ),
        },
        _ => false,
    }
}

pub fn allow_disconnections(event: &ClusterEvent) -> bool {
    let ClusterEvent::Rust { event, .. } = event else {
        return false;
    };
    match event {
        RustNodeEvent::ListenerError { .. } => true,
        RustNodeEvent::PeerConnectionError { .. } => false,
        RustNodeEvent::PeerDisconnected { .. } => true,
        RustNodeEvent::P2p { event } => match event {
            P2pEvent::Connection(_event) => false, // TODO
            P2pEvent::Channel(_event) => false,    // TODO
            P2pEvent::MioEvent(event) => matches!(
                event,
                MioEvent::ListenerError { .. } | MioEvent::IncomingConnectionDidAccept(_, Err(_)) // | MioEvent::IncomingDataDidReceive(_, Err(_))
                                                                                                  // | MioEvent::OutgoingConnectionDidConnect(_, Err(_))
                                                                                                  // | MioEvent::OutgoingDataDidSend(_, Err(_))
                                                                                                  // | MioEvent::ConnectionDidClose(_, Err(_))
            ),
        },
        _ => false,
    }
}
