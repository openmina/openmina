use crate::block_producer::vrf_evaluator::VrfEvaluatorInput;
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum BlockProducerVrfEvaluatorEffectfulAction {
    EvaluateSlot { vrf_input: VrfEvaluatorInput },
}

impl redux::EnablingCondition<crate::State> for BlockProducerVrfEvaluatorEffectfulAction {
    fn is_enabled(&self, _state: &crate::State, _time: redux::Timestamp) -> bool {
        true
    }
}

impl From<BlockProducerVrfEvaluatorEffectfulAction> for crate::Action {
    fn from(value: BlockProducerVrfEvaluatorEffectfulAction) -> Self {
        Self::BlockProducerEffectful(crate::BlockProducerEffectfulAction::VrfEvaluator(value))
    }
}
