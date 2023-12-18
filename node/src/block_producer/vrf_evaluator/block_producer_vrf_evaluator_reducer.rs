use super::{
    BlockProducerVrfEvaluatorAction, BlockProducerVrfEvaluatorActionWithMetaRef,
    BlockProducerVrfEvaluatorState, BlockProducerVrfEvaluatorStatus,
};

impl BlockProducerVrfEvaluatorState {
    pub fn reducer(&mut self, action: BlockProducerVrfEvaluatorActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            BlockProducerVrfEvaluatorAction::EpochDataUpdate(action) => {
                self.status = BlockProducerVrfEvaluatorStatus::EpochChanged { time: meta.time() };
                self.current_epoch_data.seed = action.epoch_data.seed.to_string();
                self.current_epoch_data.ledger = action.epoch_data.ledger.hash.to_string();
                self.current_epoch_data.total_currency =
                    action.epoch_data.ledger.total_currency.as_u64();
                self.next_epoch_data.seed = action.next_epoch_data.seed.to_string();
                self.next_epoch_data.ledger = action.next_epoch_data.ledger.hash.to_string();
                self.next_epoch_data.total_currency =
                    action.next_epoch_data.ledger.total_currency.as_u64();
                self.current_epoch = Some(action.new_epoch_number);
            }
            BlockProducerVrfEvaluatorAction::EvaluateVrf(_) => {
                // self.status = BlockProducerVrfEvaluatorStatus::Pending(action.vrf_input.global_slot);
                self.status = BlockProducerVrfEvaluatorStatus::SlotsRequested { time: meta.time() };
            }
            // BlockProducerVrfEvaluatorAction::EvaluationPending(_) => todo!(),
            BlockProducerVrfEvaluatorAction::EvaluationSuccess(action) => {
                let global_slot_evaluated = match &action.vrf_output {
                    vrf::VrfEvaluationOutput::SlotWon(won_slot_data) => {
                        self.won_slots
                            .insert(won_slot_data.global_slot, won_slot_data.clone());
                        won_slot_data.global_slot
                    }
                    vrf::VrfEvaluationOutput::SlotLost(global_slot) => *global_slot,
                };
                self.status = BlockProducerVrfEvaluatorStatus::SlotsReceived { time: meta.time() };
                self.latest_evaluated_slot = global_slot_evaluated;
            }
            BlockProducerVrfEvaluatorAction::UpdateProducerAndDelegates(_) => {
                self.status = BlockProducerVrfEvaluatorStatus::DataPending { time: meta.time() };
            }
            BlockProducerVrfEvaluatorAction::UpdateProducerAndDelegatesSuccess(action) => {
                self.status = BlockProducerVrfEvaluatorStatus::DataSuccess { time: meta.time() };
                self.current_epoch_data.delegator_table =
                    action.current_epoch_producer_and_delegators.clone();
                self.next_epoch_data.delegator_table =
                    action.current_epoch_producer_and_delegators.clone();
            }
        }
    }
}
