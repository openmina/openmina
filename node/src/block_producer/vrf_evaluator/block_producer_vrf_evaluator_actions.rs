use std::collections::BTreeMap;

use crate::account::AccountPublicKey;
use crate::block_producer::{vrf_evaluator::BlockProducerVrfEvaluatorStatus, BlockProducerAction};
use ledger::AccountIndex;
use mina_p2p_messages::v2::{
    ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1, LedgerHash,
};
use serde::{Deserialize, Serialize};
use vrf::VrfEvaluationOutput;

use super::VrfEvaluatorInput;

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
pub struct BlockProducerVrfEvaluatorUpdateProducerAndDelegatesAction {
    pub current_epoch_ledger_hash: LedgerHash,
    pub next_epoch_ledger_hash: LedgerHash,
    pub producer: String,
}

impl redux::EnablingCondition<crate::State>
    for BlockProducerVrfEvaluatorUpdateProducerAndDelegatesAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.vrf_evaluator.status,
                BlockProducerVrfEvaluatorStatus::EpochChanged { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerVrfEvaluatorUpdateProducerAndDelegatesSuccessAction {
    pub current_epoch_producer_and_delegators: BTreeMap<AccountIndex, (AccountPublicKey, u64)>,
    pub next_epoch_producer_and_delegators: BTreeMap<AccountIndex, (AccountPublicKey, u64)>,
}

impl redux::EnablingCondition<crate::State>
    for BlockProducerVrfEvaluatorUpdateProducerAndDelegatesSuccessAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.vrf_evaluator.status,
                BlockProducerVrfEvaluatorStatus::DataPending { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerVrfEvaluatorEvaluateVrfAction {
    pub vrf_input: VrfEvaluatorInput,
}

impl redux::EnablingCondition<crate::State> for BlockProducerVrfEvaluatorEvaluateVrfAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.vrf_evaluator.status,
                BlockProducerVrfEvaluatorStatus::SlotsReceived { .. }
                    | BlockProducerVrfEvaluatorStatus::DataSuccess { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerVrfEvaluatorEvaluationSuccessAction {
    pub vrf_output: VrfEvaluationOutput,
}

impl redux::EnablingCondition<crate::State> for BlockProducerVrfEvaluatorEvaluationSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.vrf_evaluator.status,
                BlockProducerVrfEvaluatorStatus::SlotsRequested { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerVrfEvaluatorEpochDataUpdateAction {
    pub new_epoch_number: u32,
    pub epoch_data: ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
    pub next_epoch_data: ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
}

impl redux::EnablingCondition<crate::State> for BlockProducerVrfEvaluatorEpochDataUpdateAction {
    fn is_enabled(&self, _: &crate::State) -> bool {
        true
    }
}

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::BlockProducer(BlockProducerAction::VrfEvaluator(value.into()))
            }
        }
    };
}

impl_into_global_action!(BlockProducerVrfEvaluatorEpochDataUpdateAction);
impl_into_global_action!(BlockProducerVrfEvaluatorEvaluateVrfAction);
impl_into_global_action!(BlockProducerVrfEvaluatorEvaluationSuccessAction);
impl_into_global_action!(BlockProducerVrfEvaluatorUpdateProducerAndDelegatesAction);
impl_into_global_action!(BlockProducerVrfEvaluatorUpdateProducerAndDelegatesSuccessAction);
