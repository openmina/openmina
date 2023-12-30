use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use super::super::*;
use crate::{P2pState, PeerId};

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkRpcAction {
    Init(P2pNetworkRpcInitAction),
    IncomingData(P2pNetworkRpcIncomingDataAction),
    OutgoingData(P2pNetworkRpcOutgoingDataAction),
}

impl P2pNetworkRpcAction {
    pub fn addr(&self) -> SocketAddr {
        match self {
            Self::Init(a) => a.addr,
            Self::IncomingData(a) => a.addr,
            Self::OutgoingData(a) => a.addr,
        }
    }

    pub fn stream_id(&self) -> StreamId {
        match self {
            Self::Init(a) => a.stream_id,
            Self::IncomingData(a) => a.stream_id,
            Self::OutgoingData(a) => a.stream_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkRpcInitAction {
    pub addr: SocketAddr,
    pub peer_id: PeerId,
    pub stream_id: StreamId,
    pub incoming: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkRpcIncomingDataAction {
    pub addr: SocketAddr,
    pub peer_id: PeerId,
    pub stream_id: StreamId,
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkRpcOutgoingDataAction {
    pub addr: SocketAddr,
    pub peer_id: PeerId,
    pub stream_id: StreamId,
    pub data: Data,
    pub fin: bool,
}

impl From<P2pNetworkRpcInitAction> for crate::P2pAction {
    fn from(a: P2pNetworkRpcInitAction) -> Self {
        Self::Network(P2pNetworkAction::Rpc(a.into()))
    }
}

impl From<P2pNetworkRpcIncomingDataAction> for crate::P2pAction {
    fn from(a: P2pNetworkRpcIncomingDataAction) -> Self {
        Self::Network(P2pNetworkAction::Rpc(a.into()))
    }
}

impl From<P2pNetworkRpcOutgoingDataAction> for crate::P2pAction {
    fn from(a: P2pNetworkRpcOutgoingDataAction) -> Self {
        Self::Network(P2pNetworkAction::Rpc(a.into()))
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkRpcAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        match self {
            Self::Init(v) => v.is_enabled(state),
            Self::IncomingData(v) => v.is_enabled(state),
            Self::OutgoingData(v) => v.is_enabled(state),
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkRpcInitAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkRpcIncomingDataAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkRpcOutgoingDataAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}
