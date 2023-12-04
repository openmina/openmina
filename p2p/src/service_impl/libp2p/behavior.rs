use std::collections::BTreeMap;

use libp2p::{
    gossipsub, identify,
    kad::{self, record::store::MemoryStore},
    swarm::NetworkBehaviour,
    PeerId,
};
use libp2p_rpc_behaviour as rpc;
use openmina_core::channels::mpsc;

use super::trivial;
use crate::P2pEvent;

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event")]
pub struct Behaviour<E: 'static + From<P2pEvent>> {
    pub gossipsub: gossipsub::Behaviour,
    pub identify: identify::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub trivial: trivial::Behaviour<1>,
    #[behaviour(ignore)]
    pub rendezvous_string: String,
    #[behaviour(ignore)]
    pub event_source_sender: mpsc::UnboundedSender<E>,
    // TODO(vlad9486): move maps inside `RpcBehaviour`
    // map msg_id into (tag, version)
    #[behaviour(ignore)]
    pub ongoing: BTreeMap<(PeerId, u32), (String, i32)>,
    // map from (peer, msg_id) into (stream_id, tag, version)
    //
    #[behaviour(ignore)]
    pub ongoing_incoming: BTreeMap<(PeerId, u32), (rpc::StreamId, String, i32)>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, derive_more::From)]
pub enum Event {
    Gossipsub(gossipsub::Event),
    Identify(identify::Event),
    Kademlia(kad::Event),
    Trivial((PeerId, trivial::Event)),
}
