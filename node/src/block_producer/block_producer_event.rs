use serde::{Deserialize, Serialize};

pub use super::vrf_evaluator::BlockProducerVrfEvaluatorEvent;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerEvent {
    VrfEvaluator(BlockProducerVrfEvaluatorEvent),
}

impl std::fmt::Display for BlockProducerEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockProducer, ")?;
        match self {
            Self::VrfEvaluator(e) => e.fmt(f),
        }
    }
}
