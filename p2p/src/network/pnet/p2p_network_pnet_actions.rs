use std::net::SocketAddr;

use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{Data, P2pState};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(addr), debug(data), incoming), level = trace)]
pub enum P2pNetworkPnetAction {
    IncomingData {
        addr: SocketAddr,
        data: Data,
    },
    OutgoingData {
        addr: SocketAddr,
        data: Data,
    },
    #[action_event(level = debug)]
    SetupNonce {
        addr: SocketAddr,
        nonce: Data,
        incoming: bool,
    },
    Timeout {
        addr: SocketAddr,
    },
}

impl P2pNetworkPnetAction {
    pub fn addr(&self) -> &SocketAddr {
        match self {
            Self::IncomingData { addr, .. } => addr,
            Self::OutgoingData { addr, .. } => addr,
            Self::SetupNonce { addr, .. } => addr,
            Self::Timeout { addr } => addr,
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
