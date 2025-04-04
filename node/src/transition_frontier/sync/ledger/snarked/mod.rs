//! Manages the synchronization of the snarked ledger, which is the
//! cryptographically proven state of accounts in the blockchain.
//! This module handles fetching and validating ledger data from peers.

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

/// Represents responses to ledger queries sent to peers during snarked ledger synchronization.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerLedgerQueryResponse {
    /// Response containing the hashes of the two child nodes in the ledger Merkle tree
    ChildHashes(LedgerHash, LedgerHash),
    /// Response containing a list of account data for leaf nodes
    ChildAccounts(Vec<MinaBaseAccountBinableArgStableV2>),
    /// Response containing the number of accounts and the ledger hash
    NumAccounts(u64, LedgerHash),
}

impl PeerLedgerQueryResponse {
    pub fn is_child_hashes(&self) -> bool {
        matches!(self, Self::ChildHashes(..))
    }

    pub fn is_child_accounts(&self) -> bool {
        matches!(self, Self::ChildAccounts(..))
    }

    pub fn is_num_accounts(&self) -> bool {
        matches!(self, Self::NumAccounts(..))
    }
}

/// Represents errors that can occur during ledger queries to peers.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerLedgerQueryError {
    /// The query timed out waiting for a response
    Timeout,
    /// The peer disconnected before responding
    Disconnected,
    /// The peer doesn't have the requested data
    DataUnavailable,
}
