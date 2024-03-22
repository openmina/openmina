use std::net::SocketAddr;

use mina_p2p_messages::rpc_kernel::{QueryHeader, ResponseHeader};
use serde::{Deserialize, Serialize};

use super::{super::*, *};
use crate::{P2pState, PeerId};

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkRpcAction {
    Init(P2pNetworkRpcInitAction),
    IncomingData(P2pNetworkRpcIncomingDataAction),
    IncomingMessage(P2pNetworkRpcIncomingMessageAction),
    OutgoingQuery(P2pNetworkRpcOutgoingQueryAction),
    OutgoingResponse(P2pNetworkRpcOutgoingResponseAction),
    OutgoingData(P2pNetworkRpcOutgoingDataAction),
}

pub enum RpcStreamId {
    Exact(StreamId),
    AnyIncoming,
    AnyOutgoing,
}

impl P2pNetworkRpcAction {
    pub fn stream_id(&self) -> RpcStreamId {
        match self {
            Self::Init(a) => RpcStreamId::Exact(a.stream_id),
            Self::IncomingData(a) => RpcStreamId::Exact(a.stream_id),
            Self::IncomingMessage(a) => RpcStreamId::Exact(a.stream_id),
            Self::OutgoingQuery(_) => RpcStreamId::AnyOutgoing,
            Self::OutgoingResponse(_) => RpcStreamId::AnyIncoming,
            Self::OutgoingData(a) => RpcStreamId::Exact(a.stream_id),
        }
    }

    pub fn peer_id(&self) -> PeerId {
        match self {
            Self::Init(a) => a.peer_id,
            Self::IncomingData(a) => a.peer_id,
            Self::IncomingMessage(a) => a.peer_id,
            Self::OutgoingQuery(a) => a.peer_id,
            Self::OutgoingResponse(a) => a.peer_id,
            Self::OutgoingData(a) => a.peer_id,
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
pub struct P2pNetworkRpcIncomingMessageAction {
    pub addr: SocketAddr,
    pub peer_id: PeerId,
    pub stream_id: StreamId,
    pub message: RpcMessage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkRpcOutgoingQueryAction {
    pub peer_id: PeerId,
    pub query: QueryHeader,
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkRpcOutgoingResponseAction {
    pub peer_id: PeerId,
    pub response: ResponseHeader,
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

impl From<P2pNetworkRpcIncomingMessageAction> for crate::P2pAction {
    fn from(a: P2pNetworkRpcIncomingMessageAction) -> Self {
        Self::Network(P2pNetworkAction::Rpc(a.into()))
    }
}

impl From<P2pNetworkRpcOutgoingQueryAction> for crate::P2pAction {
    fn from(a: P2pNetworkRpcOutgoingQueryAction) -> Self {
        Self::Network(P2pNetworkAction::Rpc(a.into()))
    }
}

impl From<P2pNetworkRpcOutgoingResponseAction> for crate::P2pAction {
    fn from(a: P2pNetworkRpcOutgoingResponseAction) -> Self {
        Self::Network(P2pNetworkAction::Rpc(a.into()))
    }
}

impl From<P2pNetworkRpcOutgoingDataAction> for crate::P2pAction {
    fn from(a: P2pNetworkRpcOutgoingDataAction) -> Self {
        Self::Network(P2pNetworkAction::Rpc(a.into()))
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkRpcAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        match self {
            Self::Init(v) => v.is_enabled(state, time),
            Self::IncomingData(v) => v.is_enabled(state, time),
            Self::IncomingMessage(v) => v.is_enabled(state, time),
            Self::OutgoingQuery(v) => v.is_enabled(state, time),
            Self::OutgoingResponse(v) => v.is_enabled(state, time),
            Self::OutgoingData(v) => v.is_enabled(state, time),
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkRpcInitAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkRpcIncomingDataAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkRpcIncomingMessageAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkRpcOutgoingQueryAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkRpcOutgoingResponseAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkRpcOutgoingDataAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}
