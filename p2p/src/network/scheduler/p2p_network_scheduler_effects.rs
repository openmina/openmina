use std::net::SocketAddr;

use openmina_core::error;
use redux::ActionMeta;

use crate::{
    connection::{incoming::P2pConnectionIncomingAction, outgoing::P2pConnectionOutgoingAction},
    request::{P2pNetworkKadRequestState, P2pNetworkKadRequestStatus},
    token::{RpcAlgorithm, StreamKind},
    MioCmd, P2pCryptoService, P2pMioService,
};

use super::{super::*, *};

impl P2pNetworkSchedulerAction {
    pub fn effects<Store, S>(&self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService + P2pCryptoService,
    {
        match self {
            Self::InterfaceDetected(a) => {
                let port = store.state().config.libp2p_port.unwrap_or_default();
                store
                    .service()
                    .send_mio_cmd(MioCmd::ListenOn(SocketAddr::new(a.ip, port)));

                // TODO: implement it properly, add more actions
                // let initial_peers = store.state().config.initial_peers.clone();
                // for peer in &initial_peers {
                //     let addr = match peer {
                //         P2pConnectionOutgoingInitOpts::LibP2P(v) => match &v.host {
                //             Host::Ipv4(ip) => SocketAddr::new((*ip).into(), v.port),
                //             Host::Ipv6(ip) => SocketAddr::new((*ip).into(), v.port),
                //             _ => continue,
                //         },
                //         _ => continue,
                //     };

                //     if addr.is_ipv4() == a.ip.is_ipv4() {
                //         store.service().send_mio_cmd(MioCmd::Connect(addr));
                //     }
                // }
            }
            Self::InterfaceExpired(_) => {}
            Self::IncomingConnectionIsReady(a) => {
                store.service().send_mio_cmd(MioCmd::Accept(a.listener));
            }
            Self::IncomingDidAccept(a) => {
                let Some(addr) = a.addr else {
                    return;
                };

                let nonce = store.service().generate_random_nonce();
                store.dispatch(P2pNetworkPnetSetupNonceAction {
                    addr,
                    nonce: nonce.to_vec().into(),
                    incoming: true,
                });
            }
            Self::OutgoingDidConnect(a) => {
                if a.result.is_ok() {
                    let nonce = store.service().generate_random_nonce();
                    store.dispatch(P2pNetworkPnetSetupNonceAction {
                        addr: a.addr,
                        nonce: nonce.to_vec().into(),
                        incoming: false,
                    });
                }
            }
            Self::IncomingDataIsReady(a) => {
                store
                    .service()
                    .send_mio_cmd(MioCmd::Recv(a.addr, vec![0; 0x1000].into_boxed_slice()));
            }
            Self::IncomingDataDidReceive(a) => {
                if let Ok(data) = &a.result {
                    store.dispatch(P2pNetworkPnetIncomingDataAction {
                        addr: a.addr,
                        data: data.clone(),
                    });
                }
            }
            Self::SelectDone(a) => {
                use self::token::*;

                match &a.protocol {
                    Some(Protocol::Auth(AuthKind::Noise)) => {
                        use curve25519_dalek::{constants::ED25519_BASEPOINT_TABLE as G, Scalar};

                        let ephemeral_sk = store.service().ephemeral_sk().into();
                        let static_sk = store.service().static_sk();
                        let static_sk = Scalar::from_bytes_mod_order(static_sk);
                        let signature = store
                            .service()
                            .sign_key((G * &static_sk).to_montgomery().as_bytes())
                            .into();
                        store.dispatch(P2pNetworkNoiseInitAction {
                            addr: a.addr,
                            incoming: a.incoming,
                            ephemeral_sk,
                            static_sk: static_sk.to_bytes().into(),
                            signature,
                        });
                    }
                    Some(Protocol::Mux(MuxKind::Yamux1_0_0 | MuxKind::YamuxNoNewLine1_0_0)) => {
                        let SelectKind::Multiplexing(peer_id) = a.kind else {
                            error!(meta.time(); "wrong kind for multiplexing protocol action: {:?}", a.kind);
                            return;
                        };
                        store.dispatch(P2pNetworkSchedulerYamuxDidInitAction {
                            addr: a.addr,
                            peer_id: peer_id.clone(),
                        });
                    }
                    Some(Protocol::Stream(kind)) => {
                        let SelectKind::Stream(peer_id, stream_id) = a.kind else {
                            error!(meta.time(); "wrong kind for stream protocol action: {:?}", a.kind);
                            return;
                        };
                        match kind {
                            StreamKind::Discovery(DiscoveryAlgorithm::Kademlia1_0_0) => {
                                if let Some(discovery_state) =
                                    store.state().network.scheduler.discovery_state()
                                {
                                    let request =
                                        !a.incoming && discovery_state.request(&peer_id).is_some();
                                    store.dispatch(P2pNetworkKademliaStreamAction::New {
                                        addr: a.addr,
                                        peer_id,
                                        stream_id,
                                        incoming: a.incoming,
                                    });
                                    // if our node initiated a request to the peer, notify that the stream is ready.
                                    if request {
                                        store.dispatch(P2pNetworkKadRequestAction::StreamReady {
                                            peer_id,
                                            addr: a.addr,
                                            stream_id,
                                        });
                                    }
                                }
                            }
                            StreamKind::Broadcast(BroadcastAlgorithm::Meshsub1_1_0) => {
                                unimplemented!()
                            }
                            StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1) => {
                                store.dispatch(P2pNetworkRpcInitAction {
                                    addr: a.addr,
                                    peer_id,
                                    stream_id,
                                    incoming: a.incoming,
                                });
                            }
                        }
                    }
                    None => {
                        match &a.kind {
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
                                println!("=== SelectDone(Stream(None)), {peer_id} {stream_id}");
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
            Self::SelectError(_) => {
                // TODO: close stream or connection
            }
            Self::YamuxDidInit(a) => {
                let peer_id = a.peer_id;
                let addr = a.addr;
                if let Some(cn) = store.state().network.scheduler.connections.get(&addr) {
                    // for each negotiated yamux conenction open a new outgoing RPC stream
                    // TODO(akoptelov,vlad): should we do that? shouldn't upper layer decide when to open RPC streams?
                    // Also rpc streams are short-living -- they only persist for a single request-response (?)
                    let incoming = cn.incoming;
                    let stream_id = if incoming { 2 } else { 1 };
                    store.dispatch(P2pNetworkYamuxOpenStreamAction {
                        addr: a.addr,
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
        }
    }
}
