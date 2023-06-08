use kimchi::mina_curves::pasta::{Pallas, Vesta};

pub mod accumulator_check;
pub mod caching;
mod prover;
pub mod public_input;
pub mod transition_chain;
mod urs_utils;
pub mod util;
pub mod verification;
pub mod verifier_index;

pub type VerifierIndex = kimchi::verifier_index::VerifierIndex<Pallas>;
pub type ProverProof = kimchi::proof::ProverProof<Pallas>;
pub type VerifierSRS = poly_commitment::srs::SRS<Vesta>;
