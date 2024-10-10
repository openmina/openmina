use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::identity::{EncryptableType, PeerId, PublicKey};

use super::Host;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Offer {
    pub sdp: String,
    /// Offerer's identity public key.
    pub identity_pub_key: PublicKey,
    /// Peer id that the offerer wants to connect to.
    pub target_peer_id: PeerId,
    // TODO(binier): remove host and get ip from ice candidates instead
    /// Host name or IP of the signaling server of the offerer.
    pub host: Host,
    /// Port of the signaling server of the offerer.
    pub listen_port: Option<u16>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Answer {
    pub sdp: String,
    /// Offerer's identity public key.
    pub identity_pub_key: PublicKey,
    /// Peer id that the offerer wants to connect to.
    pub target_peer_id: PeerId,
}

#[derive(Serialize, Deserialize, From, Eq, PartialEq, Debug, Clone)]
pub enum Signal {
    Offer(Offer),
    Answer(Answer),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Copy, thiserror::Error)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionResponse {
    Accepted(Box<Answer>),
    Rejected(RejectionReason),
    SignalDecryptionFailed,
    InternalError,
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

impl P2pConnectionResponse {
    pub fn internal_error_str() -> &'static str {
        "InternalError"
    }

    pub fn internal_error_json_str() -> &'static str {
        "\"InternalError\""
    }
}

/// Encrypted `webrtc::Offer`.
#[derive(Serialize, Deserialize, From, Debug, Clone)]
pub struct EncryptedOffer(Vec<u8>);

/// Encrypted `P2pConnectionResponse`.
#[derive(Serialize, Deserialize, From, Debug, Clone)]
pub struct EncryptedAnswer(Vec<u8>);

impl AsRef<[u8]> for EncryptedOffer {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<[u8]> for EncryptedAnswer {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl EncryptableType for Offer {
    type Encrypted = EncryptedOffer;
}

impl EncryptableType for P2pConnectionResponse {
    type Encrypted = EncryptedAnswer;
}
