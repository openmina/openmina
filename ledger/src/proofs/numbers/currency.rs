use crate::scan_state::currency::{self, Amount, Balance, Fee, Magnitude, MinMax, Sgn, Signed};
use std::{cmp::Ordering::Less, marker::PhantomData};

use crate::proofs::{
    to_field_elements::ToFieldElements,
    witness::{field, Boolean, Check, FieldWitness, Witness},
};

use super::common::range_check;

pub enum RangeCheckFlaggedKind {
    Add,
    Sub,
    AddOrSub,
}

fn modulus_as_field_64_bits<F: FieldWitness>() -> F {
    fn modulus_as_field_impl<F: FieldWitness>() -> F {
        let mut one = F::one();
        let two: F = 2u64.into();

        for _ in 0..64 {
            one *= two;
        }

        one
    }
    cache!(F, modulus_as_field_impl::<F>())
}

fn double_modulus_as_field_64_bits<F: FieldWitness>() -> F {
    cache!(F, modulus_as_field_64_bits::<F>().double())
}

#[derive(Clone)]
pub struct CheckedSigned<F, T>
where
    F: FieldWitness,
    T: CheckedCurrency<F>,
{
    pub magnitude: T,
    pub sgn: Sgn,
    _field: PhantomData<F>,
}

impl<F: std::fmt::Debug, T: std::fmt::Debug> std::fmt::Debug for CheckedSigned<F, T>
where
    F: FieldWitness,
    T: CheckedCurrency<F>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckedSigned")
            .field("magnitude", &self.magnitude)
            .field("sgn", &self.sgn)
            // .field("_field", &self._field)
            .finish()
    }
}

impl<F, T> CheckedSigned<F, T>
where
    F: FieldWitness + std::fmt::Debug,
    T: CheckedCurrency<F> + std::fmt::Debug,
{
    pub fn create(magnitude: T, sgn: Sgn) -> Self {
        Self {
            magnitude,
            sgn,
            _field: PhantomData,
        }
    }

    pub fn of_unsigned(magnitude: T) -> Self {
        Self {
            magnitude,
            sgn: Sgn::Pos,
            _field: PhantomData,
        }
    }

    pub fn zero() -> Self {
        Self {
            magnitude: T::zero(),
            sgn: Sgn::Pos,
            _field: PhantomData,
        }
    }

    pub fn negate(self) -> Self {
        Self {
            magnitude: self.magnitude,
            sgn: self.sgn.negate(),
            _field: PhantomData,
        }
    }

    pub fn value(&self) -> F {
        let sgn: F = self.sgn.to_field();
        let magnitude: F = self.magnitude.to_field();
        magnitude * sgn
    }

    fn unchecked(&self) -> currency::Signed<T::Inner> {
        currency::Signed {
            magnitude: self.magnitude.to_inner(),
            sgn: self.sgn,
        }
    }

    pub fn add_flagged(&self, y: &Self, w: &mut Witness<F>) -> (Self, Boolean)
    where
        T::Inner: Ord,
    {
        let x = self;

        let xv = x.value();
        let yv = y.value();

        let sgn = w.exists({
            let x = x.unchecked();
            let y = y.unchecked();
            match x.add(&y) {
                Some(r) => r.sgn,
                None => match (x.sgn, y.sgn) {
                    (Sgn::Neg, Sgn::Neg) => Sgn::Neg,
                    _ => Sgn::Pos,
                },
            }
        });

        let value = xv + yv;
        let magnitude = field::mul(sgn.to_field(), value, w);

        let (res_magnitude, overflow) =
            T::range_check_flagged(RangeCheckFlaggedKind::AddOrSub, magnitude, w);

        let _res_value = field::mul(sgn.to_field(), magnitude, w);

        let res = Self {
            magnitude: res_magnitude,
            sgn,
            _field: PhantomData,
        };
        (res, overflow)
    }

    pub fn add(&self, y: &Self, w: &mut Witness<F>) -> Self
    where
        T::Inner: Ord,
    {
        let x = self;

        let xv: F = x.value();
        let yv: F = y.value();

        let sgn = w.exists({
            let x = x.unchecked();
            let y = y.unchecked();
            x.add(&y).map(|r| r.sgn).unwrap_or(Sgn::Pos)
        });

        let res_value = w.exists(xv + yv);
        let magnitude = w.exists(sgn.to_field::<F>() * res_value);

        range_check::<F, CURRENCY_NBITS>(magnitude, w);

        Self::create(T::from_field(magnitude), sgn)
    }

    pub fn equal(&self, other: &Self, w: &mut Witness<F>) -> Boolean {
        // We decompose this way because of OCaml evaluation order
        let t2 = w.exists(other.value());
        let t1 = w.exists(self.value());
        field::equal(t1, t2, w)
    }
}

