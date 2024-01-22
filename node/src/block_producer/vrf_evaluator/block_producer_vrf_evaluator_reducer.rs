use super::{
    BlockProducerVrfEvaluatorAction, BlockProducerVrfEvaluatorActionWithMetaRef,
    BlockProducerVrfEvaluatorState, BlockProducerVrfEvaluatorStatus, EpochData, VrfWonSlotWithHash,
};

impl BlockProducerVrfEvaluatorState {
    pub fn reducer(&mut self, action: BlockProducerVrfEvaluatorActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            BlockProducerVrfEvaluatorAction::EpochDataUpdate(action) => {
                self.status = BlockProducerVrfEvaluatorStatus::EpochChanged { time: meta.time() };
                self.current_epoch_data = Some(EpochData::new(
                    action.epoch_data.seed.to_string(),
                    action.epoch_data.ledger.hash.clone(),
                    action.epoch_data.ledger.total_currency.as_u64(),
                ));
                self.next_epoch_data = Some(EpochData::new(
                    action.next_epoch_data.seed.to_string(),
                    action.next_epoch_data.ledger.hash.clone(),
                    action.next_epoch_data.ledger.total_currency.as_u64(),
                ));
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
                        self.won_slots.insert(
                            won_slot_data.global_slot,
                            VrfWonSlotWithHash::new(
                                won_slot_data.clone(),
                                action.staking_ledger_hash.clone(),
                            ),
                        );
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

                if let Some(epoch_data) = self.current_epoch_data.as_mut() {
                    epoch_data.delegator_table =
                        action.current_epoch_producer_and_delegators.clone();
                }

                if let Some(epoch_data) = self.next_epoch_data.as_mut() {
                    epoch_data.delegator_table =
                        action.current_epoch_producer_and_delegators.clone();
                }
            }
        }
    }
}
