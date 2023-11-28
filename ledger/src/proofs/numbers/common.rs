use crate::{
    proofs::witness::{field, Boolean, FieldWitness, Witness},
    scan_state::currency::Magnitude,
};

/// Helper trait for zkapps in-snark checks (with trait `InSnarkCheck`)
pub trait ForZkappCheck<F: FieldWitness>: Magnitude {
    type CheckedType;

    fn to_checked(&self) -> Self::CheckedType {
        Self::checked_from_field(self.to_field::<F>())
    }
    fn checked_from_field(field: F) -> Self::CheckedType;
    fn lte(this: &Self::CheckedType, other: &Self::CheckedType, w: &mut Witness<F>) -> Boolean;
}

fn range_check_impl<F: FieldWitness, const NBITS: usize>(number: F, w: &mut Witness<F>) -> F {
    use crate::proofs::witness::scalar_challenge::to_field_checked_prime;

    let (_, _, actual_packed) = to_field_checked_prime::<F, NBITS>(number, w);
    actual_packed
}

pub fn range_check_flag<F: FieldWitness, const NBITS: usize>(
    number: F,
    w: &mut Witness<F>,
) -> Boolean {
    let actual = range_check_impl::<F, NBITS>(number, w);
    field::equal(actual, number, w)
}

pub fn range_check<F: FieldWitness, const NBITS: usize>(number: F, w: &mut Witness<F>) {
    range_check_impl::<F, NBITS>(number, w);
}
