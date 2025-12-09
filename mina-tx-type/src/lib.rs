//! Transaction types for the Mina Protocol
//!
//! This crate provides standalone data structures representing Mina Protocol
//! transaction types. It can be used by external projects to work with Mina
//! transactions without depending on the full ledger crate.
//!
//! # Overview
//!
//! The Mina Protocol supports several transaction types:
//!
//! - **Coinbase**: Block rewards paid to the block producer
//! - **Fee Transfers**: Distribution of transaction fees to block producers
//! - **Signed Commands**: Payments and stake delegations (TODO)
//! - **zkApp Commands**: Complex multi-account operations using zero-knowledge
//!   proofs (TODO)
//!
//! # Documentation References
//!
//! For detailed information about zkApp transaction signing, see:
//! - TODO: Issue #1748 - zkApp transaction signing documentation
//! - <https://mina-rust.o1labs.org/researchers/zkapp-signing>

pub mod coinbase;
pub mod currency;

pub use coinbase::{Coinbase, CoinbaseFeeTransfer};
pub use currency::{Amount, Fee, Magnitude, MinMax, Sgn, Signed};

// Re-export mina-signer types for convenience
pub use mina_signer::CompressedPubKey;
