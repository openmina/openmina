//! # WebRTC Implementation
//!
//! This module provides WebRTC peer-to-peer communication capabilities for
//! the Mina Rust node.
//! WebRTC enables direct peer connections, NAT traversal, and efficient
//! blockchain node communication, particularly important for the Web Node
//! (browser-based Mina protocol).
//!
//! For comprehensive documentation about WebRTC concepts and this implementation,
//! see: <https://o1-labs.github.io/mina-rust/developers/webrtc>

mod host;
pub use host::Host;

mod signal;
pub use signal::{
    Answer, EncryptedAnswer, EncryptedOffer, Offer, P2pConnectionResponse, RejectionReason, Signal,
};

mod signaling_method;
pub use signaling_method::{HttpSignalingInfo, SignalingMethod, SignalingMethodParseError};

mod connection_auth;
pub use connection_auth::{ConnectionAuth, ConnectionAuthEncrypted};
