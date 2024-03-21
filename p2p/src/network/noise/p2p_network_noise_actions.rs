use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::{Data, DataSized, P2pNetworkAction, P2pState, PeerId};

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkNoiseAction {
    Init(P2pNetworkNoiseInitAction),
    /// remote peer sends the data to the noise
    IncomingData(P2pNetworkNoiseIncomingDataAction),
    IncomingChunk(P2pNetworkNoiseIncomingChunkAction),
    OutgoingChunk(P2pNetworkNoiseOutgoingChunkAction),
    // internals sends the data to the remote peer thru noise
    OutgoingData(P2pNetworkNoiseOutgoingDataAction),
    // the remote peer sends the data to internals thru noise
    DecryptedData(P2pNetworkNoiseDecryptedDataAction),
    HandshakeDone(P2pNetworkNoiseHandshakeDoneAction),
}

impl P2pNetworkNoiseAction {
    pub fn addr(&self) -> SocketAddr {
        match self {
            Self::Init(a) => a.addr,
            Self::IncomingData(a) => a.addr,
            Self::IncomingChunk(a) => a.addr,
            Self::OutgoingChunk(a) => a.addr,
            Self::OutgoingData(a) => a.addr,
            Self::DecryptedData(a) => a.addr,
            Self::HandshakeDone(a) => a.addr,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseInitAction {
    pub addr: SocketAddr,
    pub incoming: bool,
    pub ephemeral_sk: DataSized<32>,
    pub static_sk: DataSized<32>,
    pub signature: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseIncomingDataAction {
    pub addr: SocketAddr,
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseIncomingChunkAction {
    pub addr: SocketAddr,
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseOutgoingChunkAction {
    pub addr: SocketAddr,
    pub data: Vec<Data>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseOutgoingDataAction {
    pub addr: SocketAddr,
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseDecryptedDataAction {
    pub addr: SocketAddr,
    pub peer_id: Option<PeerId>,
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseHandshakeDoneAction {
    pub addr: SocketAddr,
    pub peer_id: PeerId,
    pub incoming: bool,
}

impl From<P2pNetworkNoiseInitAction> for crate::P2pAction {
    fn from(a: P2pNetworkNoiseInitAction) -> Self {
        Self::Network(P2pNetworkAction::Noise(a.into()))
    }
}

impl From<P2pNetworkNoiseIncomingDataAction> for crate::P2pAction {
    fn from(a: P2pNetworkNoiseIncomingDataAction) -> Self {
        Self::Network(P2pNetworkAction::Noise(a.into()))
    }
}

impl From<P2pNetworkNoiseIncomingChunkAction> for crate::P2pAction {
    fn from(a: P2pNetworkNoiseIncomingChunkAction) -> Self {
        Self::Network(P2pNetworkAction::Noise(a.into()))
    }
}

impl From<P2pNetworkNoiseOutgoingChunkAction> for crate::P2pAction {
    fn from(a: P2pNetworkNoiseOutgoingChunkAction) -> Self {
        Self::Network(P2pNetworkAction::Noise(a.into()))
    }
}

impl From<P2pNetworkNoiseOutgoingDataAction> for crate::P2pAction {
    fn from(a: P2pNetworkNoiseOutgoingDataAction) -> Self {
        Self::Network(P2pNetworkAction::Noise(a.into()))
    }
}

impl From<P2pNetworkNoiseDecryptedDataAction> for crate::P2pAction {
    fn from(a: P2pNetworkNoiseDecryptedDataAction) -> Self {
        Self::Network(P2pNetworkAction::Noise(a.into()))
    }
}

impl From<P2pNetworkNoiseHandshakeDoneAction> for crate::P2pAction {
    fn from(a: P2pNetworkNoiseHandshakeDoneAction) -> Self {
        Self::Network(P2pNetworkAction::Noise(a.into()))
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkNoiseAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        match self {
            Self::Init(v) => v.is_enabled(state, time),
            Self::IncomingData(v) => v.is_enabled(state, time),
            Self::IncomingChunk(v) => v.is_enabled(state, time),
            Self::OutgoingChunk(v) => v.is_enabled(state, time),
            Self::OutgoingData(v) => v.is_enabled(state, time),
            Self::DecryptedData(v) => v.is_enabled(state, time),
            Self::HandshakeDone(v) => v.is_enabled(state, time),
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkNoiseInitAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkNoiseIncomingDataAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkNoiseIncomingChunkAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkNoiseOutgoingChunkAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkNoiseOutgoingDataAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkNoiseDecryptedDataAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkNoiseHandshakeDoneAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}
