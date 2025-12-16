use mina_core::bug_condition;
use vrf::VrfEvaluationOutput;

use crate::{
    block_producer::to_epoch_and_slot,
    block_producer_effectful::vrf_evaluator_effectful::BlockProducerVrfEvaluatorEffectfulAction,
    ledger::read::{LedgerReadAction, LedgerReadInitCallback, LedgerReadRequest},
    BlockProducerAction, Substate,
};

use super::{
    BlockProducerVrfEvaluatorAction, BlockProducerVrfEvaluatorActionWithMetaRef,
    BlockProducerVrfEvaluatorState, BlockProducerVrfEvaluatorStatus, PendingEvaluation,
    SlotPositionInEpoch, VrfWonSlotWithHash,
};

impl BlockProducerVrfEvaluatorState {
    pub fn reducer(
        mut state_context: Substate<Self>,
        action: BlockProducerVrfEvaluatorActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else {
            return;
        };

        let (action, meta) = action.split();
        match action {
            BlockProducerVrfEvaluatorAction::EvaluateSlot { vrf_input } => {
                state.status = BlockProducerVrfEvaluatorStatus::SlotEvaluationPending {
                    time: meta.time(),
                    global_slot: vrf_input.global_slot,
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(BlockProducerVrfEvaluatorEffectfulAction::EvaluateSlot {
                    vrf_input: vrf_input.clone(),
                });
            }
            BlockProducerVrfEvaluatorAction::ProcessSlotEvaluationSuccess {
                vrf_output,
                staking_ledger_hash,
            } => {
                let global_slot_evaluated = match &vrf_output {
                    vrf::VrfEvaluationOutput::SlotWon(won_slot_data) => {
                        state.won_slots.insert(
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
                state.set_latest_evaluated_global_slot(&global_slot_evaluated);

                state.status = BlockProducerVrfEvaluatorStatus::SlotEvaluationReceived {
                    time: meta.time(),
                    global_slot: global_slot_evaluated,
                };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                if let Some(vrf_evaluator_state) = state.block_producer.vrf_evaluator() {
                    if let Some(pending_evaluation) = vrf_evaluator_state.current_evaluation() {
                        dispatcher.push(BlockProducerVrfEvaluatorEffectfulAction::SlotEvaluated {
                            epoch: pending_evaluation.epoch_number,
                        });
                        dispatcher.push(BlockProducerVrfEvaluatorAction::CheckEpochBounds {
                            epoch_number: pending_evaluation.epoch_number,
                            latest_evaluated_global_slot: vrf_output.global_slot(),
                        });
                    }
                }

                if matches!(vrf_output, VrfEvaluationOutput::SlotWon(_)) {
                    dispatcher.push(BlockProducerAction::WonSlotSearch);
                }
            }
            BlockProducerVrfEvaluatorAction::CheckEpochBounds {
                epoch_number,
                latest_evaluated_global_slot,
            } => {
                let latest_evaluated_global_slot = *latest_evaluated_global_slot;
                let epoch_number = *epoch_number;

                let epoch_current_bound =
                    Self::evaluate_epoch_bounds(&latest_evaluated_global_slot);
                state.status = BlockProducerVrfEvaluatorStatus::EpochBoundsCheck {
                    time: meta.time(),
                    epoch_number,
                    latest_evaluated_global_slot,
                    epoch_current_bound,
                };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                if let Some(epoch_bound) = state
                    .block_producer
                    .vrf_evaluator()
                    .and_then(|s| s.get_epoch_bound_from_check())
                {
                    match epoch_bound {
                        SlotPositionInEpoch::Beginning | SlotPositionInEpoch::Within => {
                            dispatcher.push(
                                BlockProducerVrfEvaluatorAction::ContinueEpochEvaluation {
                                    latest_evaluated_global_slot,
                                    epoch_number,
                                },
                            );
                        }
                        SlotPositionInEpoch::End => {
                            dispatcher.push(
                                BlockProducerVrfEvaluatorAction::FinishEpochEvaluation {
                                    latest_evaluated_global_slot,
                                    epoch_number,
                                },
                            );
                        }
                    }
                }
            }
            BlockProducerVrfEvaluatorAction::InitializeEvaluator { best_tip } => {
                state.status =
                    BlockProducerVrfEvaluatorStatus::InitialisationPending { time: meta.time() };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                // Note: pure function, but needs access to other parts of the state
                if state.block_producer.vrf_evaluator().is_some() {
                    if best_tip.consensus_state().epoch_count.as_u32() == 0 {
                        dispatcher.push(
                            BlockProducerVrfEvaluatorAction::FinalizeEvaluatorInitialization {
                                previous_epoch_and_height: None,
                            },
                        );
                    } else {
                        let k = best_tip.constants().k.as_u32();
                        let (epoch, slot) = to_epoch_and_slot(
                            &best_tip.consensus_state().curr_global_slot_since_hard_fork,
                        );
                        let previous_epoch = epoch.saturating_sub(1);
                        let last_height = if slot < k {
                            let found =
                                state.transition_frontier.best_chain.iter().rev().find(|b| {
                                    b.consensus_state().epoch_count.as_u32() == previous_epoch
                                });

                            if let Some(block) = found {
                                block.height()
                            } else {
                                Default::default()
                            }
                        } else if let Some(root_block) = state.transition_frontier.root() {
                            root_block.height()
                        } else {
                            Default::default()
                        };
                        dispatcher.push(
                            BlockProducerVrfEvaluatorAction::FinalizeEvaluatorInitialization {
                                previous_epoch_and_height: Some((previous_epoch, last_height)),
                            },
                        );
                    }
                }
            }
            BlockProducerVrfEvaluatorAction::FinalizeEvaluatorInitialization {
                previous_epoch_and_height,
            } => {
                if let Some((epoch, last_height)) = previous_epoch_and_height {
                    state.initialize_evaluator(*epoch, *last_height);
                }
                state.status =
                    BlockProducerVrfEvaluatorStatus::InitialisationComplete { time: meta.time() }
            }
            BlockProducerVrfEvaluatorAction::CheckEpochEvaluability {
                current_epoch,
                is_next_epoch_seed_finalized,
                best_tip_epoch,
                root_block_epoch,
                staking_epoch_data,
                next_epoch_data,
                best_tip_slot,
                best_tip_global_slot,
                next_epoch_first_slot,
            } => {
                let best_tip_epoch = *best_tip_epoch;

                state.status = BlockProducerVrfEvaluatorStatus::ReadinessCheck {
                    time: meta.time(),
                    current_epoch: *current_epoch,
                    is_next_epoch_seed_finalized: *is_next_epoch_seed_finalized,
                    best_tip_epoch,
                    root_block_epoch: *root_block_epoch,
                    is_current_epoch_evaluated: state.is_epoch_evaluated(best_tip_epoch),
                    is_next_epoch_evaluated: state
                        .is_epoch_evaluated(best_tip_epoch.checked_add(1).expect("overflow")),
                    last_evaluated_epoch: state.last_evaluated_epoch(),
                    staking_epoch_data: staking_epoch_data.clone(),
                    next_epoch_data: next_epoch_data.clone(),
                };

                state.set_epoch_context();

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let vrf_evaluator_state = state.block_producer.vrf_evaluator_with_config();

                if let Some((vrf_evaluator_state, config)) = vrf_evaluator_state {
                    if let Some(epoch_data) = vrf_evaluator_state.epoch_context().get_epoch_data() {
                        dispatcher.push(
                            BlockProducerVrfEvaluatorAction::InitializeEpochEvaluation {
                                staking_epoch_data: epoch_data,
                                producer: config.pub_key.clone().into(),
                                best_tip_global_slot: *best_tip_global_slot,
                                best_tip_epoch,
                                best_tip_slot: *best_tip_slot,
                                next_epoch_first_slot: *next_epoch_first_slot,
                            },
                        );
                    } else {
                        // If None is returned, than we are waiting for evaluation
                        dispatcher.push(BlockProducerVrfEvaluatorAction::WaitForNextEvaluation);
                    }

                    dispatcher
                        .push(BlockProducerVrfEvaluatorAction::CleanupOldSlots { best_tip_epoch });
                }
            }
            BlockProducerVrfEvaluatorAction::InitializeEpochEvaluation {
                best_tip_epoch,
                best_tip_slot,
                best_tip_global_slot,
                next_epoch_first_slot,
                staking_epoch_data,
                producer,
            } => {
                state.status = BlockProducerVrfEvaluatorStatus::ReadyToEvaluate {
                    time: meta.time(),
                    best_tip_epoch: *best_tip_epoch,
                    is_current_epoch_evaluated: state.is_epoch_evaluated(*best_tip_epoch),
                    is_next_epoch_evaluated: state
                        .is_epoch_evaluated(best_tip_epoch.checked_add(1).expect("overflow")),
                    best_tip_slot: *best_tip_slot,
                    best_tip_global_slot: *best_tip_global_slot,
                    next_epoch_first_slot: *next_epoch_first_slot,
                    staking_epoch_data: staking_epoch_data.clone(),
                    producer: producer.clone(),
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(BlockProducerVrfEvaluatorAction::BeginDelegatorTableConstruction);
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
                } = &state.status
                else {
                    return;
                };
                state.status = BlockProducerVrfEvaluatorStatus::EpochDelegatorTablePending {
                    time: meta.time(),
                    best_tip_epoch: *best_tip_epoch,
                    staking_epoch_ledger_hash: staking_epoch_data.ledger.clone(),
                    best_tip_slot: *best_tip_slot,
                    best_tip_global_slot: *best_tip_global_slot,
                    next_epoch_first_slot: *next_epoch_first_slot,
                    staking_epoch_data: staking_epoch_data.clone(),
                    producer: producer.clone(),
                };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let (staking_ledger_hash, producer) =
                    match state.block_producer.vrf_delegator_table_inputs() {
                        Some((v1, v2)) => (v1.clone(), v2.clone()),
                        None => return,
                    };

                dispatcher.push(LedgerReadAction::Init {
                    request: LedgerReadRequest::DelegatorTable(staking_ledger_hash, producer),
                    callback: LedgerReadInitCallback::None,
                })
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
                } = &state.status
                else {
                    bug_condition!("Invalid state for `BlockProducerVrfEvaluatorAction::FinalizeDelegatorTableConstruction` expected: `BlockProducerVrfEvaluatorStatus::EpochDelegatorTablePending`, found: {:?}", state.status);
                    return;
                };

                mina_core::log::warn!(
                    meta.time();
                    kind = "BlockProducerVrfEvaluatorAction::FinalizeDelegatorTableConstruction",
                    message = "Empty delegator table, account may not exist yet in the staking ledger"
                );

                let mut staking_epoch_data = staking_epoch_data.clone();
                staking_epoch_data.delegator_table = delegator_table.clone();

                state.status = BlockProducerVrfEvaluatorStatus::EpochDelegatorTableSuccess {
                    time: meta.time(),
                    best_tip_epoch: *best_tip_epoch,
                    staking_epoch_ledger_hash: staking_epoch_data.ledger.clone(),
                    best_tip_slot: *best_tip_slot,
                    best_tip_global_slot: *best_tip_global_slot,
                    next_epoch_first_slot: *next_epoch_first_slot,
                    staking_epoch_data: staking_epoch_data.clone(),
                    producer: producer.clone(),
                };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let get_slot_and_status = || {
                    let cur_global_slot = state.cur_global_slot()?;
                    let current_slot = state.current_slot()?;
                    let status = &state.block_producer.vrf_evaluator()?.status;

                    Some((cur_global_slot, current_slot, status))
                };

                let Some((
                    current_global_slot,
                    current_slot,
                    BlockProducerVrfEvaluatorStatus::EpochDelegatorTableSuccess {
                        best_tip_epoch,
                        best_tip_slot,
                        best_tip_global_slot,
                        next_epoch_first_slot,
                        staking_epoch_data,
                        ..
                    },
                )) = get_slot_and_status()
                else {
                    bug_condition!("Invalid state for `BlockProducerVrfEvaluatorAction::FinalizeDelegatorTableConstruction`");
                    return;
                };

                dispatcher.push(BlockProducerVrfEvaluatorAction::SelectInitialSlot {
                    current_global_slot,
                    current_slot,
                    best_tip_epoch: *best_tip_epoch,
                    best_tip_slot: *best_tip_slot,
                    best_tip_global_slot: *best_tip_global_slot,
                    next_epoch_first_slot: *next_epoch_first_slot,
                    staking_epoch_data: staking_epoch_data.clone(),
                });
            }
            BlockProducerVrfEvaluatorAction::BeginEpochEvaluation {
                best_tip_epoch: _,
                evaluation_epoch,
                latest_evaluated_global_slot,
                staking_epoch_data,
                best_tip_slot: _,
                best_tip_global_slot: _,
            } => {
                let latest_evaluated_global_slot = *latest_evaluated_global_slot;
                let epoch_number = *evaluation_epoch;

                state.set_pending_evaluation(PendingEvaluation {
                    epoch_number,
                    epoch_data: staking_epoch_data.clone(),
                    latest_evaluated_slot: latest_evaluated_global_slot,
                });
                state.status = BlockProducerVrfEvaluatorStatus::EpochEvaluationPending {
                    time: meta.time(),
                    epoch_number,
                    epoch_data: staking_epoch_data.clone(),
                    latest_evaluated_global_slot,
                };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                if state.block_producer.vrf_evaluator().is_some() {
                    dispatcher.push(BlockProducerVrfEvaluatorAction::ContinueEpochEvaluation {
                        latest_evaluated_global_slot,
                        epoch_number,
                    });
                }
            }
            BlockProducerVrfEvaluatorAction::ContinueEpochEvaluation {
                epoch_number,
                latest_evaluated_global_slot,
            } => {
                if let Some(pending_evaluation) = state.current_evaluation() {
                    state.status = BlockProducerVrfEvaluatorStatus::EpochEvaluationPending {
                        time: meta.time(),
                        epoch_number: *epoch_number,
                        epoch_data: pending_evaluation.epoch_data,
                        latest_evaluated_global_slot: *latest_evaluated_global_slot,
                    };
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                if let Some(vrf_evaluator_state) = state.block_producer.vrf_evaluator() {
                    if let Some(vrf_input) = vrf_evaluator_state.construct_vrf_input() {
                        dispatcher
                            .push(BlockProducerVrfEvaluatorAction::EvaluateSlot { vrf_input });
                    }
                }
            }
            BlockProducerVrfEvaluatorAction::FinishEpochEvaluation {
                epoch_number,
                latest_evaluated_global_slot: _,
            } => {
                state.unset_pending_evaluation();
                state.status = BlockProducerVrfEvaluatorStatus::EpochEvaluationSuccess {
                    time: meta.time(),
                    epoch_number: *epoch_number,
                };
                state.set_last_evaluated_epoch(*epoch_number);
            }
            BlockProducerVrfEvaluatorAction::WaitForNextEvaluation => {
                state.status =
                    BlockProducerVrfEvaluatorStatus::WaitingForNextEvaluation { time: meta.time() };
            }
            BlockProducerVrfEvaluatorAction::SelectInitialSlot {
                best_tip_epoch,
                current_global_slot,
                current_slot,
                next_epoch_first_slot,
                best_tip_slot: current_best_tip_slot,
                best_tip_global_slot: current_best_tip_global_slot,
                staking_epoch_data,
            } => {
                let (epoch_number, initial_global_slot, initial_slot) = match state.epoch_context()
                {
                    super::EpochContext::Current(_) => {
                        (*best_tip_epoch, *current_global_slot, *current_slot)
                    }
                    super::EpochContext::Next(_) => (
                        best_tip_epoch.checked_add(1).expect("overflow"),
                        next_epoch_first_slot.checked_sub(1).expect("underflow"),
                        0,
                    ),
                    super::EpochContext::Waiting => todo!(),
                };
                state.status = BlockProducerVrfEvaluatorStatus::InitialSlotSelection {
                    time: meta.time(),
                    epoch_number,
                    initial_slot: initial_global_slot,
                };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                dispatcher.push(BlockProducerVrfEvaluatorEffectfulAction::InitializeStats {
                    epoch: epoch_number,
                    initial_slot,
                });
                if let Some(initial_slot) = state
                    .block_producer
                    .vrf_evaluator()
                    .and_then(|v| v.initial_slot())
                {
                    dispatcher.push(BlockProducerVrfEvaluatorAction::BeginEpochEvaluation {
                        best_tip_epoch: *best_tip_epoch,
                        best_tip_global_slot: *current_best_tip_global_slot,
                        best_tip_slot: *current_best_tip_slot,
                        evaluation_epoch: epoch_number,
                        staking_epoch_data: staking_epoch_data.clone(),
                        latest_evaluated_global_slot: initial_slot,
                    });
                }
            }
            BlockProducerVrfEvaluatorAction::CleanupOldSlots { best_tip_epoch } => {
                state.cleanup_old_won_slots(best_tip_epoch);
            }
            BlockProducerVrfEvaluatorAction::InterruptEpochEvaluation { reason } => {
                state.status = BlockProducerVrfEvaluatorStatus::EpochEvaluationInterrupted {
                    time: meta.time(),
                    reason: reason.clone(),
                };
            }
        }
    }
}
