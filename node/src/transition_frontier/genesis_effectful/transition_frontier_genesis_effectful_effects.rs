//! Implements side effects for genesis block generation,
//! handling interactions with external services for loading the genesis ledger
//! and proving the genesis block.

use redux::ActionMeta;

use crate::Store;

use super::{TransitionFrontierGenesisEffectfulAction, TransitionFrontierGenesisService};

impl TransitionFrontierGenesisEffectfulAction {
    /// Handles side effects for genesis effectful actions.
    ///
    /// This delegates to the appropriate service methods for loading the genesis ledger
    /// or proving the genesis block.
    pub fn effects<S>(&self, _: &ActionMeta, store: &mut Store<S>)
    where
        S: redux::Service + TransitionFrontierGenesisService,
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
