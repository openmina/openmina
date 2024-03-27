use std::net::SocketAddr;

use openmina_core::error;
use redux::ActionMeta;

use crate::{
    connection::{incoming::P2pConnectionIncomingAction, outgoing::P2pConnectionOutgoingAction},
    disconnection::P2pDisconnectionAction,
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
            Self::IncomingDataDidReceive { result, addr } => {
                if let Ok(data) = &result {
                    store.dispatch(P2pNetworkPnetAction::IncomingData {
                        addr,
                        data: data.clone(),
                    });
                }
            }
            Self::SelectDone {
                addr,
                protocol,
                kind: select_kind,
                incoming,
            } => {
                use self::token::*;

                match protocol {
                    Some(Protocol::Auth(AuthKind::Noise)) => {
                        use curve25519_dalek::{constants::ED25519_BASEPOINT_TABLE as G, Scalar};

                        let ephemeral_sk = store.service().ephemeral_sk().into();
                        let static_sk = store.service().static_sk();
                        let static_sk = Scalar::from_bytes_mod_order(static_sk);
                        let signature = store
                            .service()
                            .sign_key((G * &static_sk).to_montgomery().as_bytes())
                            .into();
                        store.dispatch(P2pNetworkNoiseAction::Init {
                            addr,
                            incoming,
                            ephemeral_sk,
                            static_sk: static_sk.to_bytes().into(),
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
                            StreamKind::Broadcast(BroadcastAlgorithm::Meshsub1_1_0) => {
                                unimplemented!()
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
                                    }) = discovery_state.request(&peer_id)
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
                    let stream_id = if incoming { 2 } else { 1 };
                    store.dispatch(P2pNetworkYamuxAction::OpenStream {
                        addr,
                        stream_id,
                        stream_kind: StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1),
                    });
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
                        store.dispatch(P2pConnectionIncomingAction::FinalizeSuccess { peer_id });
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
            Self::Disconnected { addr, reason: _ } => {
                if let Some(conn_state) = store.state().network.scheduler.connections.get(&addr) {
                    // TODO(akoptelov): handle cases where connection is not yet established
                    if let Some(peer_id) = conn_state.peer_id().cloned() {
                        store.dispatch(P2pDisconnectionAction::Finish { peer_id });
                    }
                }
            }
        }
    }
}
