mod merkle_path;

pub use ledger::proofs::accumulator_check::get_srs;
pub use ledger::proofs::caching::{
    srs_from_bytes, srs_to_bytes, verifier_index_from_bytes, verifier_index_to_bytes,
};
pub use ledger::proofs::verifier_index::{get_verifier_index, VerifierKind};
pub use ledger::proofs::{ProverProof, VerifierIndex, VerifierSRS};
pub use ledger::proofs::verification::verify_block;

pub use merkle_path::calc_merkle_root_hash;

pub mod block_verify;

mod snark_actions;
pub use snark_actions::*;

mod snark_config;
pub use snark_config::*;

mod snark_state;
pub use snark_state::*;

mod snark_reducer;
pub use snark_reducer::*;

use redux::SubStore;
pub trait SnarkStore<GlobalState>:
    SubStore<GlobalState, SnarkState, SubAction = SnarkAction>
{
}
impl<S, T: SubStore<S, SnarkState, SubAction = SnarkAction>> SnarkStore<S> for T {}
