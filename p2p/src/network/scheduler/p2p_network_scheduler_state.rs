use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, SocketAddr},
};

use serde::{Deserialize, Serialize};

use crate::PeerId;

use super::super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct P2pNetworkSchedulerState {
    pub interfaces: BTreeSet<IpAddr>,
    pub listeners: BTreeSet<SocketAddr>,
    pub pnet_key: [u8; 32],
    pub connections: BTreeMap<SocketAddr, P2pNetworkConnectionState>,
    pub broadcast_state: (),
    pub discovery_state: (),
    pub rpc_incoming_streams: BTreeMap<PeerId, BTreeMap<StreamId, P2pNetworkRpcState>>,
    pub rpc_outgoing_streams: BTreeMap<PeerId, BTreeMap<StreamId, P2pNetworkRpcState>>,
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkAuthState {
    Noise(P2pNetworkNoiseState),
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
