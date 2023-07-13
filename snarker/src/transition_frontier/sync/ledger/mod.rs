mod transition_frontier_sync_ledger_state;
pub use transition_frontier_sync_ledger_state::*;

mod transition_frontier_sync_ledger_actions;
pub use transition_frontier_sync_ledger_actions::*;

mod transition_frontier_sync_ledger_reducer;
pub use transition_frontier_sync_ledger_reducer::*;

mod transition_frontier_sync_ledger_effects;
pub use transition_frontier_sync_ledger_effects::*;

mod transition_frontier_sync_ledger_service;
pub use transition_frontier_sync_ledger_service::*;

use mina_p2p_messages::v2::{LedgerHash, MinaBaseAccountBinableArgStableV2};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerLedgerQueryResponse {
    ChildHashes(LedgerHash, LedgerHash),
    Accounts(Vec<MinaBaseAccountBinableArgStableV2>),
}

impl PeerLedgerQueryResponse {
    pub fn is_child_hashes(&self) -> bool {
        matches!(self, Self::ChildHashes(..))
    }

    pub fn is_child_accounts(&self) -> bool {
        matches!(self, Self::Accounts(..))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerLedgerQueryError {
    Timeout,
    Disconnected,
    DataUnavailable,
}
