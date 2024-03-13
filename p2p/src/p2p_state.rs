use multiaddr::multiaddr;
use openmina_core::block::ArcBlockWithHash;
use redux::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use openmina_core::requests::RpcId;

use crate::channels::rpc::P2pRpcId;
use crate::channels::{ChannelId, P2pChannelsState};
use crate::connection::incoming::P2pConnectionIncomingState;
use crate::connection::outgoing::{P2pConnectionOutgoingInitOpts, P2pConnectionOutgoingState};
use crate::network::identify::P2pNetworkIdentify;
use crate::network::P2pNetworkState;
use crate::{P2pTimeouts, PeerId};

use super::connection::P2pConnectionState;
use super::P2pConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pState {
    pub config: P2pConfig,
    pub network: P2pNetworkState,
    pub peers: BTreeMap<PeerId, P2pPeerState>,
    pub kademlia: P2pKademliaState,
    pub listeners: P2pListenersState,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pKademliaState {
    pub is_ready: bool,
    pub is_bootstrapping: bool,
    pub outgoing_requests: usize,
    pub routes: BTreeMap<PeerId, Vec<P2pConnectionOutgoingInitOpts>>,
    pub known_peers: BTreeMap<PeerId, P2pConnectionOutgoingInitOpts>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pListenerState {
    Open {
        addrs: BTreeSet<multiaddr::Multiaddr>,
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
        let mut kademlia = P2pKademliaState::default();
        if cfg!(feature = "p2p-webrtc") {
            kademlia.known_peers.extend(
                config
                    .initial_peers
                    .iter()
                    .map(|opts| (*opts.peer_id(), opts.clone())),
            );
        }

        let addrs = config
            .libp2p_port
            .map(|port| multiaddr!(Ip4([127, 0, 0, 1]), Tcp((port))))
            .into_iter()
            .collect();

        let known_peers = config
            .initial_peers
            .iter()
            .filter_map(|peer| {
                if let P2pConnectionOutgoingInitOpts::LibP2P(peer) = peer {
                    Some(peer.into())
                } else {
                    None
                }
            })
            .collect();

        let peers = config
            .initial_peers
            .iter()
            .map(|peer| {
                (
                    peer.peer_id().clone(),
                    P2pPeerState {
                        dial_opts: Some(peer.clone()),
                        is_libp2p: peer.is_libp2p(),
                        status: P2pPeerStatus::Disconnected {
                            time: Timestamp::ZERO,
                        },
                        identify: None,
                    },
                )
            })
            .collect();

        let network = P2pNetworkState::new(
            config.identity_pub_key.peer_id(),
            addrs,
            known_peers,
            &config.chain_id,
            config.peer_discovery,
        );
        Self {
            config,
            network,
            listeners: Default::default(),
            peers,
            kademlia,
        }
    }

    pub fn my_id(&self) -> PeerId {
        self.config.identity_pub_key.peer_id()
    }

    pub fn peer_connection_rpc_id(&self, peer_id: &PeerId) -> Option<RpcId> {
        self.peers.get(peer_id)?.connection_rpc_id()
    }

    /// Get peer in ready state. `None` if peer isn't in `Ready` state,
    /// or if peer doesn't exist.
    pub fn get_ready_peer(&self, peer_id: &PeerId) -> Option<&P2pPeerStatusReady> {
        self.peers.get(peer_id)?.status.as_ready()
    }

    /// Get peer in ready state. `None` if peer isn't in `Ready` state,
    /// or if peer doesn't exist.
    pub fn get_ready_peer_mut(&mut self, peer_id: &PeerId) -> Option<&mut P2pPeerStatusReady> {
        self.peers.get_mut(peer_id)?.status.as_ready_mut()
    }

    pub fn any_ready_peers(&self) -> bool {
        self.peers
            .iter()
            .any(|(_, p)| p.status.as_ready().is_some())
    }

    pub fn initial_unused_peers(&self) -> Vec<P2pConnectionOutgoingInitOpts> {
        self.kademlia
            .known_peers
            .values()
            .filter(|v| {
                self.ready_peers_iter()
                    .find(|(id, _)| (*id).eq(v.peer_id()))
                    .is_none()
            })
            .cloned()
            .collect()
    }

    pub fn disconnected_peers(&self) -> impl '_ + Iterator<Item = P2pConnectionOutgoingInitOpts> {
        self.peers.iter().filter_map(|(_, state)| {
            if let P2pPeerState {
                status: P2pPeerStatus::Disconnected { .. },
                dial_opts: Some(opts),
                ..
            } = state
            {
                Some(opts.clone())
            } else {
                None
            }
        })
    }

    pub fn ready_peers_iter(&self) -> impl Iterator<Item = (&PeerId, &P2pPeerStatusReady)> {
        self.peers
            .iter()
            .filter_map(|(id, p)| Some((id, p.status.as_ready()?)))
    }

    pub fn ready_rpc_peers_iter(&self) -> impl '_ + Iterator<Item = (PeerId, P2pRpcId)> {
        self.ready_peers_iter()
            .filter(|(_, p)| p.channels.rpc.can_send_request())
            .map(|(peer_id, p)| (*peer_id, p.channels.rpc.next_local_rpc_id()))
    }

    pub fn ready_peers(&self) -> Vec<PeerId> {
        self.peers
            .iter()
            .filter(|(_, p)| p.status.as_ready().is_some())
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn connected_or_connecting_peers_count(&self) -> usize {
        self.peers
            .iter()
            .filter(|(_, p)| p.status.is_connected_or_connecting())
            .count()
    }

    pub fn is_peer_connecting(&self, peer_id: &PeerId) -> bool {
        self.peers
            .get(peer_id)
            .and_then(|p| p.status.as_connecting())
            .map_or(false, |p| !p.is_error())
    }

    pub fn is_peer_connected_or_connecting(&self, peer_id: &PeerId) -> bool {
        self.peers
            .get(peer_id)
            .map_or(false, |p| p.status.is_connected_or_connecting())
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
        self.get_ready_peer(peer_id).map_or(false, |p| {
            p.channels
                .rpc
                .is_timed_out(rpc_id, now, &self.config.timeouts)
        })
    }

    pub fn peer_rpc_timeouts(&self, now: redux::Timestamp) -> Vec<(PeerId, P2pRpcId)> {
        self.ready_peers_iter()
            .filter_map(|(peer_id, s)| {
                let rpc_id = s.channels.rpc.pending_local_rpc_id()?;
                if !s
                    .channels
                    .rpc
                    .is_timed_out(rpc_id, now, &self.config.timeouts)
                {
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

    /// The peers capacity is exceeded.
    pub fn already_has_too_many_peers(&self) -> bool {
        self.connected_or_connecting_peers_count() > self.config.max_peers
    }

    pub fn already_knows_max_peers(&self) -> bool {
        self.kademlia.known_peers.len() >= self.config.max_peers * 2
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerState {
    pub is_libp2p: bool,
    pub dial_opts: Option<P2pConnectionOutgoingInitOpts>,
    pub status: P2pPeerStatus,
    pub identify: Option<P2pNetworkIdentify>,
}

impl P2pPeerState {
    pub fn is_libp2p(&self) -> bool {
        self.is_libp2p
    }

    pub fn connection_rpc_id(&self) -> Option<RpcId> {
        match &self.status {
            P2pPeerStatus::Connecting(v) => v.rpc_id(),
            _ => None,
        }
    }

    /// Returns true if the peer can be reconnected, that is:
    /// - it has available dial options
    /// - it is never been connected yet or enough time is passed since its connection failure or disconnection.
    pub fn can_reconnect(&self, now: Timestamp, timeouts: &P2pTimeouts) -> bool {
        self.dial_opts.is_some()
            && match &self.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionIncomingState::Error { time, .. },
                )) => now.checked_sub(*time) >= timeouts.incoming_error_reconnect_timeout,
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::Error { time, .. },
                )) => now.checked_sub(*time) >= timeouts.outgoing_error_reconnect_timeout,
                P2pPeerStatus::Disconnected { time } => {
                    *time == Timestamp::ZERO || now.checked_sub(*time) >= timeouts.reconnect_timeout
                }
                _ => false,
            }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "state")]
pub enum P2pPeerStatus {
    Connecting(P2pConnectionState),
    Disconnected { time: redux::Timestamp },

    Ready(P2pPeerStatusReady),
}

impl P2pPeerStatus {
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
            Self::Connecting(s) => !s.is_error(),
            Self::Ready(_) => true,
            Self::Disconnected { .. } => false,
        }
    }

    pub fn as_connecting(&self) -> Option<&P2pConnectionState> {
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
