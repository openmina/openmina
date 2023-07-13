use std::sync::Arc;

use mina_p2p_messages::v2::MinaBlockBlockStableV2;
use mina_p2p_messages::v2::StateHash;
use serde::{Deserialize, Serialize};

use crate::consensus::ConsensusBlockStatus;
use crate::snark::block_verify::SnarkBlockVerifyId;

use super::is_short_range_fork;

pub type ConsensusActionWithMeta = redux::ActionWithMeta<ConsensusAction>;
pub type ConsensusActionWithMetaRef<'a> = redux::ActionWithMeta<&'a ConsensusAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum ConsensusAction {
    BlockReceived(ConsensusBlockReceivedAction),
    BlockSnarkVerifyPending(ConsensusBlockSnarkVerifyPendingAction),
    BlockSnarkVerifySuccess(ConsensusBlockSnarkVerifySuccessAction),
    DetectForkRange(ConsensusDetectForkRangeAction),
    ShortRangeForkResolve(ConsensusShortRangeForkResolveAction),
    LongRangeForkResolve(ConsensusLongRangeForkResolveAction),
    BestTipUpdate(ConsensusBestTipUpdateAction),
    BestTipHistoryUpdate(ConsensusBestTipHistoryUpdateAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsensusBlockReceivedAction {
    pub hash: StateHash,
    pub block: Arc<MinaBlockBlockStableV2>,
    // Sorted from newest to oldest block starting with predecessor hash.
    pub history: Option<Vec<StateHash>>,
}

impl redux::EnablingCondition<crate::State> for ConsensusBlockReceivedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.consensus.best_tip().map_or(true, |tip| {
            let height = self
                .block
                .header
                .protocol_state
                .body
                .consensus_state
                .blockchain_length
                .0
                 .0;
            let tip_height = tip.height();
            height > tip_height || (height == tip_height && &self.hash != tip.hash)
        }) && !state.consensus.blocks.contains_key(&self.hash)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsensusBlockSnarkVerifyPendingAction {
    pub req_id: SnarkBlockVerifyId,
    pub hash: StateHash,
}

impl redux::EnablingCondition<crate::State> for ConsensusBlockSnarkVerifyPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .consensus
            .blocks
            .get(&self.hash)
            .map_or(false, |block| block.status.is_received())
            && state.snark.block_verify.jobs.contains(self.req_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsensusBlockSnarkVerifySuccessAction {
    pub hash: StateHash,
}

impl redux::EnablingCondition<crate::State> for ConsensusBlockSnarkVerifySuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .consensus
            .blocks
            .get(&self.hash)
            .map_or(false, |block| block.status.is_snark_verify_pending())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsensusDetectForkRangeAction {
    pub hash: StateHash,
}

impl redux::EnablingCondition<crate::State> for ConsensusDetectForkRangeAction {
    fn is_enabled(&self, #[allow(unused_variables)] state: &crate::State) -> bool {
        state
            .consensus
            .blocks
            .get(&self.hash)
            .map_or(false, |block| {
                matches!(
                    block.status,
                    ConsensusBlockStatus::SnarkVerifySuccess { .. }
                )
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsensusShortRangeForkResolveAction {
    pub hash: StateHash,
}

impl redux::EnablingCondition<crate::State> for ConsensusShortRangeForkResolveAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .consensus
            .blocks
            .get(&self.hash)
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
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsensusLongRangeForkResolveAction {
    pub hash: StateHash,
}

impl redux::EnablingCondition<crate::State> for ConsensusLongRangeForkResolveAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .consensus
            .blocks
            .get(&self.hash)
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
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsensusBestTipUpdateAction {
    pub hash: StateHash,
}

impl redux::EnablingCondition<crate::State> for ConsensusBestTipUpdateAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .consensus
            .is_candidate_decided_to_use_as_tip(&self.hash)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsensusBestTipHistoryUpdateAction {
    pub tip_hash: StateHash,
    // Sorted from newest to oldest block starting with predecessor hash.
    pub history: Vec<StateHash>,
}

impl redux::EnablingCondition<crate::State> for ConsensusBestTipHistoryUpdateAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.consensus.best_tip.as_ref() == Some(&self.tip_hash)
    }
}

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::Consensus(value.into())
            }
        }
    };
}

impl_into_global_action!(ConsensusBlockReceivedAction);
impl_into_global_action!(ConsensusBlockSnarkVerifyPendingAction);
impl_into_global_action!(ConsensusBlockSnarkVerifySuccessAction);
impl_into_global_action!(ConsensusDetectForkRangeAction);
impl_into_global_action!(ConsensusShortRangeForkResolveAction);
impl_into_global_action!(ConsensusLongRangeForkResolveAction);
impl_into_global_action!(ConsensusBestTipUpdateAction);
impl_into_global_action!(ConsensusBestTipHistoryUpdateAction);
