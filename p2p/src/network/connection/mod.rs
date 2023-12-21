mod p2p_network_connection_action;
pub use self::p2p_network_connection_action::*;

use std::{
    collections::BTreeSet,
    net::{IpAddr, SocketAddr},
};

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]

pub struct P2pNetworkConnectionState {
    pub interfaces: BTreeSet<IpAddr>,
    pub listeners: BTreeSet<SocketAddr>,
}

impl P2pNetworkConnectionState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkConnectionAction>) {
        match action.action() {
            P2pNetworkConnectionAction::InterfaceDetected(ip) => drop(self.interfaces.insert(*ip)),
            P2pNetworkConnectionAction::InterfaceExpired(ip) => drop(self.interfaces.remove(ip)),
        }
    }
}
