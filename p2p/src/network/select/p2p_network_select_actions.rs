use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

use super::{super::P2pNetworkAction, *};

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkSelectAction {
    Init(P2pNetworkSelectInitAction),
    IncomingData(P2pNetworkSelectIncomingDataAction),
    IncomingToken(P2pNetworkSelectIncomingTokenAction),
    OutgoingTokens(P2pNetworkSelectOutgoingTokensAction),
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum SelectKind {
    #[default]
    Authentication,
    Multiplexing(PeerId),
    Stream(PeerId, u16),
}

impl SelectKind {
    pub fn peer_id(&self) -> Option<PeerId> {
        match self {
            Self::Authentication => None,
            Self::Multiplexing(v) => Some(*v),
            Self::Stream(v, _) => Some(*v),
        }
    }

    pub fn stream_id(&self) -> Option<u16> {
        match self {
            Self::Authentication => None,
            Self::Multiplexing(_) => None,
            Self::Stream(_, v) => Some(*v),
        }
    }
}

impl P2pNetworkSelectAction {
    pub fn addr(&self) -> SocketAddr {
        match self {
            Self::Init(v) => v.addr,
            Self::IncomingData(v) => v.addr,
            Self::IncomingToken(v) => v.addr,
            Self::OutgoingTokens(v) => v.addr,
        }
    }

    pub fn id(&self) -> SelectKind {
        match self {
            Self::Init(v) => v.kind,
            Self::IncomingData(v) => v.kind,
            Self::IncomingToken(v) => v.kind,
            Self::OutgoingTokens(v) => v.kind,
        }
    }
}

/// Multistream Select protocol is running multiple times:
/// When Pnet protocol is done for newly established TCP connection. We don't have `peer_id` yet.
/// When Noise protocol is done and we have a `peer_id`.
/// For each yamux stream opened, we have a `peer_id` and `stream_id` at this point.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSelectInitAction {
    pub addr: SocketAddr,
    pub kind: SelectKind,
    pub incoming: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSelectIncomingDataAction {
    pub addr: SocketAddr,
    pub kind: SelectKind,
    pub data: Box<[u8]>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSelectIncomingTokenAction {
    pub addr: SocketAddr,
    pub kind: SelectKind,
    pub token: token::Token,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSelectOutgoingTokensAction {
    pub addr: SocketAddr,
    pub kind: SelectKind,
    pub tokens: Vec<token::Token>,
}

impl From<P2pNetworkSelectInitAction> for crate::P2pAction {
    fn from(a: P2pNetworkSelectInitAction) -> Self {
        Self::Network(P2pNetworkAction::Select(a.into()))
    }
}

impl From<P2pNetworkSelectIncomingDataAction> for crate::P2pAction {
    fn from(a: P2pNetworkSelectIncomingDataAction) -> Self {
        Self::Network(P2pNetworkAction::Select(a.into()))
    }
}

impl From<P2pNetworkSelectIncomingTokenAction> for crate::P2pAction {
    fn from(a: P2pNetworkSelectIncomingTokenAction) -> Self {
        Self::Network(P2pNetworkAction::Select(a.into()))
    }
}

impl From<P2pNetworkSelectOutgoingTokensAction> for crate::P2pAction {
    fn from(a: P2pNetworkSelectOutgoingTokensAction) -> Self {
        Self::Network(P2pNetworkAction::Select(a.into()))
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSelectAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        match self {
            Self::Init(v) => v.is_enabled(state),
            Self::IncomingData(v) => v.is_enabled(state),
            Self::IncomingToken(v) => v.is_enabled(state),
            Self::OutgoingTokens(v) => v.is_enabled(state),
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSelectInitAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSelectIncomingDataAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSelectIncomingTokenAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSelectOutgoingTokensAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}
