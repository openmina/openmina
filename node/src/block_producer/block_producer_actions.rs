use std::sync::Arc;

use ledger::scan_state::transaction_logic::valid;
use mina_p2p_messages::v2::MinaBaseProofStableV2;
use openmina_core::block::ArcBlockWithHash;
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::block_producer_effectful::StagedLedgerDiffCreateOutput;

use super::vrf_evaluator::BlockProducerVrfEvaluatorAction;
use super::{BlockProducerCurrentState, BlockProducerWonSlot, BlockProducerWonSlotDiscardReason};

pub type BlockProducerActionWithMeta = redux::ActionWithMeta<BlockProducerAction>;
pub type BlockProducerActionWithMetaRef<'a> = redux::ActionWithMeta<&'a BlockProducerAction>;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = info)]
pub enum BlockProducerAction {
    VrfEvaluator(BlockProducerVrfEvaluatorAction),
    BestTipUpdate {
        best_tip: ArcBlockWithHash,
    },
    WonSlotSearch,
    #[action_event(
        level = info,
        fields(
            slot = won_slot.global_slot.slot_number.as_u32(),
            slot_time = openmina_core::log::to_rfc_3339(won_slot.slot_time)
                .unwrap_or_else(|_| "<error>".to_owned()),
            current_time = openmina_core::log::to_rfc_3339(context.timestamp())
                .unwrap_or_else(|_| "<error>".to_owned()),
        )
    )]
    WonSlot {
        won_slot: BlockProducerWonSlot,
    },
    #[action_event(
        level = info,
        fields(
            reason = format!("{reason:?}"),
        )
    )]
    WonSlotDiscard {
        reason: BlockProducerWonSlotDiscardReason,
    },
    WonSlotWait,
    WonSlotTransactionsGet,
    WonSlotTransactionsSuccess {
        transactions_by_fee: Vec<valid::UserCommand>,
    },
    WonSlotProduceInit,
    StagedLedgerDiffCreateInit,
    StagedLedgerDiffCreatePending,
    StagedLedgerDiffCreateSuccess {
        output: Arc<StagedLedgerDiffCreateOutput>,
    },
    BlockUnprovenBuild,
    BlockProveInit,
    BlockProvePending,
    BlockProveSuccess {
        proof: Arc<MinaBaseProofStableV2>,
    },
    BlockProduced,
    #[action_event(level = trace)]
    BlockInject,
    BlockInjected,
}

