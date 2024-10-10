pub mod incoming;
pub mod outgoing;

pub mod incoming_effectful;
pub mod outgoing_effectful;

mod p2p_connection_state;
pub use p2p_connection_state::*;

mod p2p_connection_actions;
pub use p2p_connection_actions::*;

mod p2p_connection_reducer;

mod p2p_connection_service;
pub use p2p_connection_service::*;

use serde::{Deserialize, Serialize};

pub use crate::webrtc::{Answer, Offer, P2pConnectionResponse, RejectionReason};

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error)]
pub enum P2pConnectionErrorResponse {
    #[error("connection rejected: {0}")]
    Rejected(RejectionReason),
    #[error("signal decryption failed")]
    SignalDecryptionFailed,
    #[error("internal error")]
    InternalError,
}
