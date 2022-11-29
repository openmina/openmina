use kimchi::mina_curves::pasta::Pallas;

pub mod accumulator_check;
mod prover;
pub mod transition_chain;
mod urs_utils;
pub mod verification;
pub mod verifier_index;

type VerifierIndex = kimchi::verifier_index::VerifierIndex<Pallas>;
type ProverProof = kimchi::proof::ProverProof<Pallas>;
