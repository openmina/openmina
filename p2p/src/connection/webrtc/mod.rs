pub mod incoming;
pub mod outgoing;

// mod p2p_connection_webrtc_state;
// pub use p2p_connection_webrtc_state::*;

mod p2p_connection_webrtc_actions;
pub use p2p_connection_webrtc_actions::*;

mod p2p_connection_webrtc_reducer;
pub use p2p_connection_webrtc_reducer::*;

mod p2p_connection_webrtc_service;
pub use p2p_connection_webrtc_service::*;

use serde::{Deserialize, Serialize};

use crate::webrtc;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, thiserror::Error)]
pub enum RejectionReason {
    #[error("peer ID and public key mismatch")]
    PeerIdAndPublicKeyMismatch,
    #[error("incorrect target peer")]
    TargetPeerIdNotMe,
    #[error("too many peers")]
    PeerCapacityFull,
    #[error("already connected")]
    AlreadyConnected,
    #[error("connecting to myself")]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionWebRTCErrorResponse {
    Rejected(RejectionReason),
    InternalError,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionWebRTCResponse {
    Accepted(webrtc::Answer),
    Rejected(RejectionReason),
    InternalError,
}

impl P2pConnectionWebRTCResponse {
    pub fn internal_error_str() -> &'static str {
        "InternalError"
    }
}
