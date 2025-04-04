//! Block producer state management module.
//! Defines the state machine for block production, including slot winning, block creation,
//! and block injection into the transition frontier.

use std::{collections::BTreeSet, sync::Arc, time::Duration};

use ledger::scan_state::transaction_logic::valid;
use mina_p2p_messages::v2;
use openmina_core::{
    block::{AppliedBlock, ArcBlockWithHash},
    consensus::consensus_take,
};
use serde::{Deserialize, Serialize};

use crate::account::AccountPublicKey;

use super::{
    vrf_evaluator::BlockProducerVrfEvaluatorState, BlockProducerConfig, BlockProducerWonSlot,
    BlockWithoutProof,
};

/// Main state container for the block producer module.
/// When None, block production is disabled.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerState(Option<BlockProducerEnabled>);

/// Active block producer state when block production is enabled.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerEnabled {
    pub config: BlockProducerConfig,
    pub vrf_evaluator: BlockProducerVrfEvaluatorState,
    /// Current state in the block production state machine.
    pub current: BlockProducerCurrentState,
    /// Blocks that were injected into transition frontier, but haven't
    /// become our best tip yet.
    pub injected_blocks: BTreeSet<v2::StateHash>,
}

/// State machine for block production process.
/// Represents all possible states in the block production workflow from
/// winning a slot to injecting a produced block into the transition frontier.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerCurrentState {
    /// No active block production.
    Idle {
        time: redux::Timestamp,
    },
    /// A won slot was discarded due to a specific reason.
    WonSlotDiscarded {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        reason: BlockProducerWonSlotDiscardReason,
    },
    /// A slot has been won but production hasn't started yet.
    WonSlot {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
    },
    /// Waiting for the right time to produce a block for a won slot.
    WonSlotWait {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
    },
    /// Initializing block production for a won slot.
    WonSlotProduceInit {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<AppliedBlock>,
    },
    /// Fetching transactions from the mempool for block production.
    WonSlotTransactionsGet {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<AppliedBlock>,
    },
    /// Successfully retrieved transactions for block production.
    WonSlotTransactionsSuccess {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<AppliedBlock>,
        transactions_by_fee: Vec<valid::UserCommand>,
    },
    /// Creating a staged ledger diff for the new block.
    StagedLedgerDiffCreatePending {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<AppliedBlock>,
        transactions_by_fee: Vec<valid::UserCommand>,
    },
    /// Successfully created a staged ledger diff for the new block.
    StagedLedgerDiffCreateSuccess {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<AppliedBlock>,
        diff: v2::StagedLedgerDiffDiffStableV2,
        /// `protocol_state.blockchain_state.body_reference`
        diff_hash: v2::ConsensusBodyReferenceStableV1,
        staged_ledger_hash: v2::MinaBaseStagedLedgerHashStableV1,
        emitted_ledger_proof: Option<Arc<v2::LedgerProofProdStableV2>>,
        pending_coinbase_update: v2::MinaBasePendingCoinbaseUpdateStableV1,
        pending_coinbase_witness: v2::MinaBasePendingCoinbaseWitnessStableV2,
        stake_proof_sparse_ledger: v2::MinaBaseSparseLedgerBaseStableV2,
    },
    /// Built an unproven block (without SNARK proof).
    BlockUnprovenBuilt {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<AppliedBlock>,
        emitted_ledger_proof: Option<Arc<v2::LedgerProofProdStableV2>>,
        pending_coinbase_update: v2::MinaBasePendingCoinbaseUpdateStableV1,
        pending_coinbase_witness: v2::MinaBasePendingCoinbaseWitnessStableV2,
        stake_proof_sparse_ledger: v2::MinaBaseSparseLedgerBaseStableV2,
        block: BlockWithoutProof,
        block_hash: v2::StateHash,
    },
    /// Generating a SNARK proof for the block.
    BlockProvePending {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<AppliedBlock>,
        emitted_ledger_proof: Option<Arc<v2::LedgerProofProdStableV2>>,
        pending_coinbase_update: v2::MinaBasePendingCoinbaseUpdateStableV1,
        pending_coinbase_witness: v2::MinaBasePendingCoinbaseWitnessStableV2,
        stake_proof_sparse_ledger: v2::MinaBaseSparseLedgerBaseStableV2,
        block: BlockWithoutProof,
        block_hash: v2::StateHash,
    },
    /// Successfully generated a SNARK proof for the block.
    BlockProveSuccess {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<AppliedBlock>,
        block: BlockWithoutProof,
        block_hash: v2::StateHash,
        proof: Arc<v2::MinaBaseProofStableV2>,
    },
    /// Block has been fully produced with proof.
    Produced {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<AppliedBlock>,
        block: ArcBlockWithHash,
    },
    /// Block has been injected into the transition frontier.
    Injected {
        time: redux::Timestamp,
        won_slot: BlockProducerWonSlot,
        /// Chain that we are extending.
        chain: Vec<AppliedBlock>,
        block: ArcBlockWithHash,
    },
}

