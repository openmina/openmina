use super::pb;
use crate::{token::BroadcastAlgorithm, ConnectionAddr, PeerId, StreamId};
use mina_p2p_messages::v2;
use openmina_core::{snark::Snark, transaction::Transaction};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPubsubState {
    pub clients: BTreeMap<PeerId, P2pNetworkPubsubClientState>,
    pub seq: u64,
    pub to_sign: VecDeque<pb::Message>,
    pub seen: VecDeque<Vec<u8>>,
    pub mcache: P2pNetworkPubsubMessageCache,
    pub incoming_block: Option<(PeerId, v2::MinaBlockBlockStableV2)>,
    pub incoming_transactions: Vec<(Transaction, u32)>,
    pub incoming_snarks: Vec<(Snark, u32)>,
    pub topics: BTreeMap<String, BTreeMap<PeerId, P2pNetworkPubsubClientTopicState>>,
}

impl P2pNetworkPubsubState {
    pub fn prune_peer_state(&mut self, peer_id: &PeerId) {
        self.clients.remove(peer_id);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPubsubClientState {
    pub protocol: BroadcastAlgorithm,
    pub addr: ConnectionAddr,
    pub outgoing_stream_id: Option<StreamId>,
    pub message: pb::Rpc,
    pub buffer: Vec<u8>,
}

// TODO: store blocks, snarks and txs separately
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPubsubMessageCache {
    pub map: BTreeMap<Vec<u8>, pb::Message>,
    pub queue: VecDeque<Vec<u8>>,
}

impl P2pNetworkPubsubMessageCache {
    const CAPACITY: usize = 100;

    pub fn put(&mut self, message: pb::Message) -> Option<Vec<u8>> {
        let id = compute_message_id(&message)?;
        self.map.insert(id.clone(), message);
        self.queue.push_back(id.clone());
        if self.queue.len() > Self::CAPACITY {
            if let Some(id) = self.queue.pop_back() {
                self.map.remove(&id);
            }
        }
        Some(id)
    }
}

// TODO: what if wasm32?
// How to test it?
pub fn compute_message_id(message: &pb::Message) -> Option<Vec<u8>> {
    let source_bytes = message
        .from
        .as_ref()
        .map(AsRef::as_ref)
        .unwrap_or(&[0, 1, 0][..]);

    let mut source_string = libp2p_identity::PeerId::from_bytes(source_bytes)
        .ok()?
        .to_base58();

    let sequence_number = message
        .seqno
        .as_ref()
        .and_then(|b| <[u8; 8]>::try_from(b.as_slice()).ok())
        .map(u64::from_be_bytes)
        .unwrap_or_default();
    source_string.push_str(&sequence_number.to_string());
    Some(source_string.into_bytes())
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct P2pNetworkPubsubClientTopicState {
    pub mesh: P2pNetworkPubsubClientMeshAddingState,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum P2pNetworkPubsubClientMeshAddingState {
    #[default]
    Initial,
    TheyRefused,
    WeRefused,
    Added,
}

impl P2pNetworkPubsubClientTopicState {
    pub fn on_mesh(&self) -> bool {
        matches!(&self.mesh, P2pNetworkPubsubClientMeshAddingState::Added)
    }
}
