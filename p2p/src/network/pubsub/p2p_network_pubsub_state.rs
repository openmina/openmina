use super::{pb, BroadcastMessageId};
use crate::{token::BroadcastAlgorithm, ConnectionAddr, PeerId, StreamId};

use libp2p_identity::ParseError;
use mina_p2p_messages::{gossip::GossipNetMessageV2, v2::TransactionHash};
use openmina_core::{
    snark::{Snark, SnarkJobId},
    transaction::Transaction,
};
use redux::Timestamp;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    time::Duration,
};

use malloc_size_of_derive::MallocSizeOf;

pub const IWANT_TIMEOUT_DURATION: Duration = Duration::from_secs(5);

/// State of the P2P Network PubSub system.
///
/// This struct maintains information about connected peers, message sequencing,
/// message caching, and topic subscriptions. It handles incoming and outgoing
/// messages, manages the mesh network topology, and ensures efficient message
/// broadcasting across the network.
#[derive(Default, Serialize, Deserialize, Debug, Clone, MallocSizeOf)]
pub struct P2pNetworkPubsubState {
    /// State of each connected peer.
    #[with_malloc_size_of_func = "measurement::clients"]
    pub clients: BTreeMap<PeerId, P2pNetworkPubsubClientState>,

    /// Current message sequence number.
    ///
    /// Increments with each new message to ensure proper ordering and uniqueness.
    pub seq: u64,

    /// Messages awaiting cryptographic signing.
    pub to_sign: VecDeque<pb::Message>,

    /// Recently seen message identifiers to prevent duplication.
    ///
    /// Keeps a limited history of message signatures to avoid processing
    /// the same message multiple times.
    pub seen: VecDeque<Vec<u8>>,

    /// Cache of published messages for efficient retrieval and broadcasting.
    ///
    /// For quick access and reducing redundant data transmission across peers.
    pub mcache: P2pNetworkPubsubMessageCache,

    /// Incoming transactions from peers along with their nonces.
    pub incoming_transactions: Vec<(Transaction, u32)>,

    /// Incoming snarks from peers along with their nonces.
    pub incoming_snarks: Vec<(Snark, u32)>,

    /// Topics and their subscribed peers.
    #[with_malloc_size_of_func = "measurement::topics"]
    pub topics: BTreeMap<String, BTreeMap<PeerId, P2pNetworkPubsubClientTopicState>>,

    /// `iwant` requests, tracking the number of times peers have expressed interest in specific messages.
    pub iwant: VecDeque<P2pNetworkPubsubIwantRequestCount>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, MallocSizeOf)]
pub struct P2pNetworkPubsubIwantRequestCount {
    pub message_id: Vec<u8>,
    #[with_malloc_size_of_func = "measurement::timestamps"]
    pub count: Vec<Timestamp>,
}

impl P2pNetworkPubsubState {
    pub fn prune_peer_state(&mut self, peer_id: &PeerId) {
        self.clients.remove(peer_id);
    }

    pub fn filter_iwant_message_ids(&mut self, message_id: &[u8], timestamp: Timestamp) -> bool {
        if self
            .mcache
            .get_message_from_raw_message_id(message_id)
            .is_some()
        {
            return false;
        }

        let message_count = self
            .iwant
            .iter_mut()
            .find(|message| message.message_id == message_id);

        match message_count {
            Some(message) => {
                let message_counts = std::mem::take(&mut message.count);

                message.count = message_counts
                    .into_iter()
                    .filter(|time| {
                        timestamp
                            .checked_sub(*time)
                            .is_some_and(|duration| duration < IWANT_TIMEOUT_DURATION)
                    })
                    .collect();

                if message.count.len() < 3 {
                    message.count.push(timestamp);
                    return true;
                }

                false
            }
            None => {
                let message_count = P2pNetworkPubsubIwantRequestCount {
                    message_id: message_id.to_vec(),
                    count: vec![timestamp],
                };

                self.iwant.push_back(message_count);
                if self.iwant.len() > 10 {
                    self.iwant.pop_front();
                }

                true
            }
        }
    }

    pub fn clear_incoming(&mut self) {
        self.incoming_transactions.clear();
        self.incoming_snarks.clear();

        self.incoming_transactions.shrink_to(0x20);
        self.incoming_snarks.shrink_to(0x20);
    }
}

