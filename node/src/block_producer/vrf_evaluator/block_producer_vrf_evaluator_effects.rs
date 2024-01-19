use std::sync::Arc;

use redux::ActionMeta;

use crate::block_producer::to_epoch_and_slot;
use crate::block_producer::vrf_evaluator::VrfEvaluatorInput;
use crate::Service;
use crate::Store;

use super::BlockProducerVrfEvaluatorAction;

impl BlockProducerVrfEvaluatorAction {
    pub fn effects<S: Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        match self {
            BlockProducerVrfEvaluatorAction::EvaluateVrf { vrf_input } => {
                store.service.evaluate(vrf_input);
            }
            BlockProducerVrfEvaluatorAction::EvaluationSuccess { .. } => {
                if let Some(vrf_evaluator_state) = store.state().block_producer.vrf_evaluator() {
                    if let Some(vrf_input) = vrf_evaluator_state.status.construct_vrf_input() {
                        store.dispatch(BlockProducerVrfEvaluatorAction::EvaluateEpoch { vrf_input });
                    }
                }
            }
            BlockProducerVrfEvaluatorAction::EvaluatorInit { best_tip } => {
                if store.state().block_producer.vrf_evaluator().is_some() {
                    if self.best_tip.consensus_state().epoch_count.as_u32() == 0 {
                        store.dispatch(BlockProducerVrfEvaluatorAction::EvaluatorInitSuccess {
                            previous_epoch_and_height: None,
                        });
                    } else {
                        let k = self.best_tip.constants().k.as_u32();
                        let (epoch, slot) =
                            to_epoch_and_slot(&self.best_tip.consensus_state().curr_global_slot);
                        let last_height = if slot < k {
                            store
                                .state()
                                .transition_frontier
                                .best_chain
                                .iter()
                                .rev()
                                .find(|b| b.consensus_state().epoch_count.as_u32() == epoch - 1)
                                .unwrap()
                                .height()
                        } else {
                            store
                                .state()
                                .transition_frontier
                                .sync
                                .root_block()
                                .unwrap()
                                .height()
                        };
                        store.dispatch(BlockProducerVrfEvaluatorAction::EvaluatorInitSuccess {
                            previous_epoch_and_height: Some((epoch - 1, last_height)),
                        });
                    }
                }
            }
            BlockProducerVrfEvaluatorAction::EvaluatorInitSuccess { .. } => {}
            BlockProducerVrfEvaluatorAction::CanEvaluateVrf {
                staking_epoch_data,
                next_epoch_data,
                current_best_tip_height,
                current_best_tip_global_slot,
                current_epoch_number,
                current_best_tip_slot,
                transition_frontier_size,
                next_epoch_first_slot,
            } => {
                let vrf_evaluator_state = store.state().block_producer.vrf_evaluator_with_config();

                if let Some((vrf_evaluator_state, config)) = vrf_evaluator_state {
                    let epoch_data = match vrf_evaluator_state.status.epoch_to_evaluate() {
                        EpochContext::Current => EpochData::new(
                            staking_epoch_data.seed.to_string(),
                            staking_epoch_data.ledger.hash,
                            staking_epoch_data.ledger.total_currency.as_u64(),
                        ),
                        EpochContext::Next => EpochData::new(
                            next_epoch_data.seed.to_string(),
                            next_epoch_data.ledger.hash,
                            next_epoch_data.ledger.total_currency.as_u64(),
                        ),
                        EpochContext::Waiting => {
                            return;
                        }
                    };
                    store.dispatch(BlockProducerVrfEvaluatorAction::EvaluateEpochInit {
                        epoch_context: vrf_evaluator_state.status.epoch_to_evaluate(),
                        staking_epoch_data: epoch_data,
                        producer: config.pub_key.clone().into(),
                        current_best_tip_height,
                        current_best_tip_global_slot,
                        current_epoch_number,
                        current_best_tip_slot,
                        transition_frontier_size,
                        next_epoch_first_slot,
                    });
                }
            }
            BlockProducerVrfEvaluatorAction::EvaluateEpochInit {
                epoch_context,
                staking_epoch_data,
                producer,
                current_epoch_number,
                current_best_tip_height,
                current_best_tip_global_slot,
                current_best_tip_slot,
                transition_frontier_size,
                next_epoch_first_slot
            } => {
                store.dispatch(BlockProducerVrfEvaluatorAction::ConstructDelegatorTable {
                    epoch_context,
                    staking_epoch_data,
                    producer,
                    current_best_tip_height,
                    current_best_tip_global_slot,
                    current_best_tip_slot,
                    current_epoch_number,
                    transition_frontier_size,
                    next_epoch_first_slot,
                });
            }
            BlockProducerVrfEvaluatorAction::ConstructDelegatorTable {
                epoch_context,
                staking_epoch_data,
                producer,
                current_epoch_number,
                current_best_tip_height,
                current_best_tip_slot,
                current_best_tip_global_slot,
                next_epoch_first_slot,
                transition_frontier_size,
            } => {
                let delegator_table = store.service().get_producer_and_delegates(
                    self.staking_epoch_data.ledger.clone(),
                    self.producer.clone(),
                );
                let mut epoch_data = self.staking_epoch_data.clone();
                epoch_data.delegator_table = Arc::new(delegator_table);
        
                store.dispatch(BlockProducerVrfEvaluatorAction::ConstructDelegatorTableSuccess {
                    epoch_context,
                    staking_epoch_data: epoch_data,
                    producer,
                    current_epoch_number,
                    current_best_tip_height,
                    current_best_tip_slot,
                    current_best_tip_global_slot,
                    next_epoch_first_slot,
                    transition_frontier_size,
                });
            }
            BlockProducerVrfEvaluatorAction::ConstructDelegatorTableSuccess {
                current_best_tip_height,
                current_best_tip_global_slot,
                current_best_tip_slot,
                current_epoch_number,
                staking_epoch_data,
                next_epoch_first_slot,
            } => {
                if let (Some(vrf_evaluator_state), Some(current_global_slot)) = (
                    store.state().block_producer.vrf_evaluator(),
                    store.state().cur_global_slot(),
                ) {
                    let latest_evaluated_global_slot = match vrf_evaluator_state.status.epoch_context() {
                        // Note: BlockProducerVrfEvaluateEpochAction increments the slot at each dispatch
                        // For current epoch evaluation start at the current_global slot
                        EpochContext::Current => current_global_slot,
                        EpochContext::Next => next_epoch_first_slot - 1,
                        EpochContext::Waiting => return,
                    };
        
                    store.dispatch(BlockProducerVrfEvaluatorAction::EvaluateEpoch {
                        epoch_context: vrf_evaluator_state.status.epoch_context(),
                        current_best_tip_heigh,
                        current_best_tip_global_slot,
                        current_best_tip_slot,
                        current_epoch_number,
                        staking_epoch_data,
                        latest_evaluated_global_slot,
                    });
                }
            }
            BlockProducerVrfEvaluatorAction::EvaluateEpoch {
                staking_epoch_data,
                latest_evaluated_global_slot,
                ..
            } => {
                let next_slot = latest_evaluated_global_slot + 1;

                let vrf_input = VrfEvaluatorInput::new(
                    staking_epoch_data.seed,
                    staking_epoch_data.delegator_table.clone(),
                    next_slot,
                    staking_epoch_data.total_currency,
                    staking_epoch_data.ledger,
                );
                store.dispatch(BlockProducerVrfEvaluatorAction::EvaluateVrf { vrf_input });
            }
            BlockProducerVrfEvaluatorAction::SaveLastBlockHeightInEpoch { .. } => {}
        }
    }
}
