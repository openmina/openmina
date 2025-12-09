//! Currency types for the Mina Protocol
//!
//! This module provides the fundamental currency types used throughout the
//! Mina Protocol for representing monetary values.
//!
//! # Unit System
//!
//! Mina uses a fixed-point representation where values are stored as integers
//! in the smallest unit (nanomina). One MINA equals 1,000,000,000 nanomina.
//!
//! # Types
//!
//! - [`Amount`]: Represents positive currency amounts (e.g., transaction
//!   amounts, coinbase rewards)
//! - [`Fee`]: Represents transaction fees paid to block producers
//! - [`Signed`]: A signed wrapper for magnitude types
//!
//! # Traits
//!
//! - [`Magnitude`]: Core trait for numeric operations on currency types
//! - [`MinMax`]: Trait for types with minimum and maximum values

use std::cmp::Ordering::{Equal, Greater, Less};

use ark_ff::BigInteger256;
use serde::{Deserialize, Serialize};

use crate::proofs::{
    field::FieldWitness,
    to_field_elements::ToFieldElements,
    witness::{Check, Witness},
};

/// Sign of a signed value.
///
/// Used with [`Signed`] to represent positive or negative values.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sgn {
    /// Positive sign
    Pos,
    /// Negative sign
    Neg,
}

impl Sgn {
    /// Returns `true` if the sign is positive.
    pub fn is_pos(&self) -> bool {
        matches!(self, Sgn::Pos)
    }

    /// Returns the negated sign.
    pub fn negate(&self) -> Self {
        match self {
            Sgn::Pos => Sgn::Neg,
            Sgn::Neg => Sgn::Pos,
        }
    }
}

/// Core trait for numeric magnitude operations.
///
/// This trait defines the fundamental operations needed for currency types
/// like [`Amount`] and [`Fee`]. It provides checked arithmetic operations
/// that return `None` on overflow/underflow, as well as wrapping operations.
///
/// # Required Methods
///
/// Implementors must provide:
/// - Arithmetic operations (add, sub, mul, div, rem) in both checked and
///   wrapping variants
/// - Zero value and zero check
/// - Absolute difference
/// - Bit width constant
pub trait Magnitude
where
    Self: Sized + PartialOrd + Copy,
{
    /// The number of bits in this type.
    const NBITS: usize;

    /// Returns the absolute difference between `self` and `rhs`.
    fn abs_diff(&self, rhs: &Self) -> Self;

    /// Wrapping addition.
    fn wrapping_add(&self, rhs: &Self) -> Self;

    /// Wrapping multiplication.
    fn wrapping_mul(&self, rhs: &Self) -> Self;

    /// Wrapping subtraction.
    fn wrapping_sub(&self, rhs: &Self) -> Self;

    /// Checked addition. Returns `None` if overflow occurred.
    fn checked_add(&self, rhs: &Self) -> Option<Self>;

    /// Checked multiplication. Returns `None` if overflow occurred.
    fn checked_mul(&self, rhs: &Self) -> Option<Self>;

    /// Checked subtraction. Returns `None` if underflow occurred.
    fn checked_sub(&self, rhs: &Self) -> Option<Self>;

    /// Checked division. Returns `None` if `rhs` is zero.
    fn checked_div(&self, rhs: &Self) -> Option<Self>;

    /// Checked remainder. Returns `None` if `rhs` is zero.
    fn checked_rem(&self, rhs: &Self) -> Option<Self>;

    /// Returns `true` if the value is zero.
    fn is_zero(&self) -> bool;

    /// Returns the zero value.
    fn zero() -> Self;

    /// Addition with overflow flag.
    ///
    /// Returns a tuple of the result and a boolean indicating whether
    /// overflow occurred.
    fn add_flagged(&self, rhs: &Self) -> (Self, bool) {
        let z = self.wrapping_add(rhs);
        (z, z < *self)
    }

    /// Subtraction with underflow flag.
    ///
    /// Returns a tuple of the result and a boolean indicating whether
    /// underflow occurred.
    fn sub_flagged(&self, rhs: &Self) -> (Self, bool) {
        (self.wrapping_sub(rhs), self < rhs)
    }

    /// Convert to a field element.
    fn to_field<F: FieldWitness>(&self) -> F;

    /// Create from a field element.
    fn of_field<F: FieldWitness>(field: F) -> Self;
}

