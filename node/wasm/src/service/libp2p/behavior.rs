use lib::p2p::rpc::P2pRpcEvent;
use libp2p::{
    futures::channel::mpsc,
    gossipsub::{Gossipsub, GossipsubEvent},
    identify::{Identify, IdentifyEvent},
    NetworkBehaviour,
};

use super::rpc::RpcBehaviour;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event")]
pub struct Behaviour {
    // pub identify: Identify,
    pub gossipsub: Gossipsub,
    pub rpc: RpcBehaviour,

    #[behaviour(ignore)]
    pub event_source_sender: mpsc::Sender<lib::event_source::Event>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, derive_more::From)]
pub enum Event {
    // Identify(IdentifyEvent),
    Gossipsub(GossipsubEvent),
    Rpc(P2pRpcEvent),
}
