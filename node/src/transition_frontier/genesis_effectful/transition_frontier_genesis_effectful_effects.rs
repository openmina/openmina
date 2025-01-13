use redux::ActionMeta;

use crate::Store;

use super::{TransitionFrontierGenesisEffectfulAction, TransitionFrontierGenesisService};

impl TransitionFrontierGenesisEffectfulAction {
    pub fn effects<S>(&self, _: &ActionMeta, store: &mut Store<S>)
    where
        S: redux::Service + TransitionFrontierGenesisService,
    {
        match self {
            TransitionFrontierGenesisEffectfulAction::LedgerLoadInit { config, is_archive } => {
                store.service.load_genesis(config.clone(), *is_archive);
            }
            TransitionFrontierGenesisEffectfulAction::ProveInit { block_hash, input } => {
                store.service.prove(block_hash.clone(), input.clone());
            }
        }
    }
}
