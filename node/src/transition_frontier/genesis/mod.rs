//! Handles the generation and management of the genesis block,
//! which is the first block in the blockchain and serves as the foundation
//! for all subsequent blocks.

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

/// Creates an empty block body for the genesis block.
///
/// This is used when initializing the genesis block, which doesn't contain
/// any transactions but needs a valid empty diff structure.
pub(super) fn empty_block_body() -> v2::StagedLedgerDiffDiffStableV2 {
    use ledger::staged_ledger::diff::with_valid_signatures_and_proofs::Diff;
    (&Diff::empty()).into()
}

/// Computes the hash of an empty block body.
///
/// This hash is used in the genesis block to reference the empty block body,
/// which is required for a valid block structure even though there are no transactions.
pub(super) fn empty_block_body_hash() -> v2::ConsensusBodyReferenceStableV1 {
    use ledger::staged_ledger::validate_block::block_body_hash;
    block_body_hash(&empty_block_body()).unwrap()
}

/// Creates an empty pending coinbase structure for the genesis block.
///
/// The pending coinbase is a data structure that tracks coinbase rewards
/// that have been included in blocks but not yet applied to accounts.
/// For genesis, we need an empty but valid structure.
pub(super) fn empty_pending_coinbase() -> PendingCoinbase {
    let mut v = PendingCoinbase::create(constraint_constants().pending_coinbase_depth);
    v.merkle_root(); // Calculate and cache the merkle root
    v
}

/// Computes the hash of an empty pending coinbase structure.
///
/// This hash is used in the genesis block to represent the state of the
/// pending coinbase tree, which is initially empty but must have a valid hash.
pub fn empty_pending_coinbase_hash() -> v2::PendingCoinbaseHash {
    v2::MinaBasePendingCoinbaseHashVersionedStableV1(
        v2::MinaBasePendingCoinbaseHashBuilderStableV1(
            empty_pending_coinbase().merkle_root().into(),
        ),
    )
    .into()
}
