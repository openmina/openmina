use openmina_core::bug_condition;
use redux::Dispatcher;

use crate::{
    ledger_effectful::LedgerEffectfulAction,
    transition_frontier::sync::{
        ledger::staged::TransitionFrontierSyncLedgerStagedAction, TransitionFrontierSyncAction,
    },
    Action, BlockProducerAction, State, Substate,
};

use super::{
    LedgerWriteAction, LedgerWriteActionWithMetaRef, LedgerWriteResponse, LedgerWriteState,
};

impl LedgerWriteState {
    pub fn reducer(mut state_context: Substate<Self>, action: LedgerWriteActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        let Ok(state) = state_context.get_substate_mut() else {
            return;
        };

        match action {
            LedgerWriteAction::Init { request, on_init } => {
                *state = Self::Init {
                    time: meta.time(),
                    request: request.clone(),
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(LedgerEffectfulAction::WriteInit {
                    request: request.clone(),
                    on_init: on_init.clone(),
                });
            }
            LedgerWriteAction::Pending => {
                if let Self::Init { request, .. } = state {
                    *state = Self::Pending {
                        time: meta.time(),
                        request: request.clone(),
                    };
                }
            }
            LedgerWriteAction::Success { response } => {
                if let Self::Pending { request, .. } = state {
                    *state = Self::Success {
                        time: meta.time(),
                        request: request.clone(),
                        response: response.clone(),
                    };
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                Self::propagate_write_response(dispatcher, state, response.clone());
                dispatcher.push(BlockProducerAction::StagedLedgerDiffCreateInit);
                dispatcher.push(TransitionFrontierSyncAction::BlocksNextApplyInit);
                dispatcher.push(TransitionFrontierSyncAction::CommitInit);
                dispatcher.push(TransitionFrontierSyncLedgerStagedAction::ReconstructInit);
            }
        }
    }

    fn propagate_write_response(
        dispatcher: &mut Dispatcher<Action, State>,
        state: &State,
        response: LedgerWriteResponse,
    ) {
        let Some(request) = &state.ledger.write.request() else {
            return;
        };
        match (request, response) {
            (
                _,
                LedgerWriteResponse::StagedLedgerReconstruct {
                    staged_ledger_hash,
                    result,
                },
            ) => {
                let sync = &state.transition_frontier.sync;
                let expected_ledger = sync
                    .ledger_target()
                    .and_then(|target| target.staged)
                    .map(|v| v.hashes.non_snark.ledger_hash);
                if expected_ledger.as_ref() == Some(&staged_ledger_hash) {
                    match result {
                        Err(error) => {
                            dispatcher.push(
                                TransitionFrontierSyncLedgerStagedAction::ReconstructError {
                                    error,
                                },
                            );
                        }
                        Ok(()) => {
                            dispatcher.push(
                                TransitionFrontierSyncLedgerStagedAction::ReconstructSuccess {
                                    ledger_hash: staged_ledger_hash,
                                },
                            );
                        }
                    }
                }
            }
            (
                _,
                LedgerWriteResponse::StagedLedgerDiffCreate {
                    pred_block_hash,
                    global_slot_since_genesis,
                    result,
                },
            ) => {
                let Some((expected_pred_block_hash, expected_global_slot)) = None.or_else(|| {
                    let pred_block = state.block_producer.current_parent_chain()?.last()?;
                    let won_slot = state.block_producer.current_won_slot()?;
                    let slot = won_slot.global_slot_since_genesis(pred_block.global_slot_diff());
                    Some((pred_block.hash(), slot))
                }) else {
                    return;
                };

                if &pred_block_hash == expected_pred_block_hash
                    && global_slot_since_genesis == expected_global_slot
                {
                    match result {
                        Err(err) => {
                            bug_condition!("StagedLedgerDiffCreate error: {err}");
                        }
                        Ok(output) => {
                            dispatcher.push(BlockProducerAction::StagedLedgerDiffCreateSuccess {
                                output,
                            });
                        }
                    }
                }
            }
            (
                _,
                LedgerWriteResponse::BlockApply {
                    block_hash: hash,
                    result,
                },
            ) => match result {
                Err(error) => {
                    dispatcher
                        .push(TransitionFrontierSyncAction::BlocksNextApplyError { hash, error });
                }
                Ok(result) => {
                    dispatcher.push(TransitionFrontierSyncAction::BlocksSendToArchive {
                        hash: hash.clone(),
                        data: result.clone(),
                    });
                    dispatcher.push(TransitionFrontierSyncAction::BlocksNextApplySuccess {
                        hash,
                        just_emitted_a_proof: result.just_emitted_a_proof,
                    });
                }
            },
            (
                _,
                LedgerWriteResponse::Commit {
                    best_tip_hash,
                    result,
                },
            ) => {
                let best_tip = state.transition_frontier.sync.best_tip();
                if best_tip.is_some_and(|tip| tip.hash() == &best_tip_hash) {
                    dispatcher.push(TransitionFrontierSyncAction::CommitSuccess { result });
                }
            }
        }
    }
}
