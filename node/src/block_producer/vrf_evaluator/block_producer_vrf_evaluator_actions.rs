use crate::account::AccountPublicKey;
use crate::block_producer::vrf_evaluator::BlockProducerVrfEvaluatorStatus;
use crate::block_producer::vrf_evaluator::EpochContext;
use mina_p2p_messages::v2::{
    ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1, LedgerHash,
};
use openmina_core::action_info;
use openmina_core::action_trace;
use openmina_core::block::ArcBlockWithHash;
use openmina_core::log::ActionEvent;
use serde::{Deserialize, Serialize};
use vrf::VrfEvaluationOutput;
use vrf::VrfWonSlot;

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
        latest_evaluated_global_slot: u32,
    },
    WaitForNextEvaluation {
        current_epoch_number: u32,
        current_best_tip_height: u32,
        current_best_tip_slot: u32,
        current_best_tip_global_slot: u32,
        last_epoch_block_height: Option<u32>,
        transition_frontier_size: u32,
    },
    CheckEpochBounds {
        epoch_number: u32,
        latest_evaluated_global_slot: u32,
    },
    CleanupOldSlots {
        current_epoch_number: u32,
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
                if this.vrf_evaluator.is_slot_requested() {
                    if let Some(current_evaluation) = this.vrf_evaluator.current_evaluation() {
                        current_evaluation.latest_evaluated_slot + 1 == vrf_output.global_slot()
                            && current_evaluation.epoch_data.ledger == *staking_ledger_hash
                    } else {
                        false
                    }
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
                            EpochContext::Current(_) | EpochContext::Next(_)
                        )
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
                    this.vrf_evaluator.is_epoch_bound_evaluated()
                        || this.vrf_evaluator.is_evaluation_pending()
                })
            }
            BlockProducerVrfEvaluatorAction::FinishEpochEvaluation { .. } => state
                .block_producer
                .with(false, |this| this.vrf_evaluator.is_epoch_bound_evaluated()),
            BlockProducerVrfEvaluatorAction::WaitForNextEvaluation { .. } => state
                .block_producer
                .with(false, |this| this.vrf_evaluator.is_readiness_check()),
            BlockProducerVrfEvaluatorAction::SelectInitialSlot { .. } => {
                state.block_producer.with(false, |this| {
                    this.vrf_evaluator.is_delegator_table_constructed()
                })
            }
            BlockProducerVrfEvaluatorAction::CheckEpochBounds { .. } => state
                .block_producer
                .with(false, |this| this.vrf_evaluator.is_slot_evaluated()),
            BlockProducerVrfEvaluatorAction::CleanupOldSlots {
                current_epoch_number,
            } => state.block_producer.with(false, |this| {
                let retention_slot = this.vrf_evaluator.retention_slot(current_epoch_number);
                if let Some((first_won_slot, _)) = this.vrf_evaluator.won_slots.first_key_value() {
                    *first_won_slot < retention_slot
                } else {
                    false
                }
            }),
        }
    }
}

impl From<BlockProducerVrfEvaluatorAction> for crate::Action {
    fn from(value: BlockProducerVrfEvaluatorAction) -> Self {
        Self::BlockProducer(crate::BlockProducerAction::VrfEvaluator(value))
    }
}

