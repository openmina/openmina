use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use shared::requests::RpcId;

use crate::connection::incoming::P2pConnectionIncomingState;
use crate::connection::outgoing::{P2pConnectionOutgoingInitOpts, P2pConnectionOutgoingState};
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

    pub fn peer_connection_outgoing_rpc_id(&self, peer_id: &PeerId) -> Option<RpcId> {
        self.peers.get(peer_id)?.connection_outgoing_rpc_id()
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
            .filter(|(_, p)| match &p.status {
                P2pPeerStatus::Connecting(s) => match s {
                    P2pConnectionState::Outgoing(s) => !matches!(
                        s,
                        P2pConnectionOutgoingState::AnswerRecvError { .. }
                            | P2pConnectionOutgoingState::FinalizeError { .. }
                            | P2pConnectionOutgoingState::Error { .. }
                    ),
                    P2pConnectionState::Incoming(s) => !matches!(
                        s,
                        P2pConnectionIncomingState::FinalizeError { .. }
                            | P2pConnectionIncomingState::Error { .. }
                    ),
                },
                P2pPeerStatus::Ready(_) => true,
                _ => false,
            })
            .count()
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
    pub dial_opts: P2pConnectionOutgoingInitOpts,
    pub status: P2pPeerStatus,
}

impl P2pPeerState {
    pub fn connection_outgoing_rpc_id(&self) -> Option<RpcId> {
        match &self.status {
            P2pPeerStatus::Connecting(v) => v.outgoing_rpc_id(),
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
pub struct P2pPeerStatusReady {}

impl P2pPeerStatusReady {
    pub fn new() -> Self {
        Self {}
    }
}
