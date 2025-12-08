//! Transaction types for the Mina Protocol
//!
//! This crate provides standalone data structures representing Mina Protocol
//! transaction types. It is designed to be `no_std` compatible for use in
//! constrained environments such as hardware wallets and WebAssembly.
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
//!
//! # no_std Support
//!
//! This crate is `no_std` by design to support embedded and constrained
//! environments such as hardware wallets and WebAssembly.

#![no_std]

extern crate alloc;

pub mod zkapp;

pub use zkapp::*;
