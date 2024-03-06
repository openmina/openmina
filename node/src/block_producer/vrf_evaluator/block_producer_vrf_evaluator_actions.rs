use crate::account::AccountPublicKey;
use crate::block_producer::vrf_evaluator::BlockProducerVrfEvaluatorStatus;
use crate::block_producer::vrf_evaluator::EpochContext;
use mina_p2p_messages::v2::{
    ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1, LedgerHash,
};
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};
use vrf::VrfEvaluationOutput;

use super::{EpochData, VrfEvaluatorInput};

pub type BlockProducerVrfEvaluatorActionWithMeta =
    redux::ActionWithMeta<BlockProducerVrfEvaluatorAction>;
pub type BlockProducerVrfEvaluatorActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a BlockProducerVrfEvaluatorAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerVrfEvaluatorAction {
    EvaluateSlot {
        vrf_input: VrfEvaluatorInput,
    },
    ProcessSlotEvaluationSuccess {
        vrf_output: VrfEvaluationOutput,
        staking_ledger_hash: LedgerHash,
    },
    InitializeEvaluator {
        best_tip: ArcBlockWithHash,
    },
    FinalizeEvaluatorInitialization {
        previous_epoch_and_height: Option<(u32, u32)>,
    },
    CheckEpochEvaluability {
        current_epoch_number: u32,
        current_best_tip_height: u32,
        current_best_tip_slot: u32,
        current_best_tip_global_slot: u32,
        next_epoch_first_slot: u32,
        staking_epoch_data: ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
        next_epoch_data: ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
        transition_frontier_size: u32,
    },
    InitializeEpochEvaluation {
        current_epoch_number: u32,
        current_best_tip_slot: u32,
        current_best_tip_height: u32,
        current_best_tip_global_slot: u32,
        next_epoch_first_slot: u32,
        staking_epoch_data: EpochData,
        producer: AccountPublicKey,
        transition_frontier_size: u32,
    },
    BeginDelegatorTableConstruction {
        staking_epoch_data: EpochData,
        producer: AccountPublicKey,
        current_epoch_number: u32,
        current_best_tip_height: u32,
        current_best_tip_slot: u32,
        current_best_tip_global_slot: u32,
        next_epoch_first_slot: u32,
        transition_frontier_size: u32,
    },
    FinalizeDelegatorTableConstruction {
        staking_epoch_data: EpochData,
        producer: AccountPublicKey,
        current_epoch_number: u32,
        current_best_tip_height: u32,
        current_best_tip_slot: u32,
        current_best_tip_global_slot: u32,
        next_epoch_first_slot: u32,
        transition_frontier_size: u32,
    },
    SelectInitialSlot {
        current_global_slot: u32,
        current_best_tip_height: u32,
        current_best_tip_slot: u32,
        current_best_tip_global_slot: u32,
        current_epoch_number: u32,
        staking_epoch_data: EpochData,
        next_epoch_first_slot: u32,
    },
    BeginEpochEvaluation {
        current_best_tip_height: u32,
        current_best_tip_slot: u32,
        current_best_tip_global_slot: u32,
        current_epoch_number: u32,
        staking_epoch_data: EpochData,
        latest_evaluated_global_slot: u32,
    },
    RecordLastBlockHeightInEpoch {
        epoch_number: u32,
        last_block_height: u32,
    },
    ContinueEpochEvaluation {
        latest_evaluated_global_slot: u32,
        epoch_number: u32,
    },
    FinishEpochEvaluation {
        epoch_number: u32,
        last_evaluated_global_slot: u32,
    },
    WaitForNextEvaluation {
        current_epoch_number: u32,
        current_best_tip_height: u32,
        current_best_tip_slot: u32,
        current_best_tip_global_slot: u32,
        last_epoch_block_height: Option<u32>,
        transition_frontier_size: u32,
    },
}

