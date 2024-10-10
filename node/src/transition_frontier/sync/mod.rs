pub mod ledger;

mod transition_frontier_sync_state;
pub use transition_frontier_sync_state::*;

mod transition_frontier_sync_actions;
pub use transition_frontier_sync_actions::*;

mod transition_frontier_sync_reducer;

mod transition_frontier_sync_effects;

use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerBlockFetchError {
    Timeout,
    Disconnected,
    DataUnavailable,
}

#[derive(thiserror::Error, Serialize, Deserialize, Debug, Clone)]
pub enum SyncError {
    #[error("sync failed due to block({}, {}) application error: {1}", .0.height(), .0.hash())]
    BlockApplyFailed(ArcBlockWithHash, String),
}

/// How close to the best tip we have to be for the full
/// verification of proofs contained in the block
/// body (zkApp txns and completed works) to be enabled.
const CATCHUP_BLOCK_VERIFY_TAIL_LENGTH: usize = 5;
