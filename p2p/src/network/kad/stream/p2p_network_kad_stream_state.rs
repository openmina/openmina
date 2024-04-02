use serde::{Deserialize, Serialize};

use crate::{P2pNetworkKademliaRpcReply, P2pNetworkKademliaRpcRequest};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum P2pNetworkKadStreamState {
    Incoming(P2pNetworkKadIncomingStreamState),
    Outgoing(P2pNetworkKadOutgoingStreamState),
}

impl P2pNetworkKadStreamState {
    pub fn new(incoming: bool) -> Self {
        if incoming {
            P2pNetworkKadStreamState::Incoming(Default::default())
        } else {
            P2pNetworkKadStreamState::Outgoing(Default::default())
        }
    }
}

/// Incoming Kademlia stream is used by a remote peer to perform a Kademlia request.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum P2pNetworkKadIncomingStreamState {
    #[default]
    Default,
    /// Waiting for the incoming request.
    WaitingForRequest { expect_close: bool },
    /// A portion of data from the stream is received.
    PartialRequestReceived { len: usize, data: Vec<u8> },
    /// Request from the stream is received.
    RequestIsReady { data: P2pNetworkKademliaRpcRequest },
    /// Waiting for an outgoing data, or for finalization of the stream (iff `expect_fin` is `true`)
    WaitingForReply,
    /// Response bytes for the remote request is ready to be written into the stream.
    ResponseBytesAreReady { bytes: Vec<u8> },
    /// Remote peer half-closed the stream.
    Closing,
    /// The stream is closed.
    Closed,
    /// Error handling the stream.
    /// TODO: use enum for errors.
    Error(String),
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum P2pNetworkKadOutgoingStreamState {
    #[default]
    Default,
    /// Waiting for an outgoing data, or for finalization of the stream (iff `expect_close` is `true`)
    WaitingForRequest { expect_close: bool },
    /// Response bytes for the remote request are ready to be written into the stream.
    RequestBytesAreReady { bytes: Vec<u8> },
    /// Waiting for the incoming reply.
    WaitingForReply,
    /// A portion of data from the stream is received.
    PartialReplyReceived { len: usize, data: Vec<u8> },
    /// Response from the stream is received.
    ResponseIsReady { data: P2pNetworkKademliaRpcReply },
    /// Closing the stream.
    Closing,
    /// The stream is closed.
    Closed,
    /// Error handling the stream.
    /// TODO: use enum for errors.
    Error(String),
}
