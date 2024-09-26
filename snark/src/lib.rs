use kimchi::mina_curves::pasta::Vesta;

mod merkle_path;

pub use ledger::proofs::caching::{
    srs_from_bytes, srs_to_bytes, verifier_index_from_bytes, verifier_index_to_bytes,
};
pub use ledger::proofs::verifiers::{BlockVerifier, TransactionVerifier};

pub use merkle_path::calc_merkle_root_hash;

pub mod block_verify;
pub mod block_verify_effectful;
pub mod user_command_verify;
pub mod user_command_verify_effectful;
pub mod work_verify;
pub mod work_verify_effectful;

mod snark_event;
pub use snark_event::*;

mod snark_actions;
pub use snark_actions::*;

mod snark_config;
pub use snark_config::*;

mod snark_state;
pub use snark_state::*;

mod snark_reducer;

pub type VerifierIndex = ledger::proofs::VerifierIndex<mina_curves::pasta::Fq>;
pub type VerifierSRS = poly_commitment::srs::SRS<Vesta>;

use redux::SubStore;
pub trait SnarkStore<GlobalState>:
    SubStore<GlobalState, SnarkState, SubAction = SnarkAction>
{
}
impl<S, T: SubStore<S, SnarkState, SubAction = SnarkAction>> SnarkStore<S> for T {}

pub fn get_srs() -> std::sync::Arc<poly_commitment::srs::SRS<Vesta>> {
    ledger::verifier::get_srs::<mina_hasher::Fp>()
}
