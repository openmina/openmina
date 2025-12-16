pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/gossipsub.rs"));
}
pub use mina_core::p2p::P2pNetworkPubsubMessageCacheId;
mod p2p_network_pubsub_actions;
pub use self::p2p_network_pubsub_actions::P2pNetworkPubsubAction;

mod p2p_network_pubsub_state;
pub use self::p2p_network_pubsub_state::{
    P2pNetworkPubsubClientState, P2pNetworkPubsubClientTopicState, P2pNetworkPubsubState,
};

#[cfg(feature = "p2p-libp2p")]
mod p2p_network_pubsub_reducer;

#[cfg(feature = "p2p-libp2p")]
const TOPIC: &str = "coda/consensus-messages/0.0.1";

pub mod pubsub_effectful;
use mina_core::snark::SnarkJobId;
pub use pubsub_effectful::P2pNetworkPubsubEffectfulAction;

use binprot::BinProtWrite;
use mina_core::bug_condition;
use mina_p2p_messages::gossip::GossipNetMessageV2;
use sha2::{Digest, Sha256};

use crate::identity::SecretKey;

#[derive(serde::Serialize, serde:: Deserialize, Debug, Clone)]
pub enum BroadcastMessageId {
    BlockHash {
        hash: mina_p2p_messages::v2::StateHash,
    },
    Snark {
        job_id: SnarkJobId,
    },
    MessageId {
        message_id: P2pNetworkPubsubMessageCacheId,
    },
}

pub(super) fn webrtc_source_sk(message: &GossipNetMessageV2) -> SecretKey {
    let mut hasher = Sha256::new();
    if let Err(err) = message.binprot_write(&mut hasher) {
        bug_condition!("trying to broadcast message which can't be binprot serialized! err: {err}");
        return SecretKey::from_bytes([0; 32]);
    }
    SecretKey::from_bytes(hasher.finalize().into())
}

pub(super) fn webrtc_source_sk_from_bytes(bytes: &[u8]) -> SecretKey {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    SecretKey::from_bytes(hasher.finalize().into())
}

pub(super) fn encode_message(message: &GossipNetMessageV2) -> std::io::Result<Vec<u8>> {
    let mut buffer = vec![0; 8];

    message.binprot_write(&mut buffer)?;

    let len = buffer.len() - 8;
    buffer[..8].clone_from_slice(&(len as u64).to_le_bytes());

    Ok(buffer)
}
