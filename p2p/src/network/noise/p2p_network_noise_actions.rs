use std::net::SocketAddr;

use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{Data, P2pNetworkAction, P2pState, PeerId};

use super::p2p_network_noise_state::{Pk, Sk};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = debug, fields(display(addr), incoming, debug(data), display(peer_id)))]
pub enum P2pNetworkNoiseAction {
    Init {
        addr: SocketAddr,
        incoming: bool,
        ephemeral_sk: Sk,
        ephemeral_pk: Pk,
        static_sk: Sk,
        static_pk: Pk,
        signature: Data,
    },
    /// remote peer sends the data to the noise
    IncomingData {
        addr: SocketAddr,
        data: Data,
    },
    IncomingChunk {
        addr: SocketAddr,
        data: Data,
    },
    OutgoingChunk {
        addr: SocketAddr,
        data: Vec<Data>,
    },
    // internals sends the data to the remote peer thru noise
    OutgoingData {
        addr: SocketAddr,
        data: Data,
    },
    // the remote peer sends the data to internals thru noise
    #[action_event(fields(display(addr), debug(data), debug(peer_id)))]
    DecryptedData {
        addr: SocketAddr,
        peer_id: Option<PeerId>,
        data: Data,
    },
    HandshakeDone {
        addr: SocketAddr,
        peer_id: PeerId,
        incoming: bool,
    },
}

impl P2pNetworkNoiseAction {
    pub fn addr(&self) -> &SocketAddr {
        match self {
            Self::Init { addr, .. } => addr,
            Self::IncomingData { addr, .. } => addr,
            Self::IncomingChunk { addr, .. } => addr,
            Self::OutgoingChunk { addr, .. } => addr,
            Self::OutgoingData { addr, .. } => addr,
            Self::DecryptedData { addr, .. } => addr,
            Self::HandshakeDone { addr, .. } => addr,
        }
    }
}

impl From<P2pNetworkNoiseAction> for crate::P2pAction {
    fn from(a: P2pNetworkNoiseAction) -> Self {
        Self::Network(P2pNetworkAction::Noise(a))
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkNoiseAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            Self::Init { .. } => true,
            Self::IncomingData { .. } => true,
            Self::IncomingChunk { .. } => true,
            Self::OutgoingChunk { .. } => true,
            Self::OutgoingData { .. } => true,
            Self::DecryptedData { .. } => true,
            Self::HandshakeDone { .. } => true,
        }
    }
}
