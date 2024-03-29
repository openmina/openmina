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

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub enum RejectionReason {
    PeerIdAndPublicKeyMismatch,
    TargetPeerIdNotMe,
    PeerCapacityFull,
    AlreadyConnected,
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
pub enum P2pConnectionErrorResponse {
    Rejected(RejectionReason),
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
