use std::collections::BTreeMap;

use mina_p2p_messages::v2::{
    MinaStateProtocolStateBodyValueStableV2, MinaStateProtocolStateValueStableV2, StateHash,
};
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

use super::sync::TransitionFrontierSyncState;
use super::TransitionFrontierConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierState {
    pub config: TransitionFrontierConfig,
    /// Current best known chain, from root of the transition frontier to best tip
    pub best_chain: Vec<ArcBlockWithHash>,
    /// Needed protocol states for applying transactions in the root
    /// scan state that we don't have in the `best_chain` list.
    pub needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    /// Last transition frontier synchronization state
    pub sync: TransitionFrontierSyncState,
}

impl TransitionFrontierState {
    pub fn new(config: TransitionFrontierConfig) -> Self {
        let k = config.protocol_constants.k.0.as_u32() as usize;
        Self {
            config,
            // TODO(binier): add genesis_block as initial best_tip.
            best_chain: Vec::with_capacity(k),
            needed_protocol_states: Default::default(),
            sync: TransitionFrontierSyncState::Idle,
        }
    }

    pub fn best_tip(&self) -> Option<&ArcBlockWithHash> {
        self.best_chain.last()
    }

    /// Looks up state body by state hash.
    pub fn get_state_body(
        &self,
        hash: &StateHash,
    ) -> Option<&MinaStateProtocolStateBodyValueStableV2> {
        self.best_chain
            .iter()
            .find_map(|block_with_hash| {
                if &block_with_hash.hash == hash {
                    Some(&block_with_hash.block.header.protocol_state.body)
                } else {
                    None
                }
            })
            .or_else(|| {
                self.needed_protocol_states
                    .iter()
                    .find_map(|(block_hash, state)| {
                        if block_hash == hash {
                            Some(&state.body)
                        } else {
                            None
                        }
                    })
            })
    }
}
