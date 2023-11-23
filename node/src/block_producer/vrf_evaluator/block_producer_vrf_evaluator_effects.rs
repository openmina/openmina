use redux::ActionMeta;

use crate::Store;

use super::BlockProducerVrfEvaluatorEpochDataUpdateAction;

impl BlockProducerVrfEvaluatorEpochDataUpdateAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {}
}