impl redux::EnablingCondition<crate::State> for BlockProducerVrfEvaluatorAction {
    fn is_enabled(&self, state: &crate::State, _time: redux::Timestamp) -> bool {
        match self {
            BlockProducerVrfEvaluatorAction::EvaluateSlot { .. } => state
                .block_producer
                .with(false, |this| this.vrf_evaluator.is_evaluating()),
            BlockProducerVrfEvaluatorAction::ProcessSlotEvaluationSuccess {
                vrf_output,
                staking_ledger_hash,
                ..
            } => state.block_producer.with(false, |this| {
                if let Some(current_evaluation) = this.vrf_evaluator.current_evaluation() {
                    current_evaluation.latest_evaluated_slot + 1 == vrf_output.global_slot()
                        && current_evaluation.epoch_data.ledger == *staking_ledger_hash
                } else {
                    false
                }
            }),
            BlockProducerVrfEvaluatorAction::InitializeEvaluator { .. } => state
                .block_producer
                .with(false, |this| !this.vrf_evaluator.status.is_initialized()),
            BlockProducerVrfEvaluatorAction::FinalizeEvaluatorInitialization { .. } => {
                state.block_producer.with(false, |this| {
                    matches!(
                        this.vrf_evaluator.status,
                        BlockProducerVrfEvaluatorStatus::InitialisationPending { .. }
                    )
                })
            }
            BlockProducerVrfEvaluatorAction::CheckEpochEvaluability { .. } => state
                .block_producer
                .with(false, |this| this.vrf_evaluator.can_check_next_evaluation()),
            BlockProducerVrfEvaluatorAction::InitializeEpochEvaluation { .. } => {
                state.block_producer.with(false, |this| {
                    this.vrf_evaluator.is_readiness_check()
                        && matches!(
                            this.vrf_evaluator.epoch_context(),
                            EpochContext::Current | EpochContext::Next
                        )
                    // && !(this.vrf_evaluator.status.is_current_epoch_evaluated(last_evaluated_epoch) || this.vrf_evaluator.status.is_next_epoch_evaluated(last_evaluated_epoch))
                })
            }
            BlockProducerVrfEvaluatorAction::BeginDelegatorTableConstruction { .. } => {
                state.block_producer.with(false, |this| {
                    this.vrf_evaluator.can_construct_delegator_table()
                })
            }
            BlockProducerVrfEvaluatorAction::FinalizeDelegatorTableConstruction { .. } => {
                state.block_producer.with(false, |this| {
                    this.vrf_evaluator.is_delegator_table_requested()
                })
            }
            BlockProducerVrfEvaluatorAction::BeginEpochEvaluation { .. } => state
                .block_producer
                .with(false, |this| this.vrf_evaluator.is_slot_selection()),
            BlockProducerVrfEvaluatorAction::RecordLastBlockHeightInEpoch { .. } => {
                state.block_producer.vrf_evaluator().is_some()
            }
            BlockProducerVrfEvaluatorAction::ContinueEpochEvaluation { .. } => {
                state.block_producer.with(false, |this| {
                    this.vrf_evaluator.is_waiting_for_slot_evaluation()
                })
            }
            BlockProducerVrfEvaluatorAction::FinishEpochEvaluation { .. } => {
                state.block_producer.with(false, |this| {
                    this.vrf_evaluator.is_waiting_for_slot_evaluation()
                })
            }
            BlockProducerVrfEvaluatorAction::WaitForNextEvaluation { .. } => state
                .block_producer
                .with(false, |this| this.vrf_evaluator.is_readiness_check()),
            BlockProducerVrfEvaluatorAction::SelectInitialSlot { .. } => {
                state.block_producer.with(false, |this| {
                    this.vrf_evaluator.is_delegator_table_constructed()
                })
            }
        }
    }
}

impl From<BlockProducerVrfEvaluatorAction> for crate::Action {
    fn from(value: BlockProducerVrfEvaluatorAction) -> Self {
        Self::BlockProducer(crate::BlockProducerAction::VrfEvaluator(value))
    }
}
