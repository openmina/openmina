pub mod ledger;

mod transition_frontier_sync_state;
pub use transition_frontier_sync_state::*;

mod transition_frontier_sync_actions;
pub use transition_frontier_sync_actions::*;

mod transition_frontier_sync_reducer;

mod transition_frontier_sync_effects;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerBlockFetchError {
    Timeout,
    Disconnected,
    DataUnavailable,
}
