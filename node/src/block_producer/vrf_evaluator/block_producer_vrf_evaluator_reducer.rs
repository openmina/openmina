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
                current_epoch,
                is_next_epoch_seed_finalized,
                best_tip_epoch,
                root_block_epoch,
                staking_epoch_data,
                next_epoch_data,
                best_tip_slot: _,
                best_tip_global_slot: _,
                next_epoch_first_slot: _,
            } => {
                self.status = BlockProducerVrfEvaluatorStatus::ReadinessCheck {
                    time: meta.time(),
                    current_epoch: *current_epoch,
                    is_next_epoch_seed_finalized: *is_next_epoch_seed_finalized,
                    best_tip_epoch: *best_tip_epoch,
                    root_block_epoch: *root_block_epoch,
                    is_current_epoch_evaluated: self.is_epoch_evaluated(*best_tip_epoch),
                    is_next_epoch_evaluated: self.is_epoch_evaluated(best_tip_epoch + 1),
                    last_evaluated_epoch: self.last_evaluated_epoch(),
                    staking_epoch_data: staking_epoch_data.clone(),
                    next_epoch_data: next_epoch_data.clone(),
                };

                self.set_epoch_context();
            }
            BlockProducerVrfEvaluatorAction::InitializeEpochEvaluation {
                best_tip_epoch,
                best_tip_slot,
                best_tip_global_slot,
                next_epoch_first_slot,
                staking_epoch_data,
                producer,
            } => {
                self.status = BlockProducerVrfEvaluatorStatus::ReadyToEvaluate {
                    time: meta.time(),
                    best_tip_epoch: *best_tip_epoch,
                    is_current_epoch_evaluated: self.is_epoch_evaluated(*best_tip_epoch),
                    is_next_epoch_evaluated: self.is_epoch_evaluated(best_tip_epoch + 1),
                    best_tip_slot: *best_tip_slot,
                    best_tip_global_slot: *best_tip_global_slot,
                    next_epoch_first_slot: *next_epoch_first_slot,
                    staking_epoch_data: staking_epoch_data.clone(),
                    producer: producer.clone(),
                }
            }
            BlockProducerVrfEvaluatorAction::BeginDelegatorTableConstruction => {
                let BlockProducerVrfEvaluatorStatus::ReadyToEvaluate {
                    best_tip_epoch,
                    best_tip_slot,
                    best_tip_global_slot,
                    next_epoch_first_slot,
                    staking_epoch_data,
                    producer,
                    time: _,
                    is_current_epoch_evaluated: _,
                    is_next_epoch_evaluated: _,
                } = &self.status
                else {
                    return;
                };
                self.status = BlockProducerVrfEvaluatorStatus::EpochDelegatorTablePending {
                    time: meta.time(),
                    best_tip_epoch: *best_tip_epoch,
                    staking_epoch_ledger_hash: staking_epoch_data.ledger.clone(),
                    best_tip_slot: *best_tip_slot,
                    best_tip_global_slot: *best_tip_global_slot,
                    next_epoch_first_slot: *next_epoch_first_slot,
                    staking_epoch_data: staking_epoch_data.clone(),
                    producer: producer.clone(),
                }
            }
            BlockProducerVrfEvaluatorAction::FinalizeDelegatorTableConstruction {
                delegator_table,
            } => {
                let BlockProducerVrfEvaluatorStatus::EpochDelegatorTablePending {
                    best_tip_epoch,
                    best_tip_slot,
                    best_tip_global_slot,
                    next_epoch_first_slot,
                    staking_epoch_data,
                    producer,
                    time: _,
                    staking_epoch_ledger_hash: _,
                } = &self.status
                else {
                    return;
                };

                let mut staking_epoch_data = staking_epoch_data.clone();
                staking_epoch_data.delegator_table = delegator_table.clone();

                self.status = BlockProducerVrfEvaluatorStatus::EpochDelegatorTableSuccess {
                    time: meta.time(),
                    best_tip_epoch: *best_tip_epoch,
                    staking_epoch_ledger_hash: staking_epoch_data.ledger.clone(),
                    best_tip_slot: *best_tip_slot,
                    best_tip_global_slot: *best_tip_global_slot,
                    next_epoch_first_slot: *next_epoch_first_slot,
                    staking_epoch_data: staking_epoch_data.clone(),
                    producer: producer.clone(),
                }
            }
            BlockProducerVrfEvaluatorAction::BeginEpochEvaluation {
                best_tip_epoch,
                latest_evaluated_global_slot,
                staking_epoch_data,
                best_tip_slot: _,
                best_tip_global_slot: _,
            } => {
                self.set_pending_evaluation(PendingEvaluation {
                    epoch_number: *best_tip_epoch,
                    epoch_data: staking_epoch_data.clone(),
                    latest_evaluated_slot: *latest_evaluated_global_slot,
                });
                self.status = BlockProducerVrfEvaluatorStatus::EpochEvaluationPending {
                    time: meta.time(),
                    epoch_number: *best_tip_epoch,
                    epoch_data: staking_epoch_data.clone(),
                    latest_evaluated_global_slot: *latest_evaluated_global_slot,
                }
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
            BlockProducerVrfEvaluatorAction::FinishEpochEvaluation {
                epoch_number,
                latest_evaluated_global_slot: _,
            } => {
                self.unset_pending_evaluation();
                self.status = BlockProducerVrfEvaluatorStatus::EpochEvaluationSuccess {
                    time: meta.time(),
                    epoch_number: *epoch_number,
                };
                self.set_last_evaluated_epoch();
            }
            BlockProducerVrfEvaluatorAction::WaitForNextEvaluation => {
                self.status =
                    BlockProducerVrfEvaluatorStatus::WaitingForNextEvaluation { time: meta.time() };
            }
            BlockProducerVrfEvaluatorAction::SelectInitialSlot {
                best_tip_epoch,
                current_global_slot,
                next_epoch_first_slot,
                best_tip_slot: _,
                best_tip_global_slot: _,
                staking_epoch_data: _,
            } => {
                let (epoch_number, initial_slot) = match self.epoch_context() {
                    super::EpochContext::Current(_) => (*best_tip_epoch, *current_global_slot),
                    super::EpochContext::Next(_) => (best_tip_epoch + 1, next_epoch_first_slot - 1),
                    super::EpochContext::Waiting => todo!(),
                };
                self.status = BlockProducerVrfEvaluatorStatus::InitialSlotSelection {
                    time: meta.time(),
                    epoch_number,
                    initial_slot,
                }
            }
            BlockProducerVrfEvaluatorAction::CleanupOldSlots { best_tip_epoch } => {
                self.cleanup_old_won_slots(best_tip_epoch);
            }
            BlockProducerVrfEvaluatorAction::InterruptEpochEvaluation { reason } => {
                self.status = BlockProducerVrfEvaluatorStatus::EpochEvaluationInterrupted {
                    time: meta.time(),
                    reason: reason.clone(),
                };
            }
        }
    }
}
