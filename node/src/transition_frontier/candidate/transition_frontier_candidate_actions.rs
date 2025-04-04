use mina_p2p_messages::v2::StateHash;
use openmina_core::block::prevalidate::BlockPrevalidationError;
use openmina_core::block::ArcBlockWithHash;
use openmina_core::consensus::consensus_take;
use openmina_core::{action_event, ActionEvent};
use serde::{Deserialize, Serialize};
use snark::block_verify::SnarkBlockVerifyError;

use crate::snark::block_verify::SnarkBlockVerifyId;

pub type TransitionFrontierCandidateActionWithMeta =
    redux::ActionWithMeta<TransitionFrontierCandidateAction>;
pub type TransitionFrontierCandidateActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierCandidateAction>;

// NOTE: `debug(hash)` must be used instead of `display(hash)` because
// for some reason the later breaks CI. `hash = display(&hash)` works too.
#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = debug, fields(debug(hash), debug(error)))]
pub enum TransitionFrontierCandidateAction {
    P2pBestTipUpdate {
        best_tip: ArcBlockWithHash,
    },
    BlockReceived {
        block: ArcBlockWithHash,
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
    TransitionFrontierSyncTargetUpdate,
    Prune,
}

impl redux::EnablingCondition<crate::State> for TransitionFrontierCandidateAction {
    fn is_enabled(&self, state: &crate::State, _time: redux::Timestamp) -> bool {
        match self {
            TransitionFrontierCandidateAction::P2pBestTipUpdate { .. } => true,
            TransitionFrontierCandidateAction::BlockReceived { block, .. } => {
                !block.is_genesis() && !state.transition_frontier.candidates.contains(block.hash())
            }
            TransitionFrontierCandidateAction::BlockPrevalidateSuccess { hash }
            | TransitionFrontierCandidateAction::BlockPrevalidateError { hash, .. } => state
                .transition_frontier
                .candidates
                .get(hash)
                .is_some_and(|block| block.status.is_received()),
            TransitionFrontierCandidateAction::BlockChainProofUpdate { hash, .. } => state
                .transition_frontier
                .candidates
                .is_chain_proof_needed(hash),
            TransitionFrontierCandidateAction::BlockSnarkVerifyPending { req_id, hash } => {
                state
                    .transition_frontier
                    .candidates
                    .get(hash)
                    .is_some_and(|block| block.status.is_prevalidated())
                    && state.snark.block_verify.jobs.contains(*req_id)
            }
            TransitionFrontierCandidateAction::BlockSnarkVerifySuccess { hash } => state
                .transition_frontier
                .candidates
                .get(hash)
                .is_some_and(|block| block.status.is_snark_verify_pending()),
            TransitionFrontierCandidateAction::BlockSnarkVerifyError { hash, .. } => state
                .transition_frontier
                .candidates
                .get(hash)
                .is_some_and(|block| block.status.is_snark_verify_pending()),
            TransitionFrontierCandidateAction::TransitionFrontierSyncTargetUpdate => {
                let Some(best_candidate) =
                    state.transition_frontier.candidates.best_verified_block()
                else {
                    return false;
                };
                // do not need to update transition frontier sync target.
                if IntoIterator::into_iter([
                    state.transition_frontier.best_tip(),
                    state.transition_frontier.sync.best_tip(),
                ])
                .flatten()
                .any(|b| {
                    b.hash() == best_candidate.hash()
                        || !consensus_take(
                            b.consensus_state(),
                            best_candidate.consensus_state(),
                            b.hash(),
                            best_candidate.hash(),
                        )
                }) {
                    return false;
                }

                // has enough data
                state
                    .transition_frontier
                    .candidates
                    .best_verified_block_chain_proof(&state.transition_frontier)
                    .is_some()
            }
            TransitionFrontierCandidateAction::Prune => {
                state.transition_frontier.candidates.best().is_some()
            }
        }
    }
}

impl From<TransitionFrontierCandidateAction> for crate::Action {
    fn from(value: TransitionFrontierCandidateAction) -> Self {
        Self::TransitionFrontier(crate::TransitionFrontierAction::Candidate(value))
    }
}
