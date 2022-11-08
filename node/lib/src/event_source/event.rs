use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Event {
    P2p(P2pEvent),
}

// TODO(binier): maybe move to p2p crate.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pEvent {
    Connection(P2pConnectionEvent),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionEvent {
    OutgoingInit(crate::p2p::PeerId, Result<(), String>),
}
