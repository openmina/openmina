use crate::{Data, P2pAction, P2pState, PeerId, StreamId};
use redux::EnablingCondition;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Floodsub stream related actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2pNetworkFloodsubStreamAction {
    /// Creates a new stream state.
    New {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
        incoming: bool,
    },
    /// Handles incoming data from the stream.
    IncomingData {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
        data: Data,
    },
    /// Start closing the stream (send FIN).
    Close {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
    },
    /// Remote peer sent FIN to close the stream.
    RemoteClose {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
    },
    /// Removes the closed stream from the state.
    Prune {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
    },
}

macro_rules! enum_field {
    ($field:ident: $field_type:ty) => {
        pub fn $field(&self) -> &$field_type {
            match self {
                P2pNetworkFloodsubStreamAction::New { $field, .. }
                | P2pNetworkFloodsubStreamAction::IncomingData { $field, .. }
                | P2pNetworkFloodsubStreamAction::Close { $field, .. }
                | P2pNetworkFloodsubStreamAction::RemoteClose { $field, .. }
                | P2pNetworkFloodsubStreamAction::Prune { $field, .. } => $field,
            }
        }
    };
}

impl P2pNetworkFloodsubStreamAction {
    enum_field!(addr: SocketAddr);
    enum_field!(peer_id: PeerId);
    enum_field!(stream_id: StreamId);
}

impl EnablingCondition<P2pState> for P2pNetworkFloodsubStreamAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        // TODO
        true
    }
}

impl From<P2pNetworkFloodsubStreamAction> for P2pAction {
    fn from(value: P2pNetworkFloodsubStreamAction) -> Self {
        P2pAction::Network(super::super::P2pNetworkFloodsubAction::Stream(value).into())
    }
}
