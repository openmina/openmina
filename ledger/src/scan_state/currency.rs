use std::cmp::Ordering::{Equal, Greater, Less};

use ark_ff::{BigInteger256, Field};
use rand::Rng;

use crate::proofs::field::FieldWitness;
use crate::proofs::to_field_elements::ToFieldElements;
use crate::proofs::transaction::Check;
use crate::proofs::witness::Witness;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sgn {
    Pos,
    Neg,
}

impl Sgn {
    pub fn negate(&self) -> Self {
        match self {
            Sgn::Pos => Sgn::Neg,
            Sgn::Neg => Sgn::Pos,
        }
    }

    pub fn to_field<F: FieldWitness>(&self) -> F {
        match self {
            Sgn::Pos => F::one(),
            Sgn::Neg => F::one().neg(),
        }
    }
}

pub trait Magnitude
where
    Self: Sized + PartialOrd + Copy,
{
    const NBITS: usize;

    fn abs_diff(&self, rhs: &Self) -> Self;
    fn wrapping_add(&self, rhs: &Self) -> Self;
    fn wrapping_mul(&self, rhs: &Self) -> Self;
    fn wrapping_sub(&self, rhs: &Self) -> Self;
    fn checked_add(&self, rhs: &Self) -> Option<Self>;
    fn checked_mul(&self, rhs: &Self) -> Option<Self>;
    fn checked_sub(&self, rhs: &Self) -> Option<Self>;
    fn checked_div(&self, rhs: &Self) -> Option<Self>;
    fn checked_rem(&self, rhs: &Self) -> Option<Self>;

    fn is_zero(&self) -> bool;
    fn zero() -> Self;

    fn add_flagged(&self, rhs: &Self) -> (Self, bool) {
        let z = self.wrapping_add(rhs);
        (z, z < *self)
    }

    fn sub_flagged(&self, rhs: &Self) -> (Self, bool) {
        (self.wrapping_sub(rhs), self < rhs)
    }

    fn to_field<F: FieldWitness>(&self) -> F;
    fn of_field<F: FieldWitness>(field: F) -> Self;
}

