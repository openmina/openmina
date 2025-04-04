//! Defines actions that trigger side effects in the block production process.
//! These actions represent events that require interaction with external services.

use super::vrf_evaluator_effectful::BlockProducerVrfEvaluatorEffectfulAction;
use crate::block_producer::{BlockProducerWonSlot, BlockProducerWonSlotDiscardReason};
use openmina_core::{block::ArcBlockWithHash, ActionEvent};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
/// Actions that represent side effects in the block production process.
/// These actions are dispatched to trigger external service interactions
/// or to record the results of those interactions.
// FACT-CHECKER-WARNING: The action enum is missing actions for block injection to the transition frontier and P2P network, which are critical parts of the block production process shown in the block_producer module.
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
    BlockProduced {
        block: ArcBlockWithHash,
    },
}

impl redux::EnablingCondition<crate::State> for BlockProducerEffectfulAction {
    fn is_enabled(&self, _state: &crate::State, _time: redux::Timestamp) -> bool {
        true
    }
}
