pub mod connection;
pub mod pubsub;
pub mod rpc;

mod p2p_actions;
pub use p2p_actions::*;

mod p2p_config;
pub use p2p_config::*;

mod p2p_state;
pub use p2p_state::*;

mod p2p_reducer;
pub use p2p_reducer::*;

mod p2p_effects;
pub use p2p_effects::*;

pub use libp2p::PeerId;

use redux::SubStore;
pub trait P2pStore<GlobalState>: SubStore<GlobalState, P2pState, SubAction = P2pAction> {}
impl<S, T: SubStore<S, P2pState, SubAction = P2pAction>> P2pStore<S> for T {}
