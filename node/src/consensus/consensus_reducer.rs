use openmina_core::{
    block::BlockHash,
    consensus::{is_short_range_fork, long_range_fork_take, short_range_fork_take},
};
use snark::block_verify::{SnarkBlockVerifyAction, SnarkBlockVerifyError};

use crate::{
    transition_frontier::sync::TransitionFrontierSyncAction, Action, State, WatchedAccountsAction,
};

use super::{
    ConsensusAction, ConsensusActionWithMetaRef, ConsensusBlockState, ConsensusBlockStatus,
    ConsensusLongRangeForkDecision, ConsensusShortRangeForkDecision, ConsensusState,
};

impl ConsensusState {
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: ConsensusActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else {
            // TODO: log or propagate
            return;
        };
        let (action, meta) = action.split();

        match action {
            ConsensusAction::BlockReceived {
                hash,
                block,
                chain_proof,
            } => {
                state.blocks.insert(
                    hash.clone(),
                    ConsensusBlockState {
                        block: block.clone(),
                        status: ConsensusBlockStatus::Received { time: meta.time() },
                        chain_proof: chain_proof.clone(),
                    },
                );

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let req_id = global_state.snark.block_verify.next_req_id();
                dispatcher.push(SnarkBlockVerifyAction::Init {
                    req_id,
                    block: (hash.clone(), block.clone()).into(),
                    on_success: redux::callback!(
                        on_received_block_snark_verify_success(hash: BlockHash) -> crate::Action {
                            ConsensusAction::BlockSnarkVerifySuccess { hash }
                        }),
                    on_error: redux::callback!(
                        on_received_block_snark_verify_error((hash: BlockHash, error: SnarkBlockVerifyError)) -> crate::Action {
                            ConsensusAction::BlockSnarkVerifyError { hash, error }
                        }),
                });
                dispatcher.push(ConsensusAction::BlockSnarkVerifyPending {
                    req_id,
                    hash: hash.clone(),
                });
            }
            ConsensusAction::BlockChainProofUpdate { hash, chain_proof } => {
                if state.best_tip.as_ref() == Some(hash) {
                    state.best_tip_chain_proof = Some(chain_proof.clone());
                } else if let Some(block) = state.blocks.get_mut(hash) {
                    block.chain_proof = Some(chain_proof.clone());
                }

                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                if global_state.consensus.best_tip.as_ref() != Some(hash) {
                    return;
                }

                transition_frontier_new_best_tip_handler(global_state, dispatcher);
            }
            ConsensusAction::BlockSnarkVerifyPending { req_id, hash } => {
                if let Some(block) = state.blocks.get_mut(hash) {
                    block.status = ConsensusBlockStatus::SnarkVerifyPending {
                        time: meta.time(),
                        req_id: *req_id,
                    };
                }
            }
            ConsensusAction::BlockSnarkVerifySuccess { hash } => {
                if let Some(block) = state.blocks.get_mut(hash) {
                    block.status = ConsensusBlockStatus::SnarkVerifySuccess { time: meta.time() };
                }

                // Dispatch
                let hash = hash.clone();
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(ConsensusAction::DetectForkRange { hash });
            }
            ConsensusAction::BlockSnarkVerifyError { .. } => {
                // TODO: handle block verification error.
            }
            ConsensusAction::DetectForkRange { hash } => {
                let candidate_hash = hash;
                let Some(candidate_state) = state.blocks.get(candidate_hash) else {
                    return;
                };
                let candidate = &candidate_state.block.header;
                let (tip_hash, short_fork) = if let Some(tip_ref) = state.best_tip() {
                    let tip = tip_ref.header;
                    (
                        Some(tip_ref.hash.clone()),
                        is_short_range_fork(
                            &candidate.protocol_state.body.consensus_state,
                            &tip.protocol_state.body.consensus_state,
                        ),
                    )
                } else {
                    (None, true)
                };
                if let Some(candidate_state) = state.blocks.get_mut(candidate_hash) {
                    candidate_state.status = ConsensusBlockStatus::ForkRangeDetected {
                        time: meta.time(),
                        compared_with: tip_hash,
                        short_fork,
                    };
                    openmina_core::log::debug!(openmina_core::log::system_time(); kind = "ConsensusAction::DetectForkRange", status = serde_json::to_string(&candidate_state.status).unwrap());
                }
                openmina_core::log::debug!(openmina_core::log::system_time(); kind = "ConsensusAction::DetectForkRange");

                // Dispatch
                let hash = hash.clone();
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(ConsensusAction::ShortRangeForkResolve { hash: hash.clone() });
                dispatcher.push(ConsensusAction::LongRangeForkResolve { hash });
            }
            ConsensusAction::ShortRangeForkResolve { hash } => {
                let candidate_hash = hash;
                if let Some(candidate) = state.blocks.get(candidate_hash) {
                    let (best_tip_hash, decision): (_, ConsensusShortRangeForkDecision) =
                        match state.best_tip() {
                            Some(tip) => (Some(tip.hash.clone()), {
                                let tip_cs = &tip.header.protocol_state.body.consensus_state;
                                let candidate_cs =
                                    &candidate.block.header.protocol_state.body.consensus_state;
                                let (take, why) = short_range_fork_take(
                                    tip_cs,
                                    candidate_cs,
                                    tip.hash,
                                    candidate_hash,
                                );
                                if take {
                                    ConsensusShortRangeForkDecision::Take(why)
                                } else {
                                    ConsensusShortRangeForkDecision::Keep(why)
                                }
                            }),
                            None => (None, ConsensusShortRangeForkDecision::TakeNoBestTip),
                        };
                    if let Some(best_tip_hash) = &best_tip_hash {
                        openmina_core::log::info!(openmina_core::log::system_time(); best_tip_hash = best_tip_hash.to_string(), candidate_hash = candidate_hash.to_string(), decision = format!("{decision:?}"));
                    }
                    if let Some(candidate) = state.blocks.get_mut(candidate_hash) {
                        if !decision.use_as_best_tip() {
                            candidate.chain_proof = None;
                        }

                        candidate.status = ConsensusBlockStatus::ShortRangeForkResolve {
                            time: meta.time(),
                            compared_with: best_tip_hash,
                            decision,
                        };
                    }
                }

                // Dispatch
                let hash = hash.clone();
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(ConsensusAction::BestTipUpdate { hash });
            }
            ConsensusAction::LongRangeForkResolve { hash } => {
                openmina_core::log::debug!(openmina_core::log::system_time(); kind = "ConsensusAction::LongRangeForkResolve");
                let candidate_hash = hash;
                let Some(tip_ref) = state.best_tip() else {
                    return;
                };
                let Some(candidate_state) = state.blocks.get(candidate_hash) else {
                    return;
                };
                openmina_core::log::debug!(openmina_core::log::system_time(); kind = "ConsensusAction::LongRangeForkResolve", pre_status = serde_json::to_string(&candidate_state.status).unwrap());
                let tip_hash = tip_ref.hash.clone();
                let tip = tip_ref.header;
                let tip_cs = &tip.protocol_state.body.consensus_state;
                let candidate = &candidate_state.block.header;
                let candidate_cs = &candidate.protocol_state.body.consensus_state;

                let (take, why) =
                    long_range_fork_take(tip_cs, candidate_cs, &tip_hash, candidate_hash);

                let Some(candidate_state) = state.blocks.get_mut(candidate_hash) else {
                    return;
                };
                candidate_state.status = ConsensusBlockStatus::LongRangeForkResolve {
                    time: meta.time(),
                    compared_with: tip_hash,
                    decision: if take {
                        ConsensusLongRangeForkDecision::Take(why)
                    } else {
                        candidate_state.chain_proof = None;
                        ConsensusLongRangeForkDecision::Keep(why)
                    },
                };
                openmina_core::log::debug!(openmina_core::log::system_time(); kind = "ConsensusAction::LongRangeForkResolve", status = serde_json::to_string(&candidate_state.status).unwrap());

                // Dispatch
                let hash = hash.clone();
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(ConsensusAction::BestTipUpdate { hash });
            }
            ConsensusAction::BestTipUpdate { hash } => {
                state.best_tip = Some(hash.clone());

                if let Some(tip) = state.blocks.get_mut(hash) {
                    state.best_tip_chain_proof = tip.chain_proof.take();
                }

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let Some(block) = global_state.consensus.best_tip_block_with_hash() else {
                    return;
                };
                for pub_key in global_state.watched_accounts.accounts() {
                    dispatcher.push(WatchedAccountsAction::LedgerInitialStateGetInit {
                        pub_key: pub_key.clone(),
                    });
                    dispatcher.push(WatchedAccountsAction::TransactionsIncludedInBlock {
                        pub_key,
                        block: block.clone(),
                    });
                }

                transition_frontier_new_best_tip_handler(global_state, dispatcher);
            }
            ConsensusAction::Prune => {
                let Some(best_tip_hash) = state.best_tip.clone() else {
                    return;
                };
                let blocks = &mut state.blocks;

                // keep at most latest 32 candidate blocks.
                let blocks_to_keep = (0..32)
                    .scan(best_tip_hash, |block_hash, _| {
                        let block_state = blocks.remove(block_hash)?;
                        let block_hash = match block_state.status.compared_with() {
                            None => block_hash.clone(),
                            Some(compared_with) => {
                                std::mem::replace(block_hash, compared_with.clone())
                            }
                        };
                        Some((block_hash, block_state))
                    })
                    .collect();
                *blocks = blocks_to_keep;
            }
        }
    }
}

