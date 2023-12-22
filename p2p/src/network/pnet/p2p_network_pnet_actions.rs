use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::P2pState;

use super::super::P2pNetworkAction;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkPnetAction {
    IncomingData(P2pNetworkPnetIncomingDataAction),
    OutgoingData(P2pNetworkPnetOutgoingDataAction),
    SetupNonce(P2pNetworkPnetSetupNonceAction),
}

impl P2pNetworkPnetAction {
    pub fn addr(&self) -> SocketAddr {
        match self {
            Self::IncomingData(a) => a.addr,
            Self::OutgoingData(a) => a.addr,
            Self::SetupNonce(a) => a.addr,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPnetIncomingDataAction {
    pub addr: SocketAddr,
    pub data: Box<[u8]>,
    pub len: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPnetOutgoingDataAction {
    pub addr: SocketAddr,
    pub data: Box<[u8]>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPnetSetupNonceAction {
    pub addr: SocketAddr,
    pub nonce: [u8; 24],
}

impl From<P2pNetworkPnetAction> for crate::P2pAction {
    fn from(a: P2pNetworkPnetAction) -> Self {
        Self::Network(a.into())
    }
}

impl From<P2pNetworkPnetIncomingDataAction> for crate::P2pAction {
    fn from(a: P2pNetworkPnetIncomingDataAction) -> Self {
        Self::Network(P2pNetworkAction::Pnet(a.into()))
    }
}

impl From<P2pNetworkPnetOutgoingDataAction> for crate::P2pAction {
    fn from(a: P2pNetworkPnetOutgoingDataAction) -> Self {
        Self::Network(P2pNetworkAction::Pnet(a.into()))
    }
}

impl From<P2pNetworkPnetSetupNonceAction> for crate::P2pAction {
    fn from(a: P2pNetworkPnetSetupNonceAction) -> Self {
        Self::Network(P2pNetworkAction::Pnet(a.into()))
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkPnetAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkPnetIncomingDataAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkPnetOutgoingDataAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkPnetSetupNonceAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}
