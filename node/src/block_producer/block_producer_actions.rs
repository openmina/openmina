use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

use super::vrf_evaluator::BlockProducerVrfEvaluatorAction;

pub type BlockProducerActionWithMeta = redux::ActionWithMeta<BlockProducerAction>;
pub type BlockProducerActionWithMetaRef<'a> = redux::ActionWithMeta<&'a BlockProducerAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerAction {
    VrfEvaluator(BlockProducerVrfEvaluatorAction),
    BestTipUpdate(BlockProducerBestTipUpdateAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerBestTipUpdateAction {
    pub best_tip: ArcBlockWithHash,
}

impl redux::EnablingCondition<crate::State> for BlockProducerBestTipUpdateAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        true
    }
}

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::BlockProducer(value.into())
            }
        }
    };
}

impl_into_global_action!(BlockProducerBestTipUpdateAction);
