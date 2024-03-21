use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::{Data, P2pState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkPnetAction {
    IncomingData {
        addr: SocketAddr,
        data: Data,
    },
    OutgoingData {
        addr: SocketAddr,
        data: Data,
    },
    SetupNonce {
        addr: SocketAddr,
        nonce: Data,
        incoming: bool,
    },
}

impl P2pNetworkPnetAction {
    pub fn addr(&self) -> &SocketAddr {
        match self {
            Self::IncomingData { addr, .. } => addr,
            Self::OutgoingData { addr, .. } => addr,
            Self::SetupNonce { addr, .. } => addr,
        }
    }
}

impl From<P2pNetworkPnetAction> for crate::P2pAction {
    fn from(a: P2pNetworkPnetAction) -> Self {
        Self::Network(a.into())
    }
}


impl redux::EnablingCondition<P2pState> for P2pNetworkPnetAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

