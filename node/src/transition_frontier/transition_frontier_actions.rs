use std::collections::BTreeSet;

use mina_p2p_messages::v2::StateHash;
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use super::genesis::TransitionFrontierGenesisAction;
use super::sync::{TransitionFrontierSyncAction, TransitionFrontierSyncState};

pub type TransitionFrontierActionWithMeta = redux::ActionWithMeta<TransitionFrontierAction>;
pub type TransitionFrontierActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum TransitionFrontierAction {
    Genesis(TransitionFrontierGenesisAction),
    /// Inject genesis block into the transition frontier.
    ///
    /// Unless we already have a better block there.
    ///
    /// If this node is block producer, we produce proof for the genesis
    /// block, otherwise we don't need it so we use dummy proof instead.
    #[action_event(level = info)]
    GenesisInject,

    Sync(TransitionFrontierSyncAction),
    /// Transition frontier synced.
    #[action_event(level = info)]
    Synced {
        /// Required protocol states for root block.
        needed_protocol_states: BTreeSet<StateHash>,
    },
}

impl redux::EnablingCondition<crate::State> for TransitionFrontierAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        match self {
            TransitionFrontierAction::Genesis(a) => a.is_enabled(state, time),
            TransitionFrontierAction::GenesisInject => {
                if state.transition_frontier.best_tip().is_some() {
                    return false;
                }
                let genesis_state = &state.transition_frontier.genesis;
                if state.block_producer.is_enabled() {
                    genesis_state.proven_block().is_some()
                } else {
                    genesis_state.block_with_dummy_proof().is_some()
                }
            }
            TransitionFrontierAction::Sync(a) => a.is_enabled(state, time),
            TransitionFrontierAction::Synced { .. } => matches!(
                state.transition_frontier.sync,
                TransitionFrontierSyncState::CommitSuccess { .. }
            ),
        }
    }
}
