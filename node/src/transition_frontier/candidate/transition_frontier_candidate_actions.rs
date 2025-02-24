use std::sync::Arc;

use mina_p2p_messages::v2::{MinaBlockBlockStableV2, StateHash};
use openmina_core::block::prevalidate::BlockPrevalidationError;
use openmina_core::block::{ArcBlockWithHash, BlockWithHash};
use openmina_core::consensus::consensus_take;
use openmina_core::{action_event, ActionEvent};
use serde::{Deserialize, Serialize};
use snark::block_verify::SnarkBlockVerifyError;

use crate::snark::block_verify::SnarkBlockVerifyId;

use super::TransitionFrontierCandidateStatus;

pub type TransitionFrontierCandidateActionWithMeta =
    redux::ActionWithMeta<TransitionFrontierCandidateAction>;
pub type TransitionFrontierCandidateActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierCandidateAction>;

// NOTE: `debug(hash)` must be used instead of `display(hash)` because
// for some reason the later breaks CI. `hash = display(&hash)` works too.
#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = debug, fields(debug(hash), debug(error)))]
pub enum TransitionFrontierCandidateAction {
    #[action_event(level = info)]
    BlockReceived {
        hash: StateHash,
        block: Arc<MinaBlockBlockStableV2>,
        chain_proof: Option<(Vec<StateHash>, ArcBlockWithHash)>,
    },
    BlockPrevalidateSuccess {
        hash: StateHash,
    },
    BlockPrevalidateError {
        hash: StateHash,
        error: BlockPrevalidationError,
    },
    BlockChainProofUpdate {
        hash: StateHash,
        chain_proof: (Vec<StateHash>, ArcBlockWithHash),
    },
    BlockSnarkVerifyPending {
        req_id: SnarkBlockVerifyId,
        hash: StateHash,
    },
    #[action_event(level = info)]
    BlockSnarkVerifySuccess {
        hash: StateHash,
    },
    #[action_event(level = warn)]
    BlockSnarkVerifyError {
        hash: StateHash,
        error: SnarkBlockVerifyError,
    },
    DetectForkRange {
        hash: StateHash,
    },
    ShortRangeForkResolve {
        hash: StateHash,
    },
    LongRangeForkResolve {
        hash: StateHash,
    },
    #[action_event(level = info)]
    BestTipUpdate {
        hash: StateHash,
    },
    TransitionFrontierSyncTargetUpdate,
    P2pBestTipUpdate {
        best_tip: BlockWithHash<Arc<MinaBlockBlockStableV2>>,
    },
    Prune,
}

impl redux::EnablingCondition<crate::State> for TransitionFrontierCandidateAction {
    fn is_enabled(&self, state: &crate::State, _time: redux::Timestamp) -> bool {
        match self {
            TransitionFrontierCandidateAction::BlockReceived { hash, block, .. } => {
                let block = ArcBlockWithHash {
                    hash: hash.clone(),
                    block: block.clone()
                };
                !block.is_genesis() && !state.transition_frontier.candidates.blocks.contains_key(hash)
            },
            TransitionFrontierCandidateAction::BlockPrevalidateSuccess { hash }
            | TransitionFrontierCandidateAction::BlockPrevalidateError { hash, .. } => state
                .transition_frontier.candidates
                .blocks
                .get(hash)
                .is_some_and(|block| block.status.is_received()),
            TransitionFrontierCandidateAction::BlockChainProofUpdate { hash, .. } => {
                (state.transition_frontier.candidates.best_tip.as_ref() == Some(hash)
                    && state.transition_frontier.candidates.best_tip_chain_proof.is_none())
                    || state.transition_frontier
                        .candidates
                        .blocks
                        .get(hash)
                        .is_some_and( |b| b.status.is_pending() && b.chain_proof.is_none())
            },
            TransitionFrontierCandidateAction::BlockSnarkVerifyPending { req_id, hash } => {
                state
                    .transition_frontier
                    .candidates
                    .blocks
                    .get(hash)
                    .is_some_and( |block| block.status.is_prevalidated())
                    && state.snark.block_verify.jobs.contains(*req_id)
            },
            TransitionFrontierCandidateAction::BlockSnarkVerifySuccess { hash } => {
                state
                    .transition_frontier
                    .candidates
                    .blocks
                    .get(hash)
                    .is_some_and( |block| block.status.is_snark_verify_pending())
            },
            TransitionFrontierCandidateAction::BlockSnarkVerifyError { hash, .. } => {
                state
                    .transition_frontier
                    .candidates
                    .blocks
                    .get(hash)
                    .is_some_and( |block| block.status.is_snark_verify_pending())
            },
            TransitionFrontierCandidateAction::DetectForkRange { hash } => {
                state
                    .transition_frontier
                    .candidates
                    .blocks
                    .get(hash)
                    .is_some_and( |block| {
                        matches!(
                            block.status,
                            TransitionFrontierCandidateStatus::SnarkVerifySuccess { .. }
                        )
                    })
            },
            TransitionFrontierCandidateAction::ShortRangeForkResolve { hash } => {
                state
                    .transition_frontier
                    .candidates
                    .blocks
                    .get(hash)
                    .is_some_and( |block| match state.transition_frontier.candidates.best_tip() {
                        Some(tip) => {
                            matches!(
                                &block.status,
                                TransitionFrontierCandidateStatus::ForkRangeDetected { compared_with, short_fork, .. }
                                    if compared_with.as_ref() == Some(tip.hash) && *short_fork
                            )
                        }
                        None => true,
                    })
            },
            TransitionFrontierCandidateAction::LongRangeForkResolve { hash } => {
                state
                    .transition_frontier
                    .candidates
                    .blocks
                    .get(hash)
                    .is_some_and( |block| match state.transition_frontier.candidates.best_tip() {
                        Some(tip) => {
                            matches!(
                                &block.status,
                                TransitionFrontierCandidateStatus::ForkRangeDetected { compared_with, short_fork, .. }
                                     if compared_with.as_ref() == Some(tip.hash) && !*short_fork
                            )
                        }
                        None => false,
                    })
            },
            TransitionFrontierCandidateAction::BestTipUpdate { hash } => {
                state
                    .transition_frontier
                    .candidates
                    .is_candidate_decided_to_use_as_tip(hash)
            },
            TransitionFrontierCandidateAction::TransitionFrontierSyncTargetUpdate => {
                let Some(best_tip) = state.transition_frontier.candidates.best_tip_block_with_hash() else {
                    return false;
                };
                // do not need to update transition frontier sync target.
                if IntoIterator::into_iter([
                    state.transition_frontier.best_tip(),
                    state.transition_frontier.sync.best_tip(),
                ])
                .flatten()
                .any(|b| b.hash() == best_tip.hash()
                    || !consensus_take(b.consensus_state(), best_tip.consensus_state(), b.hash(), best_tip.hash())) {
                    return false;
                }

                // has enough data
                state.transition_frontier.candidates.best_tip_chain_proof(&state.transition_frontier).is_some()
            },
            TransitionFrontierCandidateAction::P2pBestTipUpdate { .. } => true,
            TransitionFrontierCandidateAction::Prune => {
                state.transition_frontier.candidates.best_tip().is_some()
            },
        }
    }
}

impl From<TransitionFrontierCandidateAction> for crate::Action {
    fn from(value: TransitionFrontierCandidateAction) -> Self {
        Self::TransitionFrontier(crate::TransitionFrontierAction::Candidate(value))
    }
}
