use mina_hasher::Fp;

use crate::{
    proofs::{
        block::consensus::ConsensusConstantsChecked,
        to_field_elements::ToFieldElements,
        witness::{field, Boolean, Check, FieldWitness, Witness},
    },
    scan_state::currency::{
        BlockTime, BlockTimeSpan, Index, Length, Magnitude, MinMax, Nonce, Slot, SlotSpan,
    },
    ToInputs,
};

use super::common::{range_check, range_check_flag, ForZkappCheck};

pub trait CheckedNat<F: FieldWitness, const NBITS: usize>:
    Sized + ToFieldElements<F> + Check<F> + Clone
{
    type Inner: MinMax + Magnitude;

    fn to_field(&self) -> F;
    fn from_field(field: F) -> Self;

    fn zero() -> Self {
        Self::from_field(F::zero())
    }

    fn one() -> Self {
        Self::from_field(F::one())
    }

    fn to_inner(&self) -> Self::Inner {
        Self::Inner::of_field(self.to_field())
    }

    fn from_inner(inner: Self::Inner) -> Self {
        Self::from_field(inner.to_field())
    }

    /// >=
    /// greater than or equal
    fn gte(&self, other: &Self, w: &mut Witness<F>) -> Boolean {
        let (x, y) = (self.to_field(), other.to_field());

        let xy = w.exists(x - y);
        let yx = w.exists(xy.neg());

        let x_gte_y = range_check_flag::<F, NBITS>(xy, w);
        let y_gte_x = range_check_flag::<F, NBITS>(yx, w);

        Boolean::assert_any(&[x_gte_y, y_gte_x], w);
        x_gte_y
    }

    /// <=
    /// less than or equal
    fn lte(&self, other: &Self, w: &mut Witness<F>) -> Boolean {
        other.gte(self, w)
    }

    /// <
    /// less than
    fn less_than(&self, other: &Self, w: &mut Witness<F>) -> Boolean {
        let is_equal = field::equal(other.to_field(), self.to_field(), w);
        other.gte(self, w).and(&is_equal.neg(), w)
    }

    /// TODO: Remove this
    fn const_gte(&self, other: &Self, w: &mut Witness<F>) -> Boolean {
        let (x, y) = (self.to_field(), other.to_field());

        let xy = x - y;
        let yx = w.exists(xy.neg());

        let x_gte_y = range_check_flag::<F, NBITS>(xy, w);
        let y_gte_x = range_check_flag::<F, NBITS>(yx, w);

        Boolean::assert_any(&[x_gte_y, y_gte_x], w);
        x_gte_y
    }

    fn const_less_than(&self, other: &Self, w: &mut Witness<F>) -> Boolean {
        let is_equal = field::equal(other.to_field(), self.to_field(), w);
        other.const_gte(self, w).and(&is_equal.neg(), w)
    }

    /// >
    /// greater than
    fn greater_than(&self, other: &Self, w: &mut Witness<F>) -> Boolean {
        other.less_than(self, w)
    }

    fn const_greater_than(&self, other: &Self, w: &mut Witness<F>) -> Boolean {
        other.const_less_than(self, w)
    }

    fn equal(&self, other: &Self, w: &mut Witness<F>) -> Boolean {
        field::equal(self.to_field(), other.to_field(), w)
    }

    fn subtract_unpacking_or_zero(&self, y: &Self, w: &mut Witness<F>) -> (Boolean, Self) {
        let x = self.to_field();
        let y = y.to_field();

        let res = w.exists(x - y);
        let neg_res = w.exists(res.neg());

        let x_gte_y = range_check_flag::<F, NBITS>(res, w);
        let y_gte_x = range_check_flag::<F, NBITS>(neg_res, w);

        Boolean::assert_any(&[x_gte_y, y_gte_x], w);

        let is_equal = field::equal(x, y, w);
        let underflow = y_gte_x.and(&is_equal, w);

        let value = w.exists(match underflow {
            Boolean::True => F::zero(),
            Boolean::False => res,
        });

        (underflow, Self::from_field(value))
    }

    /// Returns (is_underflow, value)
    fn sub_or_zero(&self, y: &Self, w: &mut Witness<F>) -> (Boolean, Self) {
        self.subtract_unpacking_or_zero(y, w)
    }

    /// (div, remainder)
    fn div_mod(&self, y: &Self, w: &mut Witness<F>) -> (Self, Self) {
        let x = self.to_inner();
        let y = y.to_inner();

        let div = x.checked_div(&y).unwrap();
        let rem = x.checked_rem(&y).unwrap();

        w.exists((Self::from_inner(div), Self::from_inner(rem)))
    }

    fn add(&self, y: &Self, w: &mut Witness<F>) -> Self {
        let res = field::add(self.to_field(), y.to_field(), w);
        range_check::<F, NBITS>(res, w);
        Self::from_field(res)
    }

    fn const_add(&self, y: &Self, w: &mut Witness<F>) -> Self {
        let res = self.to_field() + y.to_field();
        range_check::<F, NBITS>(res, w);
        Self::from_field(res)
    }

    fn mul(&self, y: &Self, w: &mut Witness<F>) -> Self {
        let res = field::mul(self.to_field(), y.to_field(), w);
        range_check::<F, NBITS>(res, w);
        Self::from_field(res)
    }

    fn const_mul(&self, y: &Self, w: &mut Witness<F>) -> Self {
        let res = self.to_field() * y.to_field();
        range_check::<F, NBITS>(res, w);
        Self::from_field(res)
    }

    fn min(&self, b: &Self, w: &mut Witness<F>) -> Self {
        let a_lte_b = self.lte(b, w);
        w.exists_no_check(match a_lte_b {
            Boolean::True => self.clone(),
            Boolean::False => b.clone(),
        })
    }

    fn succ(&self) -> Self {
        let this = self.to_field();
        Self::from_field(this + F::one())
    }
}

