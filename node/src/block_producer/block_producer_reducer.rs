use super::{BlockProducerAction, BlockProducerActionWithMetaRef, BlockProducerState};

impl BlockProducerState {
    pub fn reducer(&mut self, action: BlockProducerActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            BlockProducerAction::VrfEvaluator(action) => {
                self.vrf_evaluator.reducer(meta.with_action(action))
            }
            BlockProducerAction::BestTipUpdate(_) => {}
        }
    }
}
