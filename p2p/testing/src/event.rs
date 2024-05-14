use std::net::{IpAddr, SocketAddr};

use p2p::{
    channels::{rpc::P2pChannelsRpcAction, P2pChannelsAction},
    connection::{incoming::P2pConnectionIncomingAction, outgoing::P2pConnectionOutgoingAction},
    disconnection::P2pDisconnectionAction,
    identify::P2pIdentifyAction,
    network::identify::P2pNetworkIdentify,
    peer::P2pPeerAction,
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
        info: P2pNetworkIdentify,
    },
    KadBootstrapFinished,
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
        P2pAction::Peer(action) => match action {
            P2pPeerAction::Ready { peer_id, incoming } => {
                store_event(store, RustNodeEvent::PeerConnected { peer_id, incoming })
            }
            _ => {}
        },
        P2pAction::Connection(action) => match action {
            p2p::connection::P2pConnectionAction::Outgoing(action) => match action {
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
        P2pAction::Network(p2p::P2pNetworkAction::Kad(p2p::P2pNetworkKadAction::System(
            p2p::P2pNetworkKademliaAction::BootstrapFinished,
        ))) => {
            store_event(store, RustNodeEvent::KadBootstrapFinished);
        }
        P2pAction::Channels(action) => match action {
            P2pChannelsAction::Rpc(action) => match action {
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