impl<F: FieldWitness> CheckedSlot<F> {
    pub fn diff_or_zero(&self, t2: &Self, w: &mut Witness<F>) -> (Boolean, CheckedSlotSpan<F>) {
        let t1 = self;
        let (is_underflow, diff) = Self::sub_or_zero(t1, t2, w);
        (is_underflow, CheckedSlotSpan(diff.0))
    }

    pub fn diff(&self, t2: &Self, w: &mut Witness<F>) -> CheckedSlotSpan<F> {
        let diff = field::sub(self.to_field(), t2.to_field(), w);
        range_check::<F, 32>(diff, w);
        CheckedSlotSpan::from_field(diff)
    }

    pub fn to_length(&self) -> CheckedLength<F> {
        CheckedLength::from_field(self.to_field())
    }
}

impl CheckedSlot<Fp> {
    pub fn in_seed_update_range(
        &self,
        constants: &ConsensusConstantsChecked,
        w: &mut Witness<Fp>,
    ) -> Boolean {
        // constant
        let c = |n: u32| Length::from_u32(n).to_checked();
        let third_epoch = {
            let (q, _r) = constants.slots_per_epoch.div_mod(&c(3), w);
            q
        };
        let ck_times_2 = third_epoch.const_mul(&c(2), w);
        // let ck_times_2 = third_epoch.mul(&c(2), w);
        self.to_length().less_than(&ck_times_2, w)
    }
}

impl<F: FieldWitness> CheckedLength<F> {
    pub fn to_slot(&self) -> CheckedSlot<F> {
        CheckedSlot::from_field(self.to_field())
    }

    pub fn const_succ(&self) -> Self {
        Self(self.0 + F::one())
    }
}

