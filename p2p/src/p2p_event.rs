use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::{webrtc, PeerId};

#[derive(Serialize, Deserialize, From, Debug, Clone)]
pub enum P2pEvent {
    Connection(P2pConnectionEvent),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionEvent {
    OfferSdpReady(PeerId, String),
    AnswerSdpReady(PeerId, String),
    AnswerReceived(PeerId, webrtc::Answer),
    Opened(PeerId),
    Closed(PeerId),
}
