mod p2p_channels_signaling_exchange_state;
pub use p2p_channels_signaling_exchange_state::*;

mod p2p_channels_signaling_exchange_actions;
pub use p2p_channels_signaling_exchange_actions::*;

mod p2p_channels_signaling_exchange_reducer;

use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

use crate::{
    identity::PublicKey,
    webrtc::{EncryptedAnswer, EncryptedOffer},
};

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub enum SignalingExchangeChannelMsg {
    /// Get next incoming offer to us.
    GetNext,
    /// Relayed offer from dialer to you.
    OfferToYou {
        offerer_pub_key: PublicKey,
        offer: EncryptedOffer,
    },
    /// Answer to dialer to relay, if you aren't dialer.
    Answer(Option<EncryptedAnswer>),
}
