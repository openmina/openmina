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

fn conv(
    parts: &StagedLedgerAuxAndPendingCoinbases,
) -> Result<(ScanState, PendingCoinbase, Fp), InvalidBigInt> {
    let scan_state: ScanState = (&parts.scan_state).try_into()?;
    let pending_coinbase: PendingCoinbase = (&parts.pending_coinbase).try_into()?;
    let staged_ledger_hash: Fp = parts.staged_ledger_hash.to_field()?;
    Ok((scan_state, pending_coinbase, staged_ledger_hash))
}

impl StagedLedgerAuxAndPendingCoinbasesValidated {
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
