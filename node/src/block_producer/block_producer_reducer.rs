use ledger::scan_state::currency::{Amount, Signed};
use mina_p2p_messages::{
    list::List,
    v2::{
        ConsensusProofOfStakeDataConsensusStateValueStableV2,
        ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
        ConsensusVrfOutputTruncatedStableV1, LedgerProofProdStableV2,
        MinaBaseEpochLedgerValueStableV1, MinaStateBlockchainStateValueStableV2,
        MinaStateBlockchainStateValueStableV2LedgerProofStatement,
        MinaStateProtocolStateBodyValueStableV2, MinaStateProtocolStateValueStableV2,
        StagedLedgerDiffBodyStableV1, StateBodyHash, StateHash, UnsignedExtendedUInt32StableV1,
    },
};
use openmina_core::{
    block::ArcBlockWithHash, consensus::ConsensusConstants, constants::constraint_constants,
};
use openmina_core::{
    bug_condition,
    consensus::{
        global_sub_window, in_same_checkpoint_window, in_seed_update_range, relative_sub_window,
    },
};
use redux::{callback, Dispatcher, Timestamp};

use crate::{
    transition_frontier::sync::TransitionFrontierSyncAction, Action, BlockProducerEffectfulAction,
    State, Substate, TransactionPoolAction,
};

use super::{
    calc_epoch_seed, next_epoch_first_slot, to_epoch_and_slot,
    vrf_evaluator::{
        BlockProducerVrfEvaluatorAction, BlockProducerVrfEvaluatorState, InterruptReason,
    },
    BlockProducerAction, BlockProducerActionWithMetaRef, BlockProducerCurrentState,
    BlockProducerEnabled, BlockProducerState, BlockWithoutProof,
};

impl BlockProducerState {
    pub fn reducer(state_context: Substate<State>, action: BlockProducerActionWithMetaRef<'_>) {
        BlockProducerEnabled::reducer(state_context, action);
    }
}

