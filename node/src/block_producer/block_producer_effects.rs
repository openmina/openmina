use redux::ActionMeta;

use crate::Store;

use super::vrf_evaluator::{
    BlockProducerVrfEvaluatorAction, BlockProducerVrfEvaluatorEpochDataUpdateAction,
};
use super::{BlockProducerAction, BlockProducerActionWithMeta, BlockProducerBestTipUpdateAction};

pub fn block_producer_effects<S: crate::Service>(
    store: &mut Store<S>,
    action: BlockProducerActionWithMeta,
) {
    let (action, meta) = action.split();

    match action {
        BlockProducerAction::VrfEvaluator(action) => match action {
            BlockProducerVrfEvaluatorAction::EpochDataUpdate(action) => {
                action.effects(&meta, store);
            }
        },
        BlockProducerAction::BestTipUpdate(action) => {
            action.effects(&meta, store);
        }
    }
}

impl BlockProducerBestTipUpdateAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        let protocol_state = &self.best_tip.block.header.protocol_state.body;
        store.dispatch(BlockProducerVrfEvaluatorEpochDataUpdateAction {
            epoch_data: protocol_state.consensus_state.staking_epoch_data.clone(),
        });
    }
}
