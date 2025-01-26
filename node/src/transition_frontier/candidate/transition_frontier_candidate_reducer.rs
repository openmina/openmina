use openmina_core::{
    block::{ArcBlockWithHash, BlockHash},
    bug_condition,
    consensus::{is_short_range_fork, long_range_fork_take, short_range_fork_take},
};
use snark::block_verify::{SnarkBlockVerifyAction, SnarkBlockVerifyError, SnarkBlockVerifyId};

use crate::{
    transition_frontier::sync::{
        ledger::{
            snarked::TransitionFrontierSyncLedgerSnarkedAction,
            staged::TransitionFrontierSyncLedgerStagedAction,
        },
        TransitionFrontierSyncAction,
    },
    WatchedAccountsAction,
};

use super::{
    ConsensusLongRangeForkDecision, ConsensusShortRangeForkDecision,
    TransitionFrontierCandidateAction, TransitionFrontierCandidateActionWithMetaRef,
    TransitionFrontierCandidateState, TransitionFrontierCandidateStatus,
    TransitionFrontierCandidatesState,
};

impl TransitionFrontierCandidatesState {
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: TransitionFrontierCandidateActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else {
            // TODO: log or propagate
            return;
        };
        let (action, meta) = action.split();

        match action {
            TransitionFrontierCandidateAction::BlockReceived {
                hash,
                block,
                chain_proof,
            } => {
                state.blocks.insert(
                    hash.clone(),
                    TransitionFrontierCandidateState {
                        block: block.clone(),
                        status: TransitionFrontierCandidateStatus::Received { time: meta.time() },
                        chain_proof: chain_proof.clone(),
                    },
                );

                // Dispatch
                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                let hash = hash.clone();
                let block = ArcBlockWithHash {
                    hash: hash.clone(),
                    block: block.clone(),
                };
                let allow_block_too_late = allow_block_too_late(state, &block);

                match state.prevalidate_block(&block, allow_block_too_late) {
                    Ok(()) => {
                        dispatcher.push(
                            TransitionFrontierCandidateAction::BlockPrevalidateSuccess { hash },
                        );
                    }
                    Err(error) => {
                        dispatcher.push(TransitionFrontierCandidateAction::BlockPrevalidateError {
                            hash,
                            error,
                        });
                    }
                }
            }
            TransitionFrontierCandidateAction::BlockPrevalidateSuccess { hash } => {
                let Some(block) = state.blocks.get_mut(hash) else {
                    return;
                };
                block.status = TransitionFrontierCandidateStatus::Prevalidated;

                // Dispatch
                let block = (hash.clone(), block.block.clone()).into();
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(SnarkBlockVerifyAction::Init {
                    block,
                    on_init: redux::callback!(
                        on_received_block_snark_verify_init((hash: BlockHash, req_id: SnarkBlockVerifyId)) -> crate::Action {
                            TransitionFrontierCandidateAction::BlockSnarkVerifyPending { hash, req_id }
                        }),
                    on_success: redux::callback!(
                        on_received_block_snark_verify_success(hash: BlockHash) -> crate::Action {
                            TransitionFrontierCandidateAction::BlockSnarkVerifySuccess { hash }
                        }),
                    on_error: redux::callback!(
                        on_received_block_snark_verify_error((hash: BlockHash, error: SnarkBlockVerifyError)) -> crate::Action {
                            TransitionFrontierCandidateAction::BlockSnarkVerifyError { hash, error }
                        }),
                });
            }
            TransitionFrontierCandidateAction::BlockPrevalidateError { hash, .. } => {
                state.blocks.remove(hash);
            }
            TransitionFrontierCandidateAction::BlockChainProofUpdate { hash, chain_proof } => {
                if state.best_tip.as_ref() == Some(hash) {
                    state.best_tip_chain_proof = Some(chain_proof.clone());
                } else if let Some(block) = state.blocks.get_mut(hash) {
                    block.chain_proof = Some(chain_proof.clone());
                }

                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                if global_state
                    .transition_frontier
                    .candidates
                    .best_tip
                    .as_ref()
                    != Some(hash)
                {
                    return;
                }

                dispatcher
                    .push(TransitionFrontierCandidateAction::TransitionFrontierSyncTargetUpdate);
            }
            TransitionFrontierCandidateAction::BlockSnarkVerifyPending { req_id, hash } => {
                if let Some(block) = state.blocks.get_mut(hash) {
                    block.status = TransitionFrontierCandidateStatus::SnarkVerifyPending {
                        time: meta.time(),
                        req_id: *req_id,
                    };
                }
            }
            TransitionFrontierCandidateAction::BlockSnarkVerifySuccess { hash } => {
                if let Some(block) = state.blocks.get_mut(hash) {
                    block.status =
                        TransitionFrontierCandidateStatus::SnarkVerifySuccess { time: meta.time() };
                }

                // Dispatch
                let hash = hash.clone();
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransitionFrontierCandidateAction::DetectForkRange { hash });
            }
            TransitionFrontierCandidateAction::BlockSnarkVerifyError { .. } => {
                // TODO: handle block verification error.
            }
            TransitionFrontierCandidateAction::DetectForkRange { hash } => {
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
                    candidate_state.status = TransitionFrontierCandidateStatus::ForkRangeDetected {
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
                dispatcher.push(TransitionFrontierCandidateAction::ShortRangeForkResolve {
                    hash: hash.clone(),
                });
                dispatcher.push(TransitionFrontierCandidateAction::LongRangeForkResolve { hash });
            }
            TransitionFrontierCandidateAction::ShortRangeForkResolve { hash } => {
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

                        candidate.status =
                            TransitionFrontierCandidateStatus::ShortRangeForkResolve {
                                time: meta.time(),
                                compared_with: best_tip_hash,
                                decision,
                            };
                    }
                }

