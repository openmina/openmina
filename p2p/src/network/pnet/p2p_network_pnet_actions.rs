use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::P2pState;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkPnetAction {
    IncomingData(P2pNetworkPnetIncomingDataAction),
    OutgoingData(P2pNetworkPnetOutgoingDataAction),
    SetupNonce(P2pNetworkPnetSetupNonceAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPnetIncomingDataAction {
    pub data: Box<[u8]>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPnetOutgoingDataAction {
    pub addr: SocketAddr,
    pub data: Box<[u8]>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPnetSetupNonceAction {
    pub nonce: [u8; 24],
}

impl From<P2pNetworkPnetAction> for crate::P2pAction {
    fn from(a: P2pNetworkPnetAction) -> Self {
        Self::Network(a.into())
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
