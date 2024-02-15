use std::net::SocketAddr;

use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

use crate::{P2pAction, P2pNetworkKadEntry, P2pState, PeerId, StreamId};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum P2pNetworkKadRequestAction {
    New {
        addr: SocketAddr,
        peer_id: PeerId,
        key: PeerId,
    },
    PeerIsConnecting {
        addr: SocketAddr,
    },
    MuxReady {
        addr: SocketAddr,
    },
    StreamIsCreating {
        addr: SocketAddr,
        stream_id: StreamId,
    },
    StreamReady {
        addr: SocketAddr,
        stream_id: StreamId,
    },
    RequestSent {
        addr: SocketAddr,
    },
    ReplyReceived {
        addr: SocketAddr,
        data: Vec<P2pNetworkKadEntry>,
    },
    Prune {
        addr: SocketAddr,
    },
    Error {
        addr: SocketAddr,
        error: String,
    },
}

impl P2pNetworkKadRequestAction {
    pub fn addr(&self) -> &SocketAddr {
        match self {
            P2pNetworkKadRequestAction::New { addr, .. }
            | P2pNetworkKadRequestAction::PeerIsConnecting { addr, .. }
            | P2pNetworkKadRequestAction::MuxReady { addr, .. }
            | P2pNetworkKadRequestAction::StreamIsCreating { addr, .. }
            | P2pNetworkKadRequestAction::StreamReady { addr, .. }
            | P2pNetworkKadRequestAction::RequestSent { addr, .. }
            | P2pNetworkKadRequestAction::ReplyReceived { addr, .. }
            | P2pNetworkKadRequestAction::Prune { addr, .. }
            | P2pNetworkKadRequestAction::Error { addr, .. } => addr,
        }
    }
}

impl EnablingCondition<P2pState> for P2pNetworkKadRequestAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        state
            .network
            .scheduler
            .discovery_state()
            .map_or(false, |discovery_state| {
                // no request for New, some request for anything else.
                discovery_state.request(self.addr()).is_none()
                    == matches!(self, P2pNetworkKadRequestAction::New { .. })
            })
    }
}

impl From<P2pNetworkKadRequestAction> for P2pAction {
    fn from(value: P2pNetworkKadRequestAction) -> Self {
        P2pAction::Network(super::super::P2pNetworkKadAction::Request(value).into())
    }
}