impl BlockProducerEnabled {
    /// Substate is accesses from global state, because applied blocks from transition frontier are required
    pub fn reducer(mut state_context: Substate<State>, action: BlockProducerActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        let Ok(global_state) = state_context.get_substate_mut() else {
            return;
        };
        let consensus_constants = &global_state.config.consensus_constants;

        let best_chain = &global_state.transition_frontier.best_chain;
        let Some(state) = global_state.block_producer.as_mut() else {
            return;
        };

        match action {
            BlockProducerAction::VrfEvaluator(action) => {
                BlockProducerVrfEvaluatorState::reducer(
                    Substate::from_compatible_substate(state_context),
                    meta.with_action(action),
                );
            }
            BlockProducerAction::BestTipUpdate { best_tip } => {
                state.injected_blocks.remove(best_tip.hash());
                // set the genesis timestamp on the first best tip update
                // TODO: move/remove once we can generate the genesis block
                if state.vrf_evaluator.genesis_timestamp == redux::Timestamp::ZERO {
                    state.vrf_evaluator.genesis_timestamp = best_tip.genesis_timestamp();
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                Self::dispatch_best_tip_update(dispatcher, state, best_tip);
            }
            BlockProducerAction::WonSlotSearch => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                if let Some(won_slot) = state.block_producer.with(None, |bp| {
                    let best_tip = state.transition_frontier.best_tip()?;
                    let cur_global_slot = state.cur_global_slot()?;
                    bp.vrf_evaluator.next_won_slot(cur_global_slot, best_tip)
                }) {
                    dispatcher.push(BlockProducerAction::WonSlot { won_slot });
                }
            }
            BlockProducerAction::WonSlot { won_slot } => {
                state.current = BlockProducerCurrentState::WonSlot {
                    time: meta.time(),
                    won_slot: won_slot.clone(),
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(BlockProducerEffectfulAction::WonSlot {
                    won_slot: won_slot.clone(),
                });
            }
            BlockProducerAction::WonSlotDiscard { reason } => {
                if let Some(won_slot) = state.current.won_slot() {
                    state.current = BlockProducerCurrentState::WonSlotDiscarded {
                        time: meta.time(),
                        won_slot: won_slot.clone(),
                        reason: *reason,
                    };
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(BlockProducerEffectfulAction::WonSlotDiscard { reason: *reason });
            }
            BlockProducerAction::WonSlotWait => {
                if let Some(won_slot) = state.current.won_slot() {
                    state.current = BlockProducerCurrentState::WonSlotWait {
                        time: meta.time(),
                        won_slot: won_slot.clone(),
                    };
                }
            }
            BlockProducerAction::WonSlotTransactionsGet => {
                let BlockProducerCurrentState::WonSlotProduceInit {
                    won_slot, chain, ..
                } = &mut state.current
                else {
                    return;
                };

                state.current = BlockProducerCurrentState::WonSlotTransactionsGet {
                    time: meta.time(),
                    won_slot: won_slot.clone(),
                    chain: chain.clone(),
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransactionPoolAction::CollectTransactionsByFee);
            }
            BlockProducerAction::WonSlotTransactionsSuccess {
                transactions_by_fee,
            } => {
                let BlockProducerCurrentState::WonSlotTransactionsGet {
                    won_slot, chain, ..
                } = &mut state.current
                else {
                    return;
                };

                state.current = BlockProducerCurrentState::WonSlotTransactionsSuccess {
                    time: meta.time(),
                    won_slot: won_slot.clone(),
                    chain: chain.clone(),
                    transactions_by_fee: transactions_by_fee.clone(),
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(BlockProducerAction::StagedLedgerDiffCreateInit);
            }
            BlockProducerAction::WonSlotProduceInit => {
                if let Some(won_slot) = state.current.won_slot() {
                    if let Some(chain) = best_chain.last().map(|best_tip| {
                        if best_tip.global_slot() == won_slot.global_slot() {
                            // We are producing block which replaces current best tip
                            // instead of extending it.
                            best_chain[..(best_chain.len() - 1)].to_vec()
                        } else {
                            best_chain.to_vec()
                        }
                    }) {
                        state.current = BlockProducerCurrentState::WonSlotProduceInit {
                            time: meta.time(),
                            won_slot: won_slot.clone(),
                            chain,
                        };
                    };
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(BlockProducerAction::WonSlotTransactionsGet);
            }
            BlockProducerAction::StagedLedgerDiffCreateInit => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(BlockProducerEffectfulAction::StagedLedgerDiffCreateInit);
            }
            BlockProducerAction::StagedLedgerDiffCreatePending => {
                let BlockProducerCurrentState::WonSlotTransactionsSuccess {
                    won_slot,
                    chain,
                    transactions_by_fee,
                    ..
                } = &mut state.current
                else {
                    return;
                };
                state.current = BlockProducerCurrentState::StagedLedgerDiffCreatePending {
                    time: meta.time(),
                    won_slot: won_slot.clone(),
                    chain: std::mem::take(chain),
                    transactions_by_fee: transactions_by_fee.to_vec(),
                };
            }
            BlockProducerAction::StagedLedgerDiffCreateSuccess { output } => {
                let BlockProducerCurrentState::StagedLedgerDiffCreatePending {
                    won_slot,
                    chain,
                    ..
                } = &mut state.current
                else {
                    return;
                };
                state.current = BlockProducerCurrentState::StagedLedgerDiffCreateSuccess {
                    time: meta.time(),
                    won_slot: won_slot.clone(),
                    chain: std::mem::take(chain),
                    diff: output.diff.clone(),
                    diff_hash: output.diff_hash.clone(),
                    staged_ledger_hash: output.staged_ledger_hash.clone(),
                    emitted_ledger_proof: output.emitted_ledger_proof.clone(),
                    pending_coinbase_update: output.pending_coinbase_update.clone(),
                    pending_coinbase_witness: output.pending_coinbase_witness.clone(),
                    stake_proof_sparse_ledger: output.stake_proof_sparse_ledger.clone(),
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(BlockProducerEffectfulAction::StagedLedgerDiffCreateSuccess);
            }
            BlockProducerAction::BlockUnprovenBuild => {
                state.reduce_block_unproved_build(consensus_constants, meta.time());

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(BlockProducerEffectfulAction::BlockUnprovenBuild);
            }
            BlockProducerAction::BlockProveInit => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(BlockProducerEffectfulAction::BlockProveInit);
            }
            BlockProducerAction::BlockProvePending => {
                if let BlockProducerCurrentState::BlockUnprovenBuilt {
                    won_slot,
                    chain,
                    emitted_ledger_proof,
                    pending_coinbase_update,
                    pending_coinbase_witness,
                    stake_proof_sparse_ledger,
                    block,
                    block_hash,
                    ..
                } = std::mem::take(&mut state.current)
                {
                    state.current = BlockProducerCurrentState::BlockProvePending {
                        time: meta.time(),
                        won_slot,
                        chain,
                        emitted_ledger_proof,
                        pending_coinbase_update,
                        pending_coinbase_witness,
                        stake_proof_sparse_ledger,
                        block,
                        block_hash,
                    };
                }
            }
            BlockProducerAction::BlockProveSuccess { proof } => {
                if let BlockProducerCurrentState::BlockProvePending {
                    won_slot,
                    chain,
                    block,
                    block_hash,
                    ..
                } = std::mem::take(&mut state.current)
                {
                    state.current = BlockProducerCurrentState::BlockProveSuccess {
                        time: meta.time(),
                        won_slot,
                        chain,
                        block,
                        block_hash,
                        proof: proof.clone(),
                    };
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(BlockProducerEffectfulAction::BlockProveSuccess);
            }
            BlockProducerAction::BlockProduced => {
                if let BlockProducerCurrentState::BlockProveSuccess {
                    won_slot,
                    chain,
                    block,
                    block_hash,
                    proof,
                    ..
                } = std::mem::take(&mut state.current)
                {
                    state.current = BlockProducerCurrentState::Produced {
                        time: meta.time(),
                        won_slot,
                        chain,
                        block: block.with_hash_and_proof(block_hash, *proof),
                    };
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(BlockProducerAction::BlockInject);
            }
            BlockProducerAction::BlockInject => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                let Some((best_tip, root_block, blocks_inbetween)) = None.or_else(|| {
                    let (best_tip, chain) = state.block_producer.produced_block_with_chain()?;
                    let mut iter = chain.iter();
                    let root_block = iter.next()?.block_with_hash();
                    let blocks_inbetween = iter.map(|b| b.hash().clone()).collect();
                    Some((best_tip.clone(), root_block.clone(), blocks_inbetween))
                }) else {
                    return;
                };

                let previous_root_snarked_ledger_hash = state
                    .transition_frontier
                    .root()
                    .map(|b| b.snarked_ledger_hash().clone());

                dispatcher.push(TransitionFrontierSyncAction::BestTipUpdate {
                    previous_root_snarked_ledger_hash,
                    best_tip: best_tip.clone(),
                    root_block,
                    blocks_inbetween,
                    on_success: Some(callback!(
                        on_transition_frontier_sync_best_tip_update(_p: ()) -> crate::Action{
                            BlockProducerAction::BlockInjected
                        }
                    )),
                });
            }
            BlockProducerAction::BlockInjected => {
                if let BlockProducerCurrentState::Produced {
                    won_slot,
                    chain,
                    block,
                    ..
                } = &mut state.current
                {
                    state.injected_blocks.insert(block.hash().clone());
                    state.current = BlockProducerCurrentState::Injected {
                        time: meta.time(),
                        won_slot: won_slot.clone(),
                        chain: std::mem::take(chain),
                        block: block.clone(),
                    };
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(BlockProducerAction::WonSlotSearch);
            }
        }
    }

    fn reduce_block_unproved_build(
        &mut self,
        consensus_constants: &ConsensusConstants,
        time: Timestamp,
    ) {
        let BlockProducerCurrentState::StagedLedgerDiffCreateSuccess {
            won_slot,
            chain,
            diff,
            diff_hash,
            staged_ledger_hash,
            emitted_ledger_proof,
            pending_coinbase_update,
            pending_coinbase_witness,
            stake_proof_sparse_ledger,
            ..
        } = std::mem::take(&mut self.current)
        else {
            return;
        };
        let Some(pred_block) = chain.last() else {
            return;
        };

        let pred_consensus_state = &pred_block.header().protocol_state.body.consensus_state;
        let pred_blockchain_state = &pred_block.header().protocol_state.body.blockchain_state;

        let genesis_ledger_hash = &pred_blockchain_state.genesis_ledger_hash;

        let block_timestamp = won_slot.timestamp();
        let pred_global_slot = pred_consensus_state
            .curr_global_slot_since_hard_fork
            .clone();
        let curr_global_slot_since_hard_fork = won_slot.global_slot.clone();
        let global_slot_since_genesis =
            won_slot.global_slot_since_genesis(pred_block.global_slot_diff());
        let (pred_epoch, _) = to_epoch_and_slot(&pred_global_slot);
        let (next_epoch, next_slot) = to_epoch_and_slot(&curr_global_slot_since_hard_fork);
        let has_ancestor_in_same_checkpoint_window =
            in_same_checkpoint_window(&pred_global_slot, &curr_global_slot_since_hard_fork);

        let block_stake_winner = won_slot.delegator.0.clone();
        let vrf_truncated_output: ConsensusVrfOutputTruncatedStableV1 =
            (*won_slot.vrf_output).clone().into();
        let vrf_hash = won_slot.vrf_output.hash();
        let block_creator = self.config.pub_key.clone();
        let coinbase_receiver = self.config.coinbase_receiver().clone();
        let proposed_protocol_version_opt = self.config.proposed_protocol_version.clone();

        let ledger_proof_statement = ledger_proof_statement_from_emitted_proof(
            emitted_ledger_proof.as_deref(),
            &pred_blockchain_state.ledger_proof_statement,
        );

        let supply_increase = emitted_ledger_proof.as_ref().map_or(Signed::zero(), |v| {
            Signed::from(&v.statement.supply_increase)
        });

        let total_currency = {
            let (amount, overflowed) = Amount::from(pred_consensus_state.total_currency.clone())
                .add_signed_flagged(supply_increase);
            if overflowed {
                todo!("total_currency overflowed");
            }
            amount
        };

        let (staking_epoch_data, next_epoch_data, epoch_count) = {
            let next_staking_ledger = if pred_block.snarked_ledger_hash() == genesis_ledger_hash {
                pred_consensus_state.next_epoch_data.ledger.clone()
            } else {
                MinaBaseEpochLedgerValueStableV1 {
                    hash: pred_block.snarked_ledger_hash().clone(),
                    total_currency: (&total_currency).into(),
                }
            };
            let (staking_data, next_data, epoch_count) = if next_epoch > pred_epoch {
                let staking_data =
                    next_to_staking_epoch_data(&pred_consensus_state.next_epoch_data);
                let next_data = ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 {
                    seed: pred_consensus_state.next_epoch_data.seed.clone(),
                    ledger: next_staking_ledger,
                    start_checkpoint: pred_block.hash().clone(),
                    // comment from Mina repo: (* TODO: We need to make sure issue #2328 is properly addressed. *)
                    lock_checkpoint: StateHash::zero(),
                    epoch_length: UnsignedExtendedUInt32StableV1(1.into()),
                };
                let epoch_count = UnsignedExtendedUInt32StableV1(
                    (pred_consensus_state.epoch_count.as_u32() + 1).into(),
                );
                (staking_data, next_data, epoch_count)
            } else {
                assert_eq!(pred_epoch, next_epoch);
                let mut next_data = pred_consensus_state.next_epoch_data.clone();
                next_data.epoch_length =
                    UnsignedExtendedUInt32StableV1((next_data.epoch_length.as_u32() + 1).into());
                (
                    pred_consensus_state.staking_epoch_data.clone(),
                    next_data,
                    pred_consensus_state.epoch_count,
                )
            };

            let next_data = if in_seed_update_range(next_slot, pred_block.constants()) {
                ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 {
                    seed: calc_epoch_seed(&next_data.seed, vrf_hash),
                    lock_checkpoint: pred_block.hash().clone(),
                    ..next_data
                }
            } else {
                next_data
            };

            (staking_data, next_data, epoch_count)
        };

        let (min_window_density, sub_window_densities) = {
            // TODO(binier): when should this be false?
            let incr_window = true;
            let pred_sub_window_densities = &pred_consensus_state.sub_window_densities;

            let pred_global_sub_window =
                global_sub_window(&pred_global_slot, pred_block.constants());
            let next_global_sub_window =
                global_sub_window(&curr_global_slot_since_hard_fork, pred_block.constants());

            let pred_relative_sub_window = relative_sub_window(pred_global_sub_window);
            let next_relative_sub_window = relative_sub_window(next_global_sub_window);

            let is_same_global_sub_window = pred_global_sub_window == next_global_sub_window;
            let are_windows_overlapping = pred_global_sub_window
                + constraint_constants().sub_windows_per_window as u32
                >= next_global_sub_window;

            let current_sub_window_densities = pred_sub_window_densities
                .iter()
                .enumerate()
                .map(|(i, density)| (i as u32, density.as_u32()))
                .map(|(i, density)| {
                    let gt_pred_sub_window = i > pred_relative_sub_window;
                    let lt_next_sub_window = i < next_relative_sub_window;
                    let within_range = if pred_relative_sub_window < next_relative_sub_window {
                        gt_pred_sub_window && lt_next_sub_window
                    } else {
                        gt_pred_sub_window || lt_next_sub_window
                    };
                    if is_same_global_sub_window || (are_windows_overlapping && !within_range) {
                        density
                    } else {
                        0
                    }
                })
                .collect::<Vec<_>>();

            let grace_period_end = consensus_constants.grace_period_end;
            let min_window_density = if is_same_global_sub_window
                || curr_global_slot_since_hard_fork.slot_number.as_u32() < grace_period_end
            {
                pred_consensus_state.min_window_density
            } else {
                let cur_density = current_sub_window_densities.iter().sum();
                let min_density = pred_consensus_state
                    .min_window_density
                    .as_u32()
                    .min(cur_density);
                UnsignedExtendedUInt32StableV1(min_density.into())
            };

            let next_sub_window_densities = current_sub_window_densities
                .into_iter()
                .enumerate()
                .map(|(i, density)| (i as u32, density))
                .map(|(i, density)| {
                    let is_next_sub_window = i == next_relative_sub_window;
                    if is_next_sub_window {
                        let density = if is_same_global_sub_window {
                            density
                        } else {
                            0
                        };
                        if incr_window {
                            density + 1
                        } else {
                            density
                        }
                    } else {
                        density
                    }
                })
                .map(|v| UnsignedExtendedUInt32StableV1(v.into()))
                .collect();

            (min_window_density, next_sub_window_densities)
        };

        let consensus_state = ConsensusProofOfStakeDataConsensusStateValueStableV2 {
            blockchain_length: UnsignedExtendedUInt32StableV1((pred_block.height() + 1).into()),
            epoch_count,
            min_window_density,
            sub_window_densities,
            last_vrf_output: vrf_truncated_output,
            total_currency: (&total_currency).into(),
            curr_global_slot_since_hard_fork,
            global_slot_since_genesis,
            staking_epoch_data,
            next_epoch_data,
            has_ancestor_in_same_checkpoint_window,
            block_stake_winner,
            block_creator,
            coinbase_receiver,
            // TODO(binier): Staged_ledger.can_apply_supercharged_coinbase_exn
            supercharge_coinbase: constraint_constants().supercharged_coinbase_factor != 0,
        };

        let protocol_state = MinaStateProtocolStateValueStableV2 {
            previous_state_hash: pred_block.hash().clone(),
            body: MinaStateProtocolStateBodyValueStableV2 {
                genesis_state_hash: if pred_block.is_genesis() {
                    pred_block.hash().clone()
                } else {
                    pred_block
                        .header()
                        .protocol_state
                        .body
                        .genesis_state_hash
                        .clone()
                },
                constants: pred_block.header().protocol_state.body.constants.clone(),
                blockchain_state: MinaStateBlockchainStateValueStableV2 {
                    staged_ledger_hash: staged_ledger_hash.clone(),
                    genesis_ledger_hash: genesis_ledger_hash.clone(),
                    ledger_proof_statement,
                    timestamp: block_timestamp,
                    body_reference: diff_hash.clone(),
                },
                consensus_state,
            },
        };

        let chain_proof_len = pred_block.constants().delta.as_u32() as usize;
        let delta_block_chain_proof = match chain_proof_len {
            0 => (pred_block.hash().clone(), List::new()),
            chain_proof_len => {
                // TODO(binier): test
                let mut iter = chain.iter().rev().take(chain_proof_len + 1).rev();
                if let Some(first_block) = iter.next() {
                    let first_hash = first_block.hash().clone();
                    let body_hashes = iter
                        .filter_map(|b| b.header().protocol_state.body.try_hash().ok()) // TODO: Handle error ?
                        .map(StateBodyHash::from)
                        .collect();
                    (first_hash, body_hashes)
                } else {
                    // TODO: test this as well
                    // If the chain is empty, return the same as when chain_proof_len is 0
                    (pred_block.hash().clone(), List::new())
                }
            }
        };

        let block = BlockWithoutProof {
            protocol_state,
            delta_block_chain_proof,
            current_protocol_version: pred_block.header().current_protocol_version.clone(),
            proposed_protocol_version_opt,
            body: StagedLedgerDiffBodyStableV1 {
                staged_ledger_diff: diff.clone(),
            },
        };
        let Ok(block_hash) = block.protocol_state.try_hash() else {
            openmina_core::log::inner::error!("Invalid protocol state");
            return;
        };

        self.current = BlockProducerCurrentState::BlockUnprovenBuilt {
            time,
            won_slot,
            chain,
            emitted_ledger_proof,
            pending_coinbase_update,
            pending_coinbase_witness,
            stake_proof_sparse_ledger,
            block,
            block_hash,
        };
    }

    fn dispatch_best_tip_update(
        dispatcher: &mut Dispatcher<Action, State>,
        state: &State,
        best_tip: &ArcBlockWithHash,
    ) {
        let global_slot = best_tip
            .consensus_state()
            .curr_global_slot_since_hard_fork
            .clone();

        let (best_tip_epoch, best_tip_slot) = to_epoch_and_slot(&global_slot);
        let root_block_epoch = if let Some(root_block) = state.transition_frontier.root() {
            let root_block_global_slot = root_block.curr_global_slot_since_hard_fork();
            to_epoch_and_slot(root_block_global_slot).0
        } else {
            bug_condition!("Expected to find a block at the root of the transition frontier but there was none");
            best_tip_epoch.saturating_sub(1)
        };
        let next_epoch_first_slot = next_epoch_first_slot(&global_slot);
        let current_epoch = state.current_epoch();
        let current_slot = state.current_slot();

        dispatcher.push(BlockProducerVrfEvaluatorAction::InitializeEvaluator {
            best_tip: best_tip.clone(),
        });

        // None if the evaluator is not evaluating
        let currenty_evaluated_epoch = state
            .block_producer
            .vrf_evaluator()
            .and_then(|vrf_evaluator| vrf_evaluator.currently_evaluated_epoch());

        if let Some(currently_evaluated_epoch) = currenty_evaluated_epoch {
            // if we receive a block with higher epoch than the current one, interrupt the evaluation
            if currently_evaluated_epoch < best_tip_epoch {
                dispatcher.push(BlockProducerVrfEvaluatorAction::InterruptEpochEvaluation {
                    reason: InterruptReason::BestTipWithHigherEpoch,
                });
            }
        }

        let is_next_epoch_seed_finalized = if let Some(current_slot) = current_slot {
            !in_seed_update_range(current_slot, best_tip.constants())
        } else {
            false
        };

        dispatcher.push(BlockProducerVrfEvaluatorAction::CheckEpochEvaluability {
            current_epoch,
            is_next_epoch_seed_finalized,
            root_block_epoch,
            best_tip_epoch,
            best_tip_slot,
            best_tip_global_slot: best_tip.global_slot(),
            next_epoch_first_slot,
            staking_epoch_data: Box::new(best_tip.consensus_state().staking_epoch_data.clone()),
            next_epoch_data: Box::new(best_tip.consensus_state().next_epoch_data.clone()),
        });

        if let Some(reason) = state
            .block_producer
            .with(None, |bp| bp.current.won_slot_should_discard(best_tip))
        {
            dispatcher.push(BlockProducerAction::WonSlotDiscard { reason });
        } else {
            dispatcher.push(BlockProducerAction::WonSlotSearch);
        }
    }
}

fn next_to_staking_epoch_data(
    data: &ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
) -> ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 {
    ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 {
        seed: data.seed.clone(),
        ledger: data.ledger.clone(),
        start_checkpoint: data.start_checkpoint.clone(),
        lock_checkpoint: data.lock_checkpoint.clone(),
        epoch_length: data.epoch_length,
    }
}

fn ledger_proof_statement_from_emitted_proof(
    emitted_ledger_proof: Option<&LedgerProofProdStableV2>,
    pred_proof_statement: &MinaStateBlockchainStateValueStableV2LedgerProofStatement,
) -> MinaStateBlockchainStateValueStableV2LedgerProofStatement {
    match emitted_ledger_proof.map(|proof| &proof.statement) {
        None => pred_proof_statement.clone(),
        Some(stmt) => MinaStateBlockchainStateValueStableV2LedgerProofStatement {
            source: stmt.source.clone(),
            target: stmt.target.clone(),
            connecting_ledger_left: stmt.connecting_ledger_left.clone(),
            connecting_ledger_right: stmt.connecting_ledger_right.clone(),
            supply_increase: stmt.supply_increase.clone(),
            fee_excess: stmt.fee_excess.clone(),
            sok_digest: (),
        },
    }
}
