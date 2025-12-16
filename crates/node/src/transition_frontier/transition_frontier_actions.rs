use std::{collections::BTreeSet, sync::Arc};

use mina_core::{block::ArcBlockWithHash, ActionEvent};
use mina_p2p_messages::v2::StateHash;
use serde::{Deserialize, Serialize};

use super::{
    candidate::TransitionFrontierCandidateAction,
    genesis::TransitionFrontierGenesisAction,
    genesis_effectful::TransitionFrontierGenesisEffectfulAction,
    sync::{SyncError, TransitionFrontierSyncAction, TransitionFrontierSyncState},
};

pub type TransitionFrontierActionWithMeta = redux::ActionWithMeta<TransitionFrontierAction>;
pub type TransitionFrontierActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum TransitionFrontierAction {
    Genesis(TransitionFrontierGenesisAction),
    GenesisEffect(TransitionFrontierGenesisEffectfulAction),
    /// Inject genesis block into the transition frontier.
    ///
    /// Unless we already have a better block there.
    ///
    /// If this node is block producer, we produce proof for the genesis
    /// block, otherwise we don't need it so we use dummy proof instead.
    #[action_event(level = info)]
    GenesisInject,
    #[action_event(level = info)]
    GenesisProvenInject,

    Candidate(TransitionFrontierCandidateAction),
    Sync(TransitionFrontierSyncAction),
    /// Transition frontier synced.
    Synced {
        /// Required protocol states for root block.
        needed_protocol_states: BTreeSet<StateHash>,
    },
    SyncFailed {
        best_tip: ArcBlockWithHash,
        error: SyncError,
    },
}

impl redux::EnablingCondition<crate::State> for TransitionFrontierAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        match self {
            TransitionFrontierAction::Genesis(a) => a.is_enabled(state, time),
            TransitionFrontierAction::GenesisEffect(a) => a.is_enabled(state, time),
            TransitionFrontierAction::GenesisInject => {
                state.transition_frontier.root().is_none()
                    && state
                        .transition_frontier
                        .genesis
                        .block_with_real_or_dummy_proof()
                        .is_some()
            }
            TransitionFrontierAction::GenesisProvenInject => {
                let Some(genesis) = state.transition_frontier.genesis.proven_block() else {
                    return false;
                };
                state
                    .transition_frontier
                    .root()
                    .is_none_or(|b| b.is_genesis() && !Arc::ptr_eq(&genesis.block, &b.block))
            }
            TransitionFrontierAction::Candidate(a) => a.is_enabled(state, time),
            TransitionFrontierAction::Sync(a) => a.is_enabled(state, time),
            TransitionFrontierAction::Synced { .. } => matches!(
                state.transition_frontier.sync,
                TransitionFrontierSyncState::CommitSuccess { .. }
            ),
            TransitionFrontierAction::SyncFailed { best_tip, error } => {
                let sync = &state.transition_frontier.sync;
                sync.best_tip().is_some_and(|b| b.hash() == best_tip.hash())
                    && match error {
                        SyncError::BlockApplyFailed(block, _) => sync
                            .block_state(block.hash())
                            .is_some_and(|s| s.is_apply_error()),
                    }
            }
        }
    }
}
