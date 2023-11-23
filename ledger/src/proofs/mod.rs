use kimchi::mina_curves::pasta::{Pallas, Vesta};

pub mod accumulator_check;
pub mod block;
pub mod caching;
pub mod constants;
pub mod group_map;
pub mod merge;
pub mod numbers;
pub mod opt_sponge;
mod prover;
pub mod public_input;
pub mod to_field_elements;
pub mod transition_chain;
pub mod unfinalized;
mod urs_utils;
pub mod util;
pub mod verification;
pub mod verifier_index;
pub mod witness;
pub mod wrap;
pub mod zkapp;
pub mod zkapp_interfaces;
pub mod zkapp_logic;

pub type VerifierIndex = kimchi::verifier_index::VerifierIndex<Pallas>;
pub type ProverProof = kimchi::proof::ProverProof<Pallas>;
pub type VerifierSRS = poly_commitment::srs::SRS<Vesta>;

pub const BACKEND_TICK_ROUNDS_N: usize = 16;
pub const BACKEND_TOCK_ROUNDS_N: usize = 15;
