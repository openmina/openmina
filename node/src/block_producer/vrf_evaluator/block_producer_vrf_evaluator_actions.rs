use mina_p2p_messages::v2::ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1;
use serde::{Deserialize, Serialize};

pub type BlockProducerVrfEvaluatorActionWithMeta =
    redux::ActionWithMeta<BlockProducerVrfEvaluatorAction>;
pub type BlockProducerVrfEvaluatorActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a BlockProducerVrfEvaluatorAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerVrfEvaluatorAction {
    EpochDataUpdate(BlockProducerVrfEvaluatorEpochDataUpdateAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerVrfEvaluatorEpochDataUpdateAction {
    pub epoch_data: ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
}

impl redux::EnablingCondition<crate::State> for BlockProducerVrfEvaluatorEpochDataUpdateAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        true
    }
}

use crate::block_producer::BlockProducerAction;

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
