///#![feature(trivial_bounds)]
pub mod channels;
pub mod connection;
pub mod disconnection;
pub mod discovery;
pub mod listen;
pub mod peer;

pub mod identity;
pub use identity::PeerId;

pub mod webrtc;

pub mod service_impl;

mod p2p_actions;
pub use p2p_actions::*;

mod p2p_config;
pub use p2p_config::*;

mod p2p_event;
pub use p2p_event::*;

mod p2p_state;
pub use p2p_state::*;

mod p2p_reducer;
pub use p2p_reducer::*;

use redux::SubStore;
pub trait P2pStore<GlobalState>: SubStore<GlobalState, P2pState, SubAction = P2pAction> {}
impl<S, T: SubStore<S, P2pState, SubAction = P2pAction>> P2pStore<S> for T {}
