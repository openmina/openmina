use crate::Service;
use crate::Store;
use redux::ActionMeta;

use super::BlockProducerVrfEvaluatorEffectfulAction;

impl BlockProducerVrfEvaluatorEffectfulAction {
    pub fn effects<S: Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        match self {
            BlockProducerVrfEvaluatorEffectfulAction::EvaluateSlot { vrf_input } => {
                store.service.evaluate(vrf_input);
            }
        }
    }
}
