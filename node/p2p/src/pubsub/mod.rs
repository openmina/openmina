use serde::{Deserialize, Serialize};
use std::fmt;

mod p2p_pubsub_actions;
pub use p2p_pubsub_actions::*;

mod p2p_pubsub_effects;
pub use p2p_pubsub_effects::*;

mod p2p_pubsub_service;
pub use p2p_pubsub_service::*;

pub use mina_p2p_messages::GossipNetMessageV1;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PubsubTopic {
    CodaConsensusMessage,
    MinaBlock,
    MinaTx,
    MinaSnarkWork,
    Other(String),
}

impl fmt::Display for PubsubTopic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::CodaConsensusMessage => "coda/consensus-messages/0.0.1",
            Self::MinaBlock => "mina/block/1.0.0",
            Self::MinaTx => "mina/tx/1.0.0",
            Self::MinaSnarkWork => "mina/snark-work/1.0.0",
            Self::Other(s) => s.as_str(),
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for PubsubTopic {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "coda/consensus-messages/0.0.1" => Self::CodaConsensusMessage,
            "mina/block/1.0.0" => Self::MinaBlock,
            "mina/tx/1.0.0" => Self::MinaTx,
            "mina/snark-work/1.0.0" => Self::MinaSnarkWork,
            s => Self::Other(s.to_string()),
        })
    }
}

impl From<PubsubTopic> for libp2p::gossipsub::TopicHash {
    fn from(value: PubsubTopic) -> Self {
        Self::from_raw(value.to_string())
    }
}
