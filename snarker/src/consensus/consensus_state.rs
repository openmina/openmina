use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use mina_p2p_messages::v2::{
    MinaBlockBlockStableV2, MinaBlockHeaderStableV2, StagedLedgerDiffDiffStableV2, StateHash,
};
use serde::{Deserialize, Serialize};

use shared::block::BlockWithHash;

use crate::snark::block_verify::SnarkBlockVerifyId;

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

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum ConsensusLongRangeForkDecisionIgnoreReason {}

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum ConsensusLongRangeForkResolutionKind {
    SubWindowDensity,
    ChainLength,
    Vrf,
    StateHash,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConsensusLongRangeForkDecision {
    Keep(ConsensusLongRangeForkResolutionKind),
    Take(ConsensusLongRangeForkResolutionKind),
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
    /// Set temporarily when we receive best tip. Is removed once we
    /// are done processing the block.
    pub history: Option<Vec<StateHash>>,
}

impl ConsensusBlockState {
    pub fn height(&self) -> i32 {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BestTipHistory {
    chain: VecDeque<StateHash>,
    /// Level of the first block in `chain`.
    top_level: u32,
}

impl BestTipHistory {
    pub fn bottom_level(&self) -> u32 {
        self.top_level.saturating_sub(self.chain.len() as u32)
    }

    pub fn contains(&self, level: u32, hash: &StateHash) -> bool {
        let Some(i) = self.top_level.checked_sub(level) else {
            return false;
        };
        self.chain.get(i as usize) == Some(hash)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsensusState {
    pub blocks: BTreeMap<StateHash, ConsensusBlockState>,
    // TODO(binier): rename to best candidate. Best tip will be in transition_frontier state.
    pub best_tip: Option<StateHash>,
    pub best_tip_history: BestTipHistory,
}

impl ConsensusState {
    pub fn new() -> Self {
        Self {
            blocks: BTreeMap::new(),
            best_tip: None,
            best_tip_history: BestTipHistory {
                chain: Default::default(),
                top_level: 0,
            },
        }
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
            let block = &*self.blocks.get(hash)?;
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
            let block = &*self.blocks.get(hash)?;
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

    pub fn update_best_tip_history(&mut self, new_level: u32, new_history: &[StateHash]) {
        let mut cur_level = self.best_tip_history.top_level;
        if new_level < cur_level {
            return;
        }

        while let Some(hash) = self.best_tip_history.chain.pop_front() {
            let i = (new_level - cur_level) as usize;
            let new_hash = match new_history.get(i) {
                Some(v) => v,
                None => {
                    self.best_tip_history.chain.clear();
                    cur_level = 0;
                    break;
                }
            };
            if &hash == new_hash {
                self.best_tip_history.chain.push_front(hash);
                break;
            }

            cur_level -= 1;
        }

        self.best_tip_history.top_level = new_level;
        new_history
            .iter()
            .take((new_level - cur_level) as usize)
            .rev()
            .for_each(|hash| self.best_tip_history.chain.push_front(hash.clone()));
    }

    pub fn is_best_tip_and_history_linked(&self) -> bool {
        let Some(best_tip) = self.best_tip() else {
            return false;
        };
        let pred_hash = &best_tip.header.protocol_state.previous_state_hash;

        Some(pred_hash) == self.best_tip_history.chain.front()
    }

    /// Returns `None` if answer can't be known at the moment.
    pub fn is_part_of_main_chain(&self, level: u32, hash: &StateHash) -> Option<bool> {
        let best_tip = self.best_tip()?;
        if level == best_tip.height() as u32 {
            return Some(hash == best_tip.hash);
        }

        let history = &self.best_tip_history;

        if !self.is_best_tip_and_history_linked() {
            return None;
        }

        if level >= history.bottom_level() && level <= history.top_level {
            return Some(history.contains(level, hash));
        }

        None
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

#[cfg(test)]
mod tests {
    use super::*;

    fn hashes() -> Vec<StateHash> {
        IntoIterator::into_iter([
            "3NKxvqipR18tYFpE54XoXT1y6vN67UjCNxv1ovpo2H4aYUfPQbhS",
            "3NKyFBN868VkPrLXui6E34QMpfhgkS1aED5a8HhfS26JoZfdgtCL",
            "3NLQV2E7qEDmZkpaGvxnmGYmsyGFiQBccRQTsgz3jcvEdBzCXgon",
            "3NK5mH9WGPZWpW2QP7tH8TmUMAh4YEfwPyYMkkuhaPASUvHeb4P8",
            "3NLZKTrmPQVYQ3KwXyPFwSVegrZTUKJv2LyBJGqNGC3v5ZnU9yQu",
            "3NLgKTSRDeuyFxnmqbc7MbvmYYWc8GeHaYuSKA8KHjmVyNmUGTbv",
            "3NL6pccNfWQZSS1djdkqgqS2iQkrjqdhonT29ZLDHdjwTZCGTf8E",
            "3NLe7NDvwNxj1e9bwkiXd8MJXoseyDkwfXSsH9BkhG6wt4ySE46C",
        ])
        .map(|h| serde_json::from_str(&format!("\"{}\"", h)).unwrap())
        .collect()
    }

    #[test]
    fn test_update_best_tip_history() {
        let hashes = hashes();
        let n = hashes.len();
        let mut state = ConsensusState::new();

        state.update_best_tip_history(100, &hashes[6..]);
        assert_eq!(state.best_tip_history.top_level, 100);
        assert_eq!(state.best_tip_history.chain, &hashes[6..]);

        state.update_best_tip_history(101, &hashes[5..]);
        assert_eq!(state.best_tip_history.top_level, 101);
        assert_eq!(state.best_tip_history.chain, &hashes[5..]);

        state.update_best_tip_history(103, &hashes[3..]);
        assert_eq!(state.best_tip_history.top_level, 103);
        assert_eq!(state.best_tip_history.chain, &hashes[3..]);

        state.update_best_tip_history(104, &hashes[2..(n - 1)]);
        assert_eq!(state.best_tip_history.top_level, 104);
        assert_eq!(state.best_tip_history.chain, &hashes[2..]);

        state.update_best_tip_history(106, &hashes[..(n - 3)]);
        assert_eq!(state.best_tip_history.top_level, 106);
        assert_eq!(state.best_tip_history.chain, &hashes[..]);
    }
}
