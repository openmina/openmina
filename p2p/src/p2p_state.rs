use openmina_core::block::ArcBlockWithHash;
use redux::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use openmina_core::requests::RpcId;

use crate::channels::rpc::P2pRpcId;
use crate::channels::{ChannelId, P2pChannelsState};
use crate::common::{P2pGenericAddr1, P2pGenericAddrs, P2pGenericPeer};
use crate::connection::libp2p::incoming::P2pConnectionLibP2pIncomingState;
use crate::connection::libp2p::outgoing::P2pConnectionLibP2pOutgoingState;
use crate::connection::webrtc::incoming::P2pConnectionWebRTCIncomingState;
use crate::connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingState;
use crate::connection::{ConnectionState, P2pConnectionState};
use crate::libp2p::P2pLibP2pAddr;
use crate::webrtc::SignalingMethod;
use crate::PeerId;

use super::P2pConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pState {
    pub config: P2pConfig,
    pub peers: BTreeMap<PeerId, P2pPeerState>,
    pub kademlia: P2pKademliaState,
    pub listeners: P2pListenersState,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pKademliaState {
    pub is_ready: bool,
    pub is_bootstrapping: bool,
    pub outgoing_requests: usize,
    pub routes: BTreeMap<PeerId, P2pGenericAddrs>,
    pub known_peers: BTreeMap<PeerId, P2pGenericAddrs>,
    pub saturated: Option<redux::Timestamp>,
    pub peer_timestamp: BTreeMap<PeerId, redux::Timestamp>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pListenersState(pub BTreeMap<P2pListenerId, P2pListenerState>);

#[derive(
    Default,
    Serialize,
    Deserialize,
    derive_more::From,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Clone,
    derive_more::Display,
)]
pub struct P2pListenerId(String);

#[derive(
    Default,
    Serialize,
    Deserialize,
    derive_more::From,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Clone,
    derive_more::Display,
)]
pub struct P2pConnectionId(String);

