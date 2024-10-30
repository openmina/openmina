use super::vrf_evaluator_effectful::BlockProducerVrfEvaluatorEffectfulAction;
use crate::block_producer::{BlockProducerWonSlot, BlockProducerWonSlotDiscardReason};
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum BlockProducerEffectfulAction {
    VrfEvaluator(BlockProducerVrfEvaluatorEffectfulAction),
    WonSlot {
        won_slot: BlockProducerWonSlot,
    },
    WonSlotDiscard {
        reason: BlockProducerWonSlotDiscardReason,
    },
    StagedLedgerDiffCreateInit,
    StagedLedgerDiffCreateSuccess,
    BlockUnprovenBuild,
    BlockProveInit,
    BlockProveSuccess,
}

impl redux::EnablingCondition<crate::State> for BlockProducerEffectfulAction {
    fn is_enabled(&self, _state: &crate::State, _time: redux::Timestamp) -> bool {
        true
    }
}
