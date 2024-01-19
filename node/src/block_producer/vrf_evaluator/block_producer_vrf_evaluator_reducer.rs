use super::{
    BlockProducerVrfEvaluatorAction, BlockProducerVrfEvaluatorActionWithMetaRef,
    BlockProducerVrfEvaluatorState, BlockProducerVrfEvaluatorStatus, VrfWonSlotWithHash,
};

impl BlockProducerVrfEvaluatorState {
    pub fn reducer(&mut self, action: BlockProducerVrfEvaluatorActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            BlockProducerVrfEvaluatorAction::EvaluateVrf { .. } => {}
            BlockProducerVrfEvaluatorAction::EvaluationSuccess {
                vrf_output,
                staking_ledger_hash,
            } => {
                let global_slot_evaluated = match &vrf_output {
                    vrf::VrfEvaluationOutput::SlotWon(won_slot_data) => {
                        self.won_slots.insert(
                            won_slot_data.global_slot,
                            VrfWonSlotWithHash::new(
                                won_slot_data.clone(),
                                staking_ledger_hash.clone(),
                            ),
                        );
                        won_slot_data.global_slot
                    }
                    vrf::VrfEvaluationOutput::SlotLost(global_slot) => *global_slot,
                };
                self.set_last_evaluated_global_slot(&global_slot_evaluated);
                let epoch_bound = |global_slot| -> (u32, bool) {
                    (
                        global_slot / SLOTS_PER_EPOCH,
                        (global_slot + 1) % SLOTS_PER_EPOCH == 0,
                    )
                };
                let (epoch, is_epoch_end) = epoch_bound(global_slot_evaluated);

                const SLOTS_PER_EPOCH: u32 = 7140;

                self.status = if is_epoch_end {
                    self.set_last_evaluated_epoch();
                    BlockProducerVrfEvaluatorStatus::EpochEvaluationSuccess {
                        time: meta.time(),
                        epoch_number: epoch,
                        epoch_context: self.status.epoch_context(),
                    }
                } else {
                    let pending_evaluation = self.status.current_evaluation().unwrap();
                    BlockProducerVrfEvaluatorStatus::EpochEvaluationPending {
                        time: meta.time(),
                        epoch_number: epoch,
                        epoch_data: pending_evaluation.epoch_data,
                        latest_evaluated_slot: global_slot_evaluated,
                        epoch_context: self.status.epoch_context(),
                    }
                }
            }
            BlockProducerVrfEvaluatorAction::EvaluatorInit { .. } => {
                self.status = BlockProducerVrfEvaluatorStatus::EvaluatorInitialisationPending {
                    time: meta.time(),
                }
            }
            BlockProducerVrfEvaluatorAction::EvaluatorInitSuccess {
                previous_epoch_and_height
            } => {
                if let Some((epoch, last_height)) = previous_epoch_and_height {
                    self.initialize_evaluator(epoch, last_height);
                }
                self.status =
                    BlockProducerVrfEvaluatorStatus::EvaluatorInitialized { time: meta.time() }
            }
            BlockProducerVrfEvaluatorAction::CanEvaluateVrf {
                current_epoch_number,
                current_best_tip_height,
                current_best_tip_slot,
                current_best_tip_global_slot,
                transition_frontier_size,
            } => {
                if self.status.is_evaluator_ready(self.last_evaluated_epoch()) {
                    self.status = BlockProducerVrfEvaluatorStatus::CanEvaluateVrf {
                        time: meta.time(),
                        current_epoch_number,
                        is_current_epoch_evaluated: self
                            .status
                            .is_current_epoch_evaluated(self.last_evaluated_epoch()),
                        is_next_epoch_evaluated: self
                            .status
                            .is_next_epoch_evaluated(self.last_evaluated_epoch()),
                    };
                } else {
                    let previous_epoch_last_block_height =
                        self.last_height(current_epoch_number - 1);
                    self.status = BlockProducerVrfEvaluatorStatus::WaitingForEvaluation {
                        time: meta.time(),
                        current_epoch_number,
                        current_best_tip_height,
                        current_best_tip_slot,
                        current_best_tip_global_slot,
                        last_epoch_block_height: previous_epoch_last_block_height,
                        transition_frontier_size,
                    }
                }
            }
            BlockProducerVrfEvaluatorAction::EvaluateEpochInit { epoch_context, .. } => {
                self.status = BlockProducerVrfEvaluatorStatus::EpochEvaluationInit {
                    time: meta.time(),
                    epoch_context: epoch_context.clone(),
                }
            }
            BlockProducerVrfEvaluatorAction::ConstructDelegatorTable {
                epoch_context,
                current_epoch_number,
                staking_epoch_data,
                ..
            } => {
                self.status = BlockProducerVrfEvaluatorStatus::EpochDelegatorTablePending {
                    time: meta.time(),
                    epoch_context: epoch_context.clone(),
                    epoch_number: current_epoch_number,
                    staking_epoch_ledger_hash: staking_epoch_data.ledger.clone(),
                }
            }
            BlockProducerVrfEvaluatorAction::ConstructDelegatorTableSuccess {
                epoch_context,
                current_epoch_number,
                staking_epoch_data,
                ..
            } => {
                self.status = BlockProducerVrfEvaluatorStatus::EpochDelegatorTableSuccess {
                    epoch_context: epoch_context.clone(),
                    time: meta.time(),
                    epoch_number: current_epoch_number,
                    staking_epoch_ledger_hash: staking_epoch_data.ledger.clone(),
                }
            }
            BlockProducerVrfEvaluatorAction::EvaluateEpoch {
                epoch_context,
                current_epoch_number,
                latest_evaluated_global_slot,
                staking_epoch_data,
                ..
            } => {
                self.status = BlockProducerVrfEvaluatorStatus::EpochEvaluationPending {
                    epoch_context: epoch_context.clone(),
                    time: meta.time(),
                    epoch_number: current_epoch_number,
                    epoch_data: staking_epoch_data.clone(),
                    latest_evaluated_slot: latest_evaluated_global_slot,
                }
            }
            BlockProducerVrfEvaluatorAction::SaveLastBlockHeightInEpoch {
                epoch_number,
                last_block_height,
                ..
            } => {
                self.add_last_height(epoch_number, last_block_height);
            }
        }
    }
}
