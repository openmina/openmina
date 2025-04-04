//! Defines actions related to VRF evaluation in the block production process.
//! These actions handle the evaluation of slots to determine block production eligibility.

use crate::block_producer::vrf_evaluator::VrfEvaluatorInput;
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
/// Actions related to VRF evaluation for block production.
/// These actions trigger the evaluation of slots and track evaluation statistics.
pub enum BlockProducerVrfEvaluatorEffectfulAction {
    EvaluateSlot { vrf_input: VrfEvaluatorInput },
    SlotEvaluated { epoch: u32 },
    InitializeStats { epoch: u32, initial_slot: u32 },
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
