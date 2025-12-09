//! Proof-related traits and types for the Mina Protocol.
//!
//! This module provides the core traits and types needed for zero-knowledge
//! proof generation and verification in Mina.
//!
//! # Future Migration
//!
//! The types and logic in this module are temporary placeholders. They will be
//! moved to dedicated crates when we reimplement:
//! - **Snarky** in Rust (circuit construction DSL) - see [issue #1762]
//! - **Pickles** in Rust (recursive proof composition system) - see [issue #1763]
//!
//! At that point, this module will re-export types from those crates instead of
//! defining them locally.
//!
//! [issue #1762]: https://github.com/o1-labs/mina-rust/issues/1762
//! [issue #1763]: https://github.com/o1-labs/mina-rust/issues/1763

pub mod field;
pub mod to_field_elements;
pub mod witness;

pub use field::{
    Boolean, CircuitVar, FieldWitness, FromFpFq, GroupAffine, IntoGeneric, Params, Shift,
    ShiftedValue, ShiftingValue, ToBoolean, BACKEND_TICK_ROUNDS_N, BACKEND_TOCK_ROUNDS_N,
};
pub use to_field_elements::{field_of_bits, field_to_bits, ToFieldElements};
pub use witness::{Check, Witness};