/// State of a pubsub client connected to a peer.
///
/// This struct maintains essential information about the client's protocol,
/// connection details, message buffers, and caching mechanisms. It facilitates
/// efficient message handling and broadcasting within the pubsub system.
#[derive(Serialize, Deserialize, Debug, Clone, MallocSizeOf)]
pub struct P2pNetworkPubsubClientState {
    /// Broadcast algorithm used for this client.
    pub protocol: BroadcastAlgorithm,

    /// Connection address of the peer.
    pub addr: ConnectionAddr,

    /// Outgoing stream identifier, if any.
    ///
    /// - `Some(StreamId)`: Indicates an active outgoing stream.
    /// - `None`: No outgoing stream is currently established.
    pub outgoing_stream_id: Option<StreamId>,

    /// Current RPC message being constructed or processed.
    ///
    /// - `subscriptions`: List of subscription options for various topics.
    /// - `publish`: Messages queued for publishing.
    /// - `control`: Control commands for managing the mesh network.
    pub message: pb::Rpc,

    /// Cache of recently published messages.
    pub cache: P2pNetworkPubsubRecentlyPublishCache,

    /// Buffer for incoming data fragments.
    ///
    /// Stores partial data received from peers, facilitating the assembly of complete
    /// messages when all fragments are received.
    pub buffer: Vec<u8>,

    /// Collection of incoming messages from the peer.
    ///
    /// Holds fully decoded `pb::Message` instances received from the peer,
    /// ready for further handling such as validation, caching, and broadcasting.
    pub incoming_messages: Vec<pb::Message>,
}

