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
    pub select_mux: P2pNetworkSelectState,
    pub streams: BTreeMap<u16, P2pNetworkSelectState>,
}

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
                        select_auth: P2pNetworkSelectState::default(),
                        select_mux: P2pNetworkSelectState::default(),
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
        }
    }
}
