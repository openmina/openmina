use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// This structure contains the information needed to verify snark work.
/// This is basically: the verifier index of the circuits, and the SRS.
/// 
/// Blocks are verified by `ledger::proofs::verification::verify_block`
/// which requires:
/// - The block header in wire form: the verifier extracts the protocol state,
///   and the protocol state proof.
/// - The verifier index: this is taken from `block_verifier_index` to derive
///   the verification key.
/// - The SRS: this is taken from `block_verifier_srs`.
///
/// Snark work is verified by `ledger::proofs::verification::verify_transaction`
/// which required:
/// - A list of of statements and transaction snark proofs (in wire form) where
///   each statement contains information such as: first/second pass ledgers,
///   local state, and the `SokDigest` derived from the Snarker key and fee.
/// - The verifier index: this is taken from `work_verifier_index` to derive
///   the verification key.
/// - The SRS: this is taken from `work_verifier_srs`
/// 
/// Note: both `block_verifier_srs` and `work_verifier_srs` contain the same
/// SRS. Is there any reason to keep them twice?
#[derive(Serialize, Deserialize, Clone)]
pub struct SnarkConfig {
    pub block_verifier_index: Arc<crate::VerifierIndex>,
    pub block_verifier_srs: Arc<crate::VerifierSRS>,
    pub work_verifier_index: Arc<crate::VerifierIndex>,
    pub work_verifier_srs: Arc<crate::VerifierSRS>,
}

impl std::fmt::Debug for SnarkConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnarkConfig")
            .field("block_verifier_index", &"<content too big>")
            .field("block_verifier_srs", &"<content too big>")
            .field("work_verifier_index", &"<content too big>")
            .field("work_verifier_srs", &"<content too big>")
            .finish()
    }
}
