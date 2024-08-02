use std::{net::SocketAddr, sync::OnceLock};

use openmina_core::{error, warn};
use redux::ActionMeta;

use crate::{
    connection::{
        incoming::P2pConnectionIncomingAction, outgoing::P2pConnectionOutgoingAction,
        P2pConnectionState,
    },
    disconnection::P2pDisconnectionAction,
    identify::P2pIdentifyAction,
    network::identify::P2pNetworkIdentifyStreamAction,
    request::{P2pNetworkKadRequestState, P2pNetworkKadRequestStatus},
    token::{RpcAlgorithm, StreamKind},
    MioCmd, P2pCryptoService, P2pMioService, P2pPeerStatus,
};

use super::{super::*, *};

fn keep_connection_with_unknown_stream() -> bool {
    static VAL: OnceLock<bool> = OnceLock::new();
    *VAL.get_or_init(|| {
        std::env::var("KEEP_CONNECTION_WITH_UNKNOWN_STREAM")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(false)
    })
}

impl P2pNetworkSchedulerAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService + P2pCryptoService,
    {
        match self {
            P2pNetworkSchedulerAction::InterfaceDetected { ip, .. } => {
                if let Some(port) = store.state().config.libp2p_port {
                    store
                        .service()
                        .send_mio_cmd(MioCmd::ListenOn(SocketAddr::new(ip, port)));
                }
            }
            P2pNetworkSchedulerAction::InterfaceExpired { .. } => {}
            P2pNetworkSchedulerAction::ListenerReady { listener: _ } => {}
            P2pNetworkSchedulerAction::ListenerError {
                listener: _,
                error: _,
            } => {
                // TODO: handle this error?
            }
            P2pNetworkSchedulerAction::IncomingConnectionIsReady { listener, .. } => {
                let state = store.state();
                if state.network.scheduler.connections.len()
                    >= state.config.limits.max_connections()
                {
                    store.service().send_mio_cmd(MioCmd::Refuse(listener));
                } else {
                    store.service().send_mio_cmd(MioCmd::Accept(listener));
                }
            }
            P2pNetworkSchedulerAction::IncomingDidAccept { addr, .. } => {
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
            P2pNetworkSchedulerAction::OutgoingConnect { addr } => {
                store.service().send_mio_cmd(MioCmd::Connect(addr));
            }
            P2pNetworkSchedulerAction::OutgoingDidConnect { addr, result } => match result {
                Ok(_) => {
                    let nonce = store.service().generate_random_nonce();
                    store.dispatch(P2pNetworkPnetAction::SetupNonce {
                        addr,
                        nonce: nonce.to_vec().into(),
                        incoming: false,
                    });
                }
                Err(err) => {
                    let Some((peer_id, peer_state)) = store.state().peer_with_connection(addr)
                    else {
                        error!(meta.time(); "outgoing connection to {addr} failed, but there is no peer for it");
                        return;
                    };
                    if matches!(
                        peer_state.status,
                        P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(_))
                    ) {
                        store.dispatch(P2pConnectionOutgoingAction::FinalizeError {
                            peer_id,
                            error: err.to_string(),
                        });
                    }
                }
            },
            P2pNetworkSchedulerAction::IncomingDataIsReady { addr } => {
                let Some(state) = store.state().network.scheduler.connections.get(&addr) else {
                    return;
                };

                let limit = state.limit();
                if limit > 0 {
                    store
                        .service()
                        .send_mio_cmd(MioCmd::Recv(addr, vec![0; limit].into_boxed_slice()));
                }
            }
            P2pNetworkSchedulerAction::IncomingDataDidReceive { result, addr } => match result {
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
            P2pNetworkSchedulerAction::SelectDone {
                addr,
                protocol,
                kind: select_kind,
                incoming,
                ..
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
                        let message_size_limit = store.state().config.limits.yamux_message_size();
                        store.dispatch(P2pNetworkSchedulerAction::YamuxDidInit {
                            addr,
                            peer_id,
                            message_size_limit,
                        });
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
            P2pNetworkSchedulerAction::SelectError { addr, kind, .. } => {
                match kind {
                    SelectKind::Stream(peer_id, stream_id)
                        if keep_connection_with_unknown_stream() =>
                    {
                        warn!(meta.time(); summary="select error for stream", addr = display(addr), peer_id = display(peer_id));
                        // just close the stream
                        store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                            addr,
                            stream_id,
                            data: Data::default(),
                            flags: YamuxFlags::RST,
                        });
                        store.dispatch(P2pNetworkSchedulerAction::PruneStream {
                            peer_id,
                            stream_id,
                        });
                    }
                    _ => {
                        store.dispatch(P2pNetworkSchedulerAction::Error {
                            addr,
                            error: P2pNetworkConnectionError::SelectError,
                        });
                    }
                }
                // Close state is set by reducer for the non-stream case
                if let Some(conn_state) = store.state().network.scheduler.connections.get(&addr) {
                    if let Some(reason) = conn_state.closed.clone() {
                        store.service().send_mio_cmd(MioCmd::Disconnect(addr));
                        store.dispatch(P2pNetworkSchedulerAction::Disconnected { addr, reason });
                    }
                }
            }
            P2pNetworkSchedulerAction::YamuxDidInit { peer_id, addr, .. } => {
                if let Some(cn) = store.state().network.scheduler.connections.get(&addr) {
                    let incoming = cn.incoming;
                    if incoming {
                        store.dispatch(P2pConnectionIncomingAction::Libp2pReceived { peer_id });
                    } else {
                        store.dispatch(P2pConnectionOutgoingAction::FinalizeSuccess { peer_id });
                    }

                    // for each negotiated yamux conenction open a new outgoing RPC stream
                    // TODO(akoptelov,vlad): should we do that? shouldn't upper layer decide when to open RPC streams?
                    // Also rpc streams are short-living -- they only persist for a single request-response (?)
                    let stream_id = YamuxStreamKind::Rpc.stream_id(incoming);
                    store.dispatch(P2pNetworkYamuxAction::OpenStream {
                        addr,
                        stream_id,
                        stream_kind: StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1),
                    });
                    store.dispatch(P2pNetworkYamuxAction::OpenStream {
                        addr,
                        stream_id: stream_id + 2,
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
                }
            }
            P2pNetworkSchedulerAction::Disconnect { addr, .. } => {
                if let Some(conn_state) = store.state().network.scheduler.connections.get(&addr) {
                    if let Some(reason) = conn_state.closed.clone() {
                        store.service().send_mio_cmd(MioCmd::Disconnect(addr));
                        store.dispatch(P2pNetworkSchedulerAction::Disconnected { addr, reason });
                    }
                }
            }
            P2pNetworkSchedulerAction::Error { addr, .. } => {
                if let Some(conn_state) = store.state().network.scheduler.connections.get(&addr) {
                    if let Some(reason) = conn_state.closed.clone() {
                        store.service().send_mio_cmd(MioCmd::Disconnect(addr));
                        store.dispatch(P2pNetworkSchedulerAction::Disconnected { addr, reason });
                    }
                }
            }
            P2pNetworkSchedulerAction::Disconnected { addr, reason } => {
                if let Some(conn_state) = store.state().network.scheduler.connections.get(&addr) {
                    let incoming = conn_state.incoming;
                    let peer_with_state = store.state().peer_with_connection(addr);

                    store.dispatch(P2pNetworkSchedulerAction::Prune { addr });

                    if reason.is_disconnected() {
                        // statemachine behaviour should continue with this, i.e. dispatch P2pDisconnectionAction::Finish
                        return;
                    }

                    match peer_with_state {
                        Some((peer_id, peer_state)) => {
                            // TODO: connection state type should tell if it is finalized
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
                            store.dispatch(P2pNetworkSchedulerAction::PruneStreams { peer_id });
                        }
                        None => {
                            // sanity check, should be incoming connection
                            if !incoming {
                                // TODO: error!(meta.time(); "non-existing peer connection for address {addr}");
                            } else {
                                // TODO: introduce action for incoming connection finalization without peer_id
                            }
                        }
                    }
                }
            }
            P2pNetworkSchedulerAction::Prune { .. }
            | P2pNetworkSchedulerAction::PruneStreams { .. }
            | P2pNetworkSchedulerAction::PruneStream { .. } => {}
        }
    }
}
