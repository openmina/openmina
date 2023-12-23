use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

use super::{super::P2pNetworkAction, SelectKind, *};

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkSelectAction {
    Init(P2pNetworkSelectInitAction),
    IncomingData(P2pNetworkSelectIncomingDataAction),
    IncomingToken(P2pNetworkSelectIncomingTokenAction),
}

impl P2pNetworkSelectAction {
    pub fn addr(&self) -> SocketAddr {
        match self {
            Self::Init(v) => v.addr,
            Self::IncomingData(v) => v.addr,
            Self::IncomingToken(v) => v.addr,
        }
    }

    pub fn peer_id(&self) -> Option<PeerId> {
        match self {
            Self::Init(v) => v.peer_id,
            Self::IncomingData(v) => v.peer_id,
            Self::IncomingToken(v) => v.peer_id,
        }
    }

    pub fn stream_id(&self) -> Option<u16> {
        match self {
            Self::Init(v) => v.stream_id,
            Self::IncomingData(v) => v.stream_id,
            Self::IncomingToken(v) => v.stream_id,
        }
    }

    pub fn select_kind(&self) -> SelectKind {
        if self.stream_id().is_some() {
            SelectKind::Stream
        } else if self.peer_id().is_some() {
            // when we don't have `stream_id`, but have `peer_id`
            // it means that multiplexing is needed
            SelectKind::Multiplexing
        } else {
            // when we don't have `peer_id` it means that authentication is needed
            SelectKind::Authentication
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
    pub peer_id: Option<PeerId>,
    pub stream_id: Option<u16>,
    pub incoming: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSelectIncomingDataAction {
    pub addr: SocketAddr,
    pub peer_id: Option<PeerId>,
    pub stream_id: Option<u16>,
    pub data: Box<[u8]>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSelectIncomingTokenAction {
    pub addr: SocketAddr,
    pub peer_id: Option<PeerId>,
    pub stream_id: Option<u16>,
    pub token: token::Token,
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

impl redux::EnablingCondition<P2pState> for P2pNetworkSelectAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        match self {
            Self::Init(v) => v.is_enabled(state),
            Self::IncomingData(v) => v.is_enabled(state),
            Self::IncomingToken(v) => v.is_enabled(state),
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
