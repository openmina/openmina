//! There are 2 state machines in this module:
//! 1. `discovery` - used for discovering a new target peer and initiating
//!    signaling process.
//! 2. `exchange` - used by intermediary peer to relay an offer to the
//!    target peer and receive an answer from it.
//!
//! These are the overall steps that happens in these state machines in
//! order to connect two (dialer and listener) peers to each other using
//! intermediary peer (relayer):
//! 1. [discovery] Dialer asks relayer to discover an available peer.
//! 2. [discovery] Relayer responds with available peer's (listener's) public key.
//! 3. [discovery] Dialer accepts/rejects the target peer (listener).
//! 4. [discovery] If dialer accepts the peer, it sends webrtc offer to relayer.
//! 5. [exchange] Relayer relays received webrtc offer to the listener peer.
//! 6. [exchange] Relayer receives webrtc answer from the listener peer.
//! 7. [discovery] Relayer relays the answer to the dialer.

pub mod exchange;
pub mod exchange_effectful;

mod p2p_channels_signaling_state;
pub use p2p_channels_signaling_state::*;
