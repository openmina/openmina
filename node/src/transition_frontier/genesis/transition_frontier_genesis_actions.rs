use mina_p2p_messages::v2;
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use super::{GenesisConfigLoaded, TransitionFrontierGenesisState};

pub type TransitionFrontierGenesisActionWithMeta =
    redux::ActionWithMeta<TransitionFrontierGenesisAction>;
pub type TransitionFrontierGenesisActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierGenesisAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = trace)]
pub enum TransitionFrontierGenesisAction {
    LedgerLoadInit,
    LedgerLoadPending,
    LedgerLoadSuccess {
        data: GenesisConfigLoaded,
    },
    Produce,
    ProveInit,
    /// Proving genesis block.
    #[action_event(level = info)]
    ProvePending,
    /// Genesis block proved.
    #[action_event(level = info)]
    ProveSuccess {
        proof: Box<v2::MinaBaseProofStableV2>,
    },
}

impl redux::EnablingCondition<crate::State> for TransitionFrontierGenesisAction {
    fn is_enabled(&self, state: &crate::State, _time: redux::Timestamp) -> bool {
        let genesis_state = &state.transition_frontier.genesis;
        match self {
            TransitionFrontierGenesisAction::LedgerLoadInit => {
                matches!(genesis_state, TransitionFrontierGenesisState::Idle { .. })
            }
            TransitionFrontierGenesisAction::LedgerLoadPending => {
                matches!(genesis_state, TransitionFrontierGenesisState::Idle { .. })
            }
            TransitionFrontierGenesisAction::LedgerLoadSuccess { .. } => {
                matches!(
                    genesis_state,
                    TransitionFrontierGenesisState::LedgerLoadPending { .. }
                )
            }
            TransitionFrontierGenesisAction::Produce => matches!(
                genesis_state,
                TransitionFrontierGenesisState::LedgerLoadSuccess { .. }
            ),
            TransitionFrontierGenesisAction::ProveInit => {
                state.block_producer.is_enabled()
                    && matches!(
                        genesis_state,
                        TransitionFrontierGenesisState::Produced { .. }
                    )
            }
            TransitionFrontierGenesisAction::ProvePending => matches!(
                genesis_state,
                TransitionFrontierGenesisState::Produced { .. }
            ),
            TransitionFrontierGenesisAction::ProveSuccess { .. } => matches!(
                genesis_state,
                TransitionFrontierGenesisState::ProvePending { .. }
            ),
        }
    }
}

impl From<TransitionFrontierGenesisAction> for crate::Action {
    fn from(value: TransitionFrontierGenesisAction) -> Self {
        crate::transition_frontier::TransitionFrontierAction::Genesis(value).into()
    }
}
