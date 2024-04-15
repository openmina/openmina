use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, SocketAddr},
};

use serde::{Deserialize, Serialize};

use crate::{disconnection::P2pDisconnectionReason, PeerId};

use super::super::*;

pub type StreamState<T> = BTreeMap<PeerId, BTreeMap<StreamId, T>>;

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSchedulerState {
    pub interfaces: BTreeSet<IpAddr>,
    pub listeners: BTreeSet<SocketAddr>,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub pnet_key: [u8; 32],
    pub connections: BTreeMap<SocketAddr, P2pNetworkConnectionState>,
    pub broadcast_state: (),
    pub identify_state: identify::P2pNetworkIdentifyState,
    pub floodsub_state: floodsub::P2pNetworkFloodsubState,
    pub discovery_state: Option<P2pNetworkKadState>,
    pub rpc_incoming_streams: StreamState<P2pNetworkRpcState>,
    pub rpc_outgoing_streams: StreamState<P2pNetworkRpcState>,
}

impl P2pNetworkSchedulerState {
    pub fn discovery_state(&self) -> Option<&P2pNetworkKadState> {
        self.discovery_state.as_ref()
    }

    pub fn find_peer(&self, peer_id: &PeerId) -> Option<(&SocketAddr, &P2pNetworkConnectionState)> {
        self.connections
            .iter()
            .find(|(_, conn_state)| conn_state.peer_id() == Some(peer_id))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkConnectionState {
    pub incoming: bool,
    pub pnet: P2pNetworkPnetState,
    pub select_auth: P2pNetworkSelectState,
    pub auth: Option<P2pNetworkAuthState>,
    pub select_mux: P2pNetworkSelectState,
    pub mux: Option<P2pNetworkConnectionMuxState>,
    pub streams: BTreeMap<StreamId, P2pNetworkStreamState>,
    pub closed: Option<P2pNetworkConnectionCloseReason>,
}

impl P2pNetworkConnectionState {
    pub fn peer_id(&self) -> Option<&PeerId> {
        self.auth.as_ref().and_then(P2pNetworkAuthState::peer_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum P2pNetworkConnectionCloseReason {
    #[error("peer is disconnected: {0}")]
    Disconnect(#[from] P2pDisconnectionReason),
    #[error("connection error: {0}")]
    Error(#[from] P2pNetworkConnectionError),
}

/// P2p connection error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
pub enum P2pNetworkConnectionError {
    #[error("mio error: {0}")]
    MioError(String),
    #[error("remote peer closed connection")]
    RemoteClosed,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkAuthState {
    Noise(P2pNetworkNoiseState),
}

impl P2pNetworkAuthState {
    fn peer_id(&self) -> Option<&PeerId> {
        match self {
            P2pNetworkAuthState::Noise(v) => v.peer_id(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkConnectionMuxState {
    Yamux(P2pNetworkYamuxState),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkStreamState {
    pub select: P2pNetworkSelectState,
    pub handler: Option<P2pNetworkStreamHandlerState>,
}

impl P2pNetworkStreamState {
    pub fn new(stream_kind: token::StreamKind) -> Self {
        P2pNetworkStreamState {
            select: P2pNetworkSelectState::initiator_stream(stream_kind),
            handler: None,
        }
    }

    pub fn new_incoming() -> Self {
        P2pNetworkStreamState {
            select: P2pNetworkSelectState::default(),
            handler: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkStreamHandlerState {
    Broadcast,
    Discovery,
}
