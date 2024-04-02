use std::net::SocketAddr;

use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

use crate::{P2pAction, P2pNetworkKadEntry, P2pState, PeerId, StreamId};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum P2pNetworkKadRequestAction {
    New {
        peer_id: PeerId,
        addr: SocketAddr,
        key: PeerId,
    },
    PeerIsConnecting {
        peer_id: PeerId,
    },
    MuxReady {
        peer_id: PeerId,
        addr: SocketAddr,
    },
    StreamIsCreating {
        peer_id: PeerId,
        stream_id: StreamId,
    },
    StreamReady {
        peer_id: PeerId,
        stream_id: StreamId,
        addr: SocketAddr,
    },
    RequestSent {
        peer_id: PeerId,
    },
    ReplyReceived {
        peer_id: PeerId,
        stream_id: StreamId,
        data: Vec<P2pNetworkKadEntry>,
    },
    Prune {
        peer_id: PeerId,
    },
    Error {
        peer_id: PeerId,
        error: String,
    },
}

impl P2pNetworkKadRequestAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            P2pNetworkKadRequestAction::New { peer_id, .. }
            | P2pNetworkKadRequestAction::PeerIsConnecting { peer_id, .. }
            | P2pNetworkKadRequestAction::MuxReady { peer_id, .. }
            | P2pNetworkKadRequestAction::StreamIsCreating { peer_id, .. }
            | P2pNetworkKadRequestAction::StreamReady { peer_id, .. }
            | P2pNetworkKadRequestAction::RequestSent { peer_id, .. }
            | P2pNetworkKadRequestAction::ReplyReceived { peer_id, .. }
            | P2pNetworkKadRequestAction::Prune { peer_id, .. }
            | P2pNetworkKadRequestAction::Error { peer_id, .. } => peer_id,
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
                discovery_state.request(self.peer_id()).is_none()
                    == matches!(self, P2pNetworkKadRequestAction::New { .. })
            })
    }
}

impl From<P2pNetworkKadRequestAction> for P2pAction {
    fn from(value: P2pNetworkKadRequestAction) -> Self {
        P2pAction::Network(super::super::P2pNetworkKadAction::Request(value).into())
    }
}
