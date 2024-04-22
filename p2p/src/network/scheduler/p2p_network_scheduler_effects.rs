use std::net::SocketAddr;

use openmina_core::error;
use redux::ActionMeta;

use crate::{
    connection::{incoming::P2pConnectionIncomingAction, outgoing::P2pConnectionOutgoingAction},
    disconnection::P2pDisconnectionAction,
    identify::P2pIdentifyAction,
    network::identify::P2pNetworkIdentifyStreamAction,
    request::{P2pNetworkKadRequestState, P2pNetworkKadRequestStatus},
    token::{RpcAlgorithm, StreamKind},
    MioCmd, P2pCryptoService, P2pMioService,
};

use super::{super::*, *};

impl P2pNetworkSchedulerAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService + P2pCryptoService,
    {
        match self {
            Self::InterfaceDetected { ip, .. } => {
                if let Some(port) = store.state().config.libp2p_port {
                    store
                        .service()
                        .send_mio_cmd(MioCmd::ListenOn(SocketAddr::new(ip, port)));
                }
            }
            Self::InterfaceExpired { .. } => {}
            P2pNetworkSchedulerAction::ListenerReady { listener: _ } => {}
            P2pNetworkSchedulerAction::ListenerError {
                listener: _,
                error: _,
            } => {
                // TODO: handle this error?
            }
            Self::IncomingConnectionIsReady { listener, .. } => {
                store.service().send_mio_cmd(MioCmd::Accept(listener));
            }
            Self::IncomingDidAccept { addr, .. } => {
                let Some(addr) = addr else {
                    return;
                };

                let nonce = store.service().generate_random_nonce();
                store.dispatch(P2pNetworkPnetAction::SetupNonce {
                    addr,
                    nonce: nonce.to_vec().into(),
                    incoming: true,
                });
            }
            Self::OutgoingConnect { addr } => {
                store.service().send_mio_cmd(MioCmd::Connect(addr));
            }
            Self::OutgoingDidConnect { addr, result } => {
                if result.is_ok() {
                    let nonce = store.service().generate_random_nonce();
                    store.dispatch(P2pNetworkPnetAction::SetupNonce {
                        addr,
                        nonce: nonce.to_vec().into(),
                        incoming: false,
                    });
                }
            }
            Self::IncomingDataIsReady { addr } => {
                store
                    .service()
                    .send_mio_cmd(MioCmd::Recv(addr, vec![0; 0x8000].into_boxed_slice()));
            }
            Self::IncomingDataDidReceive { result, addr } => match result {
                Ok(data) => {
                    store.dispatch(P2pNetworkPnetAction::IncomingData {
                        addr,
                        data: data.clone(),
                    });
                }
                Err(e) => {
                    store.dispatch(P2pNetworkSchedulerAction::Error {
                        addr,
                        error: P2pNetworkConnectionError::MioError(e),
                    });
                }
            },
            Self::SelectDone {
                addr,
                protocol,
                kind: select_kind,
                incoming,
            } => {
                use self::token::*;

                match protocol {
                    Some(Protocol::Auth(AuthKind::Noise)) => {
                        let ephemeral_sk = Sk::from_random(store.service().ephemeral_sk());
                        let ephemeral_pk = ephemeral_sk.pk();

                        let static_sk = Sk::from_random(store.service().static_sk());
                        let static_pk = static_sk.pk();

                        let signature = store.service().sign_key(static_pk.0.as_bytes()).into();

                        store.dispatch(P2pNetworkNoiseAction::Init {
                            addr,
                            incoming,
                            ephemeral_sk,
                            ephemeral_pk,
                            static_pk,
                            static_sk,
                            signature,
                        });
                    }
                    Some(Protocol::Mux(MuxKind::Yamux1_0_0 | MuxKind::YamuxNoNewLine1_0_0)) => {
                        let SelectKind::Multiplexing(peer_id) = select_kind else {
                            error!(meta.time(); "wrong kind for multiplexing protocol action: {select_kind:?}");
                            return;
                        };
                        store.dispatch(P2pNetworkSchedulerAction::YamuxDidInit { addr, peer_id });
                    }
                    Some(Protocol::Stream(kind)) => {
                        let SelectKind::Stream(peer_id, stream_id) = select_kind else {
                            error!(meta.time(); "wrong kind for stream protocol action: {kind:?}");
                            return;
                        };
                        match kind {
                            StreamKind::Status(_) => {
                                //unimplemented!()
                            }
                            StreamKind::Bitswap(_) => {
                                //unimplemented!()
                            }
                            StreamKind::Identify(IdentifyAlgorithm::Identify1_0_0) => {
                                store.dispatch(P2pNetworkIdentifyStreamAction::New {
                                    addr,
                                    peer_id,
                                    stream_id,
                                    incoming,
                                });
                            }
                            StreamKind::Identify(IdentifyAlgorithm::IdentifyPush1_0_0) => {
                                //unimplemented!()
                            }
                            StreamKind::Ping(PingAlgorithm::Ping1_0_0) => {
                                //unimplemented!()
                            }
                            StreamKind::Broadcast(protocol) => {
                                store.dispatch(P2pNetworkPubsubAction::NewStream {
                                    incoming,
                                    peer_id,
                                    addr,
                                    stream_id,
                                    protocol,
                                });
                            }
                            StreamKind::Discovery(DiscoveryAlgorithm::Kademlia1_0_0) => {
                                if let Some(discovery_state) =
                                    store.state().network.scheduler.discovery_state()
                                {
                                    let request =
                                        !incoming && discovery_state.request(&peer_id).is_some();
                                    store.dispatch(P2pNetworkKademliaStreamAction::New {
                                        addr,
                                        peer_id,
                                        stream_id,
                                        incoming,
                                    });
                                    // if our node initiated a request to the peer, notify that the stream is ready.
                                    if request {
                                        store.dispatch(P2pNetworkKadRequestAction::StreamReady {
                                            peer_id,
                                            addr,
                                            stream_id,
                                        });
                                    }
                                }
                            }
                            StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1) => {
                                store.dispatch(P2pNetworkRpcAction::Init {
                                    addr,
                                    peer_id,
                                    stream_id,
                                    incoming,
                                });
                            }
                        }
                    }
                    None => {
                        match &select_kind {
                            SelectKind::Authentication => {
                                // TODO: close the connection
                            }
                            SelectKind::MultiplexingNoPeerId => {
                                // WARNING: must not happen
                            }
                            SelectKind::Multiplexing(_) => {
                                // TODO: close the connection
                            }
                            SelectKind::Stream(peer_id, stream_id) => {
                                if let Some(discovery_state) =
                                    store.state().network.scheduler.discovery_state()
                                {
                                    if let Some(P2pNetworkKadRequestState {
                                        status: P2pNetworkKadRequestStatus::WaitingForKadStream(id),
                                        ..
                                    }) = discovery_state.request(peer_id)
                                    {
                                        if id == stream_id {
                                            store.dispatch(P2pNetworkKadRequestAction::Error {
                                                peer_id: *peer_id,
                                                error: "stream protocol is not negotiated".into(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Self::SelectError { .. } => {
                // TODO: close stream or connection
            }
            Self::YamuxDidInit { peer_id, addr } => {
                if let Some(cn) = store.state().network.scheduler.connections.get(&addr) {
                    // for each negotiated yamux conenction open a new outgoing RPC stream
                    // TODO(akoptelov,vlad): should we do that? shouldn't upper layer decide when to open RPC streams?
                    // Also rpc streams are short-living -- they only persist for a single request-response (?)
                    let incoming = cn.incoming;
                    store.dispatch(P2pNetworkYamuxAction::OpenStream {
                        addr,
                        stream_id: 1,
                        stream_kind: StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1),
                    });
                    store.dispatch(P2pNetworkYamuxAction::OpenStream {
                        addr,
                        stream_id: 3,
                        stream_kind: StreamKind::Broadcast(token::BroadcastAlgorithm::Meshsub1_1_0),
                    });

                    // TODO: open RPC and Kad connections only after identify reports support for it?
                    store.dispatch(P2pIdentifyAction::NewRequest { peer_id, addr });

                    // Kademlia: if the connection is initiated by Kademlia request, notify that it is ready.
                    if store
                        .state()
                        .network
                        .scheduler
                        .discovery_state()
                        .map_or(false, |state| state.request(&peer_id).is_some())
                    {
                        store.dispatch(P2pNetworkKadRequestAction::MuxReady { peer_id, addr });
                    }
                    if incoming {
                        store.dispatch(P2pConnectionIncomingAction::Libp2pReceived { peer_id });
                    } else {
                        store.dispatch(P2pConnectionOutgoingAction::FinalizeSuccess { peer_id });
                    }
                }
            }
            Self::Disconnect { addr, .. } => {
                if let Some(conn_state) = store.state().network.scheduler.connections.get(&addr) {
                    if let Some(reason) = conn_state.closed.clone() {
                        store.service().send_mio_cmd(MioCmd::Disconnect(addr));
                        store.dispatch(Self::Disconnected { addr, reason });
                    }
                }
            }
            Self::Error { addr, .. } => {
                if let Some(conn_state) = store.state().network.scheduler.connections.get(&addr) {
                    if let Some(reason) = conn_state.closed.clone() {
                        store.dispatch(Self::Disconnected { addr, reason });
                    }
                }
            }
            Self::Disconnected { addr, reason } => {
                if let Some(conn_state) = store.state().network.scheduler.connections.get(&addr) {
                    let incoming = conn_state.incoming;
                    store.dispatch(P2pNetworkSchedulerAction::Prune { addr });
                    if reason.is_disconnected() {
                        // statemachine behaviour should continue with this, i.e. dispatch P2pDisconnectionAction::Finish
                        return;
                    }
                    match store.state().peer_with_connection(addr) {
                        Some((peer_id, peer_state)) => {
                            // TODO: connection state type should tell if it is finalized
                            let peer_id = *peer_id;
                            match &peer_state.status {
                                crate::P2pPeerStatus::Connecting(
                                    crate::connection::P2pConnectionState::Incoming(_),
                                ) => {
                                    store.dispatch(P2pConnectionIncomingAction::FinalizeError {
                                        peer_id,
                                        error: reason.to_string(),
                                    });
                                }
                                crate::P2pPeerStatus::Connecting(
                                    crate::connection::P2pConnectionState::Outgoing(_),
                                ) => {
                                    store.dispatch(P2pConnectionOutgoingAction::FinalizeError {
                                        peer_id,
                                        error: reason.to_string(),
                                    });
                                }
                                crate::P2pPeerStatus::Disconnected { .. } => {
                                    // sanity check, should be incoming connection
                                    if !incoming {
                                        error!(meta.time(); "disconnected peer connection for address {addr}");
                                    } else {
                                        // TODO: introduce action for incoming connection finalization without peer_id
                                    }
                                }
                                crate::P2pPeerStatus::Ready(_) => {
                                    store.dispatch(P2pDisconnectionAction::Finish { peer_id });
                                }
                            }
                        }
                        None => {
                            // sanity check, should be incoming connection
                            if !incoming {
                                error!(meta.time(); "non-existing peer connection for address {addr}");
                            } else {
                                // TODO: introduce action for incoming connection finalization without peer_id
                            }
                        }
                    }
                }
            }
            Self::Prune { .. } => {}
        }
    }
}
