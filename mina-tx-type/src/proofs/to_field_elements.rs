//! Field element conversion trait for proof inputs.
//!
//! This module provides the [`ToFieldElements`] trait used to convert data
//! structures into field elements for use in zero-knowledge proof circuits.

use ark_ff::Field;

/// Trait for converting values to field elements.
///
/// This trait is used to serialize data structures into vectors of field
/// elements, which are the inputs to Mina's zero-knowledge proof circuits.
pub trait ToFieldElements<F: Field> {
    /// Appends field elements to the provided vector.
    fn to_field_elements(&self, fields: &mut Vec<F>);

    /// Returns a new vector containing the field elements.
    fn to_field_elements_owned(&self) -> Vec<F> {
        let mut fields = Vec::with_capacity(1024);
        self.to_field_elements(&mut fields);
        fields
    }
}
