use mina_p2p_messages::v2::MinaBaseProofStableV2;
use openmina_core::block::ArcBlockWithHash;
use openmina_core::log::ActionEvent;
use openmina_core::{action_info, action_trace};
use serde::{Deserialize, Serialize};

use super::vrf_evaluator::BlockProducerVrfEvaluatorAction;
use super::{
    BlockProducerCurrentState, BlockProducerWonSlot, BlockProducerWonSlotDiscardReason,
    StagedLedgerDiffCreateOutput,
};

pub type BlockProducerActionWithMeta = redux::ActionWithMeta<BlockProducerAction>;
pub type BlockProducerActionWithMetaRef<'a> = redux::ActionWithMeta<&'a BlockProducerAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerAction {
    VrfEvaluator(BlockProducerVrfEvaluatorAction),
    BestTipUpdate {
        best_tip: ArcBlockWithHash,
    },
    WonSlotSearch,
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

impl ActionEvent for BlockProducerAction {
    fn action_event<T>(&self, context: &T)
    where
        T: openmina_core::log::EventContext,
    {
        match self {
            BlockProducerAction::VrfEvaluator(action) => action.action_event(context),
            BlockProducerAction::WonSlot {
                won_slot:
                    BlockProducerWonSlot {
                        slot_time,
                        global_slot,
                        ..
                    },
            } => action_info!(
                context,
                summary = "Won slot",
                slot = global_slot.slot_number.as_u32(),
                slot_time = openmina_core::log::to_rfc_3339(*slot_time)
                    .unwrap_or_else(|_| "<error>".to_owned()),
                current_time = openmina_core::log::to_rfc_3339(context.timestamp())
                    .unwrap_or_else(|_| "<error>".to_owned()),
            ),
            _ => action_trace!(context),
        }
    }
}
