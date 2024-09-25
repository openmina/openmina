use field::FieldWitness;
use poly_commitment::evaluation_proof::OpeningProof;

pub mod accumulator_check;
pub mod block;
pub mod caching;
pub mod constants;
mod conv;
pub mod field;
pub mod group_map;
pub mod merge;
pub mod numbers;
pub mod opt_sponge;
mod prover;
pub mod provers;
pub mod public_input;
pub mod step;
pub mod to_field_elements;
pub mod transaction;
pub mod transition_chain;
pub mod unfinalized;
mod urs_utils;
pub mod util;
pub mod verification;
pub mod verifier_index;
pub mod witness;
pub mod wrap;
pub mod zkapp;

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
