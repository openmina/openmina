//! Transaction types for the Mina Protocol
//!
//! This crate provides standalone data structures representing Mina Protocol
//! transaction types. It can be used by external projects to work with Mina
//! transactions without depending on the full ledger crate.
//!
//! # Overview
//!
//! The Mina Protocol supports two primary user-initiated transaction types:
//!
//! - **Signed Commands**: Traditional payments and stake delegations authorized
//!   by a cryptographic signature.
//! - **zkApp Commands**: Complex multi-account operations using zero-knowledge
//!   proofs for authorization and state updates.
//!
//! # zkApp Transaction Structure
//!
//! A zkApp command consists of:
//! - A **fee payer** account that pays the transaction fee
//! - A tree of **account updates** that modify account state
//! - A **memo** field for user-defined metadata
//!
//! Each account update can specify:
//! - State updates (app state, delegate, permissions, etc.)
//! - Balance changes
//! - Preconditions that must be satisfied
//! - Authorization method (signature, proof, or none)
//!
//! # Documentation References
//!
//! For detailed information about zkApp transaction signing, see:
//! - TODO: Issue #1748 - zkApp transaction signing documentation
//! - <https://mina-rust.o1labs.org/researchers/zkapp-signing>

pub mod currency;
pub mod zkapp;

pub use currency::*;
pub use zkapp::*;

// Re-export the field type for convenience
pub use mina_curves::pasta::Fp;
pub use mina_signer::{CompressedPubKey, Signature};
