use redux::ActionMeta;

use crate::Store;

use super::{TransitionFrontierGenesisEffectfulAction, TransitionFrontierGenesisService};

impl TransitionFrontierGenesisEffectfulAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>)
    where
        S: TransitionFrontierGenesisService,
    {
        match self {
            TransitionFrontierGenesisEffectfulAction::LedgerLoadInit { config } => {
                store.service.load_genesis(config.clone());
            }
            TransitionFrontierGenesisEffectfulAction::ProveInit { block_hash, input } => {
                store.service.prove(block_hash.clone(), input.clone());
            }
        }
    }
}
