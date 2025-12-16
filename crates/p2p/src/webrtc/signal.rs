//! WebRTC Signaling Data Structures
//!
//! This module defines the core signaling data structures used in Mina Rust's WebRTC peer-to-peer
//! communication system. It provides the message types for WebRTC connection establishment,
//! including offers, answers, and connection responses.
//!
//! ## Overview
//!
//! WebRTC requires a signaling mechanism to exchange connection metadata between peers
//! before establishing a direct peer-to-peer connection. This module defines the
//! data structures for:
//!
//! - **Offers**: Initial connection requests containing SDP data and peer information
//! - **Answers**: Responses to offers containing SDP data and identity verification
//! - **Connection Responses**: Acceptance, rejection, and error handling for connections
//! - **Encryption Support**: Encrypted versions of signaling messages for security
//!
//! ## WebRTC Signaling Flow
//!
//! 1. Peer A creates an [`Offer`] with SDP data, chain ID, and target peer information
//! 2. The offer is transmitted through a signaling method (HTTP, WebSocket, etc.)
//! 3. Peer B receives and validates the offer (chain ID, peer ID, capacity)
//! 4. Peer B responds with a [`P2pConnectionResponse`]:
//!    - [`P2pConnectionResponse::Accepted`] with an [`Answer`] containing SDP data
//!    - [`P2pConnectionResponse::Rejected`] with a [`RejectionReason`]
//!    - Error variants for decryption failures or internal errors
//! 5. If accepted, both peers use the SDP data to establish the WebRTC connection
//! 6. Connection authentication occurs using [`ConnectionAuth`] derived from SDP hashes
//!
//! ## Security Features
//!
//! - **Chain ID Verification**: Ensures peers are on the same blockchain network
//! - **Identity Authentication**: Uses public key cryptography to verify peer identity
//! - **Encryption Support**: Messages can be encrypted using [`EncryptedOffer`] and [`EncryptedAnswer`]
//! - **Connection Authentication**: SDP hashes used for secure handshake verification

use binprot_derive::{BinProtRead, BinProtWrite};
use derive_more::From;
use malloc_size_of_derive::MallocSizeOf;
use mina_core::ChainId;
use serde::{Deserialize, Serialize};

use crate::identity::{EncryptableType, PeerId, PublicKey};

use super::{ConnectionAuth, Host};

/// WebRTC connection offer containing SDP data and peer information.
///
/// An `Offer` represents the initial connection request in the WebRTC signaling process.
/// It contains all necessary information for a peer to evaluate and potentially accept
/// a WebRTC connection, including:
///
/// - **SDP (Session Description Protocol)** data describing the connection capabilities
/// - **Chain ID** to ensure peers are on the same blockchain network
/// - **Identity verification** through the offerer's public key
/// - **Target peer identification** to ensure the offer reaches the intended recipient
/// - **Signaling server information** for connection establishment
///
/// # Security Considerations
///
/// - The `chain_id` must match between peers to prevent cross-chain connections
/// - The `identity_pub_key` is used for cryptographic verification of the offerer
/// - The `target_peer_id` prevents offers from being accepted by unintended peers
///
/// # Example Flow
///
/// ```text
/// Peer A creates Offer -> Signaling Method -> Peer B validates Offer
/// ```
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, MallocSizeOf)]
pub struct Offer {
    /// Session Description Protocol (SDP) data describing the WebRTC connection
    /// capabilities, including media formats, network information, and ICE candidates.
    pub sdp: String,

    /// Blockchain network identifier to ensure peers are on the same chain.
    /// Prevents accidental connections between different blockchain networks.
    #[ignore_malloc_size_of = "doesn't allocate"]
    pub chain_id: ChainId,

    /// Offerer's identity public key for cryptographic authentication.
    /// Used to verify the identity of the peer making the connection offer.
    #[ignore_malloc_size_of = "doesn't allocate"]
    pub identity_pub_key: PublicKey,

    /// Peer ID that the offerer wants to connect to.
    /// Ensures offers are only accepted by the intended target peer.
    pub target_peer_id: PeerId,

