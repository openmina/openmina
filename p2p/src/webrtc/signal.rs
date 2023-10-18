use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::identity::{PeerId, PublicKey};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Offer {
    pub sdp: String,
    /// Offerer's identity public key.
    pub identity_pub_key: PublicKey,
    /// Peer id that the offerer wants to connect to.
    pub target_peer_id: PeerId,
    // TODO(binier): remove host and get ip from ice candidates instead
    /// Host name or IP of the signalling server of the offerer.
    pub host: String,
    /// Port of the signalling server of the offerer.
    pub listen_port: u16,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Answer {
    pub sdp: String,
    /// Offerer's identity public key.
    pub identity_pub_key: PublicKey,
    /// Peer id that the offerer wants to connect to.
    pub target_peer_id: PeerId,
}

#[derive(Serialize, Deserialize, From, Eq, PartialEq, Debug, Clone)]
pub enum Signal {
    Offer(Offer),
    Answer(Answer),
}
