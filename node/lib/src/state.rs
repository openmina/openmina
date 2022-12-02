use redux::{ActionMeta, Timestamp};
use serde::{Deserialize, Serialize};

pub use crate::p2p::P2pState;
pub use crate::rpc::RpcState;
pub use crate::snark::SnarkState;
use crate::ActionWithMeta;
pub use crate::Config;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    pub p2p: P2pState,
    pub snark: SnarkState,
    pub rpc: RpcState,

    // TODO(binier): include action kind in `last_action`.
    pub last_action: ActionMeta,
    pub applied_actions_count: u64,
}

impl State {
    pub fn new(config: Config) -> Self {
        Self {
            p2p: P2pState::new(),
            snark: SnarkState::new(config.snark),
            rpc: RpcState::new(),

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