#[derive(Clone, Debug)]
pub struct CheckedSlot<F: FieldWitness>(F);
#[derive(Clone, Debug)]
pub struct CheckedSlotSpan<F: FieldWitness>(F);
#[derive(Clone, Debug)]
pub struct CheckedLength<F: FieldWitness>(F);
#[derive(Clone, Debug)]
pub struct CheckedNonce<F: FieldWitness>(F);
#[derive(Clone, Debug)]
pub struct CheckedIndex<F: FieldWitness>(F);
#[derive(Clone, Debug)]
pub struct CheckedBlockTime<F: FieldWitness>(F);
#[derive(Clone, Debug)]
pub struct CheckedBlockTimeSpan<F: FieldWitness>(F);

macro_rules! impl_nat {
    ($({$name:tt, $unchecked:tt}),*) => ($(
        impl<F: FieldWitness> CheckedNat<F, 32> for $name::<F> {
            type Inner = $unchecked;
            fn to_field(&self) -> F {
                self.0
            }
            fn from_field(field: F) -> Self {
                Self(field)
            }
        }

        impl<F: FieldWitness> ToFieldElements<F> for $name::<F> {
            fn to_field_elements(&self, fields: &mut Vec<F>) {
                let Self(this) = self;
                this.to_field_elements(fields)
            }
        }

        impl<F: FieldWitness> Check<F> for $name::<F> {
            fn check(&self, w: &mut Witness<F>) {
                range_check::<F, { 32 }>(self.0, w);
            }
        }

        impl<F: FieldWitness> ToInputs for $name<F> {
            fn to_inputs(&self, inputs: &mut crate::Inputs) {
                self.to_inner().to_inputs(inputs)
            }
        }

        impl $unchecked {
            pub fn to_checked<F: FieldWitness>(&self) -> $name<F> {
                $name::from_inner(*self)
            }
        }

        impl<F: FieldWitness> ForZkappCheck<F> for $unchecked {
            type CheckedType = $name<F>;
            fn checked_from_field(field: F) -> Self::CheckedType {
                Self::CheckedType::from_field(field)
            }
            fn lte(this: &Self::CheckedType, other: &Self::CheckedType, w: &mut Witness<F>) -> Boolean {
                Self::CheckedType::lte(this, other, w)
            }
        }
    )*)
}

impl_nat!(
    {CheckedSlot, Slot},
    {CheckedSlotSpan, SlotSpan},
    {CheckedLength, Length},
    {CheckedNonce, Nonce},
    {CheckedIndex, Index},
    {CheckedBlockTime, BlockTime},
    {CheckedBlockTimeSpan, BlockTimeSpan}
);

/// A generic 64 bits checked number
#[derive(Clone, Debug)]
pub struct CheckedN<F: FieldWitness>(F);

impl<F: FieldWitness> CheckedNat<F, 64> for CheckedN<F> {
    type Inner = crate::scan_state::currency::N;
    fn to_field(&self) -> F {
        self.0
    }
    fn from_field(field: F) -> Self {
        Self(field)
    }
}

impl<F: FieldWitness> ToFieldElements<F> for CheckedN<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self(this) = self;
        this.to_field_elements(fields)
    }
}

impl<F: FieldWitness> Check<F> for CheckedN<F> {
    fn check(&self, w: &mut Witness<F>) {
        range_check::<F, 64>(self.0, w);
    }
}

/// A generic 32 bits checked number
#[derive(Clone, Debug)]
pub struct CheckedN32<F: FieldWitness>(F);

impl<F: FieldWitness> CheckedNat<F, 32> for CheckedN32<F> {
    type Inner = crate::scan_state::currency::N;
    fn to_field(&self) -> F {
        self.0
    }
    fn from_field(field: F) -> Self {
        Self(field)
    }
}

impl<F: FieldWitness> ToFieldElements<F> for CheckedN32<F> {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self(this) = self;
        this.to_field_elements(fields)
    }
}

impl<F: FieldWitness> Check<F> for CheckedN32<F> {
    fn check(&self, w: &mut Witness<F>) {
        range_check::<F, 32>(self.0, w);
    }
}

impl<F: FieldWitness> CheckedN32<F> {
    pub fn constant(n: usize) -> Self {
        let n: u32 = n.try_into().unwrap();
        Self::from_field(n.into())
    }
}