const CURRENCY_NBITS: usize = 64;

pub trait CheckedCurrency<F: FieldWitness>:
    Sized + ToFieldElements<F> + Check<F> + std::fmt::Debug
{
    type Inner: MinMax + Magnitude + std::fmt::Debug;

    fn to_field(&self) -> F;
    fn from_field(field: F) -> Self;

    fn zero() -> Self {
        Self::from_field(F::zero())
    }

    fn to_inner(&self) -> Self::Inner {
        Self::Inner::of_field(self.to_field())
    }

    fn from_inner(inner: Self::Inner) -> Self {
        Self::from_field(inner.to_field())
    }

    fn min() -> Self {
        Self::from_inner(Self::Inner::min())
    }
    fn max() -> Self {
        Self::from_inner(Self::Inner::max())
    }

    fn modulus_as_field() -> F {
        modulus_as_field_64_bits::<F>()
    }

    fn double_modulus_as_field() -> F {
        double_modulus_as_field_64_bits::<F>()
    }

    fn equal(&self, other: &Self, w: &mut Witness<F>) -> Boolean {
        field::equal(self.to_field(), other.to_field(), w)
    }

    fn sub_flagged(&self, y: &Self, w: &mut Witness<F>) -> (Self, Boolean) {
        let (x, y) = (self.to_field(), y.to_field());
        let z = w.exists(x - y);
        Self::range_check_flagged(RangeCheckFlaggedKind::Sub, z, w)
    }

    fn sub_or_zero(&self, y: &Self, w: &mut Witness<F>) -> Self {
        let (res, underflow) = self.sub_flagged(y, w);
        w.exists_no_check(match underflow {
            Boolean::True => Self::zero(),
            Boolean::False => res,
        })
    }

    fn range_check_flagged(
        kind: RangeCheckFlaggedKind,
        t: F,
        w: &mut Witness<F>,
    ) -> (Self, Boolean) {
        use RangeCheckFlaggedKind::{Add, AddOrSub, Sub};

        let adjustment_factor = w.exists(match &kind {
            Add => {
                if let Less = t.cmp(&Self::modulus_as_field()) {
                    F::zero()
                } else {
                    F::one().neg()
                }
            }
            Sub => {
                if let Less = t.cmp(&Self::modulus_as_field()) {
                    F::zero()
                } else {
                    F::one()
                }
            }
            AddOrSub => {
                if let Less = t.cmp(&Self::modulus_as_field()) {
                    F::zero()
                } else if let Less = t.cmp(&Self::double_modulus_as_field()) {
                    F::one().neg()
                } else {
                    F::one()
                }
            }
        });

        let out_of_range = match kind {
            Add => Boolean::of_field(adjustment_factor.neg()),
            Sub => Boolean::of_field(adjustment_factor),
            AddOrSub => Boolean::of_field(field::mul(adjustment_factor, adjustment_factor, w)),
        };
        let t_ajusted: F = t + (adjustment_factor * Self::modulus_as_field());
        w.exists(t_ajusted);
        range_check::<F, CURRENCY_NBITS>(t_ajusted, w);
        (Self::from_field(t_ajusted), out_of_range)
    }

    /// <
    /// less than
    fn lt(&self, y: &Self, w: &mut Witness<F>) -> Boolean {
        let diff: F = w.exists(self.to_field() - y.to_field());
        let (_res, lt) = Self::range_check_flagged(RangeCheckFlaggedKind::Sub, diff, w);
        lt
    }

    /// <=
    /// less than or equal
    fn lte(&self, y: &Self, w: &mut Witness<F>) -> Boolean {
        let y_lt_x = y.lt(self, w);
        y_lt_x.neg()
    }

    /// >=
    /// greater than or equal
    fn gte(&self, y: &Self, w: &mut Witness<F>) -> Boolean {
        y.lte(self, w)
    }

    /// >
    /// greater than
    fn gt(&self, y: &Self, w: &mut Witness<F>) -> Boolean {
        y.lt(self, w)
    }

    fn add_signed(&self, d: CheckedSigned<F, Self>, w: &mut Witness<F>) -> Self {
        let t = self.to_field();
        let d = d.value();
        let res = w.exists(t + d);
        range_check::<F, CURRENCY_NBITS>(res, w);
        Self::from_field(res)
    }

    /// Returns (F, is_overflow)
    fn const_add_flagged(&self, y: &Self, w: &mut Witness<F>) -> (Self, Boolean) {
        let x = self;
        let z: F = x.to_field() + y.to_field();
        Self::range_check_flagged(RangeCheckFlaggedKind::Add, z, w)
    }

    /// Returns (F, is_overflow)
    fn add_flagged(&self, y: &Self, w: &mut Witness<F>) -> (Self, Boolean) {
        let x = self;
        let z: F = w.exists(x.to_field() + y.to_field());
        Self::range_check_flagged(RangeCheckFlaggedKind::Add, z, w)
    }

    fn sub(&self, y: &Self, w: &mut Witness<F>) -> Self {
        let x = self.to_field();
        let y = y.to_field();

        let res = w.exists(x - y);
        range_check::<F, { CURRENCY_NBITS }>(res, w);

        Self::from_field(res)
    }
}

