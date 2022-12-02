use ark_ec::short_weierstrass_jacobian::GroupAffine;
use kimchi::mina_curves::pasta::{Pallas, VestaParameters};

pub mod accumulator_check;
mod prover;
pub mod transition_chain;
mod urs_utils;
pub mod verification;
pub mod verifier_index;

pub type VerifierIndex = kimchi::verifier_index::VerifierIndex<Pallas>;
pub type ProverProof = kimchi::proof::ProverProof<Pallas>;
pub type VerifierSRS = commitment_dlog::srs::SRS<GroupAffine<VestaParameters>>;
