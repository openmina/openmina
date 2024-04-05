use std::net::SocketAddr;

use openmina_core::ActionEvent;
use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

use crate::{
    Data, P2pAction, P2pNetworkKademliaRpcReply, P2pNetworkKademliaRpcRequest, P2pState, PeerId,
    StreamId,
};

/// Kademlia stream related actions.
#[derive(Debug, Clone, Serialize, Deserialize, ActionEvent)]
#[action_event(fields(display(addr), display(peer_id), stream_id, incoming, debug(data)))]
pub enum P2pNetworkKademliaStreamAction {
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
    /// Remote peer sent FIN to close the stream.
    RemoteClose {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
    },

    /// Reinitializes existing stream state.
    WaitIncoming {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
    },
    /// Sets the state to wait for outgoing data.
    WaitOutgoing {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
    },

    /// Sends request to the stream.
    SendRequest {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
        data: P2pNetworkKademliaRpcRequest,
    },
    /// Sends response to the stream.
    SendResponse {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
        data: P2pNetworkKademliaRpcReply,
    },
    /// Outgoing data is ready to be sent via the stream.
    OutgoingDataReady {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
    },

    /// Start closing outgoing stream (first closing our half of the stream)
    Close {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
    },

    /// Removes the closed stream from the state.
    #[action_event(level = trace)]
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
                P2pNetworkKademliaStreamAction::New { $field, .. }
                | P2pNetworkKademliaStreamAction::IncomingData { $field, .. }
                | P2pNetworkKademliaStreamAction::WaitOutgoing { $field, .. }
                | P2pNetworkKademliaStreamAction::SendRequest { $field, .. }
                | P2pNetworkKademliaStreamAction::SendResponse { $field, .. }
                | P2pNetworkKademliaStreamAction::OutgoingDataReady { $field, .. }
                | P2pNetworkKademliaStreamAction::WaitIncoming { $field, .. }
                | P2pNetworkKademliaStreamAction::Close { $field, .. }
                | P2pNetworkKademliaStreamAction::RemoteClose { $field, .. }
                | P2pNetworkKademliaStreamAction::Prune { $field, .. } => $field,
            }
        }
    };
}

impl P2pNetworkKademliaStreamAction {
    enum_field!(addr: SocketAddr);
    enum_field!(peer_id: PeerId);
    enum_field!(stream_id: StreamId);
}

impl EnablingCondition<P2pState> for P2pNetworkKademliaStreamAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        // TODO
        true
    }
}

impl From<P2pNetworkKademliaStreamAction> for P2pAction {
    fn from(value: P2pNetworkKademliaStreamAction) -> Self {
        P2pAction::Network(super::super::P2pNetworkKadAction::Stream(value).into())
    }
}
