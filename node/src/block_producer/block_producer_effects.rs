use redux::ActionMeta;

use crate::transition_frontier::sync::TransitionFrontierSyncBestTipUpdateAction;
use crate::Store;

use super::vrf_evaluator::{
    BlockProducerVrfEvaluatorAction, BlockProducerVrfEvaluatorEpochDataUpdateAction,
};
use super::{
    BlockProducerAction, BlockProducerActionWithMeta, BlockProducerBestTipUpdateAction,
    BlockProducerBlockInjectAction, BlockProducerBlockInjectedAction,
    BlockProducerBlockProducedAction, BlockProducerBlockUnprovenBuildAction, BlockProducerService,
    BlockProducerStagedLedgerDiffCreateInitAction,
    BlockProducerStagedLedgerDiffCreatePendingAction,
    BlockProducerStagedLedgerDiffCreateSuccessAction, BlockProducerWonSlot,
    BlockProducerWonSlotAction, BlockProducerWonSlotDiscardAction,
    BlockProducerWonSlotProduceInitAction, BlockProducerWonSlotSearchAction,
    BlockProducerWonSlotWaitAction,
};

pub fn block_producer_effects<S: crate::Service>(
    store: &mut Store<S>,
    action: BlockProducerActionWithMeta,
) {
    let (action, meta) = action.split();

    match action {
        BlockProducerAction::VrfEvaluator(action) => match action {
            BlockProducerVrfEvaluatorAction::EpochDataUpdate(action) => {
                action.effects(&meta, store);
            }
            BlockProducerVrfEvaluatorAction::EvaluateVrf(action) => {
                action.effects(&meta, store);
            }
            BlockProducerVrfEvaluatorAction::EvaluationSuccess(action) => {
                let has_won_slot =
                    matches!(action.vrf_output, vrf::VrfEvaluationOutput::SlotWon(_));
                action.effects(&meta, store);
                if has_won_slot {
                    store.dispatch(BlockProducerWonSlotSearchAction {});
                }
            }
            BlockProducerVrfEvaluatorAction::UpdateProducerAndDelegates(action) => {
                action.effects(&meta, store);
            }
            BlockProducerVrfEvaluatorAction::UpdateProducerAndDelegatesSuccess(action) => {
                action.effects(&meta, store);
            }
        },
        BlockProducerAction::BestTipUpdate(action) => {
            action.effects(&meta, store);
        }
        BlockProducerAction::WonSlotSearch(action) => {
            action.effects(&meta, store);
        }
        BlockProducerAction::WonSlot(action) => {
            action.effects(&meta, store);
        }
        BlockProducerAction::WonSlotWait(_) => {}
        BlockProducerAction::WonSlotDiscard(action) => {
            action.effects(&meta, store);
        }
        BlockProducerAction::WonSlotProduceInit(action) => {
            action.effects(&meta, store);
        }
        BlockProducerAction::StagedLedgerDiffCreateInit(action) => {
            action.effects(&meta, store);
        }
        BlockProducerAction::StagedLedgerDiffCreatePending(_) => {}
        BlockProducerAction::StagedLedgerDiffCreateSuccess(action) => {
            action.effects(&meta, store);
        }
        BlockProducerAction::BlockUnprovenBuild(action) => {
            action.effects(&meta, store);
        }
        BlockProducerAction::BlockProduced(action) => {
            action.effects(&meta, store);
        }
        BlockProducerAction::BlockInject(action) => {
            action.effects(&meta, store);
        }
        BlockProducerAction::BlockInjected(action) => {
            action.effects(&meta, store);
        }
    }
}

impl BlockProducerBestTipUpdateAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        let best_tip_staking_ledger = self.best_tip.staking_epoch_ledger_hash();
        let protocol_state = &self.best_tip.block.header.protocol_state.body;

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
            store.dispatch(BlockProducerVrfEvaluatorEpochDataUpdateAction {
                new_epoch_number: protocol_state.consensus_state.epoch_count.as_u32(),
                epoch_data: protocol_state.consensus_state.staking_epoch_data.clone(),
                next_epoch_data: protocol_state.consensus_state.next_epoch_data.clone(),
            });
        }

        if let Some(reason) = store.state().block_producer.with(None, |bp| {
            bp.current.won_slot_should_discard(&self.best_tip)
        }) {
            store.dispatch(BlockProducerWonSlotDiscardAction { reason });
        }
    }
}

impl BlockProducerWonSlotSearchAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        let vrf_evaluator_state = store.state().block_producer.vrf_evaluator();

        if let Some(vrf_evaluator_state) = vrf_evaluator_state {
            if let Some((_, won_slot)) = vrf_evaluator_state
                .won_slots
                .range(vrf_evaluator_state.current_best_tip_slot + 1..)
                .next()
            {
                let genesis_timestamp = vrf_evaluator_state.genesis_timestamp;
                store.dispatch(BlockProducerWonSlotAction {
                    won_slot: BlockProducerWonSlot::from_vrf_won_slot(
                        won_slot.clone(),
                        genesis_timestamp,
                    ),
                });
            }
        }
    }
}

impl BlockProducerWonSlotAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        if !store.dispatch(BlockProducerWonSlotWaitAction {}) {
            store.dispatch(BlockProducerWonSlotProduceInitAction {});
        }
    }
}

impl BlockProducerWonSlotProduceInitAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(BlockProducerStagedLedgerDiffCreateInitAction {});
    }
}

impl BlockProducerStagedLedgerDiffCreateInitAction {
    pub fn effects<S: BlockProducerService>(self, _: &ActionMeta, store: &mut Store<S>) {
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

        store.dispatch(BlockProducerStagedLedgerDiffCreatePendingAction {});
        store.dispatch(BlockProducerStagedLedgerDiffCreateSuccessAction {
            diff: output.diff,
            diff_hash: output.diff_hash,
            staged_ledger_hash: output.staged_ledger_hash,
            emitted_ledger_proof: output.emitted_ledger_proof,
        });
    }
}

impl BlockProducerStagedLedgerDiffCreateSuccessAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(BlockProducerBlockUnprovenBuildAction {});
    }
}

impl BlockProducerBlockUnprovenBuildAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(BlockProducerBlockProducedAction {});
    }
}

impl BlockProducerBlockProducedAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(BlockProducerBlockInjectAction {});
    }
}

impl BlockProducerBlockInjectAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        let Some((best_tip, root_block, blocks_inbetween)) = None.or_else(|| {
            let (best_tip, chain) = store.state().block_producer.produced_block_with_chain()?;
            let mut iter = chain.iter();
            let root_block = iter.next()?;
            let blocks_inbetween = iter.map(|b| b.hash().clone()).collect();
            Some((best_tip.clone(), root_block.clone(), blocks_inbetween))
        }) else {
            return;
        };

        if store.dispatch(TransitionFrontierSyncBestTipUpdateAction {
            best_tip,
            root_block,
            blocks_inbetween,
        }) {
            store.dispatch(BlockProducerBlockInjectedAction {});
        }
    }
}

impl BlockProducerBlockInjectedAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(BlockProducerWonSlotSearchAction {});
    }
}

impl BlockProducerWonSlotDiscardAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(BlockProducerWonSlotSearchAction {});
    }
}
