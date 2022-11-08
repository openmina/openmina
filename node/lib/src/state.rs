use redux::{ActionMeta, Timestamp};
use serde::{Deserialize, Serialize};

pub use p2p::P2pState;

use crate::ActionWithMeta;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    pub p2p: P2pState,

    // TODO(binier): include action kind in `last_action`.
    pub last_action: ActionMeta,
    pub applied_actions_count: u64,
}

impl State {
    pub fn new() -> Self {
        Self {
            p2p: P2pState::new(),

            last_action: ActionMeta::ZERO,
            applied_actions_count: 0,
        }
    }

    #[inline(always)]
    pub fn time(&self) -> Timestamp {
        self.last_action.time()
    }

    pub fn action_applied(&mut self, action: &ActionWithMeta) {
        self.last_action = action.meta().clone();
        self.applied_actions_count += 1;
    }
}
