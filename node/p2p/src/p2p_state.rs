use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use libp2p::PeerId;

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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerState {
    pub status: P2pPeerStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pPeerStatus {
    Connecting(P2pConnectionState),

    Ready(P2pPeerStatusReady),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerStatusReady {}
