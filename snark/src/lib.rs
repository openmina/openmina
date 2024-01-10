use kimchi::mina_curves::pasta::{Pallas, Vesta};

mod merkle_path;

pub use ledger::proofs::caching::{
    srs_from_bytes, srs_to_bytes, verifier_index_from_bytes, verifier_index_to_bytes,
};
pub use ledger::proofs::verifier_index::{get_verifier_index, VerifierKind};

pub use merkle_path::calc_merkle_root_hash;

pub mod block_verify;
pub mod work_verify;

mod snark_event;
pub use snark_event::*;

mod snark_actions;
pub use snark_actions::*;

mod snark_config;
pub use snark_config::*;

mod snark_state;
pub use snark_state::*;

mod snark_reducer;

pub type VerifierIndex = kimchi::verifier_index::VerifierIndex<Pallas>;
pub type VerifierSRS = poly_commitment::srs::SRS<Vesta>;

use redux::SubStore;
pub trait SnarkStore<GlobalState>:
    SubStore<GlobalState, SnarkState, SubAction = SnarkAction>
{
}
impl<S, T: SubStore<S, SnarkState, SubAction = SnarkAction>> SnarkStore<S> for T {}

pub fn get_srs() -> std::sync::Arc<std::sync::Mutex<poly_commitment::srs::SRS<Vesta>>> {
    ledger::verifier::get_srs::<mina_hasher::Fp>()
}
