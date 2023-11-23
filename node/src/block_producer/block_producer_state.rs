use serde::{Deserialize, Serialize};

use super::vrf_evaluator::BlockProducerVrfEvaluatorState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerState {
    pub vrf_evaluator: BlockProducerVrfEvaluatorState,
}

impl BlockProducerState {
    pub fn new(now: redux::Timestamp) -> Self {
        Self {
            vrf_evaluator: BlockProducerVrfEvaluatorState::Idle { time: now },
        }
    }
}