/// Trait used for default values with `ClosedInterval`
pub trait MinMax {
    fn min() -> Self;
    fn max() -> Self;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Signed<T: Magnitude> {
    pub magnitude: T,
    pub sgn: Sgn,
}

impl<T> Signed<T>
where
    T: Magnitude + PartialOrd + Ord + Clone,
{
    const NBITS: usize = T::NBITS;

    pub fn create(magnitude: T, sgn: Sgn) -> Self {
        Self {
            magnitude,
            sgn: if magnitude.is_zero() { Sgn::Pos } else { sgn },
        }
    }

    pub fn of_unsigned(magnitude: T) -> Self {
        Self::create(magnitude, Sgn::Pos)
    }

    pub fn negate(&self) -> Self {
        if self.magnitude.is_zero() {
            Self::zero()
        } else {
            Self {
                magnitude: self.magnitude,
                sgn: self.sgn.negate(),
            }
        }
    }

    pub fn is_pos(&self) -> bool {
        matches!(self.sgn, Sgn::Pos)
    }

    /// https://github.com/MinaProtocol/mina/blob/42d2005d04b59d14aacf4eef5ccee353e9a531b7/src/lib/transaction_logic/mina_transaction_logic.ml#L1615
    pub fn is_non_neg(&self) -> bool {
        matches!(self.sgn, Sgn::Pos)
    }

    pub fn is_neg(&self) -> bool {
        matches!(self.sgn, Sgn::Neg)
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/currency/currency.ml#L441
    pub fn zero() -> Self {
        Self {
            magnitude: T::zero(),
            sgn: Sgn::Pos,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.magnitude.is_zero() //&& matches!(self.sgn, Sgn::Pos)
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/currency/currency.ml#L460
    pub fn add(&self, rhs: &Self) -> Option<Self> {
        let (magnitude, sgn) = if self.sgn == rhs.sgn {
            let magnitude = self.magnitude.checked_add(&rhs.magnitude)?;
            let sgn = self.sgn;

            (magnitude, sgn)
        } else {
            let sgn = match self.magnitude.cmp(&rhs.magnitude) {
                Less => rhs.sgn,
                Greater => self.sgn,
                Equal => return Some(Self::zero()),
            };
            let magnitude = self.magnitude.abs_diff(&rhs.magnitude);

            (magnitude, sgn)
        };

        Some(Self::create(magnitude, sgn))
    }

    pub fn add_flagged(&self, rhs: Self) -> (Self, bool) {
        match (self.sgn, rhs.sgn) {
            (Sgn::Neg, sgn @ Sgn::Neg) | (Sgn::Pos, sgn @ Sgn::Pos) => {
                let (magnitude, overflow) = self.magnitude.add_flagged(&rhs.magnitude);
                (Self { magnitude, sgn }, overflow)
            }
            (Sgn::Pos, Sgn::Neg) | (Sgn::Neg, Sgn::Pos) => {
                let sgn = match self.magnitude.cmp(&rhs.magnitude) {
                    Less => rhs.sgn,
                    Greater => self.sgn,
                    Equal => Sgn::Pos,
                };
                let magnitude = self.magnitude.abs_diff(&rhs.magnitude);
                (Self { magnitude, sgn }, false)
            }
        }
    }
}

impl Signed<Amount> {
    pub fn to_fee(self) -> Signed<Fee> {
        let Self { magnitude, sgn } = self;

        Signed {
            magnitude: Fee(magnitude.0),
            sgn,
        }
    }
}

impl<T> Signed<T>
where
    T: Magnitude + PartialOrd + Ord + Clone,
    rand::distributions::Standard: rand::distributions::Distribution<T>,
{
    pub fn gen() -> Self {
        let mut rng = rand::thread_rng();

        let magnitude: T = rng.gen();
        let sgn = if rng.gen::<bool>() {
            Sgn::Pos
        } else {
            Sgn::Neg
        };

        Self::create(magnitude, sgn)
    }
}

impl Amount {
    /// The number of nanounits in a unit. User for unit transformations.
    const UNIT_TO_NANO: u64 = 1_000_000_000;

    pub fn of_fee(fee: &Fee) -> Self {
        Self(fee.0)
    }

    pub fn add_signed_flagged(&self, rhs: Signed<Self>) -> (Self, bool) {
        if let Sgn::Pos = rhs.sgn {
            self.add_flagged(&rhs.magnitude)
        } else {
            self.sub_flagged(&rhs.magnitude)
        }
    }

    pub fn to_nanomina_int(&self) -> Self {
        *self
    }

    pub fn to_mina_int(&self) -> Self {
        Self(self.0.checked_div(Self::UNIT_TO_NANO).unwrap())
    }

    pub fn of_mina_int_exn(int: u64) -> Self {
        Self::from_u64(int).scale(Self::UNIT_TO_NANO).unwrap()
    }

    pub fn of_nanomina_int_exn(int: u64) -> Self {
        Self::from_u64(int)
    }
}

impl Balance {
    pub fn sub_amount(&self, amount: Amount) -> Option<Self> {
        self.0.checked_sub(amount.0).map(Self)
    }

    pub fn add_amount(&self, amount: Amount) -> Option<Self> {
        self.0.checked_add(amount.0).map(Self)
    }

    pub fn add_signed_flagged(&self, rhs: Signed<Self>) -> (Self, bool) {
        if let Sgn::Pos = rhs.sgn {
            self.add_flagged(&rhs.magnitude)
        } else {
            self.sub_flagged(&rhs.magnitude)
        }
    }

    pub fn add_signed_amount_flagged(&self, rhs: Signed<Amount>) -> (Self, bool) {
        let rhs = Signed {
            magnitude: Balance::from_u64(rhs.magnitude.0),
            sgn: rhs.sgn,
        };

        if let Sgn::Pos = rhs.sgn {
            self.add_flagged(&rhs.magnitude)
        } else {
            self.sub_flagged(&rhs.magnitude)
        }
    }

    pub fn to_amount(self) -> Amount {
        Amount(self.0)
    }

    pub fn of_nanomina_int_exn(int: u64) -> Self {
        Self::from_u64(int)
    }
}

impl Index {
    // TODO: Not sure if OCaml wraps around here
    pub fn incr(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

impl Nonce {
    // TODO: Not sure if OCaml wraps around here
    pub fn incr(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    pub fn succ(&self) -> Self {
        self.incr()
    }

    pub fn add_signed_flagged(&self, rhs: Signed<Self>) -> (Self, bool) {
        if let Sgn::Pos = rhs.sgn {
            self.add_flagged(&rhs.magnitude)
        } else {
            self.sub_flagged(&rhs.magnitude)
        }
    }
}

impl BlockTime {
    pub fn add(&self, span: BlockTimeSpan) -> Self {
        Self(self.0.checked_add(span.0).unwrap())
    }

    pub fn sub(&self, span: BlockTimeSpan) -> Self {
        Self(self.0.checked_sub(span.0).unwrap())
    }
}

impl BlockTimeSpan {
    pub fn of_ms(ms: u64) -> Self {
        Self(ms)
    }
}

impl Slot {
    // TODO: Not sure if OCaml wraps around here
    pub fn incr(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    pub fn succ(&self) -> Self {
        self.incr()
    }

    pub fn gen_small() -> Self {
        let mut rng = rand::thread_rng();
        Self(rng.gen::<u32>() % 10_000)
    }
}

macro_rules! impl_number {
    (32: { $($name32:ident,)* }, 64: { $($name64:ident,)* },) => {
        $(impl_number!({$name32, u32, as_u32, from_u32, next_u32, append_u32},);)+
        $(impl_number!({$name64, u64, as_u64, from_u64, next_u64, append_u64},);)+
    };
    ($({ $name:ident, $inner:ty, $as_name:ident, $from_name:ident, $next_name:ident, $append_name:ident },)*) => ($(
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(pub(super) $inner);

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_fmt(format_args!("{}({:?})", stringify!($name), self.0))
            }
        }

        impl Magnitude for $name {
            const NBITS: usize = Self::NBITS;

            fn zero() -> Self {
                Self(0)
            }

            fn is_zero(&self) -> bool {
                self.0 == 0
            }

            fn wrapping_add(&self, rhs: &Self) -> Self {
                Self(self.0.wrapping_add(rhs.0))
            }

            fn wrapping_mul(&self, rhs: &Self) -> Self {
                Self(self.0.wrapping_mul(rhs.0))
            }

            fn wrapping_sub(&self, rhs: &Self) -> Self {
                Self(self.0.wrapping_sub(rhs.0))
            }

            fn checked_add(&self, rhs: &Self) -> Option<Self> {
                self.0.checked_add(rhs.0).map(Self)
            }

            fn checked_mul(&self, rhs: &Self) -> Option<Self> {
                self.0.checked_mul(rhs.0).map(Self)
            }

            fn checked_sub(&self, rhs: &Self) -> Option<Self> {
                self.0.checked_sub(rhs.0).map(Self)
            }

            fn checked_div(&self, rhs: &Self) -> Option<Self> {
                self.0.checked_div(rhs.0).map(Self)
            }

            fn checked_rem(&self, rhs: &Self) -> Option<Self> {
                self.0.checked_rem(rhs.0).map(Self)
            }

            fn abs_diff(&self, rhs: &Self) -> Self {
                Self(self.0.abs_diff(rhs.0))
            }

            fn to_field<F: FieldWitness>(&self) -> F {
                self.to_field()
            }

            fn of_field<F: FieldWitness>(field: F) -> Self {
                let amount: BigInteger256 = field.into();
                let amount: $inner = amount.0[0].try_into().unwrap();

                Self::$from_name(amount)
            }
        }

        impl MinMax for $name {
            fn min() -> Self { Self(0) }
            fn max() -> Self { Self(<$inner>::MAX) }
        }

        impl $name {
            pub const NBITS: usize = <$inner>::BITS as usize;

            pub fn $as_name(&self) -> $inner {
                self.0
            }

            pub const fn $from_name(value: $inner) -> Self {
                Self(value)
            }

            /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/currency/currency.ml#L379
            pub const fn scale(&self, n: $inner) -> Option<Self> {
                match self.0.checked_mul(n) {
                    Some(n) => Some(Self(n)),
                    None => None
                }
            }

            pub fn min() -> Self {
                <Self as MinMax>::min()
            }

            pub fn max() -> Self {
                <Self as MinMax>::max()
            }

            /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/currency/currency.ml#L124
            pub fn of_mina_string_exn(input: &str) -> Self {
                const PRECISION: usize = 9;

                let mut s = String::with_capacity(input.len() + 9);

                if !input.contains('.') {
                    let append = "000000000";
                    assert_eq!(append.len(), PRECISION);

                    s.push_str(append);
                } else {
                    let (whole, decimal) = {
                        let mut splitted = input.split('.');
                        let whole = splitted.next().unwrap();
                        let decimal = splitted.next().unwrap();
                        assert!(splitted.next().is_none(), "Currency.of_mina_string_exn: Invalid currency input");
                        (whole, decimal)
                    };

                    let decimal_length = decimal.len();

                    if decimal_length > PRECISION {
                        s.push_str(whole);
                        s.push_str(&decimal[0..PRECISION]);
                    } else {
                        s.push_str(whole);
                        s.push_str(decimal);
                        for _ in 0..PRECISION - decimal_length {
                            s.push('0');
                        }
                    }
                }

                let n = s.parse::<$inner>().unwrap();
                Self(n)
            }

            pub fn to_bits(&self) -> [bool; <$inner>::BITS as usize] {
                use crate::proofs::transaction::legacy_input::bits_iter;

                let mut iter = bits_iter::<$inner, { <$inner>::BITS as usize }>(self.0);
                std::array::from_fn(|_| iter.next().unwrap())
            }

            pub fn to_field<F: Field + From<BigInteger256>>(&self) -> F {
                let int = self.0 as u64;

                let mut bigint: [u64; 4] = [0; 4];
                bigint[0] = int;

                let bigint = ark_ff::BigInteger256(bigint);
                F::from(bigint)
            }
        }

        impl rand::distributions::Distribution<$name> for rand::distributions::Standard {
            fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> $name {
                $name(rng.$next_name())
            }
        }

        impl crate::ToInputs for $name {
            fn to_inputs(&self, inputs: &mut crate::Inputs) {
                inputs.$append_name(self.0);
            }
        }

        impl<F: FieldWitness> ToFieldElements<F> for $name {
            fn to_field_elements(&self, fields: &mut Vec<F>) {
                fields.push(self.to_field());
            }
        }

        impl<F: FieldWitness> Check<F> for $name {
            fn check(&self, witnesses: &mut Witness<F>) {
                use crate::proofs::transaction::scalar_challenge::to_field_checked_prime;

                const NBITS: usize = <$inner>::BITS as usize;

                let number: $inner = self.$as_name();
                assert_eq!(NBITS, std::mem::size_of_val(&number) * 8);

                let number: F = number.into();
                to_field_checked_prime::<F, NBITS>(number, witnesses);
            }
        }

    )+)
}

impl_number!(
    32: { Length, Slot, Nonce, Index, SlotSpan, },
    64: { Amount, Balance, Fee, BlockTime, BlockTimeSpan, N, },
);
