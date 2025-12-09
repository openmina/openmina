//! # Mina Ledger Crate
//!
//! The ledger crate is the most complex component in the Mina Rust node
//! codebase.
//! It implements the core ledger functionality, transaction processing, and
//! proof system integration.
//!
//! ## Architecture Overview
//!
//! The ledger crate is organized into several key components:
//!
//! ### Core Ledger
//!
//! - [`base`] - BaseLedger trait providing the fundamental ledger interface
//! - [`database`] - In-memory account storage implementation
//! - [`mask`] - Layered ledger views with Arc-based sharing for efficient
//!   copy-on-write semantics
//! - [`tree`] - Merkle tree operations for cryptographic integrity
//!
//! ### Transaction Processing
//!
//! - [`transaction_pool`] - Memory pool (mempool) with fee-based transaction
//!   ordering
//! - [`staged_ledger`] - Block validation and transaction application logic
//! - [`scan_state`] - SNARK work coordination and parallel scan tree management
//!
//! ### Proof System
//!
//! - [`proofs`] - Transaction, block, and zkApp proof generation and
//!   verification
//! - [`sparse_ledger`] - Minimal ledger representation optimized for proof
//!   generation
//! - [`zkapps`] - zkApp (zero-knowledge application) transaction processing
//!
//! ### Account Management
//!
//! - [`account`] - Account structures, balances, and permission management
//! - [`address`] - Account addressing and public key management
//!
//! ## Implementation Status
//!
//! The ledger components have proven reliable on devnet despite some technical
//! debt patterns. The implementation maintains the same battle-tested logic
//! that runs the production Mina network, ensuring compatibility and
//! correctness.
//!
//! ### Known Areas for Improvement
//!
//! #### Error Handling
//!
//! - Extensive use of `.unwrap()` and `.expect()` calls, particularly in:
//!   - `scan_state/transaction_logic.rs`
//!   - `staged_ledger/staged_ledger.rs`
//!   - `transaction_pool.rs`
//! - These calls are generally in code paths with well-understood preconditions
//!   but could benefit from explicit error propagation
//! - Inconsistent error handling patterns across modules
//!
//! #### Code Organization
//!
//! - Large files with multiple responsibilities that could benefit from decomposition
//! - Some monolithic structures that make testing and maintenance more challenging
//! - Opportunities for better separation of concerns in transaction processing logic
//!
//! ## Design Principles
//!
//! The ledger implementation follows several key design principles:
//!
//! - **Immutability**: Ledger states are immutable with copy-on-write semantics
//! - **Layering**: Mask-based layering allows efficient branching and merging
//! - **Cryptographic Integrity**: All ledger operations maintain Merkle tree
//!   consistency
//! - **Protocol Compliance**: Full compatibility with Mina protocol
//!   specifications
//! - **Performance**: Optimized for high-throughput transaction processing
//!
//! ## Usage Examples
//!
//! ```rust,no_run
//! use mina_tree::{Database, Mask, BaseLedger};
//!
//! // Create a new ledger database
//! let database = Database::create(35); // depth = 35
//!
//! // Create a mask for efficient layering
//! let mask = Mask::new_root(database);
//!
//! // Ledger operations can now be performed through the mask
//! ```
//!
//! For more detailed examples and API usage, see the individual module documentation.

#![allow(dead_code)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::result_unit_err)]
// #![forbid(clippy::needless_pass_by_ref_mut)]

// Unused, we don't want to print on stdout
// /// Print logs on stdout with the prefix `[ledger]`
// macro_rules! log {
//     () => (elog!("[ledger]"));
//     ($($arg:tt)*) => ({
//         println!("[ledger] {}", format_args!($($arg)*))
//     })
// }

/// Print logs on stderr with the prefix `[ledger]`
macro_rules! elog {
    () => (elog!("[ledger]"));
    ($($arg:tt)*) => ({
        let _ = &format_args!($($arg)*);
        // eprintln!("[ledger] {}", format_args!($($arg)*));
    })
}

// We need a feature to tests both nodejs and browser
// <https://github.com/rustwasm/wasm-bindgen/issues/2571>
#[cfg(not(feature = "in_nodejs"))]
#[cfg(target_family = "wasm")]
#[cfg(test)]
mod wasm {
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);
}

#[macro_use]
mod cache;

#[cfg(any(test, feature = "fuzzing"))]
pub mod generators;

pub mod account;
pub mod address;
pub mod base;
// mod blocks;
pub mod database;
pub mod dummy;
mod hash;
pub mod mask;
pub mod ondisk;
mod port_ocaml;
pub mod proofs;
pub mod scan_state;
pub mod sparse_ledger;
pub mod staged_ledger;
pub mod transaction_pool;
pub mod tree;
mod tree_version;
mod util;
pub mod verifier;
pub mod zkapps;

pub use account::*;
pub use address::*;
pub use base::*;
// pub use blocks::*;
pub use database::*;
pub use hash::*;
pub use mask::*;
pub use tree::*;
pub use tree_version::*;
pub use util::*;
