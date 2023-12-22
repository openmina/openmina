mod p2p_network_connection_actions;
pub use self::p2p_network_connection_actions::*;

use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, SocketAddr},
};

use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use crate::{
    connection::outgoing::P2pConnectionOutgoingInitOpts, webrtc::Host, MioCmd, P2pMioService,
    P2pNetworkPnetIncomingDataAction,
};

use super::pnet::P2pNetworkPnetState;

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
                    },
                );
            }
            P2pNetworkConnectionAction::IncomingDataIsReady(_) => {}
            P2pNetworkConnectionAction::IncomingDataDidReceive(_) => {}
        }
    }
}

impl P2pNetworkConnectionAction {
    pub fn effects<Store, S>(&self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService,
        P2pNetworkPnetIncomingDataAction: redux::EnablingCondition<S>,
    {
        match self {
            Self::InterfaceDetected(a) => {
                let port = store.state().config.libp2p_port.unwrap_or_default();
                store
                    .service()
                    .send_mio_cmd(MioCmd::ListenOn(SocketAddr::new(a.ip, port)));
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
            Self::OutgoingDidConnect(_) => {}
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
