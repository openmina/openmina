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
            BlockProducerVrfEvaluatorEffectfulAction::SlotEvaluated { epoch } => {
                if let Some(stats) = store.service.stats() {
                    stats.block_producer().increment_slot_evaluated(epoch);
                }
            }
            BlockProducerVrfEvaluatorEffectfulAction::InitializeStats {
                epoch,
                initial_slot,
            } => {
                if let Some(stats) = store.service.stats() {
                    let slots_per_epoch =
                        store.state.get().config.consensus_constants.slots_per_epoch;
                    // We subtract 1 because the slots are indexed from 0
                    let remaining_slots = slots_per_epoch
                        .saturating_sub(initial_slot)
                        .saturating_sub(1);
                    stats
                        .block_producer()
                        .new_epoch_evaluation(epoch, remaining_slots);
                }
            }
        }
    }
}
