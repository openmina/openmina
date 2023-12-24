use std::net::{IpAddr, SocketAddr};

use serde::{Deserialize, Serialize};

use super::super::select::{token, SelectKind};
use crate::P2pState;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkConnectionAction {
    InterfaceDetected(P2pNetworkConnectionInterfaceDetectedAction),
    InterfaceExpired(P2pNetworkConnectionInterfaceExpiredAction),
    OutgoingDidConnect(P2pNetworkConnectionOutgoingDidConnectAction),
    IncomingDataIsReady(P2pNetworkConnectionIncomingDataIsReadyAction),
    IncomingDataDidReceive(P2pNetworkConnectionIncomingDataDidReceiveAction),
    SelectDone(P2pNetworkConnectionSelectDoneAction),
    SelectError(P2pNetworkConnectionSelectErrorAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkConnectionInterfaceDetectedAction {
    pub ip: IpAddr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkConnectionInterfaceExpiredAction {
    pub ip: IpAddr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkConnectionOutgoingDidConnectAction {
    pub addr: SocketAddr,
    pub result: Result<(), String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkConnectionIncomingDataIsReadyAction {
    pub addr: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkConnectionIncomingDataDidReceiveAction {
    pub addr: SocketAddr,
    pub result: Result<(Box<[u8]>, usize), String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkConnectionSelectDoneAction {
    pub addr: SocketAddr,
    pub kind: SelectKind,
    pub protocol: token::Protocol,
    pub incoming: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkConnectionSelectErrorAction {
    pub addr: SocketAddr,
    pub kind: SelectKind,
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

impl From<P2pNetworkConnectionOutgoingDidConnectAction> for crate::P2pAction {
    fn from(a: P2pNetworkConnectionOutgoingDidConnectAction) -> Self {
        Self::Network(P2pNetworkConnectionAction::from(a).into())
    }
}

impl From<P2pNetworkConnectionIncomingDataIsReadyAction> for crate::P2pAction {
    fn from(a: P2pNetworkConnectionIncomingDataIsReadyAction) -> Self {
        Self::Network(P2pNetworkConnectionAction::from(a).into())
    }
}

impl From<P2pNetworkConnectionIncomingDataDidReceiveAction> for crate::P2pAction {
    fn from(a: P2pNetworkConnectionIncomingDataDidReceiveAction) -> Self {
        Self::Network(P2pNetworkConnectionAction::from(a).into())
    }
}

impl From<P2pNetworkConnectionSelectDoneAction> for crate::P2pAction {
    fn from(a: P2pNetworkConnectionSelectDoneAction) -> Self {
        Self::Network(P2pNetworkConnectionAction::from(a).into())
    }
}

impl From<P2pNetworkConnectionSelectErrorAction> for crate::P2pAction {
    fn from(a: P2pNetworkConnectionSelectErrorAction) -> Self {
        Self::Network(P2pNetworkConnectionAction::from(a).into())
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkConnectionAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        match self {
            Self::InterfaceDetected(a) => a.is_enabled(state),
            Self::InterfaceExpired(a) => a.is_enabled(state),
            Self::OutgoingDidConnect(a) => a.is_enabled(state),
            Self::IncomingDataIsReady(a) => a.is_enabled(state),
            Self::IncomingDataDidReceive(a) => a.is_enabled(state),
            Self::SelectDone(a) => a.is_enabled(state),
            Self::SelectError(a) => a.is_enabled(state),
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

impl redux::EnablingCondition<P2pState> for P2pNetworkConnectionOutgoingDidConnectAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkConnectionIncomingDataIsReadyAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkConnectionIncomingDataDidReceiveAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkConnectionSelectDoneAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkConnectionSelectErrorAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}
