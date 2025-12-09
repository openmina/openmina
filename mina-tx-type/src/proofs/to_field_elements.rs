//! Field element conversion trait for proof inputs.
//!
//! This module provides the [`ToFieldElements`] trait used to convert data
//! structures into field elements for use in zero-knowledge proof circuits.

use std::borrow::Cow;

use ark_ff::{BigInteger256, Field};
use kimchi::proof::{PointEvaluations, ProofEvaluations};
use mina_curves::pasta::{Fp, Fq};
use mina_p2p_messages::string::ByteString;

use super::field::{Boolean, CircuitVar, FieldWitness, GroupAffine, IntoGeneric};

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

impl<F: Field> ToFieldElements<F> for () {
    fn to_field_elements(&self, _fields: &mut Vec<F>) {}
}

impl<F: FieldWitness, T: ToFieldElements<F>> ToFieldElements<F> for Vec<T> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.iter().for_each(|v| v.to_field_elements(fields));
    }
}

impl<F: FieldWitness, T: ToFieldElements<F>> ToFieldElements<F> for Box<[T]> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.iter().for_each(|v| v.to_field_elements(fields));
    }
}

impl<F: FieldWitness> ToFieldElements<F> for Fp {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        fields.push(self.into_gen());
    }
}

impl<F: FieldWitness, T: ToFieldElements<F> + Clone> ToFieldElements<F> for Cow<'_, T> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let this: &T = self.as_ref();
        this.to_field_elements(fields)
    }
}

struct FieldBitsIterator {
    index: usize,
    bigint: [u64; 4],
}

impl Iterator for FieldBitsIterator {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;

        let limb_index = index / 64;
        let bit_index = index % 64;

        let limb = self.bigint.get(limb_index)?;
        Some(limb & (1 << bit_index) != 0)
    }
}

fn bigint_to_bits<const NBITS: usize>(bigint: BigInteger256) -> [bool; NBITS] {
    let mut bits = FieldBitsIterator {
        index: 0,
        bigint: bigint.0,
    }
    .take(NBITS);
    std::array::from_fn(|_| bits.next().unwrap())
}

pub fn field_to_bits<F, const NBITS: usize>(field: F) -> [bool; NBITS]
where
    F: Field + Into<BigInteger256>,
{
    let bigint: BigInteger256 = field.into();
    bigint_to_bits(bigint)
}

// pack
pub fn field_of_bits<F: FieldWitness, const N: usize>(bs: &[bool; N]) -> F {
    bs.iter().rev().fold(F::zero(), |acc, b| {
        let acc = acc + acc;
        if *b {
            acc + F::one()
        } else {
            acc
        }
    })
}

impl<F: FieldWitness> ToFieldElements<F> for Fq {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        use std::any::TypeId;

        // TODO: Refactor when specialization is stable
        if TypeId::of::<F>() == TypeId::of::<Fq>() {
            fields.push(self.into_gen());
        } else {
            // `Fq` is larger than `Fp` so we have to split the field (low & high bits)
            // See:
            // <https://github.com/MinaProtocol/mina/blob/e85cf6969e42060f69d305fb63df9b8d7215d3d7/src/lib/pickles/impls.ml#L94C1-L105C45>

            let to_high_low = |fq: Fq| {
                let [low, high @ ..] = field_to_bits::<Fq, 255>(fq);
                [field_of_bits(&high), F::from(low)]
            };
            fields.extend(to_high_low(*self));
        }
    }
}

impl<F: FieldWitness, T: ToFieldElements<F>, const N: usize> ToFieldElements<F> for [T; N] {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.iter().for_each(|v| v.to_field_elements(fields));
    }
}

impl<F: FieldWitness> ToFieldElements<F> for ByteString {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let slice: &[u8] = self;
        slice.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for GroupAffine<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            x, y, infinity: _, ..
        } = self;
        y.to_field_elements(fields);
        x.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for &'_ [u8] {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        const BITS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
        fields.extend(
            self.iter()
                .flat_map(|byte| BITS.iter().map(|bit| F::from((*byte & bit != 0) as u64))),
        );
    }
}

impl<F: FieldWitness> ToFieldElements<F> for &'_ [bool] {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        fields.reserve(self.len());
        fields.extend(self.iter().copied().map(F::from))
    }
}

impl<F: FieldWitness> ToFieldElements<F> for bool {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        F::from(*self).to_field_elements(fields)
    }
}

