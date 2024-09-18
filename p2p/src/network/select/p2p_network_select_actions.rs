use super::{super::*, *};
use crate::{Data, P2pState, PeerId};
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(addr), select_kind = debug(kind), debug(data), fin, debug(token), debug(tokens)))]
pub enum P2pNetworkSelectAction {
    /// Initialize protocol selection.
    ///
    /// Multistream Select protocol is running multiple times:
    /// When Pnet protocol is done for newly established TCP connection. We don't have `peer_id` yet.
    /// When Noise protocol is done and we have a `peer_id`.
    /// For each yamux stream opened, we have a `peer_id` and `stream_id` at this point.
    Init {
        addr: ConnectionAddr,
        kind: SelectKind,
        incoming: bool,
    },
    #[action_event(level = trace)]
    IncomingDataAuth {
        addr: ConnectionAddr,
        data: Data,
        fin: bool,
    },
    #[action_event(level = trace)]
    IncomingDataMux {
        addr: ConnectionAddr,
        peer_id: Option<PeerId>,
        data: Data,
        fin: bool,
    },
    #[action_event(level = trace)]
    IncomingData {
        addr: ConnectionAddr,
        peer_id: PeerId,
        stream_id: StreamId,
        data: Data,
        fin: bool,
    },
    IncomingPayloadAuth {
        addr: ConnectionAddr,
        fin: bool,
        data: Data,
    },
    IncomingPayloadMux {
        addr: ConnectionAddr,
        peer_id: Option<PeerId>,
        fin: bool,
        data: Data,
    },
    IncomingPayload {
        addr: ConnectionAddr,
        peer_id: PeerId,
        stream_id: StreamId,
        fin: bool,
        data: Data,
    },
    IncomingToken {
        addr: ConnectionAddr,
        kind: SelectKind,
    },
    OutgoingTokens {
        addr: ConnectionAddr,
        kind: SelectKind,
        tokens: Vec<token::Token>,
    },
    Timeout {
        addr: ConnectionAddr,
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

    #[allow(dead_code)]
    pub(super) fn forward_data(
        self,
        addr: ConnectionAddr,
        data: Data,
        fin: bool,
    ) -> P2pNetworkSelectAction {
        match self {
            SelectKind::Authentication => {
                P2pNetworkSelectAction::IncomingPayloadAuth { addr, fin, data }
            }
            SelectKind::Multiplexing(peer_id) => P2pNetworkSelectAction::IncomingPayloadMux {
                addr,
                peer_id: Some(peer_id),
                fin,
                data,
            },
            SelectKind::MultiplexingNoPeerId => P2pNetworkSelectAction::IncomingPayloadMux {
                addr,
                peer_id: None,
                fin,
                data,
            },
            SelectKind::Stream(peer_id, stream_id) => P2pNetworkSelectAction::IncomingPayload {
                addr,
                peer_id,
                stream_id,
                fin,
                data,
            },
        }
    }
}

impl P2pNetworkSelectAction {
    pub fn addr(&self) -> &ConnectionAddr {
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

    pub fn select_kind(&self) -> SelectKind {
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
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        if state
            .network
            .scheduler
            .connection_state(self.addr())
            .and_then(|conn| conn.select_state(&self.select_kind()))
            .is_none()
        {
            return false;
        }

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