/// Trait for types with minimum and maximum values.
///
/// Used for defining bounds on numeric types.
pub trait MinMax {
    /// Returns the minimum value.
    fn min() -> Self;
    /// Returns the maximum value.
    fn max() -> Self;
}

/// A signed magnitude value.
///
/// Combines a magnitude (absolute value) with a sign to represent
/// positive or negative values. Used for balance changes and fee excesses.
///
/// # Type Parameters
///
/// * `T` - The magnitude type, must implement [`Magnitude`]
///
/// # Examples
///
/// ```
/// use mina_tx_type::{Signed, Sgn, Fee};
///
/// // Create a positive fee
/// let positive = Signed::of_unsigned(Fee::from_u64(100));
/// assert!(positive.is_pos());
///
/// // Create a negative fee
/// let negative = positive.negate();
/// assert!(negative.is_neg());
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signed<T: Magnitude> {
    /// The absolute value.
    pub magnitude: T,
    /// The sign (positive or negative).
    pub sgn: Sgn,
}

impl<T> Signed<T>
where
    T: Magnitude + PartialOrd + Ord + Clone,
{
    /// The number of bits in the magnitude type.
    #[allow(dead_code)]
    const NBITS: usize = T::NBITS;

    /// Creates a new signed value with the given magnitude and sign.
    ///
    /// If the magnitude is zero, the sign is always set to positive.
    pub fn create(magnitude: T, sgn: Sgn) -> Self {
        Self {
            magnitude,
            sgn: if magnitude.is_zero() { Sgn::Pos } else { sgn },
        }
    }

    /// Creates a positive signed value from an unsigned magnitude.
    pub fn of_unsigned(magnitude: T) -> Self {
        Self::create(magnitude, Sgn::Pos)
    }

    /// Returns the negated value.
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

    /// Returns `true` if the value is positive.
    pub fn is_pos(&self) -> bool {
        matches!(self.sgn, Sgn::Pos)
    }

    /// Returns `true` if the value is non-negative (positive or zero).
    pub fn is_non_neg(&self) -> bool {
        matches!(self.sgn, Sgn::Pos)
    }

    /// Returns `true` if the value is negative.
    pub fn is_neg(&self) -> bool {
        matches!(self.sgn, Sgn::Neg)
    }

    /// Returns the zero value (positive zero).
    pub fn zero() -> Self {
        Self {
            magnitude: T::zero(),
            sgn: Sgn::Pos,
        }
    }

    /// Returns `true` if the value is zero.
    pub fn is_zero(&self) -> bool {
        self.magnitude.is_zero()
    }

    /// Checked addition of two signed values.
    ///
    /// Returns `None` if overflow would occur.
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

    /// Addition with overflow flag.
    ///
    /// Returns a tuple of the result and a boolean indicating whether
    /// overflow occurred.
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
    /// Converts a signed amount to a signed fee.
    pub fn to_fee(self) -> Signed<Fee> {
        let Self { magnitude, sgn } = self;
        Signed {
            magnitude: Fee(magnitude.0),
            sgn,
        }
    }
}

