use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::P2pState;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkConnectionAction {
    InterfaceDetected(P2pNetworkConnectionInterfaceDetectedAction),
    InterfaceExpired(P2pNetworkConnectionInterfaceExpiredAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkConnectionInterfaceDetectedAction {
    pub ip: IpAddr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkConnectionInterfaceExpiredAction {
    pub ip: IpAddr,
}

impl From<P2pNetworkConnectionInterfaceDetectedAction> for crate::P2pAction {
    fn from(a: P2pNetworkConnectionInterfaceDetectedAction) -> Self {
        Self::Network(P2pNetworkConnectionAction::from(a).into())
    }
}

impl From<P2pNetworkConnectionInterfaceExpiredAction> for crate::P2pAction {
    fn from(a: P2pNetworkConnectionInterfaceExpiredAction) -> Self {
        Self::Network(P2pNetworkConnectionAction::from(a).into())
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkConnectionAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        match self {
            Self::InterfaceDetected(a) => a.is_enabled(state),
            Self::InterfaceExpired(a) => a.is_enabled(state),
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkConnectionInterfaceDetectedAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkConnectionInterfaceExpiredAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}
