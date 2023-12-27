use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, SocketAddr},
};

use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use crate::{
    connection::outgoing::P2pConnectionOutgoingInitOpts, webrtc::Host, MioCmd, P2pCryptoService,
    P2pMioService,
};

use super::{super::*, *};

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct P2pNetworkSchedulerState {
    pub interfaces: BTreeSet<IpAddr>,
    pub listeners: BTreeSet<SocketAddr>,
    pub pnet_key: [u8; 32],
    pub connections: BTreeMap<SocketAddr, P2pNetworkConnectionState>,
    pub rpc_behaviour_state: (),
    pub broadcast_behaviour_state: (),
    pub discovery_behaviour_state: (),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkConnectionState {
    pub pnet: P2pNetworkPnetState,
    pub select_auth: P2pNetworkSelectState,
    pub auth: Option<P2pNetworkAuthState>,
    pub select_mux: P2pNetworkSelectState,
    pub mux: Option<P2pNetworkConnectionMuxState>,
    pub streams: BTreeMap<u16, P2pNetworkStreamState>,
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
    pub readable: bool,
    pub writable: bool,
    pub window: i32,
    pub select: P2pNetworkSelectState,
    pub handler: Option<P2pNetworkStreamHandlerState>,
}

impl P2pNetworkStreamState {
    pub fn new(stream_kind: token::StreamKind) -> Self {
        P2pNetworkStreamState {
            readable: true,
            writable: true,
            window: 1 << 18,
            select: P2pNetworkSelectState::initiator_stream(stream_kind),
            handler: None,
        }
    }

    pub fn new_incoming() -> Self {
        P2pNetworkStreamState {
            readable: true,
            writable: true,
            window: 1 << 18,
            select: P2pNetworkSelectState::default(),
            handler: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkStreamHandlerState {
    Rpc,
    Broadcast,
    Discovery,
}

impl P2pNetworkSchedulerState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkSchedulerAction>) {
        match action.action() {
            P2pNetworkSchedulerAction::InterfaceDetected(a) => drop(self.interfaces.insert(a.ip)),
            P2pNetworkSchedulerAction::InterfaceExpired(a) => drop(self.interfaces.remove(&a.ip)),
            P2pNetworkSchedulerAction::OutgoingDidConnect(a) => {
                self.connections.insert(
                    a.addr,
                    P2pNetworkConnectionState {
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
                match &a.protocol {
                    token::Protocol::Auth(token::AuthKind::Noise) => {
                        connection.auth =
                            Some(P2pNetworkAuthState::Noise(P2pNetworkNoiseState::default()));
                    }
                    token::Protocol::Mux(token::MuxKind::Yamux1_0_0) => {
                        connection.mux = Some(P2pNetworkConnectionMuxState::Yamux(
                            P2pNetworkYamuxState::default(),
                        ));
                    }
                    token::Protocol::Stream(stream_kind) => {
                        let Some(stream_id) = a.kind.stream_id() else {
                            return;
                        };
                        if let Some(stream) = connection.streams.get_mut(&stream_id) {
                            match stream_kind {
                                token::StreamKind::Rpc(_) => {
                                    stream.handler = Some(P2pNetworkStreamHandlerState::Rpc)
                                }
                                token::StreamKind::Broadcast(_) => {
                                    stream.handler = Some(P2pNetworkStreamHandlerState::Broadcast)
                                }
                                token::StreamKind::Discovery(_) => {
                                    stream.handler = Some(P2pNetworkStreamHandlerState::Discovery)
                                }
                            }
                        }
                    }
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
    {
        match self {
            Self::InterfaceDetected(a) => {
                let port = store.state().config.libp2p_port.unwrap_or_default();
                store
                    .service()
                    .send_mio_cmd(MioCmd::ListenOn(SocketAddr::new(a.ip, port)));

                // TODO: connect all initial peers
                let addr = match &store.state().config.initial_peers[0] {
                    P2pConnectionOutgoingInitOpts::LibP2P(v) => match &v.host {
                        Host::Ipv4(ip) => SocketAddr::new((*ip).into(), v.port),
                        _ => panic!(),
                    },
                    _ => panic!(),
                };
                // let addr = SocketAddr::from(([172, 17, 0, 1], 8302));

                if addr.is_ipv4() == a.ip.is_ipv4() {
                    store.service().send_mio_cmd(MioCmd::Connect(addr));
                }
            }
            Self::InterfaceExpired(_) => {}
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
            Self::SelectDone(a) => match &a.protocol {
                token::Protocol::Auth(token::AuthKind::Noise) => {
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
                _ => {}
            },
            Self::SelectError(_) => {}
        }
    }
}
