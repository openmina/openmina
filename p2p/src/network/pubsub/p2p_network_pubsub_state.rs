use super::pb;
use crate::{token::BroadcastAlgorithm, ConnectionAddr, PeerId, StreamId};
use mina_p2p_messages::v2;
use openmina_core::{snark::Snark, transaction::Transaction};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPubsubState {
    pub clients: BTreeMap<PeerId, P2pNetworkPubsubClientState>,
    pub servers: BTreeMap<PeerId, ()>,
    pub seq: u64,
    pub to_sign: VecDeque<pb::Message>,
    pub seen: VecDeque<Vec<u8>>,
    pub incoming_block: Option<(PeerId, v2::MinaBlockBlockStableV2)>,
    pub incoming_transactions: Vec<(Transaction, u32)>,
    pub incoming_snarks: Vec<(Snark, u32)>,
}

impl P2pNetworkPubsubState {
    pub fn prune_peer_state(&mut self, peer_id: &PeerId) {
        self.clients.remove(peer_id);
        self.servers.remove(peer_id);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPubsubClientState {
    pub protocol: BroadcastAlgorithm,
    pub addr: ConnectionAddr,
    pub outgoing_stream_id: Option<StreamId>,
    pub message: pb::Rpc,
    pub buffer: Vec<u8>,
    pub topics: BTreeSet<String>,
}
