use super::{
    BlockProducerVrfEvaluatorAction, BlockProducerVrfEvaluatorActionWithMetaRef,
    BlockProducerVrfEvaluatorState, BlockProducerVrfEvaluatorStatus, PendingEvaluation,
    VrfWonSlotWithHash,
};

impl BlockProducerVrfEvaluatorState {
    pub fn reducer(&mut self, action: BlockProducerVrfEvaluatorActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            BlockProducerVrfEvaluatorAction::EvaluateSlot { vrf_input } => {
                self.status = BlockProducerVrfEvaluatorStatus::SlotEvaluationPending {
                    time: meta.time(),
                    global_slot: vrf_input.global_slot,
                };
            }
            BlockProducerVrfEvaluatorAction::ProcessSlotEvaluationSuccess {
                vrf_output,
                staking_ledger_hash,
                ..
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
                self.set_latest_evaluated_global_slot(&global_slot_evaluated);

                self.status = BlockProducerVrfEvaluatorStatus::SlotEvaluationReceived {
                    time: meta.time(),
                    global_slot: global_slot_evaluated,
                }
            }
            BlockProducerVrfEvaluatorAction::CheckEpochBounds {
                epoch_number,
                latest_evaluated_global_slot,
            } => {
                let epoch_current_bound = Self::evaluate_epoch_bounds(latest_evaluated_global_slot);
                self.status = BlockProducerVrfEvaluatorStatus::EpochBoundsCheck {
                    time: meta.time(),
                    epoch_number: *epoch_number,
                    latest_evaluated_global_slot: *latest_evaluated_global_slot,
                    epoch_current_bound,
                };
            }
            BlockProducerVrfEvaluatorAction::InitializeEvaluator { .. } => {
                self.status =
                    BlockProducerVrfEvaluatorStatus::InitialisationPending { time: meta.time() }
            }
            BlockProducerVrfEvaluatorAction::FinalizeEvaluatorInitialization {
                previous_epoch_and_height,
            } => {
                if let Some((epoch, last_height)) = previous_epoch_and_height {
                    self.initialize_evaluator(*epoch, *last_height);
                }
                self.status =
                    BlockProducerVrfEvaluatorStatus::InitialisationComplete { time: meta.time() }
            }
            BlockProducerVrfEvaluatorAction::CheckEpochEvaluability {
                current_epoch_number,
                current_best_tip_height,
                transition_frontier_size,
                staking_epoch_data,
                next_epoch_data,
                ..
            } => {
                self.status = BlockProducerVrfEvaluatorStatus::ReadinessCheck {
                    time: meta.time(),
                    current_epoch_number: *current_epoch_number,
                    is_current_epoch_evaluated: self.is_epoch_evaluated(*current_epoch_number),
                    is_next_epoch_evaluated: self.is_epoch_evaluated(current_epoch_number + 1),
                    transition_frontier_size: *transition_frontier_size,
                    current_best_tip_height: *current_best_tip_height,
                    last_evaluated_epoch: self.last_evaluated_epoch(),
                    last_epoch_block_height: self.last_height(current_epoch_number - 1),
                    staking_epoch_data: staking_epoch_data.clone(),
                    next_epoch_data: next_epoch_data.clone(),
                };

                self.set_epoch_context();
            }
            BlockProducerVrfEvaluatorAction::InitializeEpochEvaluation {
                current_epoch_number,
                current_best_tip_slot,
                current_best_tip_height,
                current_best_tip_global_slot,
                next_epoch_first_slot,
                staking_epoch_data,
                producer,
                transition_frontier_size,
                ..
            } => {
                self.status = BlockProducerVrfEvaluatorStatus::ReadyToEvaluate {
                    time: meta.time(),
                    current_epoch_number: *current_epoch_number,
                    is_current_epoch_evaluated: self.is_epoch_evaluated(*current_epoch_number),
                    is_next_epoch_evaluated: self.is_epoch_evaluated(current_epoch_number + 1),
                    current_best_tip_slot: *current_best_tip_slot,
                    current_best_tip_height: *current_best_tip_height,
                    current_best_tip_global_slot: *current_best_tip_global_slot,
                    next_epoch_first_slot: *next_epoch_first_slot,
                    staking_epoch_data: staking_epoch_data.clone(),
                    producer: producer.clone(),
                    transition_frontier_size: *transition_frontier_size,
                }
            }
            BlockProducerVrfEvaluatorAction::BeginDelegatorTableConstruction => {
                let BlockProducerVrfEvaluatorStatus::ReadyToEvaluate {
                    current_epoch_number,
                    current_best_tip_slot,
                    current_best_tip_height,
                    current_best_tip_global_slot,
                    next_epoch_first_slot,
                    staking_epoch_data,
                    producer,
                    transition_frontier_size,
                    ..
                } = &self.status
                else {
                    return;
                };
                self.status = BlockProducerVrfEvaluatorStatus::EpochDelegatorTablePending {
                    time: meta.time(),
                    epoch_number: *current_epoch_number,
                    staking_epoch_ledger_hash: staking_epoch_data.ledger.clone(),
                    current_best_tip_slot: *current_best_tip_slot,
                    current_best_tip_height: *current_best_tip_height,
                    current_best_tip_global_slot: *current_best_tip_global_slot,
                    next_epoch_first_slot: *next_epoch_first_slot,
                    staking_epoch_data: staking_epoch_data.clone(),
                    producer: producer.clone(),
                    transition_frontier_size: *transition_frontier_size,
                }
            }
            BlockProducerVrfEvaluatorAction::FinalizeDelegatorTableConstruction {
                delegator_table,
            } => {
                let BlockProducerVrfEvaluatorStatus::EpochDelegatorTablePending {
                    epoch_number,
                    current_best_tip_slot,
                    current_best_tip_height,
                    current_best_tip_global_slot,
                    next_epoch_first_slot,
                    staking_epoch_data,
                    producer,
                    transition_frontier_size,
                    ..
                } = &self.status
                else {
                    return;
                };

                let mut staking_epoch_data = staking_epoch_data.clone();
                staking_epoch_data.delegator_table = delegator_table.clone();

                self.status = BlockProducerVrfEvaluatorStatus::EpochDelegatorTableSuccess {
                    time: meta.time(),
                    epoch_number: *epoch_number,
                    staking_epoch_ledger_hash: staking_epoch_data.ledger.clone(),
                    current_best_tip_slot: *current_best_tip_slot,
                    current_best_tip_height: *current_best_tip_height,
                    current_best_tip_global_slot: *current_best_tip_global_slot,
                    next_epoch_first_slot: *next_epoch_first_slot,
                    staking_epoch_data: staking_epoch_data.clone(),
                    producer: producer.clone(),
                    transition_frontier_size: *transition_frontier_size,
                }
            }
            BlockProducerVrfEvaluatorAction::BeginEpochEvaluation {
                current_epoch_number,
                latest_evaluated_global_slot,
                staking_epoch_data,
                ..
            } => {
                self.set_pending_evaluation(PendingEvaluation {
                    epoch_number: *current_epoch_number,
                    epoch_data: staking_epoch_data.clone(),
                    latest_evaluated_slot: *latest_evaluated_global_slot,
                });
                self.status = BlockProducerVrfEvaluatorStatus::EpochEvaluationPending {
                    time: meta.time(),
                    epoch_number: *current_epoch_number,
                    epoch_data: staking_epoch_data.clone(),
                    latest_evaluated_global_slot: *latest_evaluated_global_slot,
                }
            }
            BlockProducerVrfEvaluatorAction::RecordLastBlockHeightInEpoch {
                epoch_number,
                last_block_height,
                ..
            } => {
                self.add_last_height(*epoch_number, *last_block_height);
            }
            BlockProducerVrfEvaluatorAction::ContinueEpochEvaluation {
                epoch_number,
                latest_evaluated_global_slot,
            } => {
                if let Some(pending_evaluation) = self.current_evaluation() {
                    self.status = BlockProducerVrfEvaluatorStatus::EpochEvaluationPending {
                        time: meta.time(),
                        epoch_number: *epoch_number,
                        epoch_data: pending_evaluation.epoch_data,
                        latest_evaluated_global_slot: *latest_evaluated_global_slot,
                    }
                }
            }
            BlockProducerVrfEvaluatorAction::FinishEpochEvaluation { epoch_number, .. } => {
                self.unset_pending_evaluation();
                self.status = BlockProducerVrfEvaluatorStatus::EpochEvaluationSuccess {
                    time: meta.time(),
                    epoch_number: *epoch_number,
                };
                self.set_last_evaluated_epoch();
            }
            BlockProducerVrfEvaluatorAction::WaitForNextEvaluation {
                current_epoch_number,
                current_best_tip_height,
                current_best_tip_slot,
                current_best_tip_global_slot,
                last_epoch_block_height,
                transition_frontier_size,
            } => {
                self.status = BlockProducerVrfEvaluatorStatus::WaitingForNextEvaluation {
                    time: meta.time(),
                    current_epoch_number: *current_epoch_number,
                    current_best_tip_height: *current_best_tip_height,
                    current_best_tip_global_slot: *current_best_tip_global_slot,
                    current_best_tip_slot: *current_best_tip_slot,
                    last_epoch_block_height: *last_epoch_block_height,
                    transition_frontier_size: *transition_frontier_size,
                };
            }
            BlockProducerVrfEvaluatorAction::SelectInitialSlot {
                current_epoch_number,
                current_global_slot,
                next_epoch_first_slot,
                ..
            } => {
                let (epoch_number, initial_slot) = match self.epoch_context() {
                    super::EpochContext::Current(_) => {
                        (*current_epoch_number, *current_global_slot)
                    }
                    super::EpochContext::Next(_) => {
                        (current_epoch_number + 1, next_epoch_first_slot - 1)
                    }
                    super::EpochContext::Waiting => todo!(),
                };
                self.status = BlockProducerVrfEvaluatorStatus::InitialSlotSelection {
                    time: meta.time(),
                    epoch_number,
                    initial_slot,
                }
            }
            BlockProducerVrfEvaluatorAction::CleanupOldSlots {
                current_epoch_number,
            } => {
                self.cleanup_old_won_slots(current_epoch_number);
            }
        }
    }
}
