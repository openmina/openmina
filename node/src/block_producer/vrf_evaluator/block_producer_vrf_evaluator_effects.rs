use redux::ActionMeta;
use vrf::VrfEvaluatorInput;

use crate::Store;
use crate::Service;

use super::BlockProducerVrfEvaluatorUpdateProducerAndDelegatesAction;
use super::BlockProducerVrfEvaluatorUpdateProducerAndDelegatesSuccessAction;
use super::{BlockProducerVrfEvaluatorEpochDataUpdateAction, BlockProducerVrfEvaluatorEvaluateVrfAction, BlockProducerVrfEvaluatorEvaluationSuccessAction};

impl BlockProducerVrfEvaluatorEpochDataUpdateAction {
    pub fn effects<S: Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        // TODO(adonagy): once block producer is enabled
        // if let Some(config) = store.state().block_producer.config() {
        //     store.dispatch(BlockProducerVrfEvaluatorUpdateProducerAndDelegatesAction {
        //         current_epoch_ledger_hash: self.epoch_data.ledger.hash,
        //         next_epoch_ledger_hash: self.next_epoch_data.ledger.hash,
        //         producer: config.pub_key.to_string(),
        //     });
        // }

        // let vrf_evaluator_state = store.state().block_producer.vrf_evaluator();
        if let Some(vrf_evaluator_state) = store.state().block_producer.vrf_evaluator() {
            store.dispatch(BlockProducerVrfEvaluatorUpdateProducerAndDelegatesAction {
                current_epoch_ledger_hash: self.epoch_data.ledger.hash,
                next_epoch_ledger_hash: self.next_epoch_data.ledger.hash,
                producer: vrf_evaluator_state.producer_pub_key.to_string(),
            });
        }
    }
}

impl BlockProducerVrfEvaluatorEvaluateVrfAction {
    pub fn effects<S: Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.service.evaluate(self.vrf_input);
    }
}

// impl BlockProducerVrfEvaluatorEvaluationPendingAction {
//     pub fn effects<S: Service>(self, _: &ActionMeta, store: &mut Store<S>) {}
// }

impl BlockProducerVrfEvaluatorEvaluationSuccessAction {
    pub fn effects<S: Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        let vrf_evaluator_state = store.state().block_producer.vrf_evaluator();

        if let Some(vrf_evaluator_state) = vrf_evaluator_state {
            let next_slot = vrf_evaluator_state.latest_evaluated_slot + 1;
            // TODO(adonagy): Can we get this from somewhere?
            const SLOTS_PER_EPOCH: u32 = 7140;
            // determine the epoch of the slot
            if let Some(current_epoch) = vrf_evaluator_state.current_epoch {
                let current_epoch_end = current_epoch * SLOTS_PER_EPOCH + SLOTS_PER_EPOCH - 1;
                let next_epoch_end = (current_epoch + 1) * SLOTS_PER_EPOCH + SLOTS_PER_EPOCH - 1;

                // slot is in the current epoch
                if next_slot <= current_epoch_end {
                    let vrf_input: VrfEvaluatorInput = VrfEvaluatorInput::new(
                        vrf_evaluator_state.current_epoch_data.seed.clone(),
                        vrf_evaluator_state.current_epoch_data.delegator_table.clone(),
                        next_slot,
                        vrf_evaluator_state.current_epoch_data.total_currency,
                    );
                    store.dispatch(BlockProducerVrfEvaluatorEvaluateVrfAction { vrf_input });
                // slot is in the next epoch
                } else if next_slot > current_epoch_end && next_slot <= next_epoch_end {
                    let vrf_input = VrfEvaluatorInput::new(
                        vrf_evaluator_state.next_epoch_data.seed.clone(),
                        vrf_evaluator_state.next_epoch_data.delegator_table.clone(),
                        next_slot,
                        vrf_evaluator_state.next_epoch_data.total_currency,
                    );
                    store.dispatch(BlockProducerVrfEvaluatorEvaluateVrfAction { vrf_input });
                }
            }
        }
    }
}

impl BlockProducerVrfEvaluatorUpdateProducerAndDelegatesAction {
    pub fn effects<S: Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        let current_epoch_producer_and_delegators: std::collections::BTreeMap<ledger::AccountIndex, (String, u64)> = store.service.get_producer_and_delegates(self.current_epoch_ledger_hash, self.producer.clone());
        let next_epoch_producer_and_delegators: std::collections::BTreeMap<ledger::AccountIndex, (String, u64)> = store.service.get_producer_and_delegates(self.next_epoch_ledger_hash, self.producer.clone());

        store.dispatch(BlockProducerVrfEvaluatorUpdateProducerAndDelegatesSuccessAction { current_epoch_producer_and_delegators, next_epoch_producer_and_delegators });
    }
}

impl BlockProducerVrfEvaluatorUpdateProducerAndDelegatesSuccessAction {
    pub fn effects<S: Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        let vrf_evaluator_state = store.state().block_producer.vrf_evaluator();

        if let Some(vrf_evaluator_state) = vrf_evaluator_state {
            let vrf_input: VrfEvaluatorInput = VrfEvaluatorInput::new(
                vrf_evaluator_state.current_epoch_data.seed.clone(),
                vrf_evaluator_state.current_epoch_data.delegator_table.clone(),
                vrf_evaluator_state.current_best_tip_slot + 1,
                vrf_evaluator_state.current_epoch_data.total_currency,
            );
            store.dispatch(BlockProducerVrfEvaluatorEvaluateVrfAction { vrf_input });
        }
    }
}