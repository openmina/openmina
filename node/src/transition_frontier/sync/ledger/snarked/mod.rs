mod transition_frontier_sync_ledger_snarked_state;
pub use transition_frontier_sync_ledger_snarked_state::*;

mod transition_frontier_sync_ledger_snarked_actions;
pub use transition_frontier_sync_ledger_snarked_actions::*;

mod transition_frontier_sync_ledger_snarked_reducer;

mod transition_frontier_sync_ledger_snarked_effects;

mod transition_frontier_sync_ledger_snarked_service;
pub use transition_frontier_sync_ledger_snarked_service::*;

use mina_p2p_messages::v2::{LedgerHash, MinaBaseAccountBinableArgStableV2};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerLedgerQueryResponse {
    ChildHashes(LedgerHash, LedgerHash),
    ChildAccounts(Vec<MinaBaseAccountBinableArgStableV2>),
}

impl PeerLedgerQueryResponse {
    pub fn is_child_hashes(&self) -> bool {
        matches!(self, Self::ChildHashes(..))
    }

    pub fn is_child_accounts(&self) -> bool {
        matches!(self, Self::ChildAccounts(..))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerLedgerQueryError {
    Timeout,
    Disconnected,
    DataUnavailable,
}
