use std::sync::Arc;

use mina_p2p_messages::v2::{MinaBlockBlockStableV2, StateHash};
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};
use snark::block_verify::SnarkBlockVerifyError;

use crate::consensus::ConsensusBlockStatus;
use crate::snark::block_verify::SnarkBlockVerifyId;

pub type ConsensusActionWithMeta = redux::ActionWithMeta<ConsensusAction>;
pub type ConsensusActionWithMetaRef<'a> = redux::ActionWithMeta<&'a ConsensusAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConsensusAction {
    BlockReceived {
        hash: StateHash,
        block: Arc<MinaBlockBlockStableV2>,
        chain_proof: Option<(Vec<StateHash>, ArcBlockWithHash)>,
    },
    BlockChainProofUpdate {
        hash: StateHash,
        chain_proof: (Vec<StateHash>, ArcBlockWithHash),
    },
    BlockSnarkVerifyPending {
        req_id: SnarkBlockVerifyId,
        hash: StateHash,
    },
    BlockSnarkVerifySuccess {
        hash: StateHash,
    },
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
    BestTipUpdate {
        hash: StateHash,
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
                    .map_or(false, |block| block.status.is_received())
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
            ConsensusAction::Prune => {
                state.consensus.best_tip().is_some()
            },
        }
    }
}
