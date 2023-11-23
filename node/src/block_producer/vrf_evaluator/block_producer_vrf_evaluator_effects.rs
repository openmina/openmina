use redux::ActionMeta;
use vrf::VrfEvaluatorInput;

use crate::Store;
use crate::Service;

use super::BlockProducerVrfEvaluatorNewEpochAction;
use super::BlockProducerVrfEvaluatorUpdateProducerAndDelegatesAction;
use super::BlockProducerVrfEvaluatorUpdateProducerAndDelegatesSuccessAction;
use super::{BlockProducerVrfEvaluatorEpochDataUpdateAction, BlockProducerVrfEvaluatorEvaluateVrfAction, BlockProducerVrfEvaluatorEvaluationSuccessAction};

impl BlockProducerVrfEvaluatorEpochDataUpdateAction {
    pub fn effects<S: Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        let producer = &store.state().block_producer.vrf_evaluator.producer_pub_key;

        // TODO(adonagy): move to enabling condition
        if let Some(producer) = producer {
            store.dispatch(BlockProducerVrfEvaluatorUpdateProducerAndDelegatesAction { current_epoch_ledger_hash: self.epoch_data.ledger.hash, next_epoch_ledger_hash: self.next_epoch_data.ledger.hash, producer: producer.to_string()});
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
        let next_slot = store.state().block_producer.vrf_evaluator.latest_evaluated_slot + 1;

        const SLOTS_PER_EPOCH: u32 = 7140;
        // determine the epoch of the slot
        if let Some(current_epoch) = store.state().block_producer.vrf_evaluator.current_epoch {
            let current_epoch_end = current_epoch * SLOTS_PER_EPOCH + SLOTS_PER_EPOCH - 1;
            let next_epoch_end = (current_epoch + 1) * SLOTS_PER_EPOCH + SLOTS_PER_EPOCH - 1;

            // slot is in the current epoch
            if next_slot <= current_epoch_end {
                let vrf_input: VrfEvaluatorInput = VrfEvaluatorInput::new(
                    store.state().block_producer.vrf_evaluator.current_epoch_data.seed.clone(),
                    store.state().block_producer.vrf_evaluator.current_epoch_data.delegator_table.clone(),
                    next_slot,
                    store.state().block_producer.vrf_evaluator.current_epoch_data.total_currency,
                );
                store.dispatch(BlockProducerVrfEvaluatorEvaluateVrfAction { vrf_input });
            // slot is in the next epoch
            } else if next_slot > current_epoch_end && next_slot <= next_epoch_end {
                let vrf_input = VrfEvaluatorInput::new(
                    store.state().block_producer.vrf_evaluator.next_epoch_data.seed.clone(),
                    store.state().block_producer.vrf_evaluator.next_epoch_data.delegator_table.clone(),
                    next_slot,
                    store.state().block_producer.vrf_evaluator.next_epoch_data.total_currency,
                );
                store.dispatch(BlockProducerVrfEvaluatorEvaluateVrfAction { vrf_input });
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

impl BlockProducerVrfEvaluatorNewEpochAction {
    pub fn effects<S: Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(BlockProducerVrfEvaluatorEpochDataUpdateAction {
            epoch_data: self.epoch_data.clone(),
            next_epoch_data: self.next_epoch_data.clone(),
        });
    }
}

impl BlockProducerVrfEvaluatorUpdateProducerAndDelegatesSuccessAction {
    pub fn effects<S: Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        let vrf_input: VrfEvaluatorInput = VrfEvaluatorInput::new(
            store.state().block_producer.vrf_evaluator.current_epoch_data.seed.clone(),
            store.state().block_producer.vrf_evaluator.current_epoch_data.delegator_table.clone(),
            store.state().block_producer.vrf_evaluator.current_best_tip_slot + 1,
            store.state().block_producer.vrf_evaluator.current_epoch_data.total_currency,
        );
        store.dispatch(BlockProducerVrfEvaluatorEvaluateVrfAction { vrf_input });
    }
}