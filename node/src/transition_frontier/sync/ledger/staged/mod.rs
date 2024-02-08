mod transition_frontier_sync_ledger_staged_state;
pub use transition_frontier_sync_ledger_staged_state::*;

mod transition_frontier_sync_ledger_staged_actions;
pub use transition_frontier_sync_ledger_staged_actions::*;

mod transition_frontier_sync_ledger_staged_reducer;


mod transition_frontier_sync_ledger_staged_effects;


mod transition_frontier_sync_ledger_staged_service;
pub use transition_frontier_sync_ledger_staged_service::*;

use std::sync::Arc;

use ledger::{scan_state::scan_state::ScanState, staged_ledger::hash::StagedLedgerHash};
use mina_p2p_messages::v2::MinaBaseStagedLedgerHashStableV1;
use serde::{Deserialize, Serialize};

use crate::p2p::channels::rpc::StagedLedgerAuxAndPendingCoinbases;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerStagedLedgerPartsFetchError {
    Timeout,
    Disconnected,
    DataUnavailable,
}

// TODO(binier): must be separate type
pub type StagedLedgerAuxAndPendingCoinbasesValid = StagedLedgerAuxAndPendingCoinbases;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StagedLedgerAuxAndPendingCoinbasesValidated {
    Valid(Arc<StagedLedgerAuxAndPendingCoinbasesValid>),
    Invalid(Arc<StagedLedgerAuxAndPendingCoinbases>),
}

impl StagedLedgerAuxAndPendingCoinbasesValidated {
    pub fn validate(
        parts: &Arc<StagedLedgerAuxAndPendingCoinbases>,
        expected_hash: &MinaBaseStagedLedgerHashStableV1,
    ) -> Self {
        // TODO(binier): PERF extra conversions and not caching hashes.
        let scan_state: ScanState = (&parts.scan_state).into();
        let mut pending_coinbase = (&parts.pending_coinbase).into();

        let calculated_hash = StagedLedgerHash::of_aux_ledger_and_coinbase_hash(
            scan_state.hash(),
            parts.staged_ledger_hash.to_field(),
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
