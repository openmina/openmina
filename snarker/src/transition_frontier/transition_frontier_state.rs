use serde::{Deserialize, Serialize};
use shared::block::ArcBlockWithHash;

use super::sync::TransitionFrontierSyncState;
use super::TransitionFrontierConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierState {
    pub config: TransitionFrontierConfig,
    pub best_chain: Vec<ArcBlockWithHash>,
    pub sync: TransitionFrontierSyncState,
}

impl TransitionFrontierState {
    pub fn new(config: TransitionFrontierConfig) -> Self {
        let k = config.protocol_constants.k.0.as_u32() as usize;
        Self {
            config,
            // TODO(binier): add genesis_block as initial best_tip.
            best_chain: Vec::with_capacity(k),
            sync: TransitionFrontierSyncState::Idle,
        }
    }

    pub fn best_tip(&self) -> Option<&ArcBlockWithHash> {
        self.best_chain.last()
    }
}
