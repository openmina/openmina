pub mod incoming;
pub mod outgoing;

mod p2p_connection_state;
pub use p2p_connection_state::*;

mod p2p_connection_actions;
pub use p2p_connection_actions::*;

mod p2p_connection_reducer;
pub use p2p_connection_reducer::*;

mod p2p_connection_service;
pub use p2p_connection_service::*;

use serde::{Deserialize, Serialize};

use crate::webrtc;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, thiserror::Error)]
pub enum RejectionReason {
    #[error("peer_id does not match peer's public key")]
    PeerIdAndPublicKeyMismatch,
    #[error("target peer_id is not local node's peer_id")]
    TargetPeerIdNotMe,
    #[error("too many peers")]
    PeerCapacityFull,
    #[error("peer already connected")]
    AlreadyConnected,
    #[error("self connection detected")]
    ConnectingToSelf,
}

impl RejectionReason {
    pub fn is_bad(&self) -> bool {
        match self {
            Self::PeerIdAndPublicKeyMismatch => true,
            Self::TargetPeerIdNotMe => true,
            Self::PeerCapacityFull => false,
            Self::AlreadyConnected => true,
            Self::ConnectingToSelf => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error)]
pub enum P2pConnectionErrorResponse {
    #[error("connection rejected: {0}")]
    Rejected(RejectionReason),
    #[error("internal error")]
    InternalError,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionResponse {
    Accepted(webrtc::Answer),
    Rejected(RejectionReason),
    InternalError,
}

impl P2pConnectionResponse {
    pub fn internal_error_str() -> &'static str {
        "InternalError"
    }

    pub fn internal_error_json_str() -> &'static str {
        "\"InternalError\""
    }
}