#[derive(Clone, Debug)]
pub struct CheckedAmount<F: FieldWitness>(F);
#[derive(Clone, Debug)]
pub struct CheckedFee<F: FieldWitness>(F);
#[derive(Clone, Debug)]
pub struct CheckedBalance<F: FieldWitness>(F);

impl<F: FieldWitness> CheckedBalance<F> {
    pub fn add_signed_amount(
        &self,
        d: CheckedSigned<F, CheckedAmount<F>>,
        w: &mut Witness<F>,
    ) -> Self {
        let d = CheckedSigned::<F, Self>::create(Self(d.magnitude.0), d.sgn);
        self.add_signed(d, w)
    }

    pub fn add_amount_flagged(&self, y: &CheckedAmount<F>, w: &mut Witness<F>) -> (Self, Boolean) {
        let y = Self(y.0);
        self.add_flagged(&y, w)
    }

    pub fn sub_amount_or_zero(&self, y: &CheckedAmount<F>, w: &mut Witness<F>) -> Self {
        let y = Self(y.0);
        self.sub_or_zero(&y, w)
    }

    pub fn sub_amount_flagged(&self, y: &CheckedAmount<F>, w: &mut Witness<F>) -> (Self, Boolean) {
        let y = Self(y.0);
        self.sub_flagged(&y, w)
    }
}

impl<F: FieldWitness> CheckedAmount<F> {
    pub fn of_fee(fee: &CheckedFee<F>) -> Self {
        Self(fee.0)
    }
}

impl<F: FieldWitness> CheckedSigned<F, CheckedAmount<F>> {
    pub fn to_fee(&self) -> CheckedSigned<F, CheckedFee<F>> {
        CheckedSigned {
            magnitude: CheckedFee(self.magnitude.0),
            sgn: self.sgn,
            _field: PhantomData,
        }
    }
}

macro_rules! impl_currency {
    ($({$name:tt, $unchecked:tt}),*) => ($(
        impl<F: FieldWitness> CheckedCurrency<F> for $name::<F> {
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
                self.0.to_field_elements(fields)
            }
        }

        impl<F: FieldWitness> Check<F> for $name::<F> {
            fn check(&self, w: &mut Witness<F>) {
                range_check::<F, { CURRENCY_NBITS }>(self.0, w);
            }
        }

        impl $unchecked {
            pub fn to_checked<F: FieldWitness>(&self) -> $name<F> {
                $name::from_inner(*self)
            }
        }

        impl Signed<$unchecked> {
            pub fn to_checked<F: FieldWitness>(&self) -> CheckedSigned<F, $name<F>> {
                CheckedSigned {
                    magnitude: self.magnitude.to_checked(),
                    sgn: self.sgn,
                    _field: PhantomData,
                }
            }
        }
    )*)
}

impl_currency!(
    {CheckedAmount, Amount},
    {CheckedFee, Fee},
    {CheckedBalance, Balance}
);
