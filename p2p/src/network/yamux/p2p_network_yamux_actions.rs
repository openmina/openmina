use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use super::p2p_network_yamux_state::YamuxFrame;
use crate::{Data, P2pNetworkAction, P2pState};

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkYamuxAction {
    IncomingData(P2pNetworkYamuxIncomingDataAction),
    OutgoingData(P2pNetworkYamuxOutgoingDataAction),
    IncomingFrame(P2pNetworkYamuxIncomingFrameAction),
}

impl P2pNetworkYamuxAction {
    pub fn addr(&self) -> SocketAddr {
        match self {
            Self::IncomingData(a) => a.addr,
            Self::OutgoingData(a) => a.addr,
            Self::IncomingFrame(a) => a.addr,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkYamuxIncomingDataAction {
    pub addr: SocketAddr,
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkYamuxOutgoingDataAction {
    pub addr: SocketAddr,
    pub stream_id: u16,
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkYamuxIncomingFrameAction {
    pub addr: SocketAddr,
    pub frame: YamuxFrame,
}

impl From<P2pNetworkYamuxIncomingDataAction> for crate::P2pAction {
    fn from(a: P2pNetworkYamuxIncomingDataAction) -> Self {
        Self::Network(P2pNetworkAction::Yamux(a.into()))
    }
}

impl From<P2pNetworkYamuxOutgoingDataAction> for crate::P2pAction {
    fn from(a: P2pNetworkYamuxOutgoingDataAction) -> Self {
        Self::Network(P2pNetworkAction::Yamux(a.into()))
    }
}

impl From<P2pNetworkYamuxIncomingFrameAction> for crate::P2pAction {
    fn from(a: P2pNetworkYamuxIncomingFrameAction) -> Self {
        Self::Network(P2pNetworkAction::Yamux(a.into()))
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkYamuxAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        match self {
            Self::IncomingData(v) => v.is_enabled(state),
            Self::OutgoingData(v) => v.is_enabled(state),
            Self::IncomingFrame(v) => v.is_enabled(state),
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkYamuxIncomingDataAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkYamuxOutgoingDataAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkYamuxIncomingFrameAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}
