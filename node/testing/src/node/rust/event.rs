use node::{
    p2p::{P2pConnectionEvent, P2pEvent, PeerId},
    rpc::{RpcId, RpcRequest},
};
use serde::{Deserialize, Serialize};

pub use node::event_source::Event;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NonDeterministicEvent {
    /// Non-deterministic because libp2p kademlia initiates connections
    /// without state machine knowing about it.
    P2pConnectionFinalized(PeerId, Result<(), String>),
    P2pConnectionClosed(PeerId),

    RpcReadonly(RpcId, Box<RpcRequest>),
}

impl NonDeterministicEvent {
    pub fn new(event: &Event) -> Option<Box<Self>> {
        Some(match event {
            Event::P2p(e) => match e {
                P2pEvent::Connection(e) => match e {
                    P2pConnectionEvent::Finalized(id, res) => {
                        Self::P2pConnectionFinalized(*id, res.clone()).into()
                    }
                    P2pConnectionEvent::Closed(id) => Self::P2pConnectionClosed(*id).into(),
                    _ => return None,
                },
                P2pEvent::Channel(_) => return None,
                #[cfg(feature = "p2p-libp2p")]
                P2pEvent::MioEvent(_) => return None,
            },
            Event::Rpc(id, req) => match req.as_ref() {
                RpcRequest::P2pConnectionIncoming(_) => return None,
                req => Self::RpcReadonly(*id, Box::new(req.clone())).into(),
            },
            _ => return None,
        })
    }

    pub fn should_drop_event(event: &Event) -> bool {
        Self::new(event).map_or(false, |e| e.should_drop())
    }

    pub fn should_drop(&self) -> bool {
        // TODO(binier): for cleanup.
        false
    }
}
