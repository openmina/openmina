use std::sync::Arc;

use crate::account::AccountPublicKey;
use crate::block_producer::vrf_evaluator::BlockProducerVrfEvaluatorStatus;
use mina_p2p_messages::v2::{
    ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1, LedgerHash,
};
use serde::{Deserialize, Serialize};
use vrf::VrfEvaluationOutput;

use super::{DelegatorTable, VrfEvaluatorInput};

pub type BlockProducerVrfEvaluatorActionWithMeta =
    redux::ActionWithMeta<BlockProducerVrfEvaluatorAction>;
pub type BlockProducerVrfEvaluatorActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a BlockProducerVrfEvaluatorAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerVrfEvaluatorAction {
    EpochDataUpdate {
        new_epoch_number: u32,
        epoch_data: ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
        next_epoch_data: ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    },
    EvaluateVrf {
        vrf_input: VrfEvaluatorInput,
    },
    EvaluationSuccess {
        vrf_output: VrfEvaluationOutput,
        staking_ledger_hash: LedgerHash,
    },
    UpdateProducerAndDelegates {
        current_epoch_ledger_hash: LedgerHash,
        next_epoch_ledger_hash: LedgerHash,
        producer: AccountPublicKey,
    },
    UpdateProducerAndDelegatesSuccess {
        current_epoch_producer_and_delegators: Arc<DelegatorTable>,
        next_epoch_producer_and_delegators: Arc<DelegatorTable>,
        staking_ledger_hash: LedgerHash,
    },
}

impl redux::EnablingCondition<crate::State> for BlockProducerVrfEvaluatorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        match self {
            BlockProducerVrfEvaluatorAction::UpdateProducerAndDelegates { .. } => {
                state.block_producer.with(false, |this| {
                    matches!(
                        this.vrf_evaluator.status,
                        BlockProducerVrfEvaluatorStatus::EpochChanged { .. }
                    )
                })
            }
            BlockProducerVrfEvaluatorAction::UpdateProducerAndDelegatesSuccess {
                staking_ledger_hash,
                ..
            } => state.block_producer.with(false, |this| {
                matches!(
                    this.vrf_evaluator.status,
                    BlockProducerVrfEvaluatorStatus::DataPending { .. }
                ) && this
                    .vrf_evaluator
                    .current_epoch_data
                    .as_ref()
                    .is_some_and(|epoch_data| &epoch_data.ledger == staking_ledger_hash)
            }),
            BlockProducerVrfEvaluatorAction::EvaluateVrf { .. } => {
                state.block_producer.with(false, |this| {
                    matches!(
                        this.vrf_evaluator.status,
                        BlockProducerVrfEvaluatorStatus::SlotsReceived { .. }
                            | BlockProducerVrfEvaluatorStatus::DataSuccess { .. }
                    )
                })
            }
            BlockProducerVrfEvaluatorAction::EvaluationSuccess {
                vrf_output,
                staking_ledger_hash,
            } => state.block_producer.with(false, |this| {
                this.vrf_evaluator
                    .status
                    .matches_requested_slot(vrf_output.global_slot(), staking_ledger_hash)
            }),
            BlockProducerVrfEvaluatorAction::EpochDataUpdate { .. } => true,
        }
    }
}

impl From<BlockProducerVrfEvaluatorAction> for crate::Action {
    fn from(value: BlockProducerVrfEvaluatorAction) -> Self {
        Self::BlockProducer(value.into())
    }
}
