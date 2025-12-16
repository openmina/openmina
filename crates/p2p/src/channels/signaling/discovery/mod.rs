mod p2p_channels_signaling_discovery_state;
pub use p2p_channels_signaling_discovery_state::*;

mod p2p_channels_signaling_discovery_actions;
pub use p2p_channels_signaling_discovery_actions::*;

mod p2p_channels_signaling_discovery_reducer;

use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

use crate::{
    identity::PublicKey,
    webrtc::{EncryptedAnswer, EncryptedOffer},
};

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub enum SignalingDiscoveryChannelMsg {
    /// Get next request for connecting 2 peers to each other.
    GetNext,
    /// Dialer is asking relayer to find available connected peer
    /// and start signaling with it.
    Discover,
    /// Relayer found available connected peer and ready to facilitate
    /// signaling.
    Discovered { target_public_key: PublicKey },
    /// Dialer rejected target peer.
    DiscoveredReject,
    /// Dialer accepted target peer and wants to initiate signaling.
    DiscoveredAccept(EncryptedOffer),
    /// Relayed answer Answer to dialer to relay, if you aren't dialer.
    Answer(Option<EncryptedAnswer>),
}