    // TODO(binier): remove host and get ip from ice candidates instead
    /// Host name or IP address of the signaling server of the offerer.
    /// Used for signaling server discovery and connection establishment.
    #[ignore_malloc_size_of = "neglectible"]
    pub host: Host,

    /// Port number of the signaling server of the offerer.
    /// Optional port for signaling server connections.
    pub listen_port: Option<u16>,
}

/// WebRTC connection answer responding to an offer.
///
/// An `Answer` is sent in response to an [`Offer`] when a peer accepts a WebRTC
/// connection request. It contains the answering peer's SDP data and identity
/// information necessary to complete the WebRTC connection establishment.
///
/// The answer includes:
/// - **SDP data** from the answering peer describing their connection capabilities
/// - **Identity verification** through the answerer's public key
/// - **Target confirmation** ensuring the answer reaches the original offerer
///
/// # Connection Process
///
/// After an answer is received, both peers have exchanged SDP data and can proceed
/// with ICE negotiation to establish the direct WebRTC connection. The SDP data
/// from both the offer and answer is used to create connection authentication
/// credentials via [`ConnectionAuth`].
///
/// # Example Flow
///
/// ```text
/// Peer B receives Offer -> Validates -> Creates Answer -> Peer A receives Answer
/// ```
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, MallocSizeOf)]
pub struct Answer {
    /// Session Description Protocol (SDP) data from the answering peer
    /// describing their WebRTC connection capabilities and network information.
    pub sdp: String,

    /// Answering peer's identity public key for cryptographic authentication.
    /// Used to verify the identity of the peer responding to the connection offer.
    #[ignore_malloc_size_of = "doesn't allocate"]
    pub identity_pub_key: PublicKey,

    /// Peer ID of the original offerer that this answer is responding to.
    /// Ensures the answer reaches the correct peer that initiated the connection.
    pub target_peer_id: PeerId,
}

/// Union type for WebRTC signaling messages.
///
/// `Signal` represents the different types of signaling messages that can be
/// exchanged during WebRTC connection establishment. It provides a unified
/// interface for handling both connection offers and answers.
///
/// # Variants
///
/// - [`Signal::Offer`] - Initial connection request with SDP data and peer information
/// - [`Signal::Answer`] - Response to an offer with answering peer's SDP data
///
/// This enum is typically used in signaling transport layers to handle different
/// message types uniformly while preserving their specific data structures.
#[derive(Serialize, Deserialize, From, Eq, PartialEq, Debug, Clone)]
pub enum Signal {
    /// A WebRTC connection offer containing SDP data and peer information.
    Offer(Offer),
    /// A WebRTC connection answer responding to an offer.
    Answer(Answer),
}

/// Reasons why a WebRTC connection offer might be rejected.
///
/// `RejectionReason` provides detailed information about why a peer rejected
/// a connection offer. This enables proper error handling and helps with
/// debugging connection issues.
///
/// The rejection reasons are categorized into different types of validation
/// failures that can occur during the offer evaluation process.
///
/// # Classification
///
/// Some rejection reasons are considered "bad" (potentially indicating malicious
/// behavior or protocol violations) while others are normal operational conditions.
/// Use [`RejectionReason::is_bad`] to determine if a rejection indicates a problem.
#[derive(
    Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Copy, thiserror::Error, MallocSizeOf,
)]
pub enum RejectionReason {
    /// The offering peer is on a different blockchain network.
    ///
    /// This is a normal rejection reason that occurs when peers from different
    /// blockchain networks attempt to connect. Not considered a "bad" rejection.
    #[error("peer is on a different chain")]
    ChainIdMismatch,

    /// The peer ID doesn't match the peer's public key.
    ///
    /// This indicates a potential security issue or protocol violation where
    /// the claimed peer identity doesn't match the cryptographic identity.
    /// Considered a "bad" rejection that may indicate malicious behavior.
    #[error("peer_id does not match peer's public key")]
    PeerIdAndPublicKeyMismatch,

    /// The target peer ID in the offer doesn't match the local node's peer ID.
    ///
    /// This indicates the offer was sent to the wrong peer or there was an
    /// error in peer discovery. Considered a "bad" rejection.
    #[error("target peer_id is not local node's peer_id")]
    TargetPeerIdNotMe,

