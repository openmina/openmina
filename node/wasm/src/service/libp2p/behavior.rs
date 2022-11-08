use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent},
    identify::{Identify, IdentifyEvent},
    NetworkBehaviour,
};

use libp2p::futures::channel::mpsc;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event")]
pub struct Behaviour {
    // pub identify: Identify,
    pub gossipsub: Gossipsub,

    #[behaviour(ignore)]
    pub event_source_sender: mpsc::Sender<lib::event_source::Event>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, derive_more::From)]
pub enum Event {
    Identify(IdentifyEvent),
    Gossipsub(GossipsubEvent),
}
