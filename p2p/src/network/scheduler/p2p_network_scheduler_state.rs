use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, SocketAddr},
};

use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use crate::{
    channels::{ChannelId, P2pChannelsState},
    connection::outgoing::P2pConnectionOutgoingInitOpts,
    webrtc::Host,
    MioCmd, P2pCryptoService, P2pMioService, P2pPeerState, P2pPeerStatus, P2pPeerStatusReady,
    PeerId,
};

use super::{super::*, *};

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct P2pNetworkSchedulerState {
    pub interfaces: BTreeSet<IpAddr>,
    pub listeners: BTreeSet<SocketAddr>,
    pub pnet_key: [u8; 32],
    pub connections: BTreeMap<SocketAddr, P2pNetworkConnectionState>,
    pub broadcast_state: (),
    pub discovery_state: (),
    pub rpc_incoming_streams: BTreeMap<PeerId, BTreeMap<StreamId, P2pNetworkRpcState>>,
    pub rpc_outgoing_streams: BTreeMap<PeerId, BTreeMap<StreamId, P2pNetworkRpcState>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkConnectionState {
    pub incoming: bool,
    pub pnet: P2pNetworkPnetState,
    pub select_auth: P2pNetworkSelectState,
    pub auth: Option<P2pNetworkAuthState>,
    pub select_mux: P2pNetworkSelectState,
    pub mux: Option<P2pNetworkConnectionMuxState>,
    pub streams: BTreeMap<StreamId, P2pNetworkStreamState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkAuthState {
    Noise(P2pNetworkNoiseState),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkConnectionMuxState {
    Yamux(P2pNetworkYamuxState),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkStreamState {
    pub select: P2pNetworkSelectState,
    pub handler: Option<P2pNetworkStreamHandlerState>,
}

impl P2pNetworkStreamState {
    pub fn new(stream_kind: token::StreamKind) -> Self {
        P2pNetworkStreamState {
            select: P2pNetworkSelectState::initiator_stream(stream_kind),
            handler: None,
        }
    }

    pub fn new_incoming() -> Self {
        P2pNetworkStreamState {
            select: P2pNetworkSelectState::default(),
            handler: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkStreamHandlerState {
    Broadcast,
    Discovery,
}

impl P2pNetworkSchedulerState {
    pub fn reducer(
        &mut self,
        peers: &mut BTreeMap<PeerId, P2pPeerState>,
        action: redux::ActionWithMeta<&P2pNetworkSchedulerAction>,
    ) {
        match action.action() {
            P2pNetworkSchedulerAction::InterfaceDetected(a) => drop(self.interfaces.insert(a.ip)),
            P2pNetworkSchedulerAction::InterfaceExpired(a) => drop(self.interfaces.remove(&a.ip)),
            P2pNetworkSchedulerAction::IncomingConnectionIsReady(_) => {}
            P2pNetworkSchedulerAction::IncomingDidAccept(a) => {
                let Some(addr) = a.addr else {
                    return;
                };

                self.connections.insert(
                    addr,
                    P2pNetworkConnectionState {
                        incoming: true,
                        pnet: P2pNetworkPnetState::new(self.pnet_key),
                        select_auth: P2pNetworkSelectState::default(),
                        auth: None,
                        select_mux: P2pNetworkSelectState::default(),
                        mux: None,
                        streams: BTreeMap::default(),
                    },
                );
            }
            P2pNetworkSchedulerAction::OutgoingDidConnect(a) => {
                self.connections.insert(
                    a.addr,
                    P2pNetworkConnectionState {
                        incoming: false,
                        pnet: P2pNetworkPnetState::new(self.pnet_key),
                        select_auth: P2pNetworkSelectState::initiator_auth(token::AuthKind::Noise),
                        auth: None,
                        select_mux: P2pNetworkSelectState::initiator_mux(
                            token::MuxKind::Yamux1_0_0,
                        ),
                        mux: None,
                        streams: BTreeMap::default(),
                    },
                );
            }
            P2pNetworkSchedulerAction::IncomingDataIsReady(_) => {}
            P2pNetworkSchedulerAction::IncomingDataDidReceive(a) => {
                if a.result.is_err() {
                    self.connections.remove(&a.addr);
                }
            }
            P2pNetworkSchedulerAction::SelectDone(a) => {
                let Some(connection) = self.connections.get_mut(&a.addr) else {
                    return;
                };
                match &a.kind {
                    SelectKind::Multiplexing(peer_id) => {
                        let enabled_channels = Some(ChannelId::Rpc).into_iter().collect();
                        let state = P2pPeerState {
                            is_libp2p: true,
                            dial_opts: None,
                            status: P2pPeerStatus::Ready(P2pPeerStatusReady {
                                is_incoming: a.incoming,
                                connected_since: action.time(),
                                channels: P2pChannelsState::new(&enabled_channels),
                                best_tip: None,
                            }),
                        };
                        peers.insert(*peer_id, state);
                    }
                    _ => {}
                }
                match &a.protocol {
                    Some(token::Protocol::Auth(token::AuthKind::Noise)) => {
                        connection.auth =
                            Some(P2pNetworkAuthState::Noise(P2pNetworkNoiseState::default()));
                    }
                    Some(token::Protocol::Mux(
                        token::MuxKind::Yamux1_0_0 | token::MuxKind::YamuxNoNewLine1_0_0,
                    )) => {
                        connection.mux = Some(P2pNetworkConnectionMuxState::Yamux(
                            P2pNetworkYamuxState::default(),
                        ));
                    }
                    Some(token::Protocol::Stream(stream_kind)) => {
                        let Some(stream_id) = a.kind.stream_id() else {
                            return;
                        };
                        let Some(peer_id) = a.kind.peer_id() else {
                            return;
                        };
                        match stream_kind {
                            token::StreamKind::Rpc(_) => {
                                if a.incoming {
                                    self.rpc_incoming_streams
                                        .entry(peer_id)
                                        .or_default()
                                        .insert(
                                            stream_id,
                                            P2pNetworkRpcState::new(a.addr, stream_id),
                                        );
                                } else {
                                    self.rpc_outgoing_streams
                                        .entry(peer_id)
                                        .or_default()
                                        .insert(
                                            stream_id,
                                            P2pNetworkRpcState::new(a.addr, stream_id),
                                        );
                                }
                            }
                            token::StreamKind::Broadcast(_) => unimplemented!(),
                            token::StreamKind::Discovery(_) => unimplemented!(),
                        }
                    }
                    None => {}
                }
            }
            P2pNetworkSchedulerAction::SelectError(a) => {
                if let Some(stream_id) = &a.kind.stream_id() {
                    if let Some(connection) = self.connections.get_mut(&a.addr) {
                        connection.streams.remove(stream_id);
                    }
                } else {
                    self.connections.remove(&a.addr);
                }
            }
            P2pNetworkSchedulerAction::YamuxDidInit(a) => {
                if let Some(cn) = self.connections.get_mut(&a.addr) {
                    if let Some(P2pNetworkConnectionMuxState::Yamux(yamux)) = &mut cn.mux {
                        yamux.init = true;
                    }
                }
            }
        }
    }
}

impl P2pNetworkSchedulerAction {
    pub fn effects<Store, S>(&self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService + P2pCryptoService,
        P2pNetworkPnetIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkPnetSetupNonceAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseInitAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxOpenStreamAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxPingStreamAction: redux::EnablingCondition<S>,
        P2pNetworkRpcInitAction: redux::EnablingCondition<S>,
    {
        match self {
            Self::InterfaceDetected(a) => {
                let port = store.state().config.libp2p_port.unwrap_or_default();
                store
                    .service()
                    .send_mio_cmd(MioCmd::ListenOn(SocketAddr::new(a.ip, port)));

                // TODO: implement it properly, add more actions
                let initial_peers = store.state().config.initial_peers.clone();
                for peer in &initial_peers {
                    let addr = match peer {
                        P2pConnectionOutgoingInitOpts::LibP2P(v) => match &v.host {
                            Host::Ipv4(ip) => SocketAddr::new((*ip).into(), v.port),
                            Host::Ipv6(ip) => SocketAddr::new((*ip).into(), v.port),
                            _ => continue,
                        },
                        _ => continue,
                    };

                    if addr.is_ipv4() == a.ip.is_ipv4() {
                        store.service().send_mio_cmd(MioCmd::Connect(addr));
                    }
                }
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
                let nonce = store.service().generate_random_nonce();
                store.dispatch(P2pNetworkPnetSetupNonceAction {
                    addr: a.addr,
                    nonce: nonce.to_vec().into(),
                    incoming: false,
                });
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
                        if let Some(cn) = store.state().network.scheduler.connections.get(&a.addr) {
                            // for each negotiated yamux conenction open a new outgoing RPC stream
                            let stream_id = if cn.incoming { 2 } else { 1 };
                            store.dispatch(P2pNetworkYamuxOpenStreamAction {
                                addr: a.addr,
                                stream_id,
                                stream_kind: StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1),
                            });
                        }
                    }
                    Some(Protocol::Stream(StreamKind::Discovery(
                        DiscoveryAlgorithm::Kademlia1_0_0,
                    ))) => {
                        // init the stream
                        unimplemented!()
                    }
                    Some(Protocol::Stream(StreamKind::Broadcast(
                        BroadcastAlgorithm::Meshsub1_1_0,
                    ))) => {
                        unimplemented!()
                    }
                    Some(Protocol::Stream(StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1))) => {
                        match a.kind {
                            SelectKind::Stream(peer_id, stream_id) => {
                                store.dispatch(P2pNetworkRpcInitAction {
                                    addr: a.addr,
                                    peer_id,
                                    stream_id,
                                    incoming: a.incoming,
                                });
                            }
                            _ => {}
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
                            SelectKind::Stream(_, _) => {}
                        }
                    }
                }
            }
            Self::SelectError(_) => {
                // TODO: close stream or connection
            }
            Self::YamuxDidInit(_) => {}
        }
    }
}