                // Dispatch
                let hash = hash.clone();
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransitionFrontierCandidateAction::BestTipUpdate { hash });
            }
            TransitionFrontierCandidateAction::LongRangeForkResolve { hash } => {
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
                candidate_state.status = TransitionFrontierCandidateStatus::LongRangeForkResolve {
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
                dispatcher.push(TransitionFrontierCandidateAction::BestTipUpdate { hash });
            }
            TransitionFrontierCandidateAction::BestTipUpdate { hash } => {
                state.best_tip = Some(hash.clone());

                if let Some(tip) = state.blocks.get_mut(hash) {
                    state.best_tip_chain_proof = tip.chain_proof.take();
                }

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let Some(block) = global_state
                    .transition_frontier
                    .candidates
                    .best_tip_block_with_hash()
                else {
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

                dispatcher
                    .push(TransitionFrontierCandidateAction::TransitionFrontierSyncTargetUpdate);
            }
            TransitionFrontierCandidateAction::TransitionFrontierSyncTargetUpdate => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let Some(best_tip) = state
                    .transition_frontier
                    .candidates
                    .best_tip_block_with_hash()
                else {
                    bug_condition!(
                        "ConsensusAction::TransitionFrontierSyncTargetUpdate | no chosen best tip"
                    );
                    return;
                };

                let Some((blocks_inbetween, root_block)) = state
                    .transition_frontier
                    .candidates
                    .best_tip_chain_proof(&state.transition_frontier)
                else {
                    bug_condition!("ConsensusAction::TransitionFrontierSyncTargetUpdate | no best tip chain proof");
                    return;
                };

                let previous_root_snarked_ledger_hash = state
                    .transition_frontier
                    .root()
                    .map(|b| b.snarked_ledger_hash().clone());

                dispatcher.push(TransitionFrontierSyncAction::BestTipUpdate {
                    previous_root_snarked_ledger_hash,
                    best_tip,
                    root_block,
                    blocks_inbetween,
                    on_success: None,
                });
            }
            TransitionFrontierCandidateAction::P2pBestTipUpdate { best_tip } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransitionFrontierCandidateAction::BlockReceived {
                    hash: best_tip.hash.clone(),
                    block: best_tip.block.clone(),
                    chain_proof: None,
                });

                dispatcher.push(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
                dispatcher.push(TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit);
                dispatcher.push(TransitionFrontierSyncAction::BlocksPeersQuery);
            }
            TransitionFrontierCandidateAction::Prune => {
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

/// Decide if the time-reception check should be done for this block or not.
///
/// The check is skipped if the block's global_slot is greater than the
/// current best tip and the difference greater than 2.
///
/// Ideally we would differentiate between requested blocks and blocks
/// received from gossip, but this difference doesn't really exist
/// in the WebRTC transport, hence this heuristic.
pub fn allow_block_too_late(state: &crate::State, block: &ArcBlockWithHash) -> bool {
    let (has_greater_blobal_slot, diff_with_best_tip) = state
        .transition_frontier
        .best_tip()
        .map(|b| {
            (
                block.global_slot() > b.global_slot(),
                b.global_slot().abs_diff(block.global_slot()),
            )
        })
        .unwrap_or((false, 0));

    has_greater_blobal_slot && diff_with_best_tip > 2
}
