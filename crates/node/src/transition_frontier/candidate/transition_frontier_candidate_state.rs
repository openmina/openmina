use std::collections::{BTreeMap, BTreeSet};

use mina_p2p_messages::v2::StateHash;
use serde::{Deserialize, Serialize};

use mina_core::{
    block::ArcBlockWithHash,
    consensus::{
        consensus_take, ConsensusLongRangeForkDecisionReason, ConsensusShortRangeForkDecisionReason,
    },
};

use crate::{
    snark::block_verify::SnarkBlockVerifyId, transition_frontier::TransitionFrontierState,
};

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
pub enum TransitionFrontierCandidateStatus {
    Received {
        time: redux::Timestamp,
    },
    Prevalidated,
    SnarkVerifyPending {
        time: redux::Timestamp,
        req_id: SnarkBlockVerifyId,
    },
    SnarkVerifySuccess {
        time: redux::Timestamp,
    },
}

impl TransitionFrontierCandidateStatus {
    pub fn is_received(&self) -> bool {
        matches!(self, Self::Received { .. })
    }

    pub fn is_prevalidated(&self) -> bool {
        matches!(self, Self::Prevalidated)
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierCandidateState {
    pub block: ArcBlockWithHash,
    pub status: TransitionFrontierCandidateStatus,
    pub chain_proof: Option<(Vec<StateHash>, ArcBlockWithHash)>,
}

impl Ord for TransitionFrontierCandidateState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.eq(other) {
            return std::cmp::Ordering::Equal;
        }
        let is_candidate_better = consensus_take(
            self.block.consensus_state(),
            other.block.consensus_state(),
            self.block.hash(),
            other.block.hash(),
        );
        match is_candidate_better {
            true => std::cmp::Ordering::Less,
            false => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for TransitionFrontierCandidateState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for TransitionFrontierCandidateState {}

impl PartialEq for TransitionFrontierCandidateState {
    fn eq(&self, other: &Self) -> bool {
        self.block.hash() == other.block.hash()
    }
}

impl TransitionFrontierCandidateState {
    pub fn height(&self) -> u32 {
        self.block.height()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TransitionFrontierCandidatesState {
    /// Maintains an ordered list of transition frontier Candidates,
    /// ordered using consensus rules worst to best.
    ordered: BTreeSet<TransitionFrontierCandidateState>,
    /// Candidate block hashes, which failed either the prevalidation
    /// or block proof verification. We move them here so that they
    /// consume less memory while still preventing us from triggering
    /// revalidation for an invalid block if we receive it on p2p again.
    invalid: BTreeMap<StateHash, u32>,
}

impl TransitionFrontierCandidatesState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, hash: &StateHash) -> bool {
        self.invalid.contains_key(hash) || self.get(hash).is_some()
    }

    pub(super) fn get(&self, hash: &StateHash) -> Option<&TransitionFrontierCandidateState> {
        self.ordered.iter().rev().find(|s| s.block.hash() == hash)
    }

    pub(super) fn add(
        &mut self,
        time: redux::Timestamp,
        block: ArcBlockWithHash,
        chain_proof: Option<(Vec<StateHash>, ArcBlockWithHash)>,
    ) {
        self.ordered.insert(TransitionFrontierCandidateState {
            block,
            status: TransitionFrontierCandidateStatus::Received { time },
            chain_proof,
        });
    }

    fn update(
        &mut self,
        hash: &StateHash,
        update: impl FnOnce(TransitionFrontierCandidateState) -> TransitionFrontierCandidateState,
    ) -> bool {
        let Some(state) = self.get(hash).cloned() else {
            return false;
        };
        self.ordered.remove(&state);
        self.ordered.insert(update(state));
        true
    }

    pub(super) fn update_status(
        &mut self,
        hash: &StateHash,
        update: impl FnOnce(TransitionFrontierCandidateStatus) -> TransitionFrontierCandidateStatus,
    ) -> bool {
        self.update(hash, move |mut state| {
            state.status = update(state.status);
            state
        })
    }

    pub(super) fn invalidate(&mut self, hash: &StateHash, is_forever_invalid: bool) {
        self.ordered.retain(|s| {
            if s.block.hash() == hash {
                if is_forever_invalid {
                    self.invalid.insert(hash.clone(), s.block.global_slot());
                }
                false
            } else {
                true
            }
        });
    }

    pub(super) fn set_chain_proof(
        &mut self,
        hash: &StateHash,
        chain_proof: (Vec<StateHash>, ArcBlockWithHash),
    ) -> bool {
        self.update(hash, move |mut s| {
            s.chain_proof = Some(chain_proof);
            s
        })
    }

    pub(super) fn prune(&mut self) {
        let mut has_reached_best_candidate = false;
        let Some(best_candidate_hash) = self.best_verified().map(|s| s.block.hash().clone()) else {
            return;
        };

        // prune all blocks that are worse(consensus-wise) than the best
        // verified candidate.
        self.ordered.retain(|s| {
            if s.block.hash() == &best_candidate_hash {
                // prune all invalid block hashes which are for older
                // slots than the current best candidate.
                let best_candidate_slot = s.block.global_slot();
                self.invalid.retain(|_, slot| *slot >= best_candidate_slot);

                has_reached_best_candidate = true;
            }

            has_reached_best_candidate
        });
    }

    pub(super) fn best(&self) -> Option<&TransitionFrontierCandidateState> {
        self.ordered.last()
    }

    pub fn best_verified(&self) -> Option<&TransitionFrontierCandidateState> {
        self.ordered
            .iter()
            .rev()
            .find(|s| s.status.is_snark_verify_success())
    }

    pub fn is_chain_proof_needed(&self, hash: &StateHash) -> bool {
        self.get(hash).is_some_and(|s| s.chain_proof.is_none())
    }

    pub fn best_verified_block(&self) -> Option<&ArcBlockWithHash> {
        self.best_verified().map(|s| &s.block)
    }

    pub fn best_verified_block_chain_proof(
        &self,
        transition_frontier: &TransitionFrontierState,
    ) -> Option<(Vec<StateHash>, ArcBlockWithHash)> {
        self.block_chain_proof(self.best_verified()?, transition_frontier)
    }

    fn block_chain_proof(
        &self,
        block_state: &TransitionFrontierCandidateState,
        transition_frontier: &TransitionFrontierState,
    ) -> Option<(Vec<StateHash>, ArcBlockWithHash)> {
        let pred_hash = block_state.block.pred_hash();
        block_state.chain_proof.clone().or_else(|| {
            let old_best_tip = transition_frontier.best_tip()?;
            let mut iter = transition_frontier.best_chain.iter();
            if old_best_tip.hash() == pred_hash {
                if old_best_tip.height() > old_best_tip.constants().k.as_u32() {
                    iter.next();
                }
                let root_block = iter.next()?.block_with_hash().clone();
                let hashes = iter.map(|b| b.hash().clone()).collect();
                Some((hashes, root_block))
            } else if old_best_tip.pred_hash() == pred_hash {
                let root_block = iter.next()?.block_with_hash().clone();
                let hashes = iter.rev().skip(1).rev().map(|b| b.hash().clone()).collect();
                Some((hashes, root_block))
            } else {
                None
            }
        })
    }
}
