use std::net::SocketAddr;

use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

use crate::{P2pNetworkKadEntry, PeerId, StreamId};

#[derive(Debug, Clone, Serialize, Deserialize, MallocSizeOf)]
pub struct P2pNetworkKadRequestState {
    /// ID of the peer we want to send request to.
    pub peer_id: PeerId,
    /// Request key, resulting entries will be those that closest to it.
    pub key: PeerId,
    /// Address
    #[ignore_malloc_size_of = "doesn't allocate"]
    pub addr: SocketAddr,
    /// Request status.
    pub status: P2pNetworkKadRequestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, MallocSizeOf)]
pub enum P2pNetworkKadRequestStatus {
    #[default]
    Default,
    Disconnected,
    WaitingForConnection,
    WaitingForKadStream(StreamId),
    Request(Vec<u8>),
    WaitingForReply,
    Reply(Vec<P2pNetworkKadEntry>),
    Error(String),
}
