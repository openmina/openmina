///#![feature(trivial_bounds)]
pub mod channels;
pub mod connection;
pub mod disconnection;
pub mod discovery;
pub mod identity;
pub mod peer;
pub use identity::PeerId;

pub mod webrtc;

pub mod service_impl;

#[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
pub mod identify;
#[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
pub mod network;
#[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
pub use self::network::*;

mod p2p_config;
pub use p2p_config::*;

mod p2p_event;
pub use p2p_event::*;

mod p2p_actions;
pub use p2p_actions::*;

mod p2p_state;
pub use p2p_state::*;

mod p2p_reducer;

mod p2p_effects;
pub use self::p2p_effects::*;

mod p2p_service;
pub use p2p_service::*;
pub mod service {
    pub use super::p2p_service::*;
}

use redux::SubStore;
pub trait P2pStore<GlobalState>: SubStore<GlobalState, P2pState, SubAction = P2pAction> {}
impl<S, T: SubStore<S, P2pState, SubAction = P2pAction>> P2pStore<S> for T {}

#[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
pub use libp2p_identity;
#[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
pub use multiaddr;

#[cfg(all(
    not(target_arch = "wasm32"),
    feature = "p2p-libp2p",
    feature = "fuzzing"
))]
pub mod fuzzer;

/// Returns true if duration value is configured, and, given the time is `now`,
/// that duration is passed since `then`.
fn is_time_passed(
    now: redux::Timestamp,
    then: redux::Timestamp,
    duration: Option<std::time::Duration>,
) -> bool {
    duration.map_or(false, |d| now.checked_sub(then) >= Some(d))
}
