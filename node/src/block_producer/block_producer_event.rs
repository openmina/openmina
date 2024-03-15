use mina_p2p_messages::v2::{MinaBaseProofStableV2, StateHash};
use serde::{Deserialize, Serialize};

pub use super::vrf_evaluator::BlockProducerVrfEvaluatorEvent;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerEvent {
    VrfEvaluator(BlockProducerVrfEvaluatorEvent),
    BlockProve(StateHash, Result<Box<MinaBaseProofStableV2>, String>),
}

impl std::fmt::Display for BlockProducerEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockProducer, ")?;
        match self {
            Self::VrfEvaluator(e) => e.fmt(f),
            Self::BlockProve(block_hash, res) => {
                let res = res.as_ref().map_or("Err", |_| "Ok");
                write!(f, "BlockProveSuccess, {block_hash}, {res}")
            }
        }
    }
}
