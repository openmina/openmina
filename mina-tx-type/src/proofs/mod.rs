//! Proof-related traits and types for the Mina Protocol.
//!
//! This module provides the core traits and types needed for zero-knowledge
//! proof generation and verification in Mina.

pub mod field;
pub mod to_field_elements;
pub mod witness;

pub use field::{
    Boolean, CircuitVar, FieldWitness, FromFpFq, GroupAffine, IntoGeneric, Params, Shift,
    ShiftedValue, ShiftingValue, ToBoolean, BACKEND_TICK_ROUNDS_N, BACKEND_TOCK_ROUNDS_N,
};
pub use to_field_elements::ToFieldElements;
pub use witness::{Check, Witness};
