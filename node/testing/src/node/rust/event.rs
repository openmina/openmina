use node::{
    p2p::{P2pConnectionEvent, P2pDiscoveryEvent, P2pEvent, PeerId},
    rpc::{RpcId, RpcRequest},
};
use serde::{Deserialize, Serialize};

pub use node::event_source::Event;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NonDeterministicEvent {
    P2pListen,
    /// Non-deterministic because libp2p kademlia initiates connections
    /// without state machine knowing about it.
    P2pConnectionFinalized(PeerId, Result<(), String>),
    P2pConnectionClosed(PeerId),
    #[cfg(feature = "p2p-libp2p")]
    P2pLibp2pIdentify(PeerId),

    P2pDiscoveryReady,
    P2pDiscoveryDidFindPeers(Vec<PeerId>),
    P2pDiscoveryDidFindPeersError(String),
    P2pDiscoveryAddRoute(PeerId, Vec<PeerId>),

    RpcReadonly(RpcId, RpcRequest),
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
                P2pEvent::Listen(_) => Self::P2pListen.into(),
                #[cfg(not(feature = "p2p-libp2p"))]
                P2pEvent::MioEvent(_) => return None,
                #[cfg(feature = "p2p-libp2p")]
                P2pEvent::Libp2pIdentify(peer_id, _) => Self::P2pLibp2pIdentify(*peer_id).into(),
                P2pEvent::Discovery(e) => match e {
                    P2pDiscoveryEvent::Ready => Self::P2pDiscoveryReady.into(),
                    P2pDiscoveryEvent::DidFindPeers(v) => {
                        Self::P2pDiscoveryDidFindPeers(v.clone()).into()
                    }
                    P2pDiscoveryEvent::DidFindPeersError(v) => {
                        Self::P2pDiscoveryDidFindPeersError(v.clone()).into()
                    }
                    P2pDiscoveryEvent::AddRoute(id, addrs) => {
                        let ids = addrs.iter().map(|addr| *addr.peer_id()).collect();
                        Self::P2pDiscoveryAddRoute(*id, ids).into()
                    }
                },
            },
            Event::Rpc(id, req) => match req {
                RpcRequest::P2pConnectionIncoming(_) => return None,
                req => Self::RpcReadonly(*id, req.clone()).into(),
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
