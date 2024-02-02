use mina_p2p_messages::v2::{
    ConsensusBodyReferenceStableV1, LedgerProofProdStableV2, MinaBaseStagedLedgerHashStableV1,
    StagedLedgerDiffDiffStableV2,
};
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

use super::vrf_evaluator::BlockProducerVrfEvaluatorAction;
use super::{BlockProducerCurrentState, BlockProducerWonSlot, BlockProducerWonSlotDiscardReason};

pub type BlockProducerActionWithMeta = redux::ActionWithMeta<BlockProducerAction>;
pub type BlockProducerActionWithMetaRef<'a> = redux::ActionWithMeta<&'a BlockProducerAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerAction {
    VrfEvaluator(BlockProducerVrfEvaluatorAction),
    BestTipUpdate(BlockProducerBestTipUpdateAction),
    WonSlotSearch(BlockProducerWonSlotSearchAction),
    WonSlot(BlockProducerWonSlotAction),
    WonSlotDiscard(BlockProducerWonSlotDiscardAction),
    WonSlotWait(BlockProducerWonSlotWaitAction),
    WonSlotProduceInit(BlockProducerWonSlotProduceInitAction),
    StagedLedgerDiffCreateInit(BlockProducerStagedLedgerDiffCreateInitAction),
    StagedLedgerDiffCreatePending(BlockProducerStagedLedgerDiffCreatePendingAction),
    StagedLedgerDiffCreateSuccess(BlockProducerStagedLedgerDiffCreateSuccessAction),
    BlockUnprovenBuild(BlockProducerBlockUnprovenBuildAction),
    BlockProduced(BlockProducerBlockProducedAction),
    BlockInject(BlockProducerBlockInjectAction),
    BlockInjected(BlockProducerBlockInjectedAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerBestTipUpdateAction {
    pub best_tip: ArcBlockWithHash,
}

impl redux::EnablingCondition<crate::State> for BlockProducerBestTipUpdateAction {
    fn is_enabled(&self, _state: &crate::State) -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerWonSlotSearchAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerWonSlotSearchAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
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
            .is_some_and(|v| v)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerWonSlotAction {
    pub won_slot: BlockProducerWonSlot,
}

impl redux::EnablingCondition<crate::State> for BlockProducerWonSlotAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            let Some(best_tip) = state.transition_frontier.best_tip() else {
                return false;
            };

            this.current.won_slot_should_search()
                && self.won_slot.global_slot() >= state.cur_global_slot().unwrap()
                && &self.won_slot > best_tip
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerWonSlotWaitAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerWonSlotWaitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            this.current.won_slot_should_wait(state.time())
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerWonSlotProduceInitAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerWonSlotProduceInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            this.current.won_slot_should_produce(state.time())
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerStagedLedgerDiffCreateInitAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerStagedLedgerDiffCreateInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.current,
                BlockProducerCurrentState::WonSlotProduceInit { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerStagedLedgerDiffCreatePendingAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerStagedLedgerDiffCreatePendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.current,
                BlockProducerCurrentState::WonSlotProduceInit { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerStagedLedgerDiffCreateSuccessAction {
    pub diff: StagedLedgerDiffDiffStableV2,
    pub diff_hash: ConsensusBodyReferenceStableV1,
    pub staged_ledger_hash: MinaBaseStagedLedgerHashStableV1,
    pub emitted_ledger_proof: Option<LedgerProofProdStableV2>,
}

impl redux::EnablingCondition<crate::State> for BlockProducerStagedLedgerDiffCreateSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.current,
                BlockProducerCurrentState::StagedLedgerDiffCreatePending { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerBlockUnprovenBuildAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerBlockUnprovenBuildAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.current,
                BlockProducerCurrentState::StagedLedgerDiffCreateSuccess { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerBlockProducedAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerBlockProducedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(
                this.current,
                BlockProducerCurrentState::BlockUnprovenBuilt { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerBlockInjectAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerBlockInjectAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(this.current, BlockProducerCurrentState::Produced { .. })
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerBlockInjectedAction {}

impl redux::EnablingCondition<crate::State> for BlockProducerBlockInjectedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.block_producer.with(false, |this| {
            matches!(this.current, BlockProducerCurrentState::Produced { .. })
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerWonSlotDiscardAction {
    pub reason: BlockProducerWonSlotDiscardReason,
}

impl redux::EnablingCondition<crate::State> for BlockProducerWonSlotDiscardAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        let reason = state.block_producer.with(None, |bp| {
            let best_tip = state.transition_frontier.best_tip()?;
            bp.current.won_slot_should_discard(best_tip)
        });
        Some(&self.reason) == reason.as_ref()
    }
}

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::BlockProducer(value.into())
            }
        }
    };
}

impl_into_global_action!(BlockProducerBestTipUpdateAction);
impl_into_global_action!(BlockProducerWonSlotSearchAction);
impl_into_global_action!(BlockProducerWonSlotAction);
impl_into_global_action!(BlockProducerWonSlotDiscardAction);
impl_into_global_action!(BlockProducerWonSlotWaitAction);
impl_into_global_action!(BlockProducerWonSlotProduceInitAction);
impl_into_global_action!(BlockProducerStagedLedgerDiffCreateInitAction);
impl_into_global_action!(BlockProducerStagedLedgerDiffCreatePendingAction);
impl_into_global_action!(BlockProducerStagedLedgerDiffCreateSuccessAction);
impl_into_global_action!(BlockProducerBlockUnprovenBuildAction);
impl_into_global_action!(BlockProducerBlockProducedAction);
impl_into_global_action!(BlockProducerBlockInjectAction);
impl_into_global_action!(BlockProducerBlockInjectedAction);