    /// The local node has reached its maximum peer capacity.
    ///
    /// This is a normal operational condition when a node is at its connection
    /// limit. Not considered a "bad" rejection.
    #[error("too many peers")]
    PeerCapacityFull,

    /// A connection to this peer already exists.
    ///
    /// This prevents duplicate connections to the same peer. Considered a
    /// "bad" rejection as it may indicate connection management issues.
    #[error("peer already connected")]
    AlreadyConnected,

    /// The peer is attempting to connect to itself.
    ///
    /// This is a normal condition that can occur during peer discovery.
    /// Not considered a "bad" rejection.
    #[error("self connection detected")]
    ConnectingToSelf,
}

/// Response to a WebRTC connection offer.
///
/// `P2pConnectionResponse` represents the different possible responses to a WebRTC
/// connection offer. It encapsulates the outcome of offer validation and processing,
/// providing detailed information about acceptance, rejection, or error conditions.
///
/// # Response Types
///
/// - **Accepted**: The offer was validated and accepted, includes an [`Answer`]
/// - **Rejected**: The offer was rejected with a specific reason
/// - **SignalDecryptionFailed**: The encrypted offer could not be decrypted
/// - **InternalError**: An internal error occurred during offer processing
///
/// # Usage
///
/// This enum is typically used in signaling servers and peer connection handlers
/// to communicate the result of offer processing back to the offering peer.
///
/// # Example Flow
///
/// ```text
/// Offer received -> Validation -> P2pConnectionResponse sent back
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionResponse {
    /// The connection offer was accepted.
    ///
    /// Contains an [`Answer`] with the accepting peer's SDP data and identity
    /// information. The boxed answer reduces memory usage for the enum.
    Accepted(Box<Answer>),

    /// The connection offer was rejected.
    ///
    /// Contains a [`RejectionReason`] providing specific details about why
    /// the offer was rejected, enabling proper error handling and debugging.
    Rejected(RejectionReason),

    /// Failed to decrypt the signaling message.
    ///
    /// This occurs when an encrypted offer cannot be decrypted, potentially
    /// due to incorrect encryption keys or corrupted data.
    SignalDecryptionFailed,

    /// An internal error occurred during offer processing.
    ///
    /// This is a catch-all for unexpected errors that occur during offer
    /// validation or answer generation.
    InternalError,
}

/// Computes SHA-256 hash of SDP (Session Description Protocol) data.
///
/// This function creates a cryptographic hash of SDP data that is used for
/// connection authentication. The hash serves as a tamper-evident fingerprint
/// of the connection parameters and is used in the WebRTC handshake process.
///
/// # Parameters
///
/// * `sdp` - The SDP string to hash
///
/// # Returns
///
/// A 32-byte SHA-256 hash of the SDP data
///
/// # Security
///
/// The SHA-256 hash ensures that any modification to the SDP data will be
/// detected during the connection authentication process, preventing
/// man-in-the-middle attacks on the WebRTC handshake.
fn sdp_hash(sdp: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(sdp);
    hasher.finalize().into()
}

impl Offer {
    /// Computes the SHA-256 hash of this offer's SDP data.
    ///
    /// This hash is used for connection authentication and tamper detection
    /// during the WebRTC handshake process.
    ///
    /// # Returns
    ///
    /// A 32-byte SHA-256 hash of the offer's SDP data
    pub fn sdp_hash(&self) -> [u8; 32] {
        sdp_hash(&self.sdp)
    }

    /// Creates connection authentication data from this offer and an answer.
    ///
    /// This method combines the SDP data from both the offer and answer to
    /// create [`ConnectionAuth`] credentials used for secure handshake verification.
    /// The authentication data ensures both peers have the same view of the
    /// connection parameters.
    ///
    /// # Parameters
    ///
    /// * `answer` - The answer responding to this offer
    ///
    /// # Returns
    ///
    /// [`ConnectionAuth`] containing encrypted authentication data derived
    /// from both the offer and answer SDP hashes
    pub fn conn_auth(&self, answer: &Answer) -> ConnectionAuth {
        ConnectionAuth::new(self, answer)
    }
}

