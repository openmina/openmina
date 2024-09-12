use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, SocketAddr},
};

use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::{disconnection::P2pDisconnectionReason, identity::PublicKey, PeerId};

use super::super::*;

pub type StreamState<T> = BTreeMap<PeerId, BTreeMap<StreamId, T>>;

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy)]
pub struct ConnectionAddr {
    pub sock_addr: SocketAddr,
    pub incoming: bool,
}
impl std::fmt::Display for ConnectionAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} (incoming: {})", self.sock_addr, self.incoming)
    }
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSchedulerState {
    pub interfaces: BTreeSet<IpAddr>,
    pub listeners: BTreeSet<SocketAddr>,
    pub local_pk: PublicKey,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub pnet_key: [u8; 32],
    pub connections: BTreeMap<ConnectionAddr, P2pNetworkConnectionState>,
    pub broadcast_state: P2pNetworkPubsubState,
    pub identify_state: identify::P2pNetworkIdentifyState,
    pub discovery_state: Option<P2pNetworkKadState>,
    pub rpc_incoming_streams: StreamState<P2pNetworkRpcState>,
    pub rpc_outgoing_streams: StreamState<P2pNetworkRpcState>,
}

impl P2pNetworkSchedulerState {
    pub fn discovery_state(&self) -> Option<&P2pNetworkKadState> {
        self.discovery_state.as_ref()
    }

    pub fn find_peer(
        &self,
        peer_id: &PeerId,
    ) -> Option<(&ConnectionAddr, &P2pNetworkConnectionState)> {
        self.connections
            .iter()
            .find(|(_, conn_state)| conn_state.peer_id() == Some(peer_id))
    }

    pub fn prune_peer_state(&mut self, peer_id: &PeerId) {
        self.broadcast_state.prune_peer_state(peer_id);
        self.identify_state.prune_peer_state(peer_id);

        if let Some(discovery_state) = self.discovery_state.as_mut() {
            discovery_state.streams.remove(peer_id);
        }

        self.rpc_incoming_streams.remove(peer_id);
        self.rpc_outgoing_streams.remove(peer_id);
    }

    pub fn connection_state_mut(
        &mut self,
        addr: &ConnectionAddr,
    ) -> Option<&mut P2pNetworkConnectionState> {
        self.connections.get_mut(addr)
    }

    pub fn connection_state(&self, addr: &ConnectionAddr) -> Option<&P2pNetworkConnectionState> {
        self.connections.get(addr)
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
    // the number of bytes that peer allowed to send us before yamux is negotiated
    pub limit: usize,
}

impl P2pNetworkConnectionState {
    pub const INITIAL_LIMIT: usize = 1024;

    pub fn peer_id(&self) -> Option<&PeerId> {
        self.auth.as_ref().and_then(P2pNetworkAuthState::peer_id)
    }

    pub fn limit(&self) -> usize {
        if let Some(mux) = &self.mux {
            mux.limit()
        } else {
            self.limit
        }
    }

    pub fn consume(&mut self, len: usize) {
        if let Some(mux) = &mut self.mux {
            mux.consume(len);
        } else {
            self.limit = self.limit.saturating_sub(len);
        }
    }

    pub fn noise_state_mut(&mut self) -> Option<&mut P2pNetworkNoiseState> {
        self.auth
            .as_mut()
            .map(|P2pNetworkAuthState::Noise(state)| state)
    }

    pub fn yamux_state_mut(&mut self) -> Option<&mut P2pNetworkYamuxState> {
        self.mux
            .as_mut()
            .map(|P2pNetworkConnectionMuxState::Yamux(state)| state)
    }

    pub fn yamux_state(&self) -> Option<&P2pNetworkYamuxState> {
        self.mux
            .as_ref()
            .map(|P2pNetworkConnectionMuxState::Yamux(state)| state)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, thiserror::Error)]
pub enum P2pNetworkConnectionCloseReason {
    #[error("peer is disconnected: {0}")]
    Disconnect(#[from] P2pDisconnectionReason),
    #[error("connection error: {0}")]
    Error(#[from] P2pNetworkConnectionError),
}

impl P2pNetworkConnectionCloseReason {
    /// Returns true if the reason for disconnection is the statemachine
    /// behaviour, not external error.
    pub fn is_disconnected(&self) -> bool {
        matches!(self, P2pNetworkConnectionCloseReason::Disconnect(_))
    }
}

/// P2p connection error.
#[derive(Debug, Clone, PartialEq, thiserror::Error, Serialize, Deserialize)]
pub enum P2pNetworkConnectionError {
    #[error("mio error: {0}")]
    MioError(String),
    #[error("noise handshake error: {0}")]
    Noise(#[from] NoiseError),
    #[error("remote peer closed connection")]
    RemoteClosed,
    #[error("select protocol error")]
    SelectError,
    #[error(transparent)]
    IdentifyStreamError(#[from] P2pNetworkIdentifyStreamError),
    #[error(transparent)]
    KademliaIncomingStreamError(#[from] P2pNetworkKadIncomingStreamError),
    #[error(transparent)]
    KademliaOutgoingStreamError(#[from] P2pNetworkKadOutgoingStreamError),
    #[error("peer reset yamux stream")]
    StreamReset(StreamId),
    #[error("pubsub error: {0}")]
    PubSubError(String),
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

impl P2pNetworkConnectionMuxState {
    pub fn consume(&mut self, len: usize) {
        match self {
            Self::Yamux(state) => state.consume(len),
        }
    }

    fn limit(&self) -> usize {
        match self {
            Self::Yamux(state) => state.limit(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkStreamState {
    pub select: P2pNetworkSelectState,
}

impl P2pNetworkStreamState {
    pub fn new(stream_kind: token::StreamKind, time: Timestamp) -> Self {
        P2pNetworkStreamState {
            select: P2pNetworkSelectState::initiator_stream(stream_kind, time),
        }
    }

    pub fn new_incoming(time: Timestamp) -> Self {
        P2pNetworkStreamState {
            select: P2pNetworkSelectState::default_timed(time),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkStreamHandlerState {
    Broadcast,
    Discovery,
}
