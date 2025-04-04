//! Manages the synchronization of the staged ledger, which contains
//! pending transactions and their effects that haven't yet been
//! incorporated into the snarked ledger.

mod transition_frontier_sync_ledger_staged_state;
use ark_ff::fields::arithmetic::InvalidBigInt;
use mina_hasher::Fp;
pub use transition_frontier_sync_ledger_staged_state::*;

mod transition_frontier_sync_ledger_staged_actions;
pub use transition_frontier_sync_ledger_staged_actions::*;

mod transition_frontier_sync_ledger_staged_reducer;

mod transition_frontier_sync_ledger_staged_service;
pub use transition_frontier_sync_ledger_staged_service::*;

use std::sync::Arc;

use ledger::{
    scan_state::{pending_coinbase::PendingCoinbase, scan_state::ScanState},
    staged_ledger::hash::StagedLedgerHash,
};
use mina_p2p_messages::v2::MinaBaseStagedLedgerHashStableV1;
use serde::{Deserialize, Serialize};

use crate::p2p::channels::rpc::StagedLedgerAuxAndPendingCoinbases;

/// Represents errors that can occur when fetching staged ledger parts from peers.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerStagedLedgerPartsFetchError {
    /// The fetch request timed out
    Timeout,
    /// The peer disconnected before responding
    Disconnected,
    /// The peer doesn't have the requested data
    DataUnavailable,
}

// TODO(binier): must be separate type
pub type StagedLedgerAuxAndPendingCoinbasesValid = StagedLedgerAuxAndPendingCoinbases;

/// Represents the result of validating staged ledger data received from peers.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StagedLedgerAuxAndPendingCoinbasesValidated {
    /// The staged ledger data is valid (hash matches expected value)
    Valid(Arc<StagedLedgerAuxAndPendingCoinbasesValid>),
    /// The staged ledger data is invalid (hash doesn't match expected value)
    Invalid(Arc<StagedLedgerAuxAndPendingCoinbases>),
}

/// Converts staged ledger parts from the P2P message format to internal types.
fn conv(
    parts: &StagedLedgerAuxAndPendingCoinbases,
) -> Result<(ScanState, PendingCoinbase, Fp), InvalidBigInt> {
    let scan_state: ScanState = (&parts.scan_state).try_into()?;
    let pending_coinbase: PendingCoinbase = (&parts.pending_coinbase).try_into()?;
    let staged_ledger_hash: Fp = parts.staged_ledger_hash.to_field()?;
    Ok((scan_state, pending_coinbase, staged_ledger_hash))
}

impl StagedLedgerAuxAndPendingCoinbasesValidated {
    /// Validates staged ledger data by checking that its hash matches the expected hash.
    ///
    /// This method:
    /// 1. Converts the data to internal types
    /// 2. Calculates the hash of the staged ledger
    /// 3. Compares it with the expected hash
    pub fn validate(
        parts: &Arc<StagedLedgerAuxAndPendingCoinbases>,
        expected_hash: &MinaBaseStagedLedgerHashStableV1,
    ) -> Self {
        // TODO(binier): PERF extra conversions and not caching hashes.
        let Ok((scan_state, mut pending_coinbase, staged_ledger_hash)) = conv(parts) else {
            return Self::Invalid(parts.clone());
        };

        let calculated_hash = StagedLedgerHash::of_aux_ledger_and_coinbase_hash(
            scan_state.hash(),
            staged_ledger_hash,
            &mut pending_coinbase,
        );
        let calculated_hash = (&calculated_hash).into();

        if expected_hash == &calculated_hash {
            Self::Valid(parts.clone())
        } else {
            Self::Invalid(parts.clone())
        }
    }

    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid(_))
    }
}
