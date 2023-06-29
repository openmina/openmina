use mina_p2p_messages::v2::StateHash;
use serde::{Deserialize, Serialize};
use shared::block::ArcBlockWithHash;

use super::{sync::ledger::TransitionFrontierSyncLedgerAction, TransitionFrontierSyncState};

pub type TransitionFrontierActionWithMeta = redux::ActionWithMeta<TransitionFrontierAction>;
pub type TransitionFrontierActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierAction {
    SyncInit(TransitionFrontierSyncInitAction),
    SyncBestTipUpdate(TransitionFrontierSyncBestTipUpdateAction),
    RootLedgerSyncPending(TransitionFrontierRootLedgerSyncPendingAction),

    SyncLedger(TransitionFrontierSyncLedgerAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncInitAction {
    pub best_tip: ArcBlockWithHash,
    pub root_block: ArcBlockWithHash,
    pub blocks_inbetween: Vec<StateHash>,
}

impl redux::EnablingCondition<crate::State> for TransitionFrontierSyncInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        !state.transition_frontier.sync.is_pending()
            && state
                .transition_frontier
                .best_tip()
                .map_or(true, |tip| self.best_tip.hash != tip.hash)
            && state
                .consensus
                .best_tip()
                .map_or(false, |tip| &self.best_tip.hash == tip.hash)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncBestTipUpdateAction {
    pub best_tip: ArcBlockWithHash,
    pub root_block: ArcBlockWithHash,
    pub blocks_inbetween: Vec<StateHash>,
}

impl redux::EnablingCondition<crate::State> for TransitionFrontierSyncBestTipUpdateAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.transition_frontier.sync.is_pending()
            && state
                .transition_frontier
                .best_tip()
                .map_or(true, |tip| self.best_tip.hash != tip.hash)
            && state
                .transition_frontier
                .sync
                .best_tip()
                .map_or(true, |tip| self.best_tip.hash != tip.hash)
            && state
                .consensus
                .best_tip()
                .map_or(true, |tip| &self.best_tip.hash == tip.hash)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierRootLedgerSyncPendingAction {}

impl redux::EnablingCondition<crate::State> for TransitionFrontierRootLedgerSyncPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        matches!(
            state.transition_frontier.sync,
            TransitionFrontierSyncState::Init { .. }
        )
    }
}

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::TransitionFrontier(value.into())
            }
        }
    };
}

impl_into_global_action!(TransitionFrontierSyncInitAction);
impl_into_global_action!(TransitionFrontierSyncBestTipUpdateAction);
impl_into_global_action!(TransitionFrontierRootLedgerSyncPendingAction);
