//! Route handlers for the axum HTTP server.
//!
//! Each submodule groups related endpoints by functionality.

pub mod discovery;
pub mod scan_state;
pub mod snark_pool;
pub mod snarker;
pub mod state;
pub mod status;
pub mod transaction;

#[cfg(feature = "p2p-webrtc")]
pub mod webrtc;
