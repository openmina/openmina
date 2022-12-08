use std::collections::BTreeMap;

use mina_p2p_messages::{v1::StateHashStable, v2::MinaBlockHeaderStableV2};
use serde::{Deserialize, Serialize};

use snark::block_verify::SnarkBlockVerifyId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConsensusShortRangeForkDecisionIgnoreReason {
    ShorterChain,
    SmallerVrf,
    TieBreakerSmallerStateHash,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConsensusShortRangeForkDecisionUseReason {
    NoBestTip,
    LongerChain,
    BiggerVrf,
    TieBreakerBiggerStateHash,
}

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum ConsensusShortRangeForkDecision {
    Ignore(ConsensusShortRangeForkDecisionIgnoreReason),
    UseAsBestTip(ConsensusShortRangeForkDecisionUseReason),
}

impl ConsensusShortRangeForkDecision {
    pub fn use_as_best_tip(&self) -> bool {
        matches!(self, Self::UseAsBestTip(_))
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
    ShortRangeForkResolve {
        time: redux::Timestamp,
        compared_with: Option<StateHashStable>,
        decision: ConsensusShortRangeForkDecision,
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

    pub fn compared_with(&self) -> Option<&StateHashStable> {
        match self {
            Self::ShortRangeForkResolve { compared_with, .. } => compared_with.as_ref(),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsensusBlockState {
    pub header: MinaBlockHeaderStableV2,
    pub status: ConsensusBlockStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsensusState {
    pub blocks: BTreeMap<StateHashStable, ConsensusBlockState>,
    pub best_tip: Option<StateHashStable>,
}

impl ConsensusState {
    pub fn new() -> Self {
        Self {
            blocks: BTreeMap::new(),
            best_tip: None,
        }
    }

    pub fn best_tip(&self) -> Option<BlockRef<'_>> {
        self.best_tip.as_ref().and_then(|hash| {
            let block = self.blocks.get(hash)?;
            Some(BlockRef {
                hash,
                header: &block.header,
                status: &block.status,
            })
        })
    }

    pub fn previous_best_tip(&self) -> Option<BlockRef<'_>> {
        self.best_tip.as_ref().and_then(|hash| {
            let block = self.blocks.get(hash)?;
            let pred_hash = block.status.compared_with()?;
            let pred = self.blocks.get(pred_hash)?;
            Some(BlockRef {
                hash: pred_hash,
                header: &pred.header,
                status: &pred.status,
            })
        })
    }

    pub fn is_candidate_decided_to_use_as_tip(&self, hash: &StateHashStable) -> bool {
        let Some(candidate) = self.blocks.get(hash) else { return false };
        match &candidate.status {
            ConsensusBlockStatus::Received { .. } => false,
            ConsensusBlockStatus::SnarkVerifyPending { .. } => false,
            ConsensusBlockStatus::SnarkVerifySuccess { .. } => false,
            ConsensusBlockStatus::ShortRangeForkResolve {
                compared_with,
                decision,
                ..
            } => decision.use_as_best_tip() && &self.best_tip == compared_with,
        }
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
pub struct BlockRef<'a> {
    pub hash: &'a StateHashStable,
    pub header: &'a MinaBlockHeaderStableV2,
    pub status: &'a ConsensusBlockStatus,
}

impl<'a> BlockRef<'a> {
    pub fn height(&self) -> i32 {
        self.header
            .protocol_state
            .body
            .consensus_state
            .blockchain_length
            .0
             .0
    }
}
