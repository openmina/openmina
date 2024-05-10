use mina_p2p_messages::v2::MinaBaseProofStableV2;
use openmina_core::block::ArcBlockWithHash;
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use super::vrf_evaluator::BlockProducerVrfEvaluatorAction;
use super::{
    BlockProducerCurrentState, BlockProducerWonSlot, BlockProducerWonSlotDiscardReason,
    StagedLedgerDiffCreateOutput,
};

pub type BlockProducerActionWithMeta = redux::ActionWithMeta<BlockProducerAction>;
pub type BlockProducerActionWithMetaRef<'a> = redux::ActionWithMeta<&'a BlockProducerAction>;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = trace)]
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
    WonSlotDiscard {
        reason: BlockProducerWonSlotDiscardReason,
    },
    WonSlotWait,
    WonSlotProduceInit,
    StagedLedgerDiffCreateInit,
    StagedLedgerDiffCreatePending,
    StagedLedgerDiffCreateSuccess {
        output: StagedLedgerDiffCreateOutput,
    },
    BlockUnprovenBuild,
    BlockProveInit,
    BlockProvePending,
    BlockProveSuccess {
        proof: Box<MinaBaseProofStableV2>,
    },
    BlockProduced,
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
                    && won_slot.global_slot() >= state.cur_global_slot().unwrap()
                    && won_slot > best_tip
            }),
            BlockProducerAction::WonSlotWait => state
                .block_producer
                .with(false, |this| this.current.won_slot_should_wait(time)),
            BlockProducerAction::WonSlotProduceInit => state
                .block_producer
                .with(false, |this| this.current.won_slot_should_produce(time)),
            BlockProducerAction::StagedLedgerDiffCreateInit => {
                state.block_producer.with(false, |this| {
                    matches!(
                        this.current,
                        BlockProducerCurrentState::WonSlotProduceInit { .. }
                    )
                })
            }
            BlockProducerAction::StagedLedgerDiffCreatePending => {
                state.block_producer.with(false, |this| {
                    matches!(
                        this.current,
                        BlockProducerCurrentState::WonSlotProduceInit { .. }
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
            BlockProducerAction::BlockInject => state.block_producer.with(false, |this| {
                matches!(this.current, BlockProducerCurrentState::Produced { .. })
                    && !state.transition_frontier.sync.is_commit_pending()
            }),
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
        .map_or(false, |tip| state.block_producer.is_me(tip.producer()))
}