impl From<libp2p::swarm::ConnectionId> for P2pConnectionId {
    fn from(value: libp2p::swarm::ConnectionId) -> Self {
        P2pConnectionId(format!("{:?}", value))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pListenerState {
    Open {
        addrs: BTreeSet<libp2p::Multiaddr>,
        errors: Vec<String>,
    },
    Closed,
    ClosedWithError(String),
}

impl Default for P2pListenerState {
    fn default() -> Self {
        P2pListenerState::Open {
            addrs: BTreeSet::default(),
            errors: Vec::new(),
        }
    }
}

impl P2pState {
    pub fn new(config: P2pConfig) -> Self {
        Self {
            config,
            listeners: Default::default(),
            peers: Default::default(),
            kademlia: P2pKademliaState::default(),
        }
    }

    pub fn my_id(&self) -> PeerId {
        self.config.identity_pub_key.peer_id()
    }

    pub fn peer_connection_rpc_id(&self, peer_id: &PeerId) -> Option<RpcId> {
        self.peers.get(peer_id)?.connection_rpc_id()
    }

    pub fn get_libp2p_peer(&self, peer_id: &PeerId) -> Option<&P2pLibP2pPeerState> {
        self.peers
            .get(peer_id)
            .and_then(|peer_state| match peer_state {
                P2pPeerState::Libp2p(peer_state) => Some(peer_state),
                _ => None,
            })
    }

    pub fn get_libp2p_peer_mut(&mut self, peer_id: &PeerId) -> Option<&mut P2pLibP2pPeerState> {
        self.peers
            .get_mut(peer_id)
            .and_then(|peer_state| match peer_state {
                P2pPeerState::Libp2p(peer_state) => Some(peer_state),
                _ => None,
            })
    }

    pub fn get_webrtc_peer(&self, peer_id: &PeerId) -> Option<&P2pWebRTCPeerState> {
        self.peers
            .get(peer_id)
            .and_then(|peer_state| match peer_state {
                P2pPeerState::WebRTC(peer_state) => Some(peer_state),
                _ => None,
            })
    }

    /// Get peer in ready state. `None` if peer isn't in `Ready` state,
    /// or if peer doesn't exist.
    pub fn get_ready_peer(&self, peer_id: &PeerId) -> Option<&P2pPeerStatusReady> {
        self.peers.get(peer_id)?.status_as_ready()
    }

    /// Get peer in ready state. `None` if peer isn't in `Ready` state,
    /// or if peer doesn't exist.
    pub fn get_ready_peer_mut(&mut self, peer_id: &PeerId) -> Option<&mut P2pPeerStatusReady> {
        self.peers.get_mut(peer_id)?.as_status_ready_mut()
    }

    pub fn any_ready_peers(&self) -> bool {
        self.peers
            .iter()
            .any(|(_, p)| p.status_as_ready().is_some())
    }

    // pub fn initial_unused_peers(&self) -> Vec<P2pConnectionOutgoingInitOpts> {
    //     self.kademlia
    //         .known_peers
    //         .values()
    //         .filter(|v| {
    //             self.ready_peers_iter()
    //                 .find(|(id, _)| (*id).eq(v.peer_id()))
    //                 .is_none()
    //         })
    //         .cloned()
    //         .collect()
    // }

    pub fn ready_peers_iter(&self) -> impl Iterator<Item = (&PeerId, &P2pPeerStatusReady)> {
        self.peers
            .iter()
            .filter_map(|(id, p)| Some((id, p.status_as_ready()?)))
    }

    pub fn ready_rpc_peers_iter(&self) -> impl '_ + Iterator<Item = (PeerId, P2pRpcId)> {
        self.ready_peers_iter()
            .filter(|(_, p)| p.channels.rpc.can_send_request())
            .map(|(peer_id, p)| (*peer_id, p.channels.rpc.next_local_rpc_id()))
    }

    pub fn ready_peers(&self) -> Vec<PeerId> {
        self.peers
            .iter()
            .filter(|(_, p)| p.status_as_ready().is_some())
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn connected_or_connecting_peers_count(&self) -> usize {
        self.peers
            .iter()
            .filter(|(_, p)| p.is_connected_or_connecting())
            .count()
    }

    pub fn is_peer_connected_or_connecting(&self, peer_id: &PeerId) -> bool {
        self.peers
            .get(peer_id)
            .map_or(false, |p| p.is_connected_or_connecting())
    }

    pub fn is_libp2p_peer(&self, peer_id: &PeerId) -> bool {
        self.peers.get(peer_id).map_or(false, |p| p.is_libp2p())
    }

    pub fn is_peer_rpc_timed_out(
        &self,
        peer_id: &PeerId,
        rpc_id: P2pRpcId,
        now: redux::Timestamp,
    ) -> bool {
        self.get_ready_peer(peer_id)
            .map_or(false, |p| p.channels.rpc.is_timed_out(rpc_id, now))
    }

    pub fn peer_rpc_timeouts(&self, now: redux::Timestamp) -> Vec<(PeerId, P2pRpcId)> {
        self.ready_peers_iter()
            .filter_map(|(peer_id, s)| {
                let rpc_id = s.channels.rpc.pending_local_rpc_id()?;
                if !s.channels.rpc.is_timed_out(rpc_id, now) {
                    return None;
                }

                Some((*peer_id, rpc_id))
            })
            .collect()
    }

    pub fn already_has_min_peers(&self) -> bool {
        self.connected_or_connecting_peers_count() >= self.min_peers()
    }

    pub fn already_has_max_peers(&self) -> bool {
        self.connected_or_connecting_peers_count() >= self.config.max_peers
    }

    pub fn already_knows_max_peers(&self) -> bool {
        // self.kademlia.known_peers.len() >= self.config.max_peers * 2
        true
    }

    pub fn enough_time_elapsed(&self, time: redux::Timestamp) -> bool {
        let Some(last_used) = self.kademlia.saturated else {
            return true;
        };
        time.checked_sub(last_used)
            .map(|t| t > self.config.ask_initial_peers_interval)
            .unwrap_or(false)
    }

    /// Minimal number of peers that the node should connect
    pub fn min_peers(&self) -> usize {
        (self.config.max_peers / 2).max(3)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum P2pPeerState {
    #[default]
    Default,
    WebRTC(P2pWebRTCPeerState),
    Libp2p(P2pLibP2pPeerState),
}

/// Constructors
impl P2pPeerState {
    pub fn new_libp2p_incoming() -> P2pPeerState {
        P2pPeerState::Libp2p(P2pLibP2pPeerState {
            dial_opts: Vec::new(),
            status: P2pPeerStatus::Connecting(P2pConnectionState::Incoming(Default::default())),
        })
    }

    pub fn new_webrtc_incoming() -> P2pPeerState {
        P2pPeerState::WebRTC(P2pWebRTCPeerState {
            dial_opts: None,
            status: P2pPeerStatus::Connecting(P2pConnectionState::Incoming(Default::default())),
        })
    }
}

impl P2pPeerState {
    pub fn as_webrtc(&self) -> Option<&P2pWebRTCPeerState> {
        match self {
            P2pPeerState::WebRTC(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_libp2p(&self) -> Option<&P2pLibP2pPeerState> {
        match self {
            P2pPeerState::Libp2p(v) => Some(v),
            _ => None,
        }
    }
    pub fn is_webrtc(&self) -> bool {
        self.as_webrtc().is_some()
    }
    pub fn is_libp2p(&self) -> bool {
        self.as_libp2p().is_some()
    }

    pub fn status_as_ready(&self) -> Option<&P2pPeerStatusReady> {
        match self {
            P2pPeerState::Default => None,
            P2pPeerState::WebRTC(P2pWebRTCPeerState { status, .. }) => status.as_ready(),
            P2pPeerState::Libp2p(P2pLibP2pPeerState { status, .. }) => status.as_ready(),
        }
    }
    pub fn as_status_ready_mut(&mut self) -> Option<&mut P2pPeerStatusReady> {
        match self {
            P2pPeerState::Default => None,
            P2pPeerState::WebRTC(P2pWebRTCPeerState { status, .. }) => status.as_ready_mut(),
            P2pPeerState::Libp2p(P2pLibP2pPeerState { status, .. }) => status.as_ready_mut(),
        }
    }

    pub fn is_connected_or_connecting(&self) -> bool {
        match self {
            P2pPeerState::Default => false,
            P2pPeerState::WebRTC(v) => v.status.is_connected_or_connecting(),
            P2pPeerState::Libp2p(v) => v.status.is_connected_or_connecting(),
        }
    }

    pub fn is_connecting_success(&self) -> bool {
        match self {
            P2pPeerState::Default => false,
            P2pPeerState::WebRTC(v) => v.status.is_connecting_success(),
            P2pPeerState::Libp2p(v) => v.status.is_connecting_success(),
        }
    }

    pub fn is_disconnected(&self) -> bool {
        match self {
            P2pPeerState::WebRTC(P2pWebRTCPeerState {
                status: P2pPeerStatus::Disconnected { .. },
                ..
            })
            | P2pPeerState::Libp2p(P2pLibP2pPeerState {
                status: P2pPeerStatus::Disconnected { .. },
                ..
            }) => true,
            _ => false,
        }
    }

    pub fn connecting_since(&self) -> Option<Timestamp> {
        match self {
            P2pPeerState::WebRTC(P2pWebRTCPeerState {
                status: P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(s)),
                ..
            }) => Some(s.time()),
            P2pPeerState::Libp2p(P2pLibP2pPeerState {
                status: P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(s)),
                ..
            }) => Some(s.time()),
            _ => None,
        }
    }

    pub fn is_connecting_timeout(&self, now: Timestamp) -> bool {
        self.connecting_since().map_or(false, |then| {
            if let Some(dur) = now.checked_sub(then) {
                dur > Duration::from_secs(30)
            } else {
                false
            }
        })
    }

    pub fn not_connected_since(&self) -> Option<Timestamp> {
        match self {
            P2pPeerState::WebRTC(P2pWebRTCPeerState {
                status: P2pPeerStatus::Disconnected { time },
                ..
            })
            | P2pPeerState::Libp2p(P2pLibP2pPeerState {
                status: P2pPeerStatus::Disconnected { time },
                ..
            }) => Some(*time),
            P2pPeerState::WebRTC(P2pWebRTCPeerState {
                status: P2pPeerStatus::Connecting(v),
                ..
            }) if v.is_error() => Some(v.time()),
            P2pPeerState::Libp2p(P2pLibP2pPeerState {
                status: P2pPeerStatus::Connecting(v),
                ..
            }) if v.is_error() => Some(v.time()),
            _ => None,
        }
    }
}

impl P2pPeerState {
    pub fn connection_rpc_id(&self) -> Option<RpcId> {
        match self {
            P2pPeerState::WebRTC(P2pWebRTCPeerState {
                status: P2pPeerStatus::Connecting(v),
                ..
            }) => v.rpc_id(),
            P2pPeerState::Libp2p(P2pLibP2pPeerState {
                status: P2pPeerStatus::Connecting(v),
                ..
            }) => v.rpc_id(),
            _ => None,
        }
    }
}

impl P2pPeerState {
    pub fn get_generic_peers<I>(&self, peer_id: &PeerId) -> I
    where
        I: FromIterator<P2pGenericPeer> + Default,
    {
        match self {
            P2pPeerState::Default => I::default(),
            P2pPeerState::WebRTC(P2pWebRTCPeerState { dial_opts, .. }) => I::from_iter(
                dial_opts
                    .into_iter()
                    .cloned()
                    .map(P2pGenericAddr1::WebRTC)
                    .map(|addr| P2pGenericPeer {
                        peer_id: peer_id.clone(),
                        addr,
                    }),
            ),
            P2pPeerState::Libp2p(P2pLibP2pPeerState { dial_opts, .. }) => I::from_iter(
                dial_opts
                    .into_iter()
                    .cloned()
                    .map(P2pGenericAddr1::LibP2p)
                    .map(|addr| P2pGenericPeer {
                        peer_id: peer_id.clone(),
                        addr,
                    }),
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum P2pPeerStatus<T> {
    #[default]
    Default,
    Connecting(T),
    Disconnected {
        time: redux::Timestamp,
    },
    Ready(P2pPeerStatusReady),
}

impl<T: ConnectionState> P2pPeerStatus<T> {
    /// Checks if the peer is in `Connecting` state and we have finished
    /// connecting to the peer.
    pub fn is_connecting_success(&self) -> bool {
        match self {
            Self::Connecting(v) => v.is_success(),
            _ => false,
        }
    }

    pub fn is_connected_or_connecting(&self) -> bool {
        match self {
            Self::Default => false,
            Self::Connecting(s) => !s.is_error(),
            Self::Ready(_) => true,
            Self::Disconnected { .. } => false,
        }
    }

    pub fn as_connecting(&self) -> Option<&T> {
        match self {
            Self::Connecting(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_ready(&self) -> Option<&P2pPeerStatusReady> {
        match self {
            Self::Ready(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_ready_mut(&mut self) -> Option<&mut P2pPeerStatusReady> {
        match self {
            Self::Ready(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerStatusReady {
    pub is_incoming: bool,
    pub connected_since: redux::Timestamp,
    pub channels: P2pChannelsState,
    pub best_tip: Option<ArcBlockWithHash>,
}

impl P2pPeerStatusReady {
    pub fn new(
        is_incoming: bool,
        time: redux::Timestamp,
        enabled_channels: &BTreeSet<ChannelId>,
    ) -> Self {
        Self {
            is_incoming,
            connected_since: time,
            channels: P2pChannelsState::new(enabled_channels),
            best_tip: None,
        }
    }
}

pub type P2pWebRTCPeerStatus = P2pPeerStatus<
    P2pConnectionState<P2pConnectionWebRTCIncomingState, P2pConnectionWebRTCOutgoingState>,
>;

pub type P2pLibP2pPeerStatus = P2pPeerStatus<
    P2pConnectionState<P2pConnectionLibP2pIncomingState, P2pConnectionLibP2pOutgoingState>,
>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pWebRTCPeerState {
    pub dial_opts: Option<SignalingMethod>,
    pub status: P2pWebRTCPeerStatus,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pLibP2pPeerState {
    pub dial_opts: Vec<P2pLibP2pAddr>,
    pub status: P2pLibP2pPeerStatus,
}
