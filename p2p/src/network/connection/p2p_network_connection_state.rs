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

pub struct P2pNetworkConnectionState {
    pub interfaces: BTreeSet<IpAddr>,
    pub listeners: BTreeSet<SocketAddr>,
    pub pnet_key: [u8; 32],
    pub connections: BTreeMap<SocketAddr, P2pNetworkConnectionHandshakeState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkConnectionHandshakeState {
    pub pnet: P2pNetworkPnetState,
    pub select_auth: P2pNetworkSelectState,
    pub auth: Option<P2pNetworkAuthState>,
    pub select_mux: P2pNetworkSelectState,
    pub mux: Option<()>,
    pub streams: BTreeMap<u16, P2pNetworkSelectState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkAuthState {
    Noise(P2pNetworkNoiseState),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkConnectionMuxState {}

impl P2pNetworkConnectionState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkConnectionAction>) {
        match action.action() {
            P2pNetworkConnectionAction::InterfaceDetected(a) => drop(self.interfaces.insert(a.ip)),
            P2pNetworkConnectionAction::InterfaceExpired(a) => drop(self.interfaces.remove(&a.ip)),
            P2pNetworkConnectionAction::OutgoingDidConnect(a) => {
                self.connections.insert(
                    a.addr,
                    P2pNetworkConnectionHandshakeState {
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
            P2pNetworkConnectionAction::IncomingDataIsReady(_) => {}
            P2pNetworkConnectionAction::IncomingDataDidReceive(a) => {
                if a.result.is_err() {
                    self.connections.remove(&a.addr);
                }
            }
            P2pNetworkConnectionAction::SelectDone(a) => {
                let Some(connection) = self.connections.get_mut(&a.addr) else {
                    return;
                };
                match &a.protocol {
                    token::Protocol::Auth(token::AuthKind::Noise) => {
                        connection.auth =
                            Some(P2pNetworkAuthState::Noise(P2pNetworkNoiseState::default()));
                    }
                    token::Protocol::Mux(token::MuxKind::Yamux1_0_0) => {
                        connection.mux = Some(());
                    }
                    token::Protocol::Stream(stream_kind) => {
                        let Some(stream_id) = a.stream_id else {
                            return;
                        };
                        connection.streams.insert(
                            stream_id,
                            P2pNetworkSelectState::initiator_stream(*stream_kind),
                        );
                    }
                }
            }
            P2pNetworkConnectionAction::SelectError(a) => {
                if let Some(stream_id) = &a.stream_id {
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

impl P2pNetworkConnectionAction {
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

                if addr.is_ipv4() == a.ip.is_ipv4() {
                    store.service().send_mio_cmd(MioCmd::Connect(addr));
                }
            }
            Self::InterfaceExpired(_) => {}
            Self::OutgoingDidConnect(a) => {
                let nonce = store.service().generate_random_nonce();
                store.dispatch(P2pNetworkPnetSetupNonceAction {
                    addr: a.addr,
                    nonce,
                    incoming: false,
                });
            }
            Self::IncomingDataIsReady(a) => {
                store
                    .service()
                    .send_mio_cmd(MioCmd::Recv(a.addr, vec![0; 0x1000].into_boxed_slice()));
            }
            Self::IncomingDataDidReceive(a) => {
                if let Ok((data, len)) = &a.result {
                    store.dispatch(P2pNetworkPnetIncomingDataAction {
                        addr: a.addr,
                        data: data.clone(),
                        len: *len,
                    });
                }
            }
            Self::SelectDone(a) => match &a.protocol {
                token::Protocol::Auth(token::AuthKind::Noise) => {
                    // initialize Noise
                    store.dispatch(P2pNetworkNoiseInitAction {
                        addr: a.addr,
                        incoming: a.incoming,
                    });
                }
                _ => {}
            },
            Self::SelectError(_) => {}
        }
    }
}
