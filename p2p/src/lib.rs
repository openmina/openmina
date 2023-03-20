pub mod connection;
pub mod disconnection;

mod peer_id;
pub use ed25519_dalek::{Keypair, PublicKey, SecretKey};
pub use peer_id::PeerId;

mod signaling_method;
pub use signaling_method::*;

mod p2p_actions;
pub use p2p_actions::*;

mod p2p_config;
pub use p2p_config::*;

mod p2p_state;
pub use p2p_state::*;

mod p2p_reducer;
pub use p2p_reducer::*;

use redux::SubStore;
pub trait P2pStore<GlobalState>: SubStore<GlobalState, P2pState, SubAction = P2pAction> {}
impl<S, T: SubStore<S, P2pState, SubAction = P2pAction>> P2pStore<S> for T {}