impl ActionEvent for BlockProducerVrfEvaluatorAction {
    fn action_event<T>(&self, context: &T)
    where
        T: openmina_core::log::EventContext,
    {
        match self {
            BlockProducerVrfEvaluatorAction::EvaluateSlot { vrf_input } => action_info!(
                context,
                summary = "Vrf Evaluation requested",
                input = debug(vrf_input)
            ),
            BlockProducerVrfEvaluatorAction::ProcessSlotEvaluationSuccess {
                vrf_output, ..
            } => match vrf_output {
                VrfEvaluationOutput::SlotWon(VrfWonSlot {
                    global_slot,
                    vrf_output,
                    ..
                }) => action_info!(
                    context,
                    summary = "Slot evaluation result - won slot",
                    global_slot,
                    vrf_output = display(vrf_output)
                ),
                VrfEvaluationOutput::SlotLost(_) => {
                    action_info!(context, summary = "Slot evaluation result - lost slot")
                }
            },
            BlockProducerVrfEvaluatorAction::InitializeEvaluator { .. } => {
                action_trace!(context)
            }
            BlockProducerVrfEvaluatorAction::FinalizeEvaluatorInitialization { .. } => {
                action_info!(context, summary = "Vrf evaluator initilaized")
            }
            BlockProducerVrfEvaluatorAction::CheckEpochEvaluability {
                current_epoch_number,
                current_best_tip_slot,
                current_best_tip_global_slot,
                ..
            } => action_info!(
                context,
                summary = "Checking possible Vrf evaluations",
                current_epoch_number,
                current_best_tip_slot,
                current_best_tip_global_slot
            ),
            BlockProducerVrfEvaluatorAction::InitializeEpochEvaluation {
                current_epoch_number,
                current_best_tip_slot,
                current_best_tip_global_slot,
                ..
            } => action_info!(
                context,
                summary = "Constructing delegator table", /* TODO: check the name*/
                current_epoch_number,
                current_best_tip_slot,
                current_best_tip_global_slot
            ),
            BlockProducerVrfEvaluatorAction::BeginDelegatorTableConstruction {
                current_epoch_number,
                current_best_tip_slot,
                current_best_tip_global_slot,
                ..
            } => action_info!(
                context,
                summary = "Constructing delegator table",
                current_epoch_number,
                current_best_tip_slot,
                current_best_tip_global_slot
            ),
            BlockProducerVrfEvaluatorAction::FinalizeDelegatorTableConstruction {
                current_epoch_number,
                current_best_tip_slot,
                current_best_tip_global_slot,
                ..
            } => action_info!(
                context,
                summary = "Delegator table constructed",
                current_epoch_number,
                current_best_tip_slot,
                current_best_tip_global_slot
            ),
            BlockProducerVrfEvaluatorAction::SelectInitialSlot {
                current_global_slot,
                current_best_tip_height,
                ..
            } => action_info!(
                context,
                summary = "Selecting starting slot",
                current_global_slot,
                current_best_tip_height
            ),
            BlockProducerVrfEvaluatorAction::BeginEpochEvaluation {
                current_epoch_number,
                current_best_tip_slot,
                current_best_tip_global_slot,
                ..
            } => action_info!(
                context,
                summary = "Starting epoch evaluation",
                current_epoch_number,
                current_best_tip_slot,
                current_best_tip_global_slot
            ),
            BlockProducerVrfEvaluatorAction::RecordLastBlockHeightInEpoch {
                epoch_number,
                last_block_height,
            } => action_info!(
                context,
                summary = "Saving last block height in epoch",
                epoch_number,
                last_block_height
            ),
            BlockProducerVrfEvaluatorAction::ContinueEpochEvaluation { .. } => {
                action_trace!(context)
            }
            BlockProducerVrfEvaluatorAction::FinishEpochEvaluation {
                epoch_number,
                latest_evaluated_global_slot,
            } => action_info!(
                context,
                summary = "Epoch evaluation finished",
                epoch_number,
                latest_evaluated_global_slot
            ),
            BlockProducerVrfEvaluatorAction::WaitForNextEvaluation {
                current_epoch_number,
                current_best_tip_height,
                ..
            } => action_info!(
                context,
                summary = "Waiting for epoch to evaluate",
                current_epoch_number,
                current_best_tip_height
            ),
            BlockProducerVrfEvaluatorAction::CheckEpochBounds { .. } => {
                action_trace!(context, summary = "Checking epoch bounds")
            }
            BlockProducerVrfEvaluatorAction::CleanupOldSlots { .. } => {
                action_trace!(context, summary = "Cleaning up old won slots")
            }
        }
    }
}
