use binprot_derive::{BinProtRead, BinProtWrite};
use derive_more::From;
use malloc_size_of_derive::MallocSizeOf;
use openmina_core::ChainId;
use serde::{Deserialize, Serialize};

use crate::identity::{EncryptableType, PeerId, PublicKey};

use super::{ConnectionAuth, Host};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, MallocSizeOf)]
pub struct Offer {
    pub sdp: String,
    #[ignore_malloc_size_of = "doesn't allocate"]
    pub chain_id: ChainId,
    /// Offerer's identity public key.
    #[ignore_malloc_size_of = "doesn't allocate"]
    pub identity_pub_key: PublicKey,
    /// Peer id that the offerer wants to connect to.
    pub target_peer_id: PeerId,
    // TODO(binier): remove host and get ip from ice candidates instead
    /// Host name or IP of the signaling server of the offerer.
    #[ignore_malloc_size_of = "neglectible"]
    pub host: Host,
    /// Port of the signaling server of the offerer.
    pub listen_port: Option<u16>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, MallocSizeOf)]
pub struct Answer {
    pub sdp: String,
    /// Offerer's identity public key.
    #[ignore_malloc_size_of = "doesn't allocate"]
    pub identity_pub_key: PublicKey,
    /// Peer id that the offerer wants to connect to.
    pub target_peer_id: PeerId,
}

#[derive(Serialize, Deserialize, From, Eq, PartialEq, Debug, Clone)]
pub enum Signal {
    Offer(Offer),
    Answer(Answer),
}

#[derive(
    Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Copy, thiserror::Error, MallocSizeOf,
)]
pub enum RejectionReason {
    #[error("peer is on a different chain")]
    ChainIdMismatch,
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

fn sdp_hash(sdp: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(sdp);
    hasher.finalize().into()
}

impl Offer {
    pub fn sdp_hash(&self) -> [u8; 32] {
        sdp_hash(&self.sdp)
    }

    pub fn conn_auth(&self, answer: &Answer) -> ConnectionAuth {
        ConnectionAuth::new(self, answer)
    }
}

impl Answer {
    pub fn sdp_hash(&self) -> [u8; 32] {
        sdp_hash(&self.sdp)
    }
}

impl RejectionReason {
    pub fn is_bad(&self) -> bool {
        match self {
            Self::ChainIdMismatch => false,
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
#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, From, Debug, Clone)]
pub struct EncryptedOffer(Vec<u8>);

/// Encrypted `P2pConnectionResponse`.
#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, From, Debug, Clone)]
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
