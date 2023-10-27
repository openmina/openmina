use crate::proofs::witness::{field, Boolean, FieldWitness, Witness};

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
