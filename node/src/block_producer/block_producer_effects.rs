use mina_p2p_messages::v2::{
    BlockchainSnarkBlockchainStableV2, ConsensusStakeProofStableV2,
    MinaStateSnarkTransitionValueStableV2, ProverExtendBlockchainInputStableV2,
};
use openmina_core::bug_condition;

use crate::account::AccountSecretKey;
use crate::ledger::write::{LedgerWriteAction, LedgerWriteRequest};
use crate::transition_frontier::sync::TransitionFrontierSyncAction;
use crate::{Store, TransactionPoolAction};

use super::vrf_evaluator::{BlockProducerVrfEvaluatorAction, InterruptReason};
use super::{
    next_epoch_first_slot, to_epoch_and_slot, BlockProducerAction, BlockProducerActionWithMeta,
    BlockProducerCurrentState,
};

pub fn block_producer_effects<S: crate::Service>(
    store: &mut Store<S>,
    action: BlockProducerActionWithMeta,
) {
    let (action, meta) = action.split();

    match action {
        BlockProducerAction::VrfEvaluator(a) => {
            // TODO: does the order matter? can this clone be avoided?
            let has_won_slot = match &a {
                BlockProducerVrfEvaluatorAction::ProcessSlotEvaluationSuccess {
                    vrf_output,
                    ..
                } => {
                    matches!(vrf_output, vrf::VrfEvaluationOutput::SlotWon(_))
                }
                _ => false,
            };
            a.effects(&meta, store);
            if has_won_slot {
                store.dispatch(BlockProducerAction::WonSlotSearch);
            }
        }
        BlockProducerAction::BestTipUpdate { best_tip } => {
            let global_slot = best_tip
                .consensus_state()
                .curr_global_slot_since_hard_fork
                .clone();

            let (best_tip_epoch, best_tip_slot) = to_epoch_and_slot(&global_slot);
            let root_block_epoch = if let Some(root_block) =
                store.state().transition_frontier.root()
            {
                let root_block_global_slot = root_block.curr_global_slot_since_hard_fork();
                to_epoch_and_slot(root_block_global_slot).0
            } else {
                bug_condition!("Expected to find a block at the root of the transition frontier but there was none");
                best_tip_epoch.saturating_sub(1)
            };
            let next_epoch_first_slot = next_epoch_first_slot(&global_slot);
            let current_epoch = store.state().current_epoch();

            store.dispatch(BlockProducerVrfEvaluatorAction::InitializeEvaluator {
                best_tip: best_tip.clone(),
            });

            // None if the evaluator is not evaluating
            let currenty_evaluated_epoch = store
                .state()
                .block_producer
                .vrf_evaluator()
                .and_then(|vrf_evaluator| vrf_evaluator.currently_evaluated_epoch());

            if let Some(currently_evaluated_epoch) = currenty_evaluated_epoch {
                // if we receive a block with higher epoch than the current one, interrupt the evaluation
                if currently_evaluated_epoch < best_tip_epoch {
                    store.dispatch(BlockProducerVrfEvaluatorAction::InterruptEpochEvaluation {
                        reason: InterruptReason::BestTipWithHigherEpoch,
                    });
                }
            }

            store.dispatch(BlockProducerVrfEvaluatorAction::CheckEpochEvaluability {
                current_epoch,
                root_block_epoch,
                best_tip_epoch,
                best_tip_slot,
                best_tip_global_slot: best_tip.global_slot(),
                next_epoch_first_slot,
                staking_epoch_data: Box::new(best_tip.consensus_state().staking_epoch_data.clone()),
                next_epoch_data: Box::new(best_tip.consensus_state().next_epoch_data.clone()),
            });

            if let Some(reason) = store
                .state()
                .block_producer
                .with(None, |bp| bp.current.won_slot_should_discard(&best_tip))
            {
                store.dispatch(BlockProducerAction::WonSlotDiscard { reason });
            } else {
                store.dispatch(BlockProducerAction::WonSlotSearch);
            }
        }
        BlockProducerAction::WonSlotSearch => {
            if let Some(won_slot) = store.state().block_producer.with(None, |bp| {
                let best_tip = store.state().transition_frontier.best_tip()?;
                let cur_global_slot = store.state().cur_global_slot()?;
                bp.vrf_evaluator.next_won_slot(cur_global_slot, best_tip)
            }) {
                store.dispatch(BlockProducerAction::WonSlot { won_slot });
            }
        }
        BlockProducerAction::WonSlot { won_slot } => {
            if let Some(stats) = store.service.stats() {
                stats.block_producer().scheduled(meta.time(), &won_slot);
            }
            if !store.dispatch(BlockProducerAction::WonSlotWait) {
                store.dispatch(BlockProducerAction::WonSlotProduceInit);
            }
        }
        BlockProducerAction::WonSlotWait => {}
        BlockProducerAction::WonSlotProduceInit => {
            store.dispatch(BlockProducerAction::WonSlotTransactionsGet);
        }
        BlockProducerAction::WonSlotTransactionsGet => {
            store.dispatch(TransactionPoolAction::CollectTransactionsByFee);
        }
        BlockProducerAction::WonSlotTransactionsSuccess { .. } => {
            store.dispatch(BlockProducerAction::StagedLedgerDiffCreateInit);
        }
        BlockProducerAction::StagedLedgerDiffCreateInit => {
            if let Some(stats) = store.service.stats() {
                stats
                    .block_producer()
                    .staged_ledger_diff_create_start(meta.time());
            }
            let state = store.state.get();
            let Some((won_slot, pred_block, producer, coinbase_receiver)) = None.or_else(|| {
                let pred_block = state.block_producer.current_parent_chain()?.last()?;
                let won_slot = state.block_producer.current_won_slot()?;
                let config = state.block_producer.config()?;
                Some((
                    won_slot,
                    pred_block,
                    &config.pub_key,
                    config.coinbase_receiver(),
                ))
            }) else {
                return;
            };

            let completed_snarks = state
                .snark_pool
                .completed_snarks_iter()
                .map(|snark| (snark.job_id(), snark.clone()))
                .collect();
            // TODO(binier)
            let supercharge_coinbase = true;
            // We want to know if this is a new epoch to decide which staking ledger to use
            // (staking epoch ledger or next epoch ledger).
            let is_new_epoch = won_slot.epoch()
                > pred_block
                    .header()
                    .protocol_state
                    .body
                    .consensus_state
                    .epoch_count
                    .as_u32();

            let transactions_by_fee = state.block_producer.pending_transactions();

            store.dispatch(LedgerWriteAction::Init {
                request: LedgerWriteRequest::StagedLedgerDiffCreate {
                    pred_block: pred_block.clone(),
                    global_slot_since_genesis: won_slot
                        .global_slot_since_genesis(pred_block.global_slot_diff()),
                    is_new_epoch,
                    producer: producer.clone(),
                    delegator: won_slot.delegator.0.clone(),
                    coinbase_receiver: coinbase_receiver.clone(),
                    completed_snarks,
                    supercharge_coinbase,
                    transactions_by_fee,
                },
                on_init: redux::callback!(
                    on_staged_ledger_diff_create_init(_request: LedgerWriteRequest) -> crate::Action {
                        BlockProducerAction::StagedLedgerDiffCreatePending
                    }
                ),
            });
        }
        BlockProducerAction::StagedLedgerDiffCreatePending => {}
        BlockProducerAction::StagedLedgerDiffCreateSuccess { .. } => {
            if let Some(stats) = store.service.stats() {
                stats
                    .block_producer()
                    .staged_ledger_diff_create_end(meta.time());
            }
            store.dispatch(BlockProducerAction::BlockUnprovenBuild);
        }
        BlockProducerAction::BlockUnprovenBuild => {
            if let Some(stats) = store.service.stats() {
                let bp = &store.state.get().block_producer;
                if let Some((block_hash, block)) = bp.with(None, |bp| match &bp.current {
                    BlockProducerCurrentState::BlockUnprovenBuilt {
                        block, block_hash, ..
                    } => Some((block_hash, block)),
                    _ => None,
                }) {
                    stats
                        .block_producer()
                        .produced(meta.time(), block_hash, block);
                }
            }

            store.dispatch(BlockProducerAction::BlockProveInit);
        }
        BlockProducerAction::BlockProveInit => {
            let service = &mut store.service;

            if let Some(stats) = service.stats() {
                stats.block_producer().proof_create_start(meta.time());
            }
            let Some((block_hash, input)) = store.state.get().block_producer.with(None, |bp| {
                let BlockProducerCurrentState::BlockUnprovenBuilt {
                    won_slot,
                    chain,
                    emitted_ledger_proof,
                    pending_coinbase_update,
                    pending_coinbase_witness,
                    stake_proof_sparse_ledger,
                    block,
                    block_hash,
                    ..
                } = &bp.current
                else {
                    return None;
                };

                let pred_block = chain.last()?;

                let producer_public_key = block
                    .protocol_state
                    .body
                    .consensus_state
                    .block_creator
                    .clone();

                let input = Box::new(ProverExtendBlockchainInputStableV2 {
                    chain: BlockchainSnarkBlockchainStableV2 {
                        state: pred_block.header().protocol_state.clone(),
                        proof: pred_block.header().protocol_state_proof.clone(),
                    },
                    next_state: block.protocol_state.clone(),
                    block: MinaStateSnarkTransitionValueStableV2 {
                        blockchain_state: block.protocol_state.body.blockchain_state.clone(),
                        consensus_transition: block
                            .protocol_state
                            .body
                            .consensus_state
                            .curr_global_slot_since_hard_fork
                            .slot_number
                            .clone(),
                        pending_coinbase_update: pending_coinbase_update.clone(),
                    },
                    ledger_proof: emitted_ledger_proof.as_ref().map(|proof| (**proof).clone()),
                    prover_state: ConsensusStakeProofStableV2 {
                        delegator: won_slot.delegator.1.into(),
                        delegator_pk: won_slot.delegator.0.clone(),
                        coinbase_receiver_pk: block
                            .protocol_state
                            .body
                            .consensus_state
                            .coinbase_receiver
                            .clone(),
                        ledger: stake_proof_sparse_ledger.clone(),
                        // it is replaced with correct keys in the service.
                        producer_private_key: AccountSecretKey::genesis_producer().into(),
                        producer_public_key,
                    },
                    pending_coinbase: pending_coinbase_witness.clone(),
                });
                Some((block_hash.clone(), input))
            }) else {
                return;
            };
            service.prove(block_hash, input);
            store.dispatch(BlockProducerAction::BlockProvePending);
        }
        BlockProducerAction::BlockProvePending => {}
        BlockProducerAction::BlockProveSuccess { .. } => {
            if let Some(stats) = store.service.stats() {
                stats.block_producer().proof_create_end(meta.time());
            }
            store.dispatch(BlockProducerAction::BlockProduced);
        }
        BlockProducerAction::BlockProduced => {
            store.dispatch(BlockProducerAction::BlockInject);
        }
        BlockProducerAction::BlockInject => {
            let Some((best_tip, root_block, blocks_inbetween)) = None.or_else(|| {
                let (best_tip, chain) = store.state().block_producer.produced_block_with_chain()?;
                let mut iter = chain.iter();
                let root_block = iter.next()?.block_with_hash();
                let blocks_inbetween = iter.map(|b| b.hash().clone()).collect();
                Some((best_tip.clone(), root_block.clone(), blocks_inbetween))
            }) else {
                return;
            };

            let previous_root_snarked_ledger_hash = store
                .state()
                .transition_frontier
                .root()
                .map(|b| b.snarked_ledger_hash().clone());

            if store.dispatch(TransitionFrontierSyncAction::BestTipUpdate {
                previous_root_snarked_ledger_hash,
                best_tip: best_tip.clone(),
                root_block,
                blocks_inbetween,
            }) {
                store.dispatch(BlockProducerAction::BlockInjected);
            }
        }
        BlockProducerAction::BlockInjected => {
            store.dispatch(BlockProducerAction::WonSlotSearch);
        }
        BlockProducerAction::WonSlotDiscard { reason } => {
            if let Some(stats) = store.service.stats() {
                stats.block_producer().discarded(meta.time(), reason);
            }
            store.dispatch(BlockProducerAction::WonSlotSearch);
        }
    }
}