impl Answer {
    /// Computes the SHA-256 hash of this answer's SDP data.
    ///
    /// This hash is used for connection authentication and tamper detection
    /// during the WebRTC handshake process, complementing the offer's SDP hash.
    ///
    /// # Returns
    ///
    /// A 32-byte SHA-256 hash of the answer's SDP data
    pub fn sdp_hash(&self) -> [u8; 32] {
        sdp_hash(&self.sdp)
    }
}

impl RejectionReason {
    /// Determines if this rejection reason indicates a potential problem.
    ///
    /// Some rejection reasons are normal operational conditions (like capacity
    /// limits or chain ID mismatches), while others may indicate protocol
    /// violations, security issues, or implementation bugs.
    ///
    /// # Returns
    ///
    /// * `true` if the rejection indicates a potentially problematic condition
    /// * `false` if the rejection is a normal operational condition
    ///
    /// # "Bad" Rejection Reasons
    ///
    /// - [`PeerIdAndPublicKeyMismatch`] - Identity verification failure
    /// - [`TargetPeerIdNotMe`] - Targeting error or discovery issue
    /// - [`AlreadyConnected`] - Connection management issue
    ///
    /// # Normal Rejection Reasons
    ///
    /// - [`ChainIdMismatch`] - Cross-chain connection attempt
    /// - [`PeerCapacityFull`] - Resource limitation
    /// - [`ConnectingToSelf`] - Self-connection detection
    ///
    /// [`PeerIdAndPublicKeyMismatch`]: RejectionReason::PeerIdAndPublicKeyMismatch
    /// [`TargetPeerIdNotMe`]: RejectionReason::TargetPeerIdNotMe
    /// [`AlreadyConnected`]: RejectionReason::AlreadyConnected
    /// [`ChainIdMismatch`]: RejectionReason::ChainIdMismatch
    /// [`PeerCapacityFull`]: RejectionReason::PeerCapacityFull
    /// [`ConnectingToSelf`]: RejectionReason::ConnectingToSelf
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
    /// Returns the string representation of the internal error response.
    ///
    /// This is used for consistent error messaging across the system when
    /// internal errors occur during connection processing.
    pub fn internal_error_str() -> &'static str {
        "InternalError"
    }

    /// Returns the JSON string representation of the internal error response.
    ///
    /// This provides a properly quoted JSON string for the internal error
    /// response, used in JSON serialization contexts.
    pub fn internal_error_json_str() -> &'static str {
        "\"InternalError\""
    }
}

/// Encrypted WebRTC offer for secure signaling.
///
/// `EncryptedOffer` wraps an [`Offer`] that has been encrypted for secure
/// transmission through untrusted signaling channels. This provides confidentiality
/// for the offer data including SDP information and peer identities.
///
/// The encrypted data is stored as a byte vector and can be transmitted through
/// any signaling method while maintaining security properties.
#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, From, Debug, Clone)]
pub struct EncryptedOffer(Vec<u8>);

/// Encrypted WebRTC connection response for secure signaling.
///
/// `EncryptedAnswer` wraps a [`P2pConnectionResponse`] that has been encrypted
/// for secure transmission. This ensures that connection responses, including
/// answers with SDP data, are protected during transmission through untrusted
/// signaling channels.
///
/// The encrypted data maintains confidentiality of the response while allowing
/// transmission through any signaling transport method.
#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, From, Debug, Clone)]
pub struct EncryptedAnswer(Vec<u8>);

impl AsRef<[u8]> for EncryptedOffer {
    /// Provides access to the underlying encrypted data as a byte slice.
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<[u8]> for EncryptedAnswer {
    /// Provides access to the underlying encrypted data as a byte slice.
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl EncryptableType for Offer {
    /// Associates [`Offer`] with its encrypted counterpart [`EncryptedOffer`].
    type Encrypted = EncryptedOffer;
}

impl EncryptableType for P2pConnectionResponse {
    /// Associates [`P2pConnectionResponse`] with its encrypted counterpart [`EncryptedAnswer`].
    type Encrypted = EncryptedAnswer;
}