fn transition_frontier_new_best_tip_handler(
    state: &State,
    dispatcher: &mut redux::Dispatcher<Action, State>,
) {
    let Some(best_tip) = state.consensus.best_tip_block_with_hash() else {
        return;
    };
    let pred_hash = best_tip.pred_hash();

    let Some((blocks_inbetween, root_block)) =
        state.consensus.best_tip_chain_proof.clone().or_else(|| {
            let old_best_tip = state.transition_frontier.best_tip()?;
            let mut iter = state.transition_frontier.best_chain.iter();
            if old_best_tip.hash() == pred_hash {
                if old_best_tip.height() > old_best_tip.constants().k.as_u32() {
                    iter.next();
                }
                let root_block = iter.next()?.clone();
                let hashes = iter.map(|b| b.hash.clone()).collect();
                Some((hashes, root_block))
            } else if old_best_tip.pred_hash() == pred_hash {
                let root_block = iter.next()?.clone();
                let hashes = iter.rev().skip(1).rev().map(|b| b.hash.clone()).collect();
                Some((hashes, root_block))
            } else {
                None
            }
        })
    else {
        return;
    };

    if !state.transition_frontier.sync.is_pending() && !state.transition_frontier.sync.is_synced() {
        dispatcher.push(TransitionFrontierSyncAction::Init {
            best_tip,
            root_block,
            blocks_inbetween,
        });
    } else {
        let previous_root_snarked_ledger_hash = state
            .transition_frontier
            .root()
            .map(|b| b.snarked_ledger_hash().clone());

        dispatcher.push(TransitionFrontierSyncAction::BestTipUpdate {
            previous_root_snarked_ledger_hash,
            best_tip,
            root_block,
            blocks_inbetween,
        });
    }
}
