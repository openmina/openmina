use serde::{Deserialize, Serialize};

pub use crate::p2p::{P2pConnectionEvent, P2pEvent};
pub use crate::rpc::{RpcId, RpcRequest};
pub use crate::snark::SnarkEvent;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum Event {
    P2p(P2pEvent),
    Snark(SnarkEvent),
    Rpc(RpcId, RpcRequest),
}
