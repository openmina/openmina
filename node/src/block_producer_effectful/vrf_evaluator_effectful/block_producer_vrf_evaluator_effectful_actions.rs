use crate::block_producer::vrf_evaluator::VrfEvaluatorInput;
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum BlockProducerVrfEvaluatorEffectfulAction {
    EvaluateSlotsBatch {
        vrf_input: VrfEvaluatorInput,
        start_slot: u32,
        batch_size: u32,
    },
    SlotEvaluated {
        epoch: u32,
    },
    InitializeStats {
        epoch: u32,
        initial_slot: u32,
    },
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
