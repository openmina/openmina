use mina_p2p_messages::{v1::StateHashStable, v2::MinaBlockHeaderStableV2};
use serde::{Deserialize, Serialize};

use crate::snark::block_verify::SnarkBlockVerifyId;

use super::is_short_range_fork;

pub type ConsensusActionWithMeta = redux::ActionWithMeta<ConsensusAction>;
pub type ConsensusActionWithMetaRef<'a> = redux::ActionWithMeta<&'a ConsensusAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum ConsensusAction {
    BlockReceived(ConsensusBlockReceivedAction),
    BlockSnarkVerifyPending(ConsensusBlockSnarkVerifyPendingAction),
    BlockSnarkVerifySuccess(ConsensusBlockSnarkVerifySuccessAction),
    ShortRangeForkResolve(ConsensusShortRangeForkResolveAction),
    BestTipUpdate(ConsensusBestTipUpdateAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsensusBlockReceivedAction {
    pub hash: StateHashStable,
    pub header: MinaBlockHeaderStableV2,
}

impl redux::EnablingCondition<crate::State> for ConsensusBlockReceivedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.consensus.best_tip().map_or(true, |tip| {
            let height = self
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
    pub hash: StateHashStable,
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
    pub hash: StateHashStable,
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
pub struct ConsensusShortRangeForkResolveAction {
    pub hash: StateHashStable,
}

impl redux::EnablingCondition<crate::State> for ConsensusShortRangeForkResolveAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .consensus
            .blocks
            .get(&self.hash)
            .filter(|block| block.status.is_snark_verify_success())
            .map_or(false, |block| match state.consensus.best_tip() {
                Some(tip) => is_short_range_fork(
                    &tip.header.protocol_state.body,
                    &block.header.protocol_state.body,
                ),
                None => true,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsensusBestTipUpdateAction {
    pub hash: StateHashStable,
}

impl redux::EnablingCondition<crate::State> for ConsensusBestTipUpdateAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .consensus
            .is_candidate_decided_to_use_as_tip(&self.hash)
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
impl_into_global_action!(ConsensusShortRangeForkResolveAction);
impl_into_global_action!(ConsensusBestTipUpdateAction);
