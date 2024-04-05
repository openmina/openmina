use std::net::SocketAddr;

use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{Data, P2pState, PeerId};

use super::{super::*, *};

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(addr), select_kind = debug(kind), debug(data), send_handshake, fin, debug(token), debug(tokens)))]
pub enum P2pNetworkSelectAction {
    /// Initialize protocol selection.
    ///
    /// Multistream Select protocol is running multiple times:
    /// When Pnet protocol is done for newly established TCP connection. We don't have `peer_id` yet.
    /// When Noise protocol is done and we have a `peer_id`.
    /// For each yamux stream opened, we have a `peer_id` and `stream_id` at this point.
    Init {
        addr: SocketAddr,
        kind: SelectKind,
        incoming: bool,
        send_handshake: bool,
    },
    #[action_event(level = trace)]
    IncomingData {
        addr: SocketAddr,
        kind: SelectKind,
        data: Data,
        fin: bool,
    },
    IncomingToken {
        addr: SocketAddr,
        kind: SelectKind,
        token: token::Token,
    },
    OutgoingTokens {
        addr: SocketAddr,
        kind: SelectKind,
        tokens: Vec<token::Token>,
    },
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum SelectKind {
    #[default]
    Authentication,
    MultiplexingNoPeerId,
    Multiplexing(PeerId),
    Stream(PeerId, StreamId),
}

impl SelectKind {
    pub fn peer_id(&self) -> Option<PeerId> {
        match self {
            Self::Authentication => None,
            Self::MultiplexingNoPeerId => None,
            Self::Multiplexing(v) => Some(*v),
            Self::Stream(v, _) => Some(*v),
        }
    }

    pub fn stream_id(&self) -> Option<StreamId> {
        match self {
            Self::Authentication => None,
            Self::MultiplexingNoPeerId => None,
            Self::Multiplexing(_) => None,
            Self::Stream(_, v) => Some(*v),
        }
    }
}

impl P2pNetworkSelectAction {
    pub fn addr(&self) -> &SocketAddr {
        match self {
            Self::Init { addr, .. } => addr,
            Self::IncomingData { addr, .. } => addr,
            Self::IncomingToken { addr, .. } => addr,
            Self::OutgoingTokens { addr, .. } => addr,
        }
    }

    pub fn id(&self) -> &SelectKind {
        match self {
            Self::Init { kind, .. } => kind,
            Self::IncomingData { kind, .. } => kind,
            Self::IncomingToken { kind, .. } => kind,
            Self::OutgoingTokens { kind, .. } => kind,
        }
    }
}

impl From<P2pNetworkSelectAction> for crate::P2pAction {
    fn from(a: P2pNetworkSelectAction) -> Self {
        Self::Network(a.into())
    }
}
impl redux::EnablingCondition<P2pState> for P2pNetworkSelectAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            Self::Init { .. } => true,
            Self::IncomingData { .. } => true,
            Self::IncomingToken { .. } => true,
            Self::OutgoingTokens { .. } => true,
        }
    }
}
