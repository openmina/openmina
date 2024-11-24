use std::sync::Arc;

use mina_p2p_messages::v2::{MinaBlockBlockStableV2, StateHash};
use openmina_core::block::{ArcBlockWithHash, BlockWithHash};
use openmina_core::consensus::consensus_take;
use openmina_core::{action_event, ActionEvent};
use serde::{Deserialize, Serialize};
use snark::block_verify::SnarkBlockVerifyError;

use crate::consensus::ConsensusBlockStatus;
use crate::snark::block_verify::SnarkBlockVerifyId;
use crate::state::BlockPrevalidationError;

pub type ConsensusActionWithMeta = redux::ActionWithMeta<ConsensusAction>;
pub type ConsensusActionWithMetaRef<'a> = redux::ActionWithMeta<&'a ConsensusAction>;

// NOTE: `debug(hash)` must be used instead of `display(hash)` because
// for some reason the later breaks CI. `hash = display(&hash)` works too.
#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = debug, fields(debug(hash), debug(error)))]
pub enum ConsensusAction {
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

impl redux::EnablingCondition<crate::State> for ConsensusAction {
    fn is_enabled(&self, state: &crate::State, _time: redux::Timestamp) -> bool {
        match self {
            ConsensusAction::BlockReceived { hash, block, .. } => {
                let block = ArcBlockWithHash {
                    hash: hash.clone(),
                    block: block.clone()
                };
                !block.is_genesis() && !state.consensus.blocks.contains_key(hash)
            },
            ConsensusAction::BlockPrevalidateSuccess { hash }
            | ConsensusAction::BlockPrevalidateError { hash, .. } => state
                .consensus
                .blocks
                .get(hash)
                .map_or(false, |block| block.status.is_received()),
            ConsensusAction::BlockChainProofUpdate { hash, .. } => {
                (state.consensus.best_tip.as_ref() == Some(hash)
                    && state.consensus.best_tip_chain_proof.is_none())
                    || state
                        .consensus
                        .blocks
                        .get(hash)
                        .map_or(false, |b| b.status.is_pending() && b.chain_proof.is_none())
            },
            ConsensusAction::BlockSnarkVerifyPending { req_id, hash } => {
                state
                    .consensus
                    .blocks
                    .get(hash)
                    .map_or(false, |block| block.status.is_prevalidated())
                    && state.snark.block_verify.jobs.contains(*req_id)
            },
            ConsensusAction::BlockSnarkVerifySuccess { hash } => {
                state
                    .consensus
                    .blocks
                    .get(hash)
                    .map_or(false, |block| block.status.is_snark_verify_pending())
            },
            ConsensusAction::BlockSnarkVerifyError { hash, .. } => {
                state
                    .consensus
                    .blocks
                    .get(hash)
                    .map_or(false, |block| block.status.is_snark_verify_pending())
            },
            ConsensusAction::DetectForkRange { hash } => {
                state
                    .consensus
                    .blocks
                    .get(hash)
                    .map_or(false, |block| {
                        matches!(
                            block.status,
                            ConsensusBlockStatus::SnarkVerifySuccess { .. }
                        )
                    })
            },
            ConsensusAction::ShortRangeForkResolve { hash } => {
                state
                    .consensus
                    .blocks
                    .get(hash)
                    .map_or(false, |block| match state.consensus.best_tip() {
                        Some(tip) => {
                            matches!(
                                &block.status,
                                ConsensusBlockStatus::ForkRangeDetected { compared_with, short_fork, .. }
                                    if compared_with.as_ref() == Some(tip.hash) && *short_fork
                            )
                        }
                        None => true,
                    })
            },
            ConsensusAction::LongRangeForkResolve { hash } => {
                state
                    .consensus
                    .blocks
                    .get(hash)
                    .map_or(false, |block| match state.consensus.best_tip() {
                        Some(tip) => {
                            matches!(
                                &block.status,
                                ConsensusBlockStatus::ForkRangeDetected { compared_with, short_fork, .. }
                                     if compared_with.as_ref() == Some(tip.hash) && !*short_fork
                            )
                        }
                        None => false,
                    })
            },
            ConsensusAction::BestTipUpdate { hash } => {
                state
                    .consensus
                    .is_candidate_decided_to_use_as_tip(hash)
            },
            ConsensusAction::TransitionFrontierSyncTargetUpdate => {
                let Some(best_tip) = state.consensus.best_tip_block_with_hash() else {
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
                state.consensus.best_tip_chain_proof(&state.transition_frontier).is_some()
            },
            ConsensusAction::P2pBestTipUpdate { .. } => true,
            ConsensusAction::Prune => {
                state.consensus.best_tip().is_some()
            },
        }
    }
}
