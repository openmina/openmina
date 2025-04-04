//! Implements effect handlers for VRF evaluation actions in block production.
//! Manages the interaction with VRF evaluation services and statistics tracking.

use crate::Service;
use crate::Store;
use redux::ActionMeta;

use super::BlockProducerVrfEvaluatorEffectfulAction;

impl BlockProducerVrfEvaluatorEffectfulAction {
    /// Handles side effects for VRF evaluator actions.
    ///
    /// This method processes VRF evaluation requests, tracks statistics,
    /// and manages epoch transitions for the VRF evaluation process.
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
                    // We add 1 because the evaluation starts from the next slot, except for the first slot 0
                    let initial_slot = if initial_slot == 0 {
                        0
                    } else {
                        initial_slot.saturating_add(1)
                    };
                    let remaining_slots = slots_per_epoch.saturating_sub(initial_slot);
                    stats
                        .block_producer()
                        .new_epoch_evaluation(epoch, remaining_slots);
                }
            }
        }
    }
}
