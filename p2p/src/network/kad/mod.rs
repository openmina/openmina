pub mod bootstrap;
pub mod request;
pub mod stream;

pub use self::bootstrap::P2pNetworkKadBootstrapAction;
pub use self::request::P2pNetworkKadRequestAction;
pub use self::stream::P2pNetworkKademliaStreamAction;

mod p2p_network_kad_state;
pub use self::p2p_network_kad_state::*;

mod p2p_network_kad_actions;
pub use self::p2p_network_kad_actions::*;

#[cfg(feature = "p2p-libp2p")]
mod p2p_network_kad_reducer;

mod p2p_network_kad_message;
pub use self::p2p_network_kad_message::*;

mod p2p_network_kad_protocol;
pub use self::p2p_network_kad_protocol::*;

mod p2p_network_kad_internals;
pub use self::p2p_network_kad_internals::*;

const ALPHA: usize = 3;

pub mod kad_effectful;
pub use kad_effectful::P2pNetworkKadEffectfulAction;
