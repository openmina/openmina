use serde::{Deserialize, Serialize};

use crate::{P2pNetworkKademliaRpcReply, P2pNetworkKademliaRpcRequest};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum P2pNetworkKadStreamKind {
    Incoming,
    Outgoing,
}

impl From<bool> for P2pNetworkKadStreamKind {
    fn from(incoming: bool) -> Self {
        if incoming {
            P2pNetworkKadStreamKind::Incoming
        } else {
            P2pNetworkKadStreamKind::Outgoing
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum P2pNetworkKadStreamState {
    #[default]
    Default,
    /// Waiting for the incoming request.
    WaitingIncoming {
        kind: P2pNetworkKadStreamKind,
        expect_data: bool,
        expect_close: bool,
    },
    /// A portion of data from the stream is received.
    IncomingPartialData {
        kind: P2pNetworkKadStreamKind,
        len: usize,
        data: Vec<u8>,
    },
    /// Request from the stream is received.
    IncomingRequest { data: P2pNetworkKademliaRpcRequest },
    /// Request from the stream is received.
    IncomingReply { data: P2pNetworkKademliaRpcReply },
    /// Waiting for an outgoing data, or for finalization of the stream (iff `expect_fin` is `true`)
    WaitingOutgoing {
        kind: P2pNetworkKadStreamKind,
        expect_close: bool,
    },
    /// Response bytes for the remote request is ready to be written into the stream.
    OutgoingBytes {
        kind: P2pNetworkKadStreamKind,
        bytes: Vec<u8>,
    },
    /// The stream is closed.
    Closed,
    /// Error handling the stream.
    /// TODO: use enum for errors.
    Error(String),
}

impl P2pNetworkKadStreamState {
    pub fn new() -> Self {
        P2pNetworkKadStreamState::Default
    }
}

impl From<P2pNetworkKademliaRpcRequest> for P2pNetworkKadStreamState {
    fn from(data: P2pNetworkKademliaRpcRequest) -> Self {
        P2pNetworkKadStreamState::IncomingRequest { data }
    }
}

impl From<P2pNetworkKademliaRpcReply> for P2pNetworkKadStreamState {
    fn from(data: P2pNetworkKademliaRpcReply) -> Self {
        P2pNetworkKadStreamState::IncomingReply { data }
    }
}
