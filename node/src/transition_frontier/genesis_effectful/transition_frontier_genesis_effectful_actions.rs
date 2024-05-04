use std::sync::Arc;

use mina_p2p_messages::v2::{ProverExtendBlockchainInputStableV2, StateHash};
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::transition_frontier::genesis::GenesisConfig;

pub type TransitionFrontierGenesisEffectfulActionWithMeta =
    redux::ActionWithMeta<TransitionFrontierGenesisEffectfulAction>;
pub type TransitionFrontierGenesisEffectfulActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierGenesisEffectfulAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = trace)]
pub enum TransitionFrontierGenesisEffectfulAction {
    LedgerLoadInit {
        config: Arc<GenesisConfig>,
    },
    ProveInit {
        block_hash: StateHash,
        input: Box<ProverExtendBlockchainInputStableV2>,
    },
}

impl redux::EnablingCondition<crate::State> for TransitionFrontierGenesisEffectfulAction {
    fn is_enabled(&self, _state: &crate::State, _time: redux::Timestamp) -> bool {
        true
    }
}

impl From<TransitionFrontierGenesisEffectfulAction> for crate::Action {
    fn from(value: TransitionFrontierGenesisEffectfulAction) -> Self {
        crate::transition_frontier::TransitionFrontierAction::GenesisEffect(value).into()
    }
}