/// Reasons why a won slot might be discarded instead of producing a block.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub enum BlockProducerWonSlotDiscardReason {
    /// The staking ledger hash of the best tip is different from the one used in VRF evaluation.
    BestTipStakingLedgerDifferent,
    /// The global slot of the best tip is higher than the won slot.
    BestTipGlobalSlotHigher,
    /// The best tip is superior according to consensus rules.
    BestTipSuperior,
}

impl BlockProducerState {
    pub fn new(now: redux::Timestamp, config: Option<BlockProducerConfig>) -> Self {
        Self(config.map(|config| BlockProducerEnabled {
            config: config.clone(),
            vrf_evaluator: BlockProducerVrfEvaluatorState::new(now),
            current: BlockProducerCurrentState::Idle { time: now },
            injected_blocks: Default::default(),
        }))
    }

    pub fn with<'a, F, R: 'a>(&'a self, default: R, fun: F) -> R
    where
        F: FnOnce(&'a BlockProducerEnabled) -> R,
    {
        self.0.as_ref().map_or(default, fun)
    }

    pub fn as_mut(&mut self) -> Option<&mut BlockProducerEnabled> {
        self.0.as_mut()
    }

    pub fn as_ref(&self) -> Option<&BlockProducerEnabled> {
        self.0.as_ref()
    }

    pub fn is_enabled(&self) -> bool {
        self.0.is_some()
    }

    pub fn config(&self) -> Option<&BlockProducerConfig> {
        self.with(None, |this| Some(&this.config))
    }

    pub fn is_me(&self, producer: &v2::NonZeroCurvePoint) -> bool {
        self.with(false, |this| producer == &this.config.pub_key)
    }

    /// Checks if the block was produced by us recently.
    pub fn is_produced_by_me(&self, block: &ArcBlockWithHash) -> bool {
        self.with(false, |this| {
            block.producer() == &this.config.pub_key && this.injected_blocks.contains(block.hash())
        })
    }

    pub fn is_producing(&self) -> bool {
        self.with(false, |this| this.current.is_producing())
    }

    pub fn current_won_slot(&self) -> Option<&BlockProducerWonSlot> {
        self.with(None, |this| this.current.won_slot())
    }

    pub fn current_parent_chain(&self) -> Option<&[AppliedBlock]> {
        self.with(None, |this| this.current.parent_chain())
    }

    /// Won slot that we are in the middle of producing.
    pub fn producing_won_slot(&self) -> Option<&BlockProducerWonSlot> {
        self.current_won_slot().filter(|_| self.is_producing())
    }

    pub fn produced_block(&self) -> Option<&ArcBlockWithHash> {
        self.with(None, |this| this.current.produced_block())
    }

    pub fn produced_block_with_chain(&self) -> Option<(&ArcBlockWithHash, &[AppliedBlock])> {
        self.with(None, |this| this.current.produced_block_with_chain())
    }

    pub fn vrf_evaluator(&self) -> Option<&BlockProducerVrfEvaluatorState> {
        self.with(None, |this| Some(&this.vrf_evaluator))
    }

    pub fn vrf_evaluator_with_config(
        &self,
    ) -> Option<(&BlockProducerVrfEvaluatorState, &BlockProducerConfig)> {
        self.with(None, |this| Some((&this.vrf_evaluator, &this.config)))
    }

    /// If we need to construct delegator table, get it's inputs.
    pub fn vrf_delegator_table_inputs(&self) -> Option<(&v2::LedgerHash, &AccountPublicKey)> {
        self.vrf_evaluator()?.vrf_delegator_table_inputs()
    }

    pub fn pending_transactions(&self) -> Vec<valid::UserCommand> {
        self.with(Vec::new(), |this| this.current.pending_transactions())
    }
}

impl BlockProducerCurrentState {
    pub fn won_slot_should_search(&self) -> bool {
        match self {
            Self::Idle { .. } | Self::WonSlotDiscarded { .. } | Self::Injected { .. } => true,
            Self::WonSlot { .. }
            | Self::WonSlotWait { .. }
            | Self::WonSlotProduceInit { .. }
            | Self::WonSlotTransactionsGet { .. }
            | Self::WonSlotTransactionsSuccess { .. }
            | Self::StagedLedgerDiffCreatePending { .. }
            | Self::StagedLedgerDiffCreateSuccess { .. }
            | Self::BlockUnprovenBuilt { .. }
            | Self::BlockProvePending { .. }
            | Self::BlockProveSuccess { .. }
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
            | Self::WonSlotTransactionsGet { won_slot, .. }
            | Self::WonSlotTransactionsSuccess { won_slot, .. }
            | Self::StagedLedgerDiffCreatePending { won_slot, .. }
            | Self::StagedLedgerDiffCreateSuccess { won_slot, .. }
            | Self::BlockUnprovenBuilt { won_slot, .. }
            | Self::BlockProvePending { won_slot, .. }
            | Self::BlockProveSuccess { won_slot, .. }
            | Self::Produced { won_slot, .. }
            | Self::Injected { won_slot, .. } => Some(won_slot),
        }
    }

    pub fn parent_chain(&self) -> Option<&[AppliedBlock]> {
        match self {
            Self::Idle { .. }
            | Self::WonSlotDiscarded { .. }
            | Self::WonSlot { .. }
            | Self::WonSlotWait { .. } => None,
            Self::WonSlotProduceInit { chain, .. }
            | Self::WonSlotTransactionsGet { chain, .. }
            | Self::WonSlotTransactionsSuccess { chain, .. }
            | Self::StagedLedgerDiffCreatePending { chain, .. }
            | Self::StagedLedgerDiffCreateSuccess { chain, .. }
            | Self::BlockUnprovenBuilt { chain, .. }
            | Self::BlockProvePending { chain, .. }
            | Self::BlockProveSuccess { chain, .. }
            | Self::Produced { chain, .. }
            | Self::Injected { chain, .. } => Some(chain),
        }
    }

    /// Determines if we should wait before producing a block for a won slot.
    pub fn won_slot_should_wait(&self, now: redux::Timestamp) -> bool {
        matches!(self, Self::WonSlot { .. }) && !self.won_slot_should_produce(now)
    }

    /// Determines if it's the right time to produce a block for a won slot.
    ///
    /// Ensures block production happens within the slot interval and accounts for
    /// estimated production time to avoid producing blocks too late in the slot.
    pub fn won_slot_should_produce(&self, now: redux::Timestamp) -> bool {
        // TODO(binier): maybe have runtime estimate
        #[cfg(not(target_arch = "wasm32"))]
        const BLOCK_PRODUCTION_ESTIMATE: u64 = Duration::from_secs(6).as_nanos() as u64;
        #[cfg(target_arch = "wasm32")]
        const BLOCK_PRODUCTION_ESTIMATE: u64 = Duration::from_secs(20).as_nanos() as u64;

        let slot_interval = Duration::from_secs(3 * 60).as_nanos() as u64;
        match self {
            Self::WonSlot { won_slot, .. } | Self::WonSlotWait { won_slot, .. } => {
                // Make sure to only produce blocks when in the slot interval
                let slot_upper_bound = won_slot
                    .slot_time
                    .checked_add(slot_interval)
                    .expect("overflow");
                let estimated_produced_time = now
                    .checked_add(BLOCK_PRODUCTION_ESTIMATE)
                    .expect("overflow");
                estimated_produced_time >= won_slot.slot_time && now < slot_upper_bound
            }
            _ => false,
        }
    }

    /// Determines if a won slot should be discarded based on the current best tip.
    ///
    /// Checks several conditions that would make block production for this slot invalid:
    /// 1. If the best tip's global slot is higher than our won slot
    /// 2. If the staking ledger hash used for VRF evaluation doesn't match the best tip's ledger hashes
    /// 3. If the best tip is superior according to consensus rules
    pub fn won_slot_should_discard(
        &self,
        best_tip: &ArcBlockWithHash,
    ) -> Option<BlockProducerWonSlotDiscardReason> {
        let won_slot = self.won_slot()?;
        if won_slot.global_slot() < best_tip.global_slot() {
            return Some(BlockProducerWonSlotDiscardReason::BestTipGlobalSlotHigher);
        }

        if &won_slot.staking_ledger_hash != best_tip.staking_epoch_ledger_hash()
            && &won_slot.staking_ledger_hash != best_tip.next_epoch_ledger_hash()
        {
            return Some(BlockProducerWonSlotDiscardReason::BestTipStakingLedgerDifferent);
        }

        if won_slot < best_tip
            || self.produced_block().is_some_and(|block| {
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
            | Self::WonSlotTransactionsGet { .. }
            | Self::WonSlotTransactionsSuccess { .. }
            | Self::StagedLedgerDiffCreatePending { .. }
            | Self::StagedLedgerDiffCreateSuccess { .. }
            | Self::BlockUnprovenBuilt { .. }
            | Self::BlockProvePending { .. }
            | Self::BlockProveSuccess { .. }
            | Self::Produced { .. } => true,
        }
    }

    pub fn produced_block(&self) -> Option<&ArcBlockWithHash> {
        match self {
            Self::Produced { block, .. } => Some(block),
            _ => None,
        }
    }

    pub fn injected_block(&self) -> Option<&ArcBlockWithHash> {
        match self {
            Self::Injected { block, .. } => Some(block),
            _ => None,
        }
    }

    pub fn produced_block_with_chain(&self) -> Option<(&ArcBlockWithHash, &[AppliedBlock])> {
        match self {
            Self::Produced { chain, block, .. } => Some((block, chain)),
            _ => None,
        }
    }

    pub fn pending_transactions(&self) -> Vec<valid::UserCommand> {
        match self {
            Self::WonSlotTransactionsSuccess {
                transactions_by_fee,
                ..
            }
            | Self::StagedLedgerDiffCreatePending {
                transactions_by_fee,
                ..
            } => transactions_by_fee.to_vec(),
            _ => vec![],
        }
    }
}

impl Default for BlockProducerCurrentState {
    fn default() -> Self {
        Self::Idle {
            time: redux::Timestamp::ZERO,
        }
    }
}
