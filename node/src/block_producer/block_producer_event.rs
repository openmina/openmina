//! Block producer event module.
//! Defines the events that can be emitted by the block producer module.
//! These events represent the results of asynchronous operations.

use std::sync::Arc;

use mina_p2p_messages::v2::{MinaBaseProofStableV2, StateHash};
use serde::{Deserialize, Serialize};

pub use super::vrf_evaluator::BlockProducerVrfEvaluatorEvent;

/// Events emitted by the block producer module.
/// These events typically represent the results of asynchronous operations.
#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerEvent {
    VrfEvaluator(BlockProducerVrfEvaluatorEvent),
    BlockProve(StateHash, Result<Arc<MinaBaseProofStableV2>, String>),
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
