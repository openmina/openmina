use mina_p2p_messages::v2::{
    ConsensusBodyReferenceStableV1, LedgerProofProdStableV2, MinaBaseStagedLedgerHashStableV1,
    NonZeroCurvePoint, StagedLedgerDiffDiffStableV2,
};
use openmina_core::{block::ArcBlockWithHash, consensus::consensus_take};
use serde::{Deserialize, Serialize};

use super::{
    vrf_evaluator::BlockProducerVrfEvaluatorState, BlockProducerConfig, BlockProducerWonSlot,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerState(Option<BlockProducerEnabled>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerEnabled {
    pub config: BlockProducerConfig,
    pub vrf_evaluator: BlockProducerVrfEvaluatorState,
    pub current: BlockProducerCurrentState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerCurrentState {
    Idle {
        time: redux::Timestamp,
    },
    WonSlotDiscarded {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        reason: BlockProducerWonSlotDiscardReason,
    },
    WonSlot {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
    },
    WonSlotWait {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
    },
    WonSlotProduceInit {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<ArcBlockWithHash>,
    },
    StagedLedgerDiffCreatePending {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<ArcBlockWithHash>,
        // TODO(binier): transactions from pool, once we have that implemented.
        transactions: (),
    },
    StagedLedgerDiffCreateSuccess {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<ArcBlockWithHash>,
        diff: StagedLedgerDiffDiffStableV2,
        /// `protocol_state.blockchain_state.body_reference`
        diff_hash: ConsensusBodyReferenceStableV1,
        staged_ledger_hash: MinaBaseStagedLedgerHashStableV1,
        emitted_ledger_proof: Option<LedgerProofProdStableV2>,
    },
    BlockUnprovenBuilt {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<ArcBlockWithHash>,
        block: ArcBlockWithHash,
    },
    Produced {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<ArcBlockWithHash>,
        block: ArcBlockWithHash,
    },
    Injected {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<ArcBlockWithHash>,
        block: ArcBlockWithHash,
    },
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum BlockProducerWonSlotDiscardReason {
    BestTipStakingLedgerDifferent,
    BestTipGlobalSlotHigher,
    BestTipSuperior,
}

impl BlockProducerState {
    pub fn new(now: redux::Timestamp, config: Option<BlockProducerConfig>) -> Self {
        Self(config.map(|config| BlockProducerEnabled {
            config: config.clone(),
            vrf_evaluator: BlockProducerVrfEvaluatorState::new(now, config),
            current: BlockProducerCurrentState::Idle { time: now },
        }))
    }

    #[inline(always)]
    pub(super) fn with<'a, F, R: 'a>(&'a self, default: R, fun: F) -> R
    where
        F: FnOnce(&'a BlockProducerEnabled) -> R,
    {
        self.0.as_ref().map_or(default, fun)
    }

    #[inline(always)]
    pub(super) fn with_mut<F, R>(&mut self, default: R, fun: F) -> R
    where
        F: FnOnce(&mut BlockProducerEnabled) -> R,
    {
        self.0.as_mut().map_or(default, fun)
    }

    pub fn config(&self) -> Option<&BlockProducerConfig> {
        self.with(None, |this| Some(&this.config))
    }

    pub fn is_me(&self, producer: &NonZeroCurvePoint) -> bool {
        self.with(false, |this| producer == &this.config.pub_key)
    }

    pub fn is_producing(&self) -> bool {
        self.with(false, |this| this.current.is_producing())
    }

    pub fn current_won_slot(&self) -> Option<&BlockProducerWonSlot> {
        self.with(None, |this| this.current.won_slot())
    }

    pub fn current_parent_chain(&self) -> Option<&[ArcBlockWithHash]> {
        self.with(None, |this| this.current.parent_chain())
    }

    /// Won slot that we are in the middle of producing.
    pub fn producing_won_slot(&self) -> Option<&BlockProducerWonSlot> {
        self.current_won_slot().filter(|_| self.is_producing())
    }

    pub fn produced_block(&self) -> Option<&ArcBlockWithHash> {
        self.with(None, |this| this.current.produced_block())
    }

    pub fn produced_block_with_chain(&self) -> Option<(&ArcBlockWithHash, &[ArcBlockWithHash])> {
        self.with(None, |this| this.current.produced_block_with_chain())
    }

    pub fn vrf_evaluator(&self) -> Option<&BlockProducerVrfEvaluatorState> {
        self.with(None, |this| Some(&this.vrf_evaluator))
    }
}

impl BlockProducerCurrentState {
    pub fn won_slot_should_search(&self) -> bool {
        match self {
            Self::Idle { .. } | Self::WonSlotDiscarded { .. } | Self::Injected { .. } => true,
            Self::WonSlot { .. }
            | Self::WonSlotWait { .. }
            | Self::WonSlotProduceInit { .. }
            | Self::StagedLedgerDiffCreatePending { .. }
            | Self::StagedLedgerDiffCreateSuccess { .. }
            | Self::BlockUnprovenBuilt { .. }
            | Self::Produced { .. } => false,
        }
    }

    pub fn won_slot(&self) -> Option<&BlockProducerWonSlot> {
        match self {
            Self::Idle { .. } => None,
            Self::WonSlotDiscarded { won_slot, .. }
            | Self::WonSlot { won_slot, .. }
            | Self::WonSlotWait { won_slot, .. }
            | Self::WonSlotProduceInit { won_slot, .. }
            | Self::StagedLedgerDiffCreatePending { won_slot, .. }
            | Self::StagedLedgerDiffCreateSuccess { won_slot, .. }
            | Self::BlockUnprovenBuilt { won_slot, .. }
            | Self::Produced { won_slot, .. }
            | Self::Injected { won_slot, .. } => Some(won_slot),
        }
    }

    pub fn parent_chain(&self) -> Option<&[ArcBlockWithHash]> {
        match self {
            Self::Idle { .. }
            | Self::WonSlotDiscarded { .. }
            | Self::WonSlot { .. }
            | Self::WonSlotWait { .. } => None,
            Self::WonSlotProduceInit { chain, .. }
            | Self::StagedLedgerDiffCreatePending { chain, .. }
            | Self::StagedLedgerDiffCreateSuccess { chain, .. }
            | Self::BlockUnprovenBuilt { chain, .. }
            | Self::Produced { chain, .. }
            | Self::Injected { chain, .. } => Some(chain),
        }
    }

    pub fn won_slot_should_wait(&self, now: redux::Timestamp) -> bool {
        match self {
            Self::WonSlot { won_slot, .. } => now < won_slot.slot_time,
            _ => false,
        }
    }

    pub fn won_slot_should_produce(&self, now: redux::Timestamp) -> bool {
        match self {
            Self::WonSlot { won_slot, .. } | Self::WonSlotWait { won_slot, .. } => {
                now >= won_slot.slot_time
            }
            _ => false,
        }
    }

    pub fn won_slot_should_discard(
        &self,
        best_tip: &ArcBlockWithHash,
    ) -> Option<BlockProducerWonSlotDiscardReason> {
        let won_slot = self.won_slot()?;
        if won_slot.global_slot_since_genesis.as_u32() < best_tip.global_slot() {
            return Some(BlockProducerWonSlotDiscardReason::BestTipGlobalSlotHigher);
        }
        // TODO(binier): check if staking ledger changed

        if won_slot < best_tip
            || self.produced_block().map_or(false, |block| {
                !consensus_take(
                    best_tip.consensus_state(),
                    block.consensus_state(),
                    best_tip.hash(),
                    block.hash(),
                )
            })
        {
            return Some(BlockProducerWonSlotDiscardReason::BestTipSuperior);
        }

        None
    }

    pub fn is_producing(&self) -> bool {
        match self {
            Self::Idle { .. }
            | Self::WonSlotDiscarded { .. }
            | Self::WonSlot { .. }
            | Self::WonSlotWait { .. }
            | Self::Injected { .. } => false,
            Self::WonSlotProduceInit { .. }
            | Self::StagedLedgerDiffCreatePending { .. }
            | Self::StagedLedgerDiffCreateSuccess { .. }
            | Self::BlockUnprovenBuilt { .. }
            | Self::Produced { .. } => true,
        }
    }

    pub fn produced_block(&self) -> Option<&ArcBlockWithHash> {
        match self {
            Self::Produced { block, .. } => Some(block),
            _ => None,
        }
    }

    pub fn produced_block_with_chain(&self) -> Option<(&ArcBlockWithHash, &[ArcBlockWithHash])> {
        match self {
            Self::Produced { chain, block, .. } => Some((block, chain)),
            _ => None,
        }
    }
}
