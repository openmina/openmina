use field::FieldWitness;
use poly_commitment::evaluation_proof::OpeningProof;

pub mod accumulator_check;
pub mod block;
// REVIEW(dw): STATUS: DONE
pub mod caching;
mod circuit_blobs;
// REVIEW(dw): STATUS: NOT REVIEW BUT CRITICAL!!!!
pub mod constants;
// REVIEW(dw): STATUS: Require checking mapping
mod conv;
// REVIEW(dw): STATUS - Require discussion
pub mod field;
pub mod group_map;
pub mod merge;
// REVIEW(dw): STATUS: DONE
pub mod numbers;
// REVIEW(dw): STATUS: DONE
pub mod opt_sponge;
mod prover;
pub mod provers;
pub mod public_input;
pub mod step;
pub mod to_field_elements;
pub mod transaction;
// OK
pub mod transition_chain;
pub mod unfinalized;
// REVIEW(dw): STATUS: DONE
mod urs_utils;
// REVIEW(dw): STATUS: DONE
pub mod util;
pub mod verification;
pub mod verifiers;
pub mod witness;
pub mod wrap;
pub mod zkapp;

// REVIEW(dw): check!!!
pub const BACKEND_TICK_ROUNDS_N: usize = 16;
pub const BACKEND_TOCK_ROUNDS_N: usize = 15;

pub type VerifierIndex<F> = kimchi::verifier_index::VerifierIndex<
    <F as FieldWitness>::OtherCurve,
    OpeningProof<<F as FieldWitness>::OtherCurve>,
>;
pub type ProverIndex<F> = kimchi::prover_index::ProverIndex<
    <F as FieldWitness>::OtherCurve,
    OpeningProof<<F as FieldWitness>::OtherCurve>,
>;
pub type ProverProof<F> = kimchi::proof::ProverProof<
    <F as FieldWitness>::OtherCurve,
    OpeningProof<<F as FieldWitness>::OtherCurve>,
>;

pub fn generate_tx_proof(
    params: transaction::TransactionParams,
) -> Result<wrap::WrapProof, transaction::ProofError> {
    use {mina_hasher::Fp, witness::Witness};
    let mut w: Witness<Fp> = Witness::new::<constants::StepTransactionProof>();
    transaction::generate_tx_proof(params, &mut w)
}
pub fn generate_merge_proof(
    params: merge::MergeParams,
) -> Result<wrap::WrapProof, transaction::ProofError> {
    use {mina_hasher::Fp, witness::Witness};
    let mut w: Witness<Fp> = Witness::new::<constants::StepMergeProof>();
    merge::generate_merge_proof(params, &mut w)
}
pub fn generate_block_proof(
    params: block::BlockParams,
) -> Result<wrap::WrapProof, transaction::ProofError> {
    use {mina_hasher::Fp, witness::Witness};
    let mut w: Witness<Fp> = Witness::new::<constants::StepBlockProof>();
    block::generate_block_proof(params, &mut w)
}
pub use zkapp::generate_zkapp_proof;