impl P2pNetworkPubsubClientState {
    pub fn publish(&mut self, message: &pb::Message) {
        let Ok(id) = P2pNetworkPubsubMessageCacheId::compute_message_id(message) else {
            self.message.publish.push(message.clone());
            return;
        };
        if self.cache.map.insert(id) {
            self.message.publish.push(message.clone());
        }
        self.cache.queue.push_back(id);
        if self.cache.queue.len() > 50 {
            if let Some(id) = self.cache.queue.pop_front() {
                self.cache.map.remove(&id);
            }
        }
    }

    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
        self.buffer.shrink_to(0x2000);
    }

    pub fn clear_incoming(&mut self) {
        self.incoming_messages.clear();
        self.incoming_messages.shrink_to(0x20)
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPubsubRecentlyPublishCache {
    pub map: BTreeSet<P2pNetworkPubsubMessageCacheId>,
    pub queue: VecDeque<P2pNetworkPubsubMessageCacheId>,
}

// TODO: store blocks, snarks and txs separately
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPubsubMessageCache {
    pub map: BTreeMap<P2pNetworkPubsubMessageCacheId, P2pNetworkPubsubMessageCacheMessage>,
    pub queue: VecDeque<P2pNetworkPubsubMessageCacheId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkPubsubMessageCacheMessage {
    Init {
        message: pb::Message,
        content: GossipNetMessageV2,
        peer_id: PeerId,
        time: Timestamp,
    },
    PreValidatedBlockMessage {
        block_hash: mina_p2p_messages::v2::StateHash,
        message: pb::Message,
        peer_id: PeerId,
        time: Timestamp,
    },
    PreValidatedSnark {
        job_id: SnarkJobId,
        message: pb::Message,
        peer_id: PeerId,
        time: Timestamp,
    },
    PreValidatedTransactions {
        tx_hashes: Vec<TransactionHash>,
        message: pb::Message,
        peer_id: PeerId,
        time: Timestamp,
    },
    PreValidated {
        message: pb::Message,
        peer_id: PeerId,
        time: Timestamp,
    },
    Validated {
        message: pb::Message,
        peer_id: PeerId,
        time: Timestamp,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub struct P2pNetworkPubsubMessageCacheId {
    pub source: libp2p_identity::PeerId,
    pub seqno: u64,
}

impl P2pNetworkPubsubMessageCacheId {
    // TODO: what if wasm32?
    // How to test it?
    pub fn compute_message_id(
        message: &pb::Message,
    ) -> Result<P2pNetworkPubsubMessageCacheId, ParseError> {
        let source = source_from_message(message)?;

        let seqno = message
            .seqno
            .as_ref()
            .and_then(|b| <[u8; 8]>::try_from(b.as_slice()).ok())
            .map(u64::from_be_bytes)
            .unwrap_or_default();

        Ok(P2pNetworkPubsubMessageCacheId { source, seqno })
    }

    pub fn to_raw_bytes(&self) -> Vec<u8> {
        let mut message_id = self.source.to_base58();
        message_id.push_str(&self.seqno.to_string());
        message_id.into_bytes()
    }
}

macro_rules! enum_field {
    ($field:ident: $field_type:ty) => {
        pub fn $field(&self) -> &$field_type {
            match self {
                Self::Init { $field, .. }
                | Self::PreValidated { $field, .. }
                | Self::PreValidatedBlockMessage { $field, .. }
                | Self::PreValidatedSnark { $field, .. }
                | Self::PreValidatedTransactions { $field, .. }
                | Self::Validated { $field, .. } => $field,
            }
        }
    };
}

impl P2pNetworkPubsubMessageCacheMessage {
    enum_field!(message: pb::Message);
    enum_field!(time: Timestamp);
    enum_field!(peer_id: PeerId);
}

impl P2pNetworkPubsubMessageCache {
    const CAPACITY: usize = 100;

    pub fn put(
        &mut self,
        message: pb::Message,
        content: GossipNetMessageV2,
        peer_id: PeerId,
        time: Timestamp,
    ) -> Result<P2pNetworkPubsubMessageCacheId, ParseError> {
        let id = P2pNetworkPubsubMessageCacheId::compute_message_id(&message)?;
        self.map.insert(
            id,
            P2pNetworkPubsubMessageCacheMessage::Init {
                message,
                content,
                time,
                peer_id,
            },
        );

        self.queue.push_back(id);
        if self.queue.len() > Self::CAPACITY {
            if let Some(id) = self.queue.pop_front() {
                self.map.remove(&id);
            }
        }
        Ok(id)
    }

    pub fn get_message(&self, id: &P2pNetworkPubsubMessageCacheId) -> Option<&GossipNetMessageV2> {
        let message = self.map.get(id)?;
        match message {
            P2pNetworkPubsubMessageCacheMessage::Init { content, .. } => Some(content),
            _ => None,
        }
    }

    pub fn contains_broadcast_id(&self, message_id: &BroadcastMessageId) -> bool {
        match message_id {
            BroadcastMessageId::BlockHash { hash } => self
                .map
                .values()
                .any(|message| matches!(message, P2pNetworkPubsubMessageCacheMessage::PreValidatedBlockMessage { block_hash, .. } if block_hash == hash)),
            BroadcastMessageId::MessageId { message_id } => {
                self.map.contains_key(message_id)
            },
            BroadcastMessageId::Snark { job_id: snark_job_id } => {
                self
                    .map
                    .values()
                    .any(|message| matches!(message, P2pNetworkPubsubMessageCacheMessage::PreValidatedSnark { job_id,.. } if job_id == snark_job_id))
            },
            BroadcastMessageId::TransactionDiff { txs } => {
                self
                    .map
                    .values()
                    .any(|message| matches!(message, P2pNetworkPubsubMessageCacheMessage::PreValidatedTransactions { tx_hashes, .. } if compare_transaction_diff(tx_hashes, txs)))
            }
        }
    }

    pub fn get_message_id_and_message(
        &mut self,
        message_id: &BroadcastMessageId,
    ) -> Option<(
        P2pNetworkPubsubMessageCacheId,
        &mut P2pNetworkPubsubMessageCacheMessage,
    )> {
        match message_id {
            BroadcastMessageId::BlockHash { hash } => {
                self.map
                    .iter_mut()
                    .find_map(|(message_id, message)| match message {
                        P2pNetworkPubsubMessageCacheMessage::PreValidatedBlockMessage {
                            block_hash,
                            ..
                        } if block_hash == hash => Some((*message_id, message)),
                        _ => None,
                    })
            }
            BroadcastMessageId::MessageId { message_id } => self
                .map
                .get_mut(message_id)
                .map(|content| (*message_id, content)),
            BroadcastMessageId::Snark {
                job_id: snark_job_id,
            } => {
                self.map
                    .iter_mut()
                    .find_map(|(message_id, message)| match message {
                        P2pNetworkPubsubMessageCacheMessage::PreValidatedSnark {
                            job_id, ..
                        } if job_id == snark_job_id => Some((*message_id, message)),
                        _ => None,
                    })
            }
            BroadcastMessageId::TransactionDiff { txs } => {
                self.map
                    .iter_mut()
                    .find_map(|(message_id, message)| match message {
                        P2pNetworkPubsubMessageCacheMessage::PreValidatedTransactions {
                            tx_hashes,
                            ..
                        } if compare_transaction_diff(tx_hashes.as_slice(), txs) => {
                            Some((*message_id, message))
                        }
                        _ => None,
                    })
            }
        }
    }

    pub fn remove_message(
        &mut self,
        message_id: P2pNetworkPubsubMessageCacheId,
    ) -> Option<P2pNetworkPubsubMessageCacheMessage> {
        let message = self.map.remove(&message_id);
        if let Some(position) = self.queue.iter().position(|id| id == &message_id) {
            self.queue.remove(position);
        }
        message
    }

    pub fn get_message_from_raw_message_id(
        &self,
        message_id: &[u8],
    ) -> Option<&P2pNetworkPubsubMessageCacheMessage> {
        self.map.iter().find_map(|(key, value)| {
            if key.to_raw_bytes() == message_id {
                Some(value)
            } else {
                None
            }
        })
    }
}

pub fn source_from_message(message: &pb::Message) -> Result<libp2p_identity::PeerId, ParseError> {
    let source_bytes = message
        .from
        .as_ref()
        .map(AsRef::as_ref)
        .unwrap_or(&[0, 1, 0][..]);

    libp2p_identity::PeerId::from_bytes(source_bytes)
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

impl P2pNetworkPubsubClientState {
    pub fn message_is_empty(&self) -> bool {
        self.message.subscriptions.is_empty()
            && self.message.publish.is_empty()
            && self.message.control.is_none()
    }
}

impl P2pNetworkPubsubClientTopicState {
    pub fn on_mesh(&self) -> bool {
        matches!(&self.mesh, P2pNetworkPubsubClientMeshAddingState::Added)
    }
}

fn compare_transaction_diff(diffa: &[TransactionHash], diffb: &[TransactionHash]) -> bool {
    if diffa.len() != diffb.len() {
        false
    } else {
        diffa.iter().all(|a| diffb.contains(a)) && diffb.iter().all(|b| diffa.contains(b))
    }
}

mod measurement {
    use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
    use std::mem;

    use super::*;

    pub fn clients(
        val: &BTreeMap<PeerId, P2pNetworkPubsubClientState>,
        ops: &mut MallocSizeOfOps,
    ) -> usize {
        val.values().map(|v| v.size_of(ops)).sum()
    }

    pub fn topics(
        val: &BTreeMap<String, BTreeMap<PeerId, P2pNetworkPubsubClientTopicState>>,
        ops: &mut MallocSizeOfOps,
    ) -> usize {
        val.iter()
            .map(|(k, v)| k.size_of(ops) + v.size_of(ops))
            .sum()
    }

    pub fn timestamps(val: &Vec<Timestamp>, _ops: &mut MallocSizeOfOps) -> usize {
        val.capacity() * mem::size_of::<Timestamp>()
    }

    impl MallocSizeOf for P2pNetworkPubsubRecentlyPublishCache {
        fn size_of(&self, _ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
            let map_size = self.map.len() * size_of::<P2pNetworkPubsubMessageCacheId>();
            let queue_size = self.queue.capacity() * size_of::<P2pNetworkPubsubMessageCacheId>();
            map_size + queue_size
        }
    }

    impl MallocSizeOf for P2pNetworkPubsubMessageCache {
        fn size_of(&self, _ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
            let map_size = self.map.len()
                * (size_of::<P2pNetworkPubsubMessageCacheId>()
                    + size_of::<P2pNetworkPubsubMessageCacheMessage>());
            let queue_size = self.queue.capacity() * size_of::<P2pNetworkPubsubMessageCacheId>();
            map_size + queue_size
        }
    }

    impl MallocSizeOf for P2pNetworkPubsubClientTopicState {
        fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
            0
        }
    }
}
