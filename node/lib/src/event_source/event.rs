use serde::{Deserialize, Serialize};

use crate::rpc::{RpcId, RpcRequest};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Event {
    P2p(P2pEvent),
    Rpc(RpcId, RpcRequest),
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