impl redux::EnablingCondition<crate::State> for BlockProducerAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        match self {
            BlockProducerAction::VrfEvaluator(a) => a.is_enabled(state, time),
            BlockProducerAction::BestTipUpdate { .. } => true,
            BlockProducerAction::WonSlotSearch => state
                .block_producer
                .with(None, |this| {
                    if !this.current.won_slot_should_search() {
                        return None;
                    }
                    if is_syncing_to_produced_block(state) {
                        return None;
                    }
                    let best_tip = state.transition_frontier.best_tip()?;
                    let cur_global_slot = state.cur_global_slot()?;
                    let next = this.vrf_evaluator.next_won_slot(cur_global_slot, best_tip);
                    Some(next.is_some())
                })
                .is_some_and(|v| v),
            BlockProducerAction::WonSlot { won_slot } => state.block_producer.with(false, |this| {
                let Some(best_tip) = state.transition_frontier.best_tip() else {
                    return false;
                };
                if is_syncing_to_produced_block(state) {
                    return false;
                }

                this.current.won_slot_should_search()
                    && Some(won_slot.global_slot()) >= state.cur_global_slot()
                    && won_slot > best_tip
            }),
            BlockProducerAction::WonSlotWait => state
                .block_producer
                .with(false, |this| this.current.won_slot_should_wait(time)),
            BlockProducerAction::WonSlotProduceInit { .. } => {
                state.block_producer.with(false, |this| {
                    let has_genesis_proven_if_needed = || {
                        state.transition_frontier.best_tip().is_some_and(|tip| {
                            let proven_block = state.transition_frontier.genesis.proven_block();
                            !tip.is_genesis()
                                || proven_block.is_some_and(|b| Arc::ptr_eq(&b.block, &tip.block))
                        })
                    };
                    this.current.won_slot_should_produce(time)
                        && has_genesis_proven_if_needed()
                        // don't start block production (particularly staged ledger diff creation),
                        // if transition frontier sync commit is pending,
                        // as in case when fork is being committed, there
                        // is no guarantee that staged ledger for the current
                        // best tip (chosen as a parent for the new block being produced),
                        // will be still there, once the staged ledger
                        // diff creation request reaches the ledger service.
                        // So we would be trying to build on top of
                        // non-existent staged ledger causing a failure.
                        && !state.transition_frontier.sync.is_commit_pending()
                })
            }
            BlockProducerAction::WonSlotTransactionsGet => {
                state.block_producer.with(false, |this| {
                    matches!(
                        this.current,
                        BlockProducerCurrentState::WonSlotProduceInit { .. }
                    )
                })
            }
            BlockProducerAction::WonSlotTransactionsSuccess { .. } => {
                state.block_producer.with(false, |this| {
                    matches!(
                        this.current,
                        BlockProducerCurrentState::WonSlotTransactionsGet { .. }
                    )
                })
            }
            BlockProducerAction::StagedLedgerDiffCreateInit => {
                state.block_producer.with(false, |this| {
                    matches!(
                        this.current,
                        BlockProducerCurrentState::WonSlotTransactionsSuccess { .. }
                    )
                })
            }
            BlockProducerAction::StagedLedgerDiffCreatePending => {
                state.block_producer.with(false, |this| {
                    matches!(
                        this.current,
                        BlockProducerCurrentState::WonSlotTransactionsSuccess { .. }
                    )
                })
            }
            BlockProducerAction::StagedLedgerDiffCreateSuccess { .. } => {
                state.block_producer.with(false, |this| {
                    matches!(
                        this.current,
                        BlockProducerCurrentState::StagedLedgerDiffCreatePending { .. }
                    )
                })
            }
            BlockProducerAction::BlockUnprovenBuild => state.block_producer.with(false, |this| {
                matches!(
                    this.current,
                    BlockProducerCurrentState::StagedLedgerDiffCreateSuccess { .. }
                )
            }),
            BlockProducerAction::BlockProveInit => state.block_producer.with(false, |this| {
                matches!(
                    this.current,
                    BlockProducerCurrentState::BlockUnprovenBuilt { .. }
                )
            }),
            BlockProducerAction::BlockProvePending => state.block_producer.with(false, |this| {
                matches!(
                    this.current,
                    BlockProducerCurrentState::BlockUnprovenBuilt { .. }
                )
            }),
            BlockProducerAction::BlockProveSuccess { .. } => {
                state.block_producer.with(false, |this| {
                    matches!(
                        this.current,
                        BlockProducerCurrentState::BlockProvePending { .. }
                    )
                })
            }
            BlockProducerAction::BlockProduced => state.block_producer.with(false, |this| {
                matches!(
                    this.current,
                    BlockProducerCurrentState::BlockProveSuccess { .. }
                )
            }),
            BlockProducerAction::BlockInject => {
                state
                    .block_producer
                    .with(false, |this| match &this.current {
                        BlockProducerCurrentState::Produced { block, .. } => {
                            block
                                .timestamp()
                                // broadcast 1s late to account for time drift between nodes
                                .checked_add(1_000_000_000)
                                .is_some_and(|block_time| time >= block_time)
                                && !state.transition_frontier.sync.is_commit_pending()
                        }
                        _ => false,
                    })
            }
            BlockProducerAction::BlockInjected => state.block_producer.with(false, |this| {
                matches!(this.current, BlockProducerCurrentState::Produced { .. })
            }),
            BlockProducerAction::WonSlotDiscard { reason } => {
                let current_reason = state.block_producer.with(None, |bp| {
                    let best_tip = state.transition_frontier.best_tip()?;
                    bp.current.won_slot_should_discard(best_tip)
                });
                Some(reason) == current_reason.as_ref()
            }
        }
    }
}

fn is_syncing_to_produced_block(state: &crate::State) -> bool {
    state
        .transition_frontier
        .sync
        .best_tip()
        .is_some_and(|tip| state.block_producer.is_me(tip.producer()))
}
