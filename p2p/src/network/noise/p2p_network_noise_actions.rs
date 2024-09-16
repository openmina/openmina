use super::p2p_network_noise_state::Sk;
use crate::{ConnectionAddr, Data, P2pNetworkAction, P2pState, PeerId};
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = debug, fields(display(addr), incoming, debug(data), display(peer_id)))]
pub enum P2pNetworkNoiseAction {
    Init {
        addr: ConnectionAddr,
        incoming: bool,
        ephemeral_sk: Sk,
        static_sk: Sk,
        signature: Data,
    },
    /// remote peer sends the data to the noise
    IncomingData {
        addr: ConnectionAddr,
        data: Data,
    },
    IncomingChunk {
        addr: ConnectionAddr,
    },
    OutgoingChunk {
        addr: ConnectionAddr,
        data: Vec<Data>,
    },
    OutgoingChunkSelectMux {
        addr: ConnectionAddr,
        data: Vec<Data>,
    },
    // internals sends the data to the remote peer thru noise
    OutgoingData {
        addr: ConnectionAddr,
        data: Data,
    },
    OutgoingDataSelectMux {
        addr: ConnectionAddr,
        data: Data,
    },
    // the remote peer sends the data to internals thru noise
    #[action_event(fields(display(addr), debug(data), debug(peer_id)))]
    DecryptedData {
        addr: ConnectionAddr,
        peer_id: Option<PeerId>,
        data: Data,
    },
    HandshakeDone {
        addr: ConnectionAddr,
        peer_id: PeerId,
        incoming: bool,
    },
}

impl P2pNetworkNoiseAction {
    pub fn addr(&self) -> &ConnectionAddr {
        match self {
            Self::Init { addr, .. } => addr,
            Self::IncomingData { addr, .. } => addr,
            Self::IncomingChunk { addr, .. } => addr,
            Self::OutgoingChunk { addr, .. } => addr,
            Self::OutgoingChunkSelectMux { addr, .. } => addr,
            Self::OutgoingData { addr, .. } => addr,
            Self::OutgoingDataSelectMux { addr, .. } => addr,
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
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        let Some(_noise_state) = state
            .network
            .scheduler
            .connection_state(self.addr())
            .and_then(|state| state.noise_state())
        else {
            return false;
        };

        match self {
            Self::Init { .. } => true,
            Self::IncomingData { .. } => true,
            Self::IncomingChunk { .. } => true,
            Self::OutgoingChunk { .. } => true,
            Self::OutgoingChunkSelectMux { .. } => true,
            Self::OutgoingData { .. } => true,
            Self::OutgoingDataSelectMux { .. } => true,
            Self::DecryptedData { .. } => true,
            Self::HandshakeDone { .. } => true,
        }
    }
}
