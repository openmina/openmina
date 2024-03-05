use crate::transition_frontier::sync::TransitionFrontierSyncAction;
use crate::Store;

use super::vrf_evaluator::BlockProducerVrfEvaluatorAction;
use super::{
    next_epoch_first_slot, to_epoch_and_slot, BlockProducerAction, BlockProducerActionWithMeta,
};

pub fn block_producer_effects<S: crate::Service>(
    store: &mut Store<S>,
    action: BlockProducerActionWithMeta,
) {
    let (action, meta) = action.split();

    match action {
        BlockProducerAction::VrfEvaluator(ref a) => {
            // TODO: does the order matter? can this clone be avoided?
            a.clone().effects(&meta, store);
            if let BlockProducerVrfEvaluatorAction::ProcessSlotEvaluationSuccess {
                vrf_output,
                ..
            } = a
            {
                let has_won_slot = matches!(vrf_output, vrf::VrfEvaluationOutput::SlotWon(_));
                if has_won_slot {
                    store.dispatch(BlockProducerAction::WonSlotSearch);
                }
            }
        }
        BlockProducerAction::BestTipUpdate { best_tip } => {
            let global_slot = best_tip
                .consensus_state()
                .curr_global_slot_since_hard_fork
                .clone();

            let (epoch, slot) = to_epoch_and_slot(&global_slot);
            let next_epoch_first_slot = next_epoch_first_slot(&global_slot);

            store.dispatch(BlockProducerVrfEvaluatorAction::InitializeEvaluator {
                best_tip: best_tip.clone(),
            });

            store.dispatch(
                BlockProducerVrfEvaluatorAction::RecordLastBlockHeightInEpoch {
                    epoch_number: epoch,
                    last_block_height: best_tip.height(),
                },
            );

            store.dispatch(BlockProducerVrfEvaluatorAction::CheckEpochEvaluability {
                current_epoch_number: epoch,
                current_best_tip_height: best_tip.height(),
                current_best_tip_slot: slot,
                current_best_tip_global_slot: best_tip.global_slot(),
                next_epoch_first_slot,
                staking_epoch_data: best_tip.consensus_state().staking_epoch_data.clone(),
                next_epoch_data: best_tip.consensus_state().next_epoch_data.clone(),
                transition_frontier_size: best_tip.constants().k.as_u32(),
            });

            if let Some(reason) = store
                .state()
                .block_producer
                .with(None, |bp| bp.current.won_slot_should_discard(&best_tip))
            {
                store.dispatch(BlockProducerAction::WonSlotDiscard { reason });
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
        BlockProducerAction::WonSlot { .. } => {
            if !store.dispatch(BlockProducerAction::WonSlotWait) {
                store.dispatch(BlockProducerAction::WonSlotProduceInit);
            }
        }
        BlockProducerAction::WonSlotProduceInit => {
            store.dispatch(BlockProducerAction::StagedLedgerDiffCreateInit);
        }
        BlockProducerAction::StagedLedgerDiffCreateInit => {
            let state = store.state.get();
            let Some((won_slot, pred_block, coinbase_receiver)) = None.or_else(|| {
                let pred_block = state.block_producer.current_parent_chain()?.last()?;
                let won_slot = state.block_producer.current_won_slot()?;
                let coinbase_receiver = state.block_producer.config()?.coinbase_receiver();
                Some((won_slot, pred_block, coinbase_receiver))
            }) else {
                return;
            };

            let completed_snarks = state
                .snark_pool
                .completed_snarks_iter()
                .map(|snark| (snark.job_id(), snark.clone()))
                .collect();
            // TODO(binier)
            let supercharge_coinbase = false;

            // TODO(binier): error handling
            let output = store
                .service
                .staged_ledger_diff_create(
                    pred_block,
                    won_slot,
                    coinbase_receiver,
                    completed_snarks,
                    supercharge_coinbase,
                )
                .unwrap();

            store.dispatch(BlockProducerAction::StagedLedgerDiffCreatePending);
            store.dispatch(BlockProducerAction::StagedLedgerDiffCreateSuccess {
                diff: output.diff,
                diff_hash: output.diff_hash,
                staged_ledger_hash: output.staged_ledger_hash,
                emitted_ledger_proof: output.emitted_ledger_proof,
            });
        }
        BlockProducerAction::StagedLedgerDiffCreateSuccess { .. } => {
            store.dispatch(BlockProducerAction::BlockUnprovenBuild);
        }
        BlockProducerAction::BlockUnprovenBuild => {
            store.dispatch(BlockProducerAction::BlockProduced);
        }
        BlockProducerAction::BlockProduced => {
            store.dispatch(BlockProducerAction::BlockInject);
        }
        BlockProducerAction::BlockInject => {
            let Some((best_tip, root_block, blocks_inbetween)) = None.or_else(|| {
                let (best_tip, chain) = store.state().block_producer.produced_block_with_chain()?;
                let mut iter = chain.iter();
                let root_block = iter.next()?;
                let blocks_inbetween = iter.map(|b| b.hash().clone()).collect();
                Some((best_tip.clone(), root_block.clone(), blocks_inbetween))
            }) else {
                return;
            };

            if store.dispatch(TransitionFrontierSyncAction::BestTipUpdate {
                best_tip,
                root_block,
                blocks_inbetween,
            }) {
                store.dispatch(BlockProducerAction::BlockInjected);
            }
        }
        BlockProducerAction::BlockInjected => {
            store.dispatch(BlockProducerAction::WonSlotSearch);
        }
        BlockProducerAction::WonSlotDiscard { .. } => {
            store.dispatch(BlockProducerAction::WonSlotSearch);
        }
        BlockProducerAction::StagedLedgerDiffCreatePending => {}
        BlockProducerAction::WonSlotWait => {}
    }
}
