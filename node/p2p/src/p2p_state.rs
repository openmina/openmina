use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use libp2p::PeerId;

use shared::requests::RpcId;

use crate::rpc::{P2pRpcId, P2pRpcState};

use super::connection::P2pConnectionState;
use super::P2pConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pState {
    pub config: P2pConfig,
    pub peers: BTreeMap<PeerId, P2pPeerState>,
}

impl P2pState {
    pub fn new() -> Self {
        Self {
            config: P2pConfig {},
            peers: Default::default(),
        }
    }

    pub fn peer_connection_outgoing_rpc_id(&self, peer_id: &PeerId) -> Option<RpcId> {
        self.peers.get(&peer_id)?.connection_outgoing_rpc_id()
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

    /// Get peer which has least pending rpcs to initiate new rpc.
    pub fn get_free_peer_id_for_rpc(&self) -> Option<(PeerId, P2pRpcId)> {
        self.peers
            .iter()
            .filter_map(|(id, p)| Some((id, p.status.as_ready()?)))
            .min_by(|a, b| a.1.rpc.outgoing.len().cmp(&b.1.rpc.outgoing.len()))
            .map(|(id, p)| (id.clone(), p.rpc.outgoing.next_req_id()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerState {
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
pub struct P2pPeerStatusReady {
    pub rpc: P2pRpcState,
}

impl P2pPeerStatusReady {
    pub fn new() -> Self {
        Self {
            rpc: P2pRpcState::new(),
        }
    }
}
