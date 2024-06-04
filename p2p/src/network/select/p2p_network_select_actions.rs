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
    IncomingDataAuth {
        addr: SocketAddr,
        data: Data,
        fin: bool,
    },
    #[action_event(level = trace)]
    IncomingDataMux {
        addr: SocketAddr,
        peer_id: Option<PeerId>,
        data: Data,
        fin: bool,
    },
    #[action_event(level = trace)]
    IncomingData {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
        data: Data,
        fin: bool,
    },
    IncomingPayloadAuth {
        addr: SocketAddr,
        fin: bool,
        data: Data,
    },
    IncomingPayloadMux {
        addr: SocketAddr,
        peer_id: Option<PeerId>,
        fin: bool,
        data: Data,
    },
    IncomingPayload {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
        fin: bool,
        data: Data,
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
    Timeout {
        addr: SocketAddr,
        kind: SelectKind,
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
    pub fn stream_id(&self) -> Option<StreamId> {
        match self {
            Self::Authentication => None,
            Self::MultiplexingNoPeerId => None,
            Self::Multiplexing(_) => None,
            Self::Stream(_, v) => Some(*v),
        }
    }

    pub fn peer_id(&self) -> Option<PeerId> {
        match self {
            Self::Authentication => None,
            Self::MultiplexingNoPeerId => None,
            Self::Multiplexing(peer_id) => Some(*peer_id),
            Self::Stream(peer_id, _) => Some(*peer_id),
        }
    }
}

impl P2pNetworkSelectAction {
    pub fn addr(&self) -> &SocketAddr {
        match self {
            Self::Init { addr, .. } => addr,
            Self::IncomingDataAuth { addr, .. } => addr,
            Self::IncomingDataMux { addr, .. } => addr,
            Self::IncomingData { addr, .. } => addr,
            Self::IncomingPayloadAuth { addr, .. } => addr,
            Self::IncomingPayloadMux { addr, .. } => addr,
            Self::IncomingPayload { addr, .. } => addr,
            Self::IncomingToken { addr, .. } => addr,
            Self::OutgoingTokens { addr, .. } => addr,
            Self::Timeout { addr, .. } => addr,
        }
    }

    pub fn id(&self) -> SelectKind {
        match self {
            Self::Init { kind, .. } => *kind,
            Self::IncomingDataAuth { .. } => SelectKind::Authentication,
            Self::IncomingDataMux {
                peer_id: Some(peer_id),
                ..
            } => SelectKind::Multiplexing(*peer_id),
            Self::IncomingDataMux { peer_id: None, .. } => SelectKind::MultiplexingNoPeerId,
            Self::IncomingData {
                peer_id, stream_id, ..
            } => SelectKind::Stream(*peer_id, *stream_id),
            Self::IncomingPayloadAuth { .. } => SelectKind::Authentication,
            Self::IncomingPayloadMux {
                peer_id: Some(peer_id),
                ..
            } => SelectKind::Multiplexing(*peer_id),
            Self::IncomingPayloadMux { peer_id: None, .. } => SelectKind::MultiplexingNoPeerId,
            Self::IncomingPayload {
                peer_id, stream_id, ..
            } => SelectKind::Stream(*peer_id, *stream_id),
            Self::IncomingToken { kind, .. } => *kind,
            Self::OutgoingTokens { kind, .. } => *kind,
            Self::Timeout { kind, .. } => *kind,
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
            Self::IncomingDataAuth { .. } => true,
            Self::IncomingDataMux { .. } => true,
            Self::IncomingData { .. } => true,
            Self::IncomingPayloadAuth { .. } => true,
            Self::IncomingPayloadMux { .. } => true,
            Self::IncomingPayload { .. } => true,
            Self::IncomingToken { .. } => true,
            Self::OutgoingTokens { .. } => true,
            Self::Timeout { .. } => true,
        }
    }
}
