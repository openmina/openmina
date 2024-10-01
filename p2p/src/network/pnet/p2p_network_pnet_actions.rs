use crate::{ConnectionAddr, Data, P2pState};
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(addr), debug(data), incoming), level = trace)]
pub enum P2pNetworkPnetAction {
    IncomingData {
        addr: ConnectionAddr,
        data: Data,
    },
    OutgoingData {
        addr: ConnectionAddr,
        data: Data,
    },
    #[action_event(level = debug)]
    SetupNonce {
        addr: ConnectionAddr,
        nonce: Data,
        incoming: bool,
    },
    Timeout {
        addr: ConnectionAddr,
    },
}

impl P2pNetworkPnetAction {
    pub fn addr(&self) -> &ConnectionAddr {
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
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        state
            .network
            .scheduler
            .connection_state(self.addr())
            .is_some()
    }
}
