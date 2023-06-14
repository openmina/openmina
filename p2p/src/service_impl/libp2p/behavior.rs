use libp2p::{gossipsub, swarm::NetworkBehaviour};
use tokio::sync::mpsc;

use crate::{P2pChannelEvent, P2pEvent};

use super::rpc::RpcBehaviour;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event")]
pub struct Behaviour<E: 'static + From<P2pEvent>> {
    pub gossipsub: gossipsub::Behaviour,
    pub rpc: RpcBehaviour,
    #[behaviour(ignore)]
    pub event_source_sender: mpsc::UnboundedSender<E>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, derive_more::From)]
pub enum Event {
    // Identify(IdentifyEvent),
    Gossipsub(gossipsub::Event),
    Rpc(P2pChannelEvent),
}
