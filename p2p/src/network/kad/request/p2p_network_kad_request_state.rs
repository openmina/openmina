use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::{P2pNetworkKadEntry, PeerId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pNetworkKadRequestState {
    /// ID of the peer we want to send request to.
    pub peer_id: PeerId,
    /// Request key, resulting entries will be those that closest to it.
    pub key: PeerId,
    /// Address
    pub addr: SocketAddr,
    /// Request status.
    pub status: P2pNetworkKadRequestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum P2pNetworkKadRequestStatus {
    #[default]
    Default,
    Disconnected,
    WaitingForConnection,
    WaitingForKadStream,
    Request(Vec<u8>),
    WaitingForReply,
    Reply(Vec<P2pNetworkKadEntry>),
    Error(String),
}
