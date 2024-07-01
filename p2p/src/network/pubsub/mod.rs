mod pb {
    include!(concat!(env!("OUT_DIR"), "/gossipsub.rs"));
}

mod p2p_network_pubsub_actions;
pub use self::p2p_network_pubsub_actions::P2pNetworkPubsubAction;

mod p2p_network_pubsub_state;
pub use self::p2p_network_pubsub_state::{P2pNetworkPubsubClientState, P2pNetworkPubsubState};

#[cfg(feature = "p2p-libp2p")]
mod p2p_network_pubsub_reducer;

#[cfg(feature = "p2p-libp2p")]
mod p2p_network_pubsub_effects;

#[cfg(feature = "p2p-libp2p")]
const TOPIC: &str = "coda/consensus-messages/0.0.1";
