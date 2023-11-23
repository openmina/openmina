use std::collections::BTreeMap;

use ledger::AccountIndex;
use mina_p2p_messages::v2::{ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1, LedgerHash, ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1};
use mina_signer::Keypair;
use serde::{Deserialize, Serialize};
use vrf::{VrfEvaluatorInput, VrfEvaluationOutput};
use crate::block_producer::{BlockProducerAction, vrf_evaluator::{BlockProducerVrfEvaluatorState, BlockProducerVrfEvaluatorStatus}};

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
    UpdateProducerAndDelegatesSuccess(BlockProducerVrfEvaluatorUpdateProducerAndDelegatesSuccessAction),
    NewEpoch(BlockProducerVrfEvaluatorNewEpochAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerVrfEvaluatorNewEpochAction {
    pub new_epoch_number: u32,
    pub epoch_data: ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
    pub next_epoch_data: ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
}

impl redux::EnablingCondition<crate::State> for BlockProducerVrfEvaluatorNewEpochAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        true
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerVrfEvaluatorUpdateProducerAndDelegatesAction {
    pub current_epoch_ledger_hash: LedgerHash,
    pub next_epoch_ledger_hash: LedgerHash,
    pub producer: String,
}

impl redux::EnablingCondition<crate::State> for BlockProducerVrfEvaluatorUpdateProducerAndDelegatesAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerVrfEvaluatorUpdateProducerAndDelegatesSuccessAction {
    pub current_epoch_producer_and_delegators: BTreeMap<AccountIndex, (String, u64)>,
    pub next_epoch_producer_and_delegators: BTreeMap<AccountIndex, (String, u64)>,
}

impl redux::EnablingCondition<crate::State> for BlockProducerVrfEvaluatorUpdateProducerAndDelegatesSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerVrfEvaluatorEvaluateVrfAction {
    pub vrf_input: VrfEvaluatorInput,
}

impl redux::EnablingCondition<crate::State> for BlockProducerVrfEvaluatorEvaluateVrfAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        // TODO(adonagy): global_slot in the input should be greater that the current global_slot in the state
        true
    }
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct BlockProducerVrfEvaluatorEvaluationPendingAction {
//     pub global_slot: u32,
// }

// impl redux::EnablingCondition<crate::State> for BlockProducerVrfEvaluatorEvaluationPendingAction {
//     fn is_enabled(&self, state: &crate::State) -> bool {
//         !matches!(state.block_producer.vrf_evaluator, BlockProducerVrfEvaluatorState::Pending(_))
//     }
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerVrfEvaluatorEvaluationSuccessAction {
    pub vrf_output: VrfEvaluationOutput,
}

impl redux::EnablingCondition<crate::State> for BlockProducerVrfEvaluatorEvaluationSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        // TODO(adonagy)
        // let global_slot = match &self.vrf_output {
        //     VrfEvaluationOutput::SlotWon(output) => output.global_slot,
        //     VrfEvaluationOutput::SlotLost(global_slot) => *global_slot,
        // };
        // matches!(state.block_producer.vrf_evaluator.evaluator_status, BlockProducerVrfEvaluatorStatus::Pending(global_slot))
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerVrfEvaluatorEpochDataUpdateAction {
    pub epoch_data: ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
    pub next_epoch_data: ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
}

impl redux::EnablingCondition<crate::State> for BlockProducerVrfEvaluatorEpochDataUpdateAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
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
impl_into_global_action!(BlockProducerVrfEvaluatorNewEpochAction);