use serde::{Deserialize, Serialize};

use crate::p2p::pubsub::PubsubTopic;
pub use crate::p2p::rpc::P2pRpcEvent;
use crate::p2p::PeerId;
use crate::rpc::{RpcId, RpcRequest};
use crate::snark::block_verify::{SnarkBlockVerifyError, SnarkBlockVerifyId};

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum Event {
    P2p(P2pEvent),
    Snark(SnarkEvent),
    Rpc(RpcId, RpcRequest),
}

// TODO(binier): maybe move to p2p crate.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pEvent {
    Connection(P2pConnectionEvent),
    Pubsub(P2pPubsubEvent),
    Rpc(P2pRpcEvent),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionEvent {
    OutgoingInit(crate::p2p::PeerId, Result<(), String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pPubsubEvent {
    BytesReceived {
        author: PeerId,
        sender: PeerId,
        topic: PubsubTopic,
        bytes: Vec<u8>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkEvent {
    BlockVerify(SnarkBlockVerifyId, Result<(), SnarkBlockVerifyError>),
}
