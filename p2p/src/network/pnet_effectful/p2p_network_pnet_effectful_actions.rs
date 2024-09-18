use crate::{ConnectionAddr, Data, P2pState};
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(addr), debug(data), incoming), level = trace)]
pub enum P2pNetworkPnetEffectfulAction {
    OutgoingData {
        addr: ConnectionAddr,
        data: Vec<u8>,
    },
    #[action_event(level = debug)]
    SetupNonce {
        addr: ConnectionAddr,
        nonce: Data,
        incoming: bool,
    },
}

impl P2pNetworkPnetEffectfulAction {
    pub fn addr(&self) -> &ConnectionAddr {
        match self {
            Self::OutgoingData { addr, .. } => addr,
            Self::SetupNonce { addr, .. } => addr,
        }
    }
}

impl From<P2pNetworkPnetEffectfulAction> for crate::P2pAction {
    fn from(a: P2pNetworkPnetEffectfulAction) -> Self {
        Self::Network(a.into())
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkPnetEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}
