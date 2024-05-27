use std::collections::BTreeMap;
use std::sync::Arc;

use mina_p2p_messages::v2::{
    MinaBlockBlockStableV2, MinaBlockHeaderStableV2, StagedLedgerDiffDiffStableV2, StateHash,
};
use serde::{Deserialize, Serialize};

use openmina_core::block::{ArcBlockWithHash, BlockWithHash};
use openmina_core::consensus::{
    ConsensusLongRangeForkDecisionReason, ConsensusShortRangeForkDecisionReason,
};

use crate::snark::block_verify::SnarkBlockVerifyId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConsensusShortRangeForkDecision {
    TakeNoBestTip,
    Take(ConsensusShortRangeForkDecisionReason),
    Keep(ConsensusShortRangeForkDecisionReason),
}

impl ConsensusShortRangeForkDecision {
    pub fn use_as_best_tip(&self) -> bool {
        matches!(self, Self::TakeNoBestTip | Self::Take(_))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConsensusLongRangeForkDecision {
    Keep(ConsensusLongRangeForkDecisionReason),
    Take(ConsensusLongRangeForkDecisionReason),
}

impl ConsensusLongRangeForkDecision {
    pub fn use_as_best_tip(&self) -> bool {
        matches!(self, Self::Take(_))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConsensusBlockStatus {
    Received {
        time: redux::Timestamp,
    },
    SnarkVerifyPending {
        time: redux::Timestamp,
        req_id: SnarkBlockVerifyId,
    },
    SnarkVerifySuccess {
        time: redux::Timestamp,
    },
    ForkRangeDetected {
        time: redux::Timestamp,
        compared_with: Option<StateHash>,
        short_fork: bool,
    },
    ShortRangeForkResolve {
        time: redux::Timestamp,
        compared_with: Option<StateHash>,
        decision: ConsensusShortRangeForkDecision,
    },
    LongRangeForkResolve {
        time: redux::Timestamp,
        compared_with: StateHash,
        decision: ConsensusLongRangeForkDecision,
    },
}

impl ConsensusBlockStatus {
    pub fn is_received(&self) -> bool {
        matches!(self, Self::Received { .. })
    }

    pub fn is_snark_verify_pending(&self) -> bool {
        matches!(self, Self::SnarkVerifyPending { .. })
    }

    pub fn is_snark_verify_success(&self) -> bool {
        matches!(self, Self::SnarkVerifySuccess { .. })
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::SnarkVerifyPending { .. })
    }

    pub fn compared_with(&self) -> Option<&StateHash> {
        match self {
            Self::ShortRangeForkResolve { compared_with, .. } => compared_with.as_ref(),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsensusBlockState {
    pub block: Arc<MinaBlockBlockStableV2>,
    pub status: ConsensusBlockStatus,
    pub chain_proof: Option<(Vec<StateHash>, ArcBlockWithHash)>,
}

impl ConsensusBlockState {
    pub fn height(&self) -> u32 {
        self.block
            .header
            .protocol_state
            .body
            .consensus_state
            .blockchain_length
            .0
             .0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ConsensusState {
    pub blocks: BTreeMap<StateHash, ConsensusBlockState>,
    // TODO(binier): rename to best candidate. Best tip will be in transition_frontier state.
    pub best_tip: Option<StateHash>,
    pub best_tip_chain_proof: Option<(Vec<StateHash>, ArcBlockWithHash)>,
}

impl ConsensusState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn best_tip_block_with_hash(&self) -> Option<BlockWithHash<Arc<MinaBlockBlockStableV2>>> {
        let hash = self.best_tip.as_ref()?;
        let block = self.blocks.get(hash)?;
        Some(BlockWithHash {
            hash: hash.clone(),
            block: block.block.clone(),
        })
    }

    pub fn best_tip(&self) -> Option<BlockRef<'_>> {
        self.best_tip.as_ref().and_then(|hash| {
            let block = self.blocks.get(hash)?;
            Some(BlockRef {
                hash,
                header: &block.block.header,
                body: &block.block.body.staged_ledger_diff,
                status: &block.status,
            })
        })
    }

    pub fn previous_best_tip(&self) -> Option<BlockRef<'_>> {
        self.best_tip.as_ref().and_then(|hash| {
            let block = self.blocks.get(hash)?;
            let prev_hash = block.status.compared_with()?;
            let prev = self.blocks.get(prev_hash)?;
            Some(BlockRef {
                hash: prev_hash,
                header: &prev.block.header,
                body: &prev.block.body.staged_ledger_diff,
                status: &prev.status,
            })
        })
    }

    pub fn is_candidate_decided_to_use_as_tip(&self, hash: &StateHash) -> bool {
        let Some(candidate) = self.blocks.get(hash) else {
            return false;
        };
        match &candidate.status {
            ConsensusBlockStatus::Received { .. } => false,
            ConsensusBlockStatus::SnarkVerifyPending { .. } => false,
            ConsensusBlockStatus::SnarkVerifySuccess { .. } => false,
            ConsensusBlockStatus::ForkRangeDetected { .. } => false,
            ConsensusBlockStatus::ShortRangeForkResolve {
                compared_with,
                decision,
                ..
            } => decision.use_as_best_tip() && &self.best_tip == compared_with,
            ConsensusBlockStatus::LongRangeForkResolve {
                compared_with,
                decision,
                ..
            } => decision.use_as_best_tip() && self.best_tip.as_ref() == Some(compared_with),
        }
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
pub struct BlockRef<'a> {
    pub hash: &'a StateHash,
    pub header: &'a MinaBlockHeaderStableV2,
    pub body: &'a StagedLedgerDiffDiffStableV2,
    pub status: &'a ConsensusBlockStatus,
}

impl<'a> BlockRef<'a> {
    pub fn height(&self) -> u32 {
        self.header
            .protocol_state
            .body
            .consensus_state
            .blockchain_length
            .0
             .0
    }
}
