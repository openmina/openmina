use std::collections::BTreeMap;

use libp2p::{gossipsub, identify, swarm::NetworkBehaviour, PeerId};
use openmina_core::channels::mpsc;

use crate::P2pEvent;

use libp2p_rpc_behaviour::{Behaviour as RpcBehaviour, Event as RpcEvent, StreamId};

use libp2p::kad::{self, record::store::MemoryStore};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event")]
pub struct Behaviour<E: 'static + From<P2pEvent>> {
    pub gossipsub: gossipsub::Behaviour,
    pub rpc: RpcBehaviour,
    pub identify: identify::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
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
    pub ongoing_incoming: BTreeMap<(PeerId, u32), (StreamId, String, i32)>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, derive_more::From)]
pub enum Event {
    // Identify(IdentifyEvent),
    Gossipsub(gossipsub::Event),
    Rpc((PeerId, RpcEvent)),
    Identify(identify::Event),
    Kademlia(kad::Event),
}
