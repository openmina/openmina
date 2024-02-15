use std::net::{IpAddr, SocketAddr};

use serde::{Deserialize, Serialize};

use super::super::{
    select::{token, SelectKind},
    Data,
};
use crate::{P2pState, PeerId};

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkSchedulerAction {
    InterfaceDetected(P2pNetworkSchedulerInterfaceDetectedAction),
    InterfaceExpired(P2pNetworkSchedulerInterfaceExpiredAction),
    IncomingConnectionIsReady(P2pNetworkSchedulerIncomingConnectionIsReadyAction),
    IncomingDidAccept(P2pNetworkSchedulerIncomingDidAcceptAction),
    OutgoingDidConnect(P2pNetworkSchedulerOutgoingDidConnectAction),
    IncomingDataIsReady(P2pNetworkSchedulerIncomingDataIsReadyAction),
    IncomingDataDidReceive(P2pNetworkSchedulerIncomingDataDidReceiveAction),
    SelectDone(P2pNetworkSchedulerSelectDoneAction),
    SelectError(P2pNetworkSchedulerSelectErrorAction),
    YamuxDidInit(P2pNetworkSchedulerYamuxDidInitAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSchedulerInterfaceDetectedAction {
    pub ip: IpAddr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSchedulerInterfaceExpiredAction {
    pub ip: IpAddr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSchedulerIncomingConnectionIsReadyAction {
    pub listener: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSchedulerIncomingDidAcceptAction {
    pub addr: Option<SocketAddr>,
    pub result: Result<(), String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSchedulerOutgoingDidConnectAction {
    pub addr: SocketAddr,
    pub result: Result<(), String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSchedulerIncomingDataIsReadyAction {
    pub addr: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSchedulerIncomingDataDidReceiveAction {
    pub addr: SocketAddr,
    pub result: Result<Data, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSchedulerSelectDoneAction {
    pub addr: SocketAddr,
    pub kind: SelectKind,
    pub protocol: Option<token::Protocol>,
    pub incoming: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSchedulerSelectErrorAction {
    pub addr: SocketAddr,
    pub kind: SelectKind,
    pub error: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSchedulerYamuxDidInitAction {
    pub peer_id: PeerId,
    pub addr: SocketAddr,
}

impl From<P2pNetworkSchedulerInterfaceDetectedAction> for crate::P2pAction {
    fn from(a: P2pNetworkSchedulerInterfaceDetectedAction) -> Self {
        Self::Network(P2pNetworkSchedulerAction::from(a).into())
    }
}

impl From<P2pNetworkSchedulerInterfaceExpiredAction> for crate::P2pAction {
    fn from(a: P2pNetworkSchedulerInterfaceExpiredAction) -> Self {
        Self::Network(P2pNetworkSchedulerAction::from(a).into())
    }
}

impl From<P2pNetworkSchedulerIncomingConnectionIsReadyAction> for crate::P2pAction {
    fn from(a: P2pNetworkSchedulerIncomingConnectionIsReadyAction) -> Self {
        Self::Network(P2pNetworkSchedulerAction::from(a).into())
    }
}

impl From<P2pNetworkSchedulerIncomingDidAcceptAction> for crate::P2pAction {
    fn from(a: P2pNetworkSchedulerIncomingDidAcceptAction) -> Self {
        Self::Network(P2pNetworkSchedulerAction::from(a).into())
    }
}

impl From<P2pNetworkSchedulerOutgoingDidConnectAction> for crate::P2pAction {
    fn from(a: P2pNetworkSchedulerOutgoingDidConnectAction) -> Self {
        Self::Network(P2pNetworkSchedulerAction::from(a).into())
    }
}

impl From<P2pNetworkSchedulerIncomingDataIsReadyAction> for crate::P2pAction {
    fn from(a: P2pNetworkSchedulerIncomingDataIsReadyAction) -> Self {
        Self::Network(P2pNetworkSchedulerAction::from(a).into())
    }
}

impl From<P2pNetworkSchedulerIncomingDataDidReceiveAction> for crate::P2pAction {
    fn from(a: P2pNetworkSchedulerIncomingDataDidReceiveAction) -> Self {
        Self::Network(P2pNetworkSchedulerAction::from(a).into())
    }
}

impl From<P2pNetworkSchedulerSelectDoneAction> for crate::P2pAction {
    fn from(a: P2pNetworkSchedulerSelectDoneAction) -> Self {
        Self::Network(P2pNetworkSchedulerAction::from(a).into())
    }
}

impl From<P2pNetworkSchedulerSelectErrorAction> for crate::P2pAction {
    fn from(a: P2pNetworkSchedulerSelectErrorAction) -> Self {
        Self::Network(P2pNetworkSchedulerAction::from(a).into())
    }
}

impl From<P2pNetworkSchedulerYamuxDidInitAction> for crate::P2pAction {
    fn from(a: P2pNetworkSchedulerYamuxDidInitAction) -> Self {
        Self::Network(P2pNetworkSchedulerAction::from(a).into())
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        match self {
            Self::InterfaceDetected(a) => a.is_enabled(state, time),
            Self::InterfaceExpired(a) => a.is_enabled(state, time),
            Self::IncomingConnectionIsReady(a) => a.is_enabled(state, time),
            Self::IncomingDidAccept(a) => a.is_enabled(state, time),
            Self::OutgoingDidConnect(a) => a.is_enabled(state, time),
            Self::IncomingDataIsReady(a) => a.is_enabled(state, time),
            Self::IncomingDataDidReceive(a) => a.is_enabled(state, time),
            Self::SelectDone(a) => a.is_enabled(state, time),
            Self::SelectError(a) => a.is_enabled(state, time),
            Self::YamuxDidInit(a) => a.is_enabled(state, time),
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerInterfaceDetectedAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerInterfaceExpiredAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerIncomingConnectionIsReadyAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerIncomingDidAcceptAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerOutgoingDidConnectAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerIncomingDataIsReadyAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerIncomingDataDidReceiveAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerSelectDoneAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerSelectErrorAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerYamuxDidInitAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}