/// Macro to implement numeric currency types.
///
/// This macro generates a newtype wrapper around a primitive integer type
/// with implementations of [`Magnitude`], [`MinMax`], and various utility
/// methods.
///
/// # Generated Items
///
/// For each type, the macro generates:
/// - A newtype struct wrapping the inner type
/// - `Debug`, `Serialize`, `Deserialize` implementations
/// - [`Magnitude`] trait implementation
/// - [`MinMax`] trait implementation
/// - Accessor methods (`as_u64`, `from_u64`, etc.)
/// - Utility methods (`scale`, `of_mina_string_exn`)
macro_rules! impl_number {
    (32: { $($name32:ident,)* }, 64: { $($name64:ident,)* },) => {
        $(impl_number!({$name32, u32, as_u32, from_u32},);)*
        $(impl_number!({$name64, u64, as_u64, from_u64},);)*
    };
    ($({ $name:ident, $inner:ty, $as_name:ident, $from_name:ident },)*) => ($(
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
                 Deserialize, Serialize)]
        pub struct $name(pub $inner);

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
                let int = self.0 as u64;
                F::from(int)
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

        impl<F: FieldWitness> ToFieldElements<F> for $name {
            fn to_field_elements(&self, fields: &mut Vec<F>) {
                fields.push(self.to_field());
            }
        }

        impl<F: FieldWitness> Check<F> for $name {
            fn check(&self, w: &mut Witness<F>) {
                // Note: Full implementation requires to_field_checked_prime from
                // the transaction module. For now, we just add the field element
                // to the witness without range checking.
                // TODO: Implement proper range checking when snarky-rs is available.
                let _ = w.exists_no_check(self.to_field::<F>());
            }
        }

        impl $name {
            /// The number of bits in this type.
            pub const NBITS: usize = <$inner>::BITS as usize;

            /// Returns the inner value.
            pub fn $as_name(&self) -> $inner {
                self.0
            }

            /// Creates a new value from the inner type.
            pub const fn $from_name(value: $inner) -> Self {
                Self(value)
            }

            /// Multiplies by a scalar, returning `None` on overflow.
            pub const fn scale(&self, n: $inner) -> Option<Self> {
                match self.0.checked_mul(n) {
                    Some(n) => Some(Self(n)),
                    None => None,
                }
            }

            /// Returns the minimum value.
            pub fn min() -> Self {
                <Self as MinMax>::min()
            }

            /// Returns the maximum value.
            pub fn max() -> Self {
                <Self as MinMax>::max()
            }

            /// Parses a MINA amount from a string.
            ///
            /// The string can be in the format "123" or "123.456789".
            /// Values are converted to nanomina (9 decimal places).
            ///
            /// # Panics
            ///
            /// Panics if the string is not a valid currency format.
            pub fn of_mina_string_exn(input: &str) -> Self {
                const PRECISION: usize = 9;

                let mut s = String::with_capacity(input.len() + 9);

                if !input.contains('.') {
                    let append = "000000000";
                    assert_eq!(append.len(), PRECISION);

                    s.push_str(input);
                    s.push_str(append);
                } else {
                    let (whole, decimal) = {
                        let mut splitted = input.split('.');
                        let whole = splitted.next().unwrap();
                        let decimal = splitted.next().unwrap();
                        assert!(
                            splitted.next().is_none(),
                            "Currency.of_mina_string_exn: Invalid currency input"
                        );
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
        }
    )*)
}

// Generate the currency types using the macro
impl_number!(
    32: { },
    64: { Amount, Fee, },
);

// Additional Amount-specific implementations
impl Amount {
    /// The number of nanomina in one MINA.
    pub const NANOMINA_PER_MINA: u64 = 1_000_000_000;

    /// Converts a [`Fee`] to an [`Amount`].
    ///
    /// Since fees and amounts use the same underlying unit (nanomina),
    /// this is a direct conversion.
    pub const fn of_fee(fee: &Fee) -> Self {
        Self(fee.0)
    }

    /// Adds a signed amount, returning the result and overflow flag.
    pub fn add_signed_flagged(&self, rhs: Signed<Self>) -> (Self, bool) {
        if let Sgn::Pos = rhs.sgn {
            self.add_flagged(&rhs.magnitude)
        } else {
            self.sub_flagged(&rhs.magnitude)
        }
    }

    /// Returns the amount in nanomina (identity function).
    pub fn to_nanomina_int(&self) -> Self {
        *self
    }

    /// Converts nanomina to whole MINA (truncates).
    pub fn to_mina_int(&self) -> Self {
        Self(self.0.checked_div(Self::NANOMINA_PER_MINA).unwrap())
    }

    /// Creates an amount from whole MINA.
    ///
    /// # Panics
    ///
    /// Panics if the result would overflow.
    pub fn of_mina_int_exn(int: u64) -> Self {
        Self::from_u64(int).scale(Self::NANOMINA_PER_MINA).unwrap()
    }

    /// Creates an amount from nanomina.
    pub fn of_nanomina_int_exn(int: u64) -> Self {
        Self::from_u64(int)
    }
}

// Additional Fee-specific implementations
impl Fee {
    /// Creates a fee from nanomina.
    pub const fn of_nanomina_int_exn(int: u64) -> Self {
        Self::from_u64(int)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_from_u64() {
        let amount = Amount::from_u64(1_000_000_000);
        assert_eq!(amount.as_u64(), 1_000_000_000);
    }

    #[test]
    fn test_amount_checked_add() {
        let a = Amount::from_u64(100);
        let b = Amount::from_u64(200);
        assert_eq!(a.checked_add(&b), Some(Amount::from_u64(300)));
    }

    #[test]
    fn test_amount_checked_sub() {
        let a = Amount::from_u64(300);
        let b = Amount::from_u64(100);
        assert_eq!(a.checked_sub(&b), Some(Amount::from_u64(200)));

        let c = Amount::from_u64(50);
        assert_eq!(c.checked_sub(&a), None);
    }

    #[test]
    fn test_fee_operations() {
        let fee = Fee::from_u64(10_000_000);
        assert!(!fee.is_zero());
        assert_eq!(fee.as_u64(), 10_000_000);

        let zero_fee = Fee::zero();
        assert!(zero_fee.is_zero());
    }

    #[test]
    fn test_amount_of_fee() {
        let fee = Fee::from_u64(5_000_000);
        let amount = Amount::of_fee(&fee);
        assert_eq!(amount.as_u64(), 5_000_000);
    }

    #[test]
    fn test_magnitude_trait() {
        let a = Amount::from_u64(100);
        let b = Amount::from_u64(50);

        assert_eq!(a.abs_diff(&b), Amount::from_u64(50));
        assert_eq!(b.abs_diff(&a), Amount::from_u64(50));

        assert_eq!(a.wrapping_add(&b), Amount::from_u64(150));
        assert_eq!(a.wrapping_sub(&b), Amount::from_u64(50));
    }

    #[test]
    fn test_signed_operations() {
        let pos = Signed::of_unsigned(Amount::from_u64(100));
        assert!(pos.is_pos());
        assert!(!pos.is_neg());

        let neg = pos.negate();
        assert!(neg.is_neg());
        assert!(!neg.is_pos());

        // Adding positive and negative of same magnitude gives zero
        let sum = pos.add(&neg).unwrap();
        assert!(sum.is_zero());
    }

    #[test]
    fn test_signed_add() {
        let a = Signed::of_unsigned(Amount::from_u64(100));
        let b = Signed::of_unsigned(Amount::from_u64(50));

        // Positive + Positive
        let sum = a.add(&b).unwrap();
        assert_eq!(sum.magnitude.as_u64(), 150);
        assert!(sum.is_pos());

        // Positive + Negative (result positive)
        let neg_b = b.negate();
        let diff = a.add(&neg_b).unwrap();
        assert_eq!(diff.magnitude.as_u64(), 50);
        assert!(diff.is_pos());

        // Positive + Negative (result negative)
        let neg_a = a.negate();
        let diff2 = b.add(&neg_a).unwrap();
        assert_eq!(diff2.magnitude.as_u64(), 50);
        assert!(diff2.is_neg());
    }

    #[test]
    fn test_of_mina_string_exn() {
        assert_eq!(Amount::of_mina_string_exn("1").as_u64(), 1_000_000_000);
        assert_eq!(Amount::of_mina_string_exn("1.5").as_u64(), 1_500_000_000);
        assert_eq!(Amount::of_mina_string_exn("0.000000001").as_u64(), 1);
        assert_eq!(
            Amount::of_mina_string_exn("123.456789012").as_u64(),
            123_456_789_012
        );
    }

    #[test]
    fn test_signed_to_fee() {
        let signed_amount = Signed::create(Amount::from_u64(100), Sgn::Neg);
        let signed_fee = signed_amount.to_fee();

        assert_eq!(signed_fee.magnitude.as_u64(), 100);
        assert!(signed_fee.is_neg());
    }

    #[test]
    fn test_minmax() {
        assert_eq!(Amount::min().as_u64(), 0);
        assert_eq!(Amount::max().as_u64(), u64::MAX);

        assert_eq!(Fee::min().as_u64(), 0);
        assert_eq!(Fee::max().as_u64(), u64::MAX);
    }
}
