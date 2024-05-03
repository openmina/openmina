use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    net::SocketAddr,
};

use mina_p2p_messages::v2;
use openmina_core::snark::Snark;
use serde::{Deserialize, Serialize};

use crate::{token::BroadcastAlgorithm, PeerId, StreamId};

use super::pb;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPubsubState {
    pub clients: BTreeMap<PeerId, P2pNetworkPubsubClientState>,
    pub servers: BTreeMap<PeerId, ()>,
    pub seq: u64,
    pub to_sign: VecDeque<pb::Message>,
    pub seen: VecDeque<Vec<u8>>,
    pub incoming_block: Option<v2::MinaBlockBlockStableV2>,
    pub incoming_snarks: Vec<(Snark, u32)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPubsubClientState {
    pub protocol: BroadcastAlgorithm,
    pub addr: SocketAddr,
    pub outgoing_stream_id: Option<StreamId>,
    pub topics: BTreeSet<String>,
    pub message: pb::Rpc,
    pub buffer: Vec<u8>,
}
