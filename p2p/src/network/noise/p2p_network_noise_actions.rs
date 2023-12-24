use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::{Data, P2pNetworkAction, P2pState};

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkNoiseAction {
    Init(P2pNetworkNoiseInitAction),
    IncomingData(P2pNetworkNoiseIncomingDataAction),
}

impl P2pNetworkNoiseAction {
    pub fn addr(&self) -> SocketAddr {
        match self {
            Self::Init(a) => a.addr,
            Self::IncomingData(a) => a.addr,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseInitAction {
    pub addr: SocketAddr,
    pub incoming: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseIncomingDataAction {
    pub addr: SocketAddr,
    pub data: Data,
}

impl From<P2pNetworkNoiseInitAction> for crate::P2pAction {
    fn from(a: P2pNetworkNoiseInitAction) -> Self {
        Self::Network(P2pNetworkAction::Noise(a.into()))
    }
}

impl From<P2pNetworkNoiseIncomingDataAction> for crate::P2pAction {
    fn from(a: P2pNetworkNoiseIncomingDataAction) -> Self {
        Self::Network(P2pNetworkAction::Noise(a.into()))
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkNoiseAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        match self {
            Self::Init(v) => v.is_enabled(state),
            Self::IncomingData(v) => v.is_enabled(state),
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkNoiseInitAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkNoiseIncomingDataAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}
