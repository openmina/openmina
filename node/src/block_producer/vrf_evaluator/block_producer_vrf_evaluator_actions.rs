use crate::account::AccountPublicKey;
use crate::block_producer::vrf_evaluator::EpochContext;
use crate::block_producer::vrf_evaluator::BlockProducerVrfEvaluatorStatus;
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

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerVrfEvaluatorAction {
    EpochDataUpdate(BlockProducerVrfEvaluatorEpochDataUpdateAction),
    EvaluateVrf(BlockProducerVrfEvaluatorEvaluateVrfAction),
    EvaluationSuccess(BlockProducerVrfEvaluatorEvaluationSuccessAction),
    UpdateProducerAndDelegates(BlockProducerVrfEvaluatorUpdateProducerAndDelegatesAction),
    UpdateProducerAndDelegatesSuccess(
        BlockProducerVrfEvaluatorUpdateProducerAndDelegatesSuccessAction,
    ),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerVrfEvaluatorAction {
    EvaluateVrf {
        vrf_input: VrfEvaluatorInput,
    },
    EvaluationSuccess {
        vrf_output: VrfEvaluationOutput,
        staking_ledger_hash: LedgerHash,
    },
    EvaluatorInit {
        best_tip: ArcBlockWithHash,
    },
    EvaluatorInitSuccess {
        previous_epoch_and_height: Option<(u32, u32)>,
    },
    CanEvaluateVrf {
        current_epoch_number: u32,
        current_best_tip_height: u32,
        current_best_tip_slot: u32,
        current_best_tip_global_slot: u32,
        next_epoch_first_slot: u32,
        staking_epoch_data: ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
        next_epoch_data: ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
        transition_frontier_size: u32,
    },
    EvaluateEpochInit {
        epoch_context: EpochContext,
        current_epoch_number: u32,
        current_best_tip_slot: u32,
        current_best_tip_height: u32,
        current_best_tip_global_slot: u32,
        next_epoch_first_slot: u32,
        staking_epoch_data: EpochData,
        producer: AccountPublicKey,
        transition_frontier_size: u32,
    },
    ConstructDelegatorTable {
        epoch_context: EpochContext,
        staking_epoch_data: EpochData,
        producer: AccountPublicKey,
        current_epoch_number: u32,
        current_best_tip_height: u32,
        current_best_tip_slot: u32,
        current_best_tip_global_slot: u32,
        next_epoch_first_slot: u32,
        transition_frontier_size: u32,
    },
    ConstructDelegatorTableSuccess {
        epoch_context: EpochContext,
        staking_epoch_data: EpochData,
        producer: AccountPublicKey,
        current_epoch_number: u32,
        current_best_tip_height: u32,
        current_best_tip_slot: u32,
        current_best_tip_global_slot: u32,
        next_epoch_first_slot: u32,
        transition_frontier_size: u32,
    },
    EvaluateEpoch {
        epoch_context: EpochContext,
        current_best_tip_height: u32,
        current_best_tip_slot: u32,
        current_best_tip_global_slot: u32,
        current_epoch_number: u32,
        staking_epoch_data: EpochData,
        latest_evaluated_global_slot: u32,
    },
    SaveLastBlockHeightInEpoch {
        epoch_number: u32,
        last_block_height: u32,
    },
}

impl redux::EnablingCondition<crate::State> for BlockProducerVrfEvaluatorAction {
    fn is_enabled(&self, state: &crate::State, _time: redux::Timestamp) -> bool {
        match self {
            BlockProducerVrfEvaluatorAction::EvaluateVrf { .. } => {
                state
                    .block_producer
                    .with(false, |this| this.vrf_evaluator.status.is_evaluating())
            }
            BlockProducerVrfEvaluatorAction::EvaluationSuccess {
                vrf_output,
                staking_ledger_hash,
            } => state.block_producer.with(false, |this| {
                    if let Some(current_evaluation) = this.vrf_evaluator.status.current_evaluation() {
                        current_evaluation.latest_evaluated_slot + 1 == self.vrf_output.global_slot()
                            && current_evaluation.epoch_data.ledger == self.staking_ledger_hash
                    } else {
                        false
                    }
            })
            BlockProducerVrfEvaluatorAction::EvaluatorInit { .. } => {
                state
                    .block_producer
                    .with(false, |this| !this.vrf_evaluator.status.is_initialized())
            }
            BlockProducerVrfEvaluatorAction::EvaluatorInitSuccess { .. } => {
                state.block_producer.with(false, |this| {
                    matches!(
                        this.vrf_evaluator.status,
                        BlockProducerVrfEvaluatorStatus::EvaluatorInitialisationPending { .. }
                    )
                })
            }
            BlockProducerVrfEvaluatorAction::CanEvaluateVrf { .. } => {
                state.block_producer.with(false, |this| {
                    let last_evaluated_epoch = this.vrf_evaluator.last_evaluated_epoch();
                    this.vrf_evaluator
                        .status
                        .is_evaluator_ready(last_evaluated_epoch)
                })
            }
            BlockProducerVrfEvaluatorAction::EvaluateEpochInit { .. } => {
                state.block_producer.with(false, |this| {
                    matches!(
                        this.vrf_evaluator.status.epoch_context(),
                        EpochContext::Waiting
                    )
                })
            }
            BlockProducerVrfEvaluatorAction::ConstructDelegatorTable { .. } => {
                state.block_producer.with(false, |this| {
                    this.vrf_evaluator.status.can_construct_delegator_table()
                })
            }
            BlockProducerVrfEvaluatorAction::ConstructDelegatorTableSuccess { .. } => {
                state.block_producer.with(false, |this| {
                    this.vrf_evaluator.status.is_delegator_table_requested()
                })
            }
            BlockProducerVrfEvaluatorAction::EvaluateEpoch { .. } => {
                state.block_producer.with(false, |this| {
                    this.vrf_evaluator
                        .status
                        .can_start_current_epoch_evaluation()
                        || this.vrf_evaluator.status.can_start_next_epoch_evaluation()
                })
            }
            BlockProducerVrfEvaluatorAction::SaveLastBlockHeightInEpoch { .. } => {
                // TODO(adonagy): any restrictions here?
                true
            }
        }
    }
}

impl From<BlockProducerVrfEvaluatorAction> for crate::Action {
    fn from(value: BlockProducerVrfEvaluatorAction) -> Self {
        Self::BlockProducer(crate::BlockProducerAction::VrfEvaluator(value.into()))
    }
}
