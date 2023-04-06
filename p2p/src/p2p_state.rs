use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use shared::requests::RpcId;

use crate::channels::{ChannelId, P2pChannelsState};
use crate::connection::outgoing::P2pConnectionOutgoingInitOpts;
use crate::PeerId;

use super::connection::P2pConnectionState;
use super::P2pConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pState {
    pub config: P2pConfig,
    pub peers: BTreeMap<PeerId, P2pPeerState>,
}

impl P2pState {
    pub fn new(config: P2pConfig) -> Self {
        Self {
            config,
            peers: Default::default(),
        }
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
        self.config
            .initial_peers
            .iter()
            .filter(|v| !self.peers.contains_key(&v.peer_id))
            .cloned()
            .collect()
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

    pub fn is_peer_connected_or_connecting(&self, peer_id: &PeerId) -> bool {
        self.peers
            .get(peer_id)
            .map_or(false, |p| p.status.is_connected_or_connecting())
    }

    pub fn already_has_min_peers(&self) -> bool {
        self.connected_or_connecting_peers_count() >= (self.config.max_peers / 2).max(3)
    }

    pub fn already_has_max_peers(&self) -> bool {
        self.connected_or_connecting_peers_count() >= self.config.max_peers
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerState {
    pub dial_opts: Option<P2pConnectionOutgoingInitOpts>,
    pub status: P2pPeerStatus,
}

impl P2pPeerState {
    pub fn connection_rpc_id(&self) -> Option<RpcId> {
        match &self.status {
            P2pPeerStatus::Connecting(v) => v.rpc_id(),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub channels: P2pChannelsState,
}

impl P2pPeerStatusReady {
    pub fn new(enabled_channels: &BTreeSet<ChannelId>) -> Self {
        Self {
            channels: P2pChannelsState::new(enabled_channels),
        }
    }
}
