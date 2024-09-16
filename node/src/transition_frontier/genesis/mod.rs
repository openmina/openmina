mod transition_frontier_genesis_config;
pub use transition_frontier_genesis_config::*;

mod transition_frontier_genesis_state;
pub use transition_frontier_genesis_state::*;

mod transition_frontier_genesis_actions;
pub use transition_frontier_genesis_actions::*;

mod transition_frontier_genesis_reducer;

use ledger::scan_state::pending_coinbase::PendingCoinbase;
use mina_p2p_messages::v2;
use openmina_core::constants::constraint_constants;

pub(super) fn empty_block_body() -> v2::StagedLedgerDiffDiffStableV2 {
    use ledger::staged_ledger::diff::with_valid_signatures_and_proofs::Diff;
    (&Diff::empty()).into()
}

pub(super) fn empty_block_body_hash() -> v2::ConsensusBodyReferenceStableV1 {
    use ledger::staged_ledger::validate_block::block_body_hash;
    block_body_hash(&empty_block_body()).unwrap()
}

pub(super) fn empty_pending_coinbase() -> PendingCoinbase {
    let mut v = PendingCoinbase::create(constraint_constants().pending_coinbase_depth);
    v.merkle_root();
    v
}

pub fn empty_pending_coinbase_hash() -> v2::PendingCoinbaseHash {
    v2::MinaBasePendingCoinbaseHashVersionedStableV1(
        v2::MinaBasePendingCoinbaseHashBuilderStableV1(
            empty_pending_coinbase().merkle_root().into(),
        ),
    )
    .into()
}
