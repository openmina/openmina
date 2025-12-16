//! # SNARK Verification Orchestration
//!
//! The SNARK crate provides zero-knowledge proof verification capabilities for
//! the Mina Rust node, orchestrating the verification of blocks, transactions,
//! and SNARK work through a Redux-style state machine architecture.
//!
//! ## Overview
//!
//! This crate handles three main types of proof verification:
//! - **Block Verification**: Validates blockchain blocks and their proofs
//! - **Transaction Verification**: Verifies user commands and zkApp
//!   transactions
//! - **Work Verification**: Validates SNARK work proofs from workers
//!
//! ## Architecture
//!
//! The crate follows the Mina node's Redux architecture pattern with:
//! - **State**: [`SnarkState`] - Centralized verification state
//! - **Actions**: [`SnarkAction`] - Events triggering verification operations
//! - **Enabling Conditions**: [`redux::EnablingCondition`] - Guards preventing
//!   invalid state transitions
//! - **Reducers**: Pure functions managing state transitions
//! - **Effects**: Service interactions for actual proof verification
//!
//! You can find more information regarding the Redux pattern in the
//! documentation at
//! <https://o1-labs.github.io/mina-rust/docs/developers/architecture>.
//!
//! ## Core Components
//!
//! ### Verification State Machine
//!
//! Each verification type maintains its own state machine:
//! - [`block_verify`] - Block proof verification state machine
//! - [`user_command_verify`] - User command verification state machine
//! - [`work_verify`] - SNARK work verification state machine
//!
//! ### Effectful Operations
//!
//! Operations run in separate service threads:
//! - [`block_verify_effectful`] - Block verification services
//! - [`user_command_verify_effectful`] - Transaction verification services
//! - [`work_verify_effectful`] - Work verification services
//!
//! ## Configuration
//!
//! The [`SnarkConfig`] contains verifier indices and SRS parameters required
//! for proof verification. These are network-specific and loaded during
//! initialization.
//!
//! ## Integration
//!
//! The SNARK crate integrates with:
//! - **Ledger**: Uses cryptographic primitives from the ledger crate
//! - **Kimchi**: Leverages the Kimchi proving system for verification, used
//!   since Berkeley.
//! - **Node**: Provides verification services to the main node
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use snark::{SnarkConfig, SnarkState};
//!
//! // Initialize SNARK state with configuration
//! let config = SnarkConfig { /* ... */ };
//! let state = SnarkState::new(config);
//!
//! // The state machine handles verification requests through actions
//! // dispatched by the main node's Redux store
//! ```
//!
//! ## Performance Considerations
//!
//! - Verifier indices and SRS parameters are cached for reuse
//! - Multiple verification operations can run concurrently
//!
//! For detailed API documentation, see the individual module documentation.

use kimchi::mina_curves::pasta::Vesta;

mod merkle_path;

pub use ledger::proofs::{
    caching::{srs_from_bytes, srs_to_bytes, verifier_index_from_bytes, verifier_index_to_bytes},
    verifiers::{BlockVerifier, TransactionVerifier},
};

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
pub type VerifierSRS = poly_commitment::ipa::SRS<Vesta>;

use redux::SubStore;
pub trait SnarkStore<GlobalState>:
    SubStore<GlobalState, SnarkState, SubAction = SnarkAction>
{
}
impl<S, T: SubStore<S, SnarkState, SubAction = SnarkAction>> SnarkStore<S> for T {}

/// Returns the Structured Reference String (SRS) for SNARK verification.
/// Delegates to `ledger::verifier::get_srs` with Fp field type.
///
/// TODO: Use directly from proof-systems (<https://github.com/o1-labs/mina-rust/issues/1749>)
pub fn get_srs() -> std::sync::Arc<poly_commitment::ipa::SRS<Vesta>> {
    ledger::verifier::get_srs::<mina_curves::pasta::Fp>()
}
