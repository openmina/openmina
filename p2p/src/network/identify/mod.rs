mod pb {
    include!(concat!(env!("OUT_DIR"), "/identify.rs"));
}

pub mod stream;
pub mod stream_effectful;

pub use self::stream::P2pNetworkIdentifyStreamAction;

mod p2p_network_identify_protocol;
pub use self::p2p_network_identify_protocol::*;

mod p2p_network_identify_actions;
pub use self::p2p_network_identify_actions::P2pNetworkIdentifyAction;

mod p2p_network_identify_state;
pub use self::p2p_network_identify_state::*;

#[cfg(feature = "p2p-libp2p")]
mod p2p_network_identify_reducer;

#[cfg(feature = "p2p-libp2p")]
mod p2p_network_identify_effects;

mod keys_proto;
pub use self::keys_proto::*;