impl<F: FieldWitness> ToFieldElements<F> for u64 {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        F::from(*self).to_field_elements(fields)
    }
}

impl<F: FieldWitness> ToFieldElements<F> for u32 {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        F::from(*self).to_field_elements(fields)
    }
}

impl<F: FieldWitness, T: ToFieldElements<F>> ToFieldElements<F> for PointEvaluations<T> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self { zeta, zeta_omega } = self;
        zeta.to_field_elements(fields);
        zeta_omega.to_field_elements(fields);
    }
}

impl<F: FieldWitness, T: ToFieldElements<F>> ToFieldElements<F> for ProofEvaluations<T> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            public: _,
            w,
            z,
            s,
            coefficients,
            generic_selector,
            poseidon_selector,
            complete_add_selector,
            mul_selector,
            emul_selector,
            endomul_scalar_selector,
            range_check0_selector,
            range_check1_selector,
            foreign_field_add_selector,
            foreign_field_mul_selector,
            xor_selector,
            rot_selector,
            lookup_aggregation,
            lookup_table,
            lookup_sorted,
            runtime_lookup_table,
            runtime_lookup_table_selector,
            xor_lookup_selector,
            lookup_gate_lookup_selector,
            range_check_lookup_selector,
            foreign_field_mul_lookup_selector,
        } = self;

        let mut push = |value: &T| {
            value.to_field_elements(fields);
        };

        w.iter().for_each(&mut push);
        coefficients.iter().for_each(&mut push);
        push(z);
        s.iter().for_each(&mut push);
        push(generic_selector);
        push(poseidon_selector);
        push(complete_add_selector);
        push(mul_selector);
        push(emul_selector);
        push(endomul_scalar_selector);
        range_check0_selector.as_ref().map(&mut push);
        range_check1_selector.as_ref().map(&mut push);
        foreign_field_add_selector.as_ref().map(&mut push);
        foreign_field_mul_selector.as_ref().map(&mut push);
        xor_selector.as_ref().map(&mut push);
        rot_selector.as_ref().map(&mut push);
        lookup_aggregation.as_ref().map(&mut push);
        lookup_table.as_ref().map(&mut push);
        lookup_sorted.iter().for_each(|v| {
            v.as_ref().map(&mut push);
        });
        runtime_lookup_table.as_ref().map(&mut push);
        runtime_lookup_table_selector.as_ref().map(&mut push);
        xor_lookup_selector.as_ref().map(&mut push);
        lookup_gate_lookup_selector.as_ref().map(&mut push);
        range_check_lookup_selector.as_ref().map(&mut push);
        foreign_field_mul_lookup_selector.as_ref().map(&mut push);
    }
}

impl<F: FieldWitness, A: ToFieldElements<F>, B: ToFieldElements<F>> ToFieldElements<F> for (A, B) {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let (a, b) = self;
        a.to_field_elements(fields);
        b.to_field_elements(fields);
    }
}

// Implementation for references
impl<F: FieldWitness, T: ToFieldElements<F>> ToFieldElements<F> for &T {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        (*self).to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for Boolean {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.to_field::<F>().to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for CircuitVar<Boolean> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        self.as_boolean().to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for mina_signer::CompressedPubKey {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self { x, is_odd } = self;
        x.to_field_elements(fields);
        is_odd.to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for mina_signer::Signature {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self { rx, s } = self;

        rx.to_field_elements(fields);
        let s_bits = field_to_bits::<_, 255>(*s);
        s_bits.to_field_elements(fields);
    }
}

impl ToFieldElements<Fp> for mina_signer::PubKey {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let GroupAffine::<Fp> { x, y, .. } = self.point();
        x.to_field_elements(fields);
        y.to_field_elements(fields);
    }
}

impl<F: FieldWitness> ToFieldElements<F>
    for mina_p2p_messages::v2::MinaBaseProtocolConstantsCheckedValueStableV1
{
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            k,
            slots_per_epoch,
            slots_per_sub_window,
            grace_period_slots,
            delta,
            genesis_state_timestamp,
        } = self;

        k.as_u32().to_field_elements(fields);
        slots_per_epoch.as_u32().to_field_elements(fields);
        slots_per_sub_window.as_u32().to_field_elements(fields);
        grace_period_slots.as_u32().to_field_elements(fields);
        delta.as_u32().to_field_elements(fields);
        genesis_state_timestamp.as_u64().to_field_elements(fields);
    }
}
