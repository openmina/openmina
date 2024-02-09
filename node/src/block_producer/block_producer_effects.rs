use crate::transition_frontier::sync::TransitionFrontierSyncAction;
use crate::Store;

use super::vrf_evaluator::BlockProducerVrfEvaluatorAction;
use super::{BlockProducerAction, BlockProducerActionWithMeta};

pub fn block_producer_effects<S: crate::Service>(
    store: &mut Store<S>,
    action: BlockProducerActionWithMeta,
) {
    let (action, meta) = action.split();

    match action {
        BlockProducerAction::VrfEvaluator(ref a) => {
            // TODO: does the order matter? can this clone be avoided?
            a.clone().effects(&meta, store);
            match a {
                BlockProducerVrfEvaluatorAction::EvaluationSuccess { vrf_output, .. } => {
                    let has_won_slot = matches!(vrf_output, vrf::VrfEvaluationOutput::SlotWon(_));
                    if has_won_slot {
                        store.dispatch(BlockProducerAction::WonSlotSearch);
                    }
                }
                _ => {}
            }
        }
        BlockProducerAction::BestTipUpdate { best_tip } => {
            let best_tip_staking_ledger = best_tip.staking_epoch_ledger_hash();
            let protocol_state = &best_tip.block.header.protocol_state.body;

            let vrf_evaluator_current_epoch_ledger = store
                .state()
                .block_producer
                .vrf_evaluator()
                .and_then(|vrf_evaluator| {
                    vrf_evaluator
                        .current_epoch_data
                        .as_ref()
                        .map(|epoch_data| &epoch_data.ledger)
                });

            if vrf_evaluator_current_epoch_ledger != Some(best_tip_staking_ledger) {
                store.dispatch(BlockProducerVrfEvaluatorAction::EpochDataUpdate {
                    new_epoch_number: protocol_state.consensus_state.epoch_count.as_u32(),
                    epoch_data: protocol_state.consensus_state.staking_epoch_data.clone(),
                    next_epoch_data: protocol_state.consensus_state.next_epoch_data.clone(),
                });
            }

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
