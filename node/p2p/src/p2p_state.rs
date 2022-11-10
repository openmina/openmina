use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use libp2p::PeerId;

use shared::requests::RpcId;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerStatusReady {}
