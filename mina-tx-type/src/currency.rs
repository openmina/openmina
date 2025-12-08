//! Currency and numeric types for the Mina Protocol
//!
//! This module defines the fundamental numeric types used throughout Mina
//! transactions, including amounts, balances, fees, nonces, and slots.
//!
//! # Type Overview
//!
//! - [`Amount`]: Token amount in nanomina (1 MINA = 10^9 nanomina)
//! - [`Balance`]: Account balance in nanomina
//! - [`Fee`]: Transaction fee in nanomina
//! - [`Nonce`]: Account transaction sequence number
//! - [`Slot`]: Global slot number since genesis
//! - [`Length`]: Blockchain length (block height)
//!
//! # Signed Values
//!
//! The [`Signed`] type wraps any magnitude type to represent positive or
//! negative values, commonly used for balance changes in transactions.

use core::cmp::Ordering::{Equal, Greater, Less};

use ark_ff::{BigInteger256, Field};
use serde::{Deserialize, Serialize};

/// Trait bound for field types that support conversion to/from BigInteger256.
///
/// This is satisfied by `Fp` (Pallas base field) and any type implementing
/// the ledger's `FieldWitness` trait.
pub trait FieldLike: Field + From<BigInteger256> + Into<BigInteger256> {}

impl<F> FieldLike for F where F: Field + From<BigInteger256> + Into<BigInteger256> {}

/// Sign of a value (positive or negative).
///
/// # References
///
/// OCaml: `src/lib/currency/currency.ml` (Sgn)
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sgn {
    /// Positive value.
    #[default]
    Pos,
    /// Negative value.
    Neg,
}

impl Sgn {
    /// Returns `true` if positive.
    pub fn is_pos(&self) -> bool {
        matches!(self, Sgn::Pos)
    }

    /// Returns `true` if negative.
    pub fn is_neg(&self) -> bool {
        matches!(self, Sgn::Neg)
    }

    /// Returns the negated sign.
    pub fn negate(&self) -> Self {
        match self {
            Sgn::Pos => Sgn::Neg,
            Sgn::Neg => Sgn::Pos,
        }
    }

    /// Converts to a field element (+1 or -1).
    pub fn to_field<F: FieldLike>(&self) -> F {
        match self {
            Sgn::Pos => F::one(),
            Sgn::Neg => F::one().neg(),
        }
    }
}

/// Trait for numeric magnitude types.
///
/// This trait provides arithmetic operations for currency types,
/// including field conversion methods for zero-knowledge proof operations.
pub trait Magnitude: Sized + PartialOrd + Copy {
    /// Number of bits in this type.
    const NBITS: usize;

    /// Returns the absolute difference between two values.
    fn abs_diff(&self, rhs: &Self) -> Self;

    /// Wrapping addition.
    fn wrapping_add(&self, rhs: &Self) -> Self;

    /// Wrapping multiplication.
    fn wrapping_mul(&self, rhs: &Self) -> Self;

    /// Wrapping subtraction.
    fn wrapping_sub(&self, rhs: &Self) -> Self;

    /// Checked addition.
    fn checked_add(&self, rhs: &Self) -> Option<Self>;

    /// Checked multiplication.
    fn checked_mul(&self, rhs: &Self) -> Option<Self>;

    /// Checked subtraction.
    fn checked_sub(&self, rhs: &Self) -> Option<Self>;

    /// Checked division.
    fn checked_div(&self, rhs: &Self) -> Option<Self>;

    /// Checked remainder.
    fn checked_rem(&self, rhs: &Self) -> Option<Self>;

    /// Returns `true` if the value is zero.
    fn is_zero(&self) -> bool;

    /// Returns the zero value.
    fn zero() -> Self;

    /// Add with overflow flag.
    fn add_flagged(&self, rhs: &Self) -> (Self, bool) {
        let z = self.wrapping_add(rhs);
        (z, z < *self)
    }

    /// Subtract with underflow flag.
    fn sub_flagged(&self, rhs: &Self) -> (Self, bool) {
        (self.wrapping_sub(rhs), self < rhs)
    }

    /// Converts to a field element.
    fn to_field<F: FieldLike>(&self) -> F;

    /// Creates from a field element.
    fn of_field<F: FieldLike>(field: F) -> Self;
}

/// Trait for types with minimum and maximum values.
///
/// Used for defining valid ranges in preconditions.
pub trait MinMax {
    /// Returns the minimum value.
    fn min() -> Self;
    /// Returns the maximum value.
    fn max() -> Self;
}

/// Trait for converting currency types to their checked (circuit) representation.
///
/// This trait enables conversion from unchecked currency types (like [`Amount`],
/// [`Fee`], [`Balance`]) to their checked equivalents used in zero-knowledge
/// proof circuits.
///
/// The checked types are parameterized by a field type `F` that represents the
/// field used in the proof system (typically `Fp` for the Pallas curve).
///
/// # Type Parameters
///
/// - `F`: The field type used in the proof circuit (must implement `FieldLike`)
/// - `Checked`: The checked type returned by the conversion
///
/// # Example
///
/// ```ignore
/// use mina_tx_type::currency::{Amount, ToChecked};
///
/// let amount = Amount::of_mina(10).unwrap();
/// let checked: CheckedAmount<Fp> = amount.to_checked();
/// ```
pub trait ToChecked<F: FieldLike> {
    /// The checked type that this converts to.
    type Checked;

    /// Converts to the checked representation for use in proof circuits.
    fn to_checked(&self) -> Self::Checked;
}

/// A signed value with magnitude and sign.
///
/// Used to represent balance changes in transactions where the value
/// can be positive (receiving) or negative (sending).
///
/// # References
///
/// OCaml: `src/lib/currency/currency.ml` (Signed)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// Number of bits in the magnitude type.
    pub const NBITS: usize = T::NBITS;

    /// Creates a signed value, normalizing zero to positive.
    pub fn create(magnitude: T, sgn: Sgn) -> Self {
        Self {
            magnitude,
            sgn: if magnitude.is_zero() { Sgn::Pos } else { sgn },
        }
    }

    /// Creates a positive (unsigned) value.
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

    /// Returns `true` if positive.
    pub fn is_pos(&self) -> bool {
        matches!(self.sgn, Sgn::Pos)
    }

    /// Returns `true` if non-negative (positive or zero).
    pub fn is_non_neg(&self) -> bool {
        matches!(self.sgn, Sgn::Pos)
    }

    /// Returns `true` if negative.
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

    /// Adds two signed values, returning `None` on overflow.
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

    /// Adds two signed values with overflow flag.
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

impl<T: Magnitude> Default for Signed<T> {
    fn default() -> Self {
        Self {
            magnitude: T::zero(),
            sgn: Sgn::Pos,
        }
    }
}

// ============================================================================
// Macro for generating numeric types
// ============================================================================

/// Macro for generating numeric currency types.
macro_rules! impl_number {
    (32: { $($name32:ident,)* }, 64: { $($name64:ident,)* },) => {
        $(impl_number!({$name32, u32, as_u32, from_u32},);)*
        $(impl_number!({$name64, u64, as_u64, from_u64},);)*
    };
    ($({ $name:ident, $inner:ty, $as_name:ident, $from_name:ident },)*) => ($(
        #[doc = concat!("A ", stringify!($name), " value.")]
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
        pub struct $name(pub $inner);

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}({:?})", stringify!($name), self.0)
            }
        }

        impl Magnitude for $name {
            const NBITS: usize = <$inner>::BITS as usize;

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

            fn to_field<F: FieldLike>(&self) -> F {
                let int = self.0 as u64;
                F::from(int)
            }

            fn of_field<F: FieldLike>(field: F) -> Self {
                let bigint: BigInteger256 = field.into();
                let value: $inner = bigint.0[0].try_into().unwrap();
                Self(value)
            }
        }

        impl MinMax for $name {
            fn min() -> Self { Self(0) }
            fn max() -> Self { Self(<$inner>::MAX) }
        }

        impl $name {
            /// Number of bits in this type.
            pub const NBITS: usize = <$inner>::BITS as usize;

            /// Returns the inner value.
            #[inline]
            pub fn as_inner(&self) -> $inner {
                self.0
            }

            /// Creates from the inner type.
            #[inline]
            pub const fn from_inner(value: $inner) -> Self {
                Self(value)
            }

            #[doc = concat!("Returns the value as ", stringify!($inner), ".")]
            #[inline]
            pub fn $as_name(&self) -> $inner {
                self.0
            }

            #[doc = concat!("Creates from a ", stringify!($inner), " value.")]
            #[inline]
            pub const fn $from_name(value: $inner) -> Self {
                Self(value)
            }

            /// Scales by a factor, returning `None` on overflow.
            pub const fn scale(&self, n: $inner) -> Option<Self> {
                match self.0.checked_mul(n) {
                    Some(n) => Some(Self(n)),
                    None => None
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

            /// Parses from a MINA string format (e.g., "1.5" for 1.5 MINA).
            ///
            /// # Panics
            ///
            /// Panics if the input is not a valid currency string.
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

            /// Converts to bit array representation.
            pub fn to_bits(&self) -> [bool; <$inner>::BITS as usize] {
                let value = self.0;
                let mut bits = [false; <$inner>::BITS as usize];
                for i in 0..<$inner>::BITS as usize {
                    bits[i] = (value >> i) & 1 == 1;
                }
                bits
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    )*)
}

impl_number!(
    32: { Length, Slot, Nonce, Index, SlotSpan, TxnVersion, Epoch, },
    64: { Amount, Balance, Fee, BlockTime, BlockTimeSpan, },
);

// ============================================================================
// Type-specific implementations
// ============================================================================

impl Amount {
    /// The number of nanounits in a unit (1 MINA = 10^9 nanomina).
    pub const UNIT_TO_NANO: u64 = 1_000_000_000;

    /// Creates from a fee.
    pub fn of_fee(fee: &Fee) -> Self {
        Self(fee.0)
    }

    /// Add a signed amount with overflow flag.
    pub fn add_signed_flagged(&self, rhs: Signed<Self>) -> (Self, bool) {
        if let Sgn::Pos = rhs.sgn {
            self.add_flagged(&rhs.magnitude)
        } else {
            self.sub_flagged(&rhs.magnitude)
        }
    }

    /// Returns the amount in nanomina.
    pub fn to_nanomina(&self) -> u64 {
        self.0
    }

    /// Converts to MINA (integer part only).
    pub fn to_mina(&self) -> u64 {
        self.0 / Self::UNIT_TO_NANO
    }

    /// Creates from MINA amount.
    pub fn of_mina(mina: u64) -> Option<Self> {
        mina.checked_mul(Self::UNIT_TO_NANO).map(Self)
    }

    /// Creates from nanomina amount.
    pub fn of_nanomina(nanomina: u64) -> Self {
        Self(nanomina)
    }

    /// Returns self (for compatibility).
    pub fn to_nanomina_int(&self) -> Self {
        *self
    }

    /// Converts to MINA units (integer division).
    pub fn to_mina_int(&self) -> Self {
        Self(self.0.checked_div(Self::UNIT_TO_NANO).unwrap())
    }

    /// Creates from MINA int (panics on overflow).
    pub fn of_mina_int_exn(int: u64) -> Self {
        Self::from_u64(int).scale(Self::UNIT_TO_NANO).unwrap()
    }

    /// Creates from nanomina int.
    pub fn of_nanomina_int_exn(int: u64) -> Self {
        Self::from_u64(int)
    }
}

impl Balance {
    /// Subtracts an amount, returning `None` on underflow.
    pub fn sub_amount(&self, amount: Amount) -> Option<Self> {
        self.0.checked_sub(amount.0).map(Self)
    }

    /// Adds an amount, returning `None` on overflow.
    pub fn add_amount(&self, amount: Amount) -> Option<Self> {
        self.0.checked_add(amount.0).map(Self)
    }

    /// Adds a signed balance with overflow flag.
    pub fn add_signed_flagged(&self, rhs: Signed<Self>) -> (Self, bool) {
        if let Sgn::Pos = rhs.sgn {
            self.add_flagged(&rhs.magnitude)
        } else {
            self.sub_flagged(&rhs.magnitude)
        }
    }

    /// Adds a signed amount with overflow flag.
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

    /// Converts to amount.
    pub fn to_amount(self) -> Amount {
        Amount(self.0)
    }

    /// Creates from MINA amount.
    pub fn from_mina(mina: u64) -> Option<Self> {
        mina.checked_mul(Amount::UNIT_TO_NANO).map(Self)
    }

    /// Creates from nanomina amount.
    pub fn of_nanomina(nanomina: u64) -> Self {
        Self(nanomina)
    }

    /// Creates from nanomina int (for compatibility).
    pub fn of_nanomina_int_exn(int: u64) -> Self {
        Self::from_u64(int)
    }
}

impl Fee {
    /// Creates from nanomina amount.
    pub const fn of_nanomina(nanomina: u64) -> Self {
        Self(nanomina)
    }

    /// Creates from nanomina int (for compatibility).
    pub const fn of_nanomina_int_exn(int: u64) -> Self {
        Self(int)
    }

    /// Returns the fee in nanomina.
    pub fn to_nanomina(&self) -> u64 {
        self.0
    }
}

impl Index {
    /// Increments the index.
    pub fn incr(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

impl Nonce {
    /// Increments the nonce.
    pub fn incr(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    /// Returns the successor nonce.
    pub fn succ(&self) -> Self {
        self.incr()
    }

    /// Add a signed nonce with overflow flag.
    pub fn add_signed_flagged(&self, rhs: Signed<Self>) -> (Self, bool) {
        if let Sgn::Pos = rhs.sgn {
            self.add_flagged(&rhs.magnitude)
        } else {
            self.sub_flagged(&rhs.magnitude)
        }
    }

    /// Returns `true` if `low <= self <= high`.
    pub fn between(&self, low: &Self, high: &Self) -> bool {
        low <= self && self <= high
    }
}

impl Slot {
    /// Increments the slot.
    pub fn incr(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    /// Returns the successor slot.
    pub fn succ(&self) -> Self {
        self.incr()
    }

    /// Adds a slot span.
    pub fn add(&self, span: SlotSpan) -> Self {
        Self(self.0.checked_add(span.0).unwrap())
    }
}

impl SlotSpan {
    /// Creates from a number of slots.
    pub const fn of_slots(slots: u32) -> Self {
        Self(slots)
    }

    /// Returns the number of slots.
    pub fn to_slots(&self) -> u32 {
        self.0
    }
}

impl Length {
    /// Increments the length.
    pub fn incr(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    /// Returns the successor length.
    pub fn succ(&self) -> Self {
        self.incr()
    }
}

impl BlockTime {
    /// Adds a time span.
    pub fn add(&self, span: BlockTimeSpan) -> Self {
        Self(self.0.checked_add(span.0).unwrap())
    }

    /// Subtracts a time span.
    pub fn sub(&self, span: BlockTimeSpan) -> Self {
        Self(self.0.checked_sub(span.0).unwrap())
    }

    /// Returns the difference as a time span.
    pub fn diff(&self, other: Self) -> BlockTimeSpan {
        BlockTimeSpan(self.0 - other.0)
    }

    /// Converts to milliseconds since epoch.
    pub fn to_ms(&self) -> u64 {
        self.0
    }

    /// Creates from milliseconds since epoch.
    pub fn of_ms(ms: u64) -> Self {
        Self(ms)
    }

    /// Converts to span since epoch.
    pub fn to_span_since_epoch(&self) -> BlockTimeSpan {
        BlockTimeSpan(self.0)
    }

    /// Creates from span since epoch.
    pub fn of_span_since_epoch(span: BlockTimeSpan) -> Self {
        Self(span.0)
    }
}

impl BlockTimeSpan {
    /// Creates from milliseconds.
    pub fn of_ms(ms: u64) -> Self {
        Self(ms)
    }

    /// Returns the span in milliseconds.
    pub fn to_ms(&self) -> u64 {
        self.0
    }
}

impl Signed<Amount> {
    /// Converts to a signed fee.
    pub fn to_fee(self) -> Signed<Fee> {
        Signed {
            magnitude: Fee(self.magnitude.0),
            sgn: self.sgn,
        }
    }
}

// ============================================================================
// Conversions with mina-p2p-messages types
// ============================================================================

use mina_p2p_messages::v2::{
    BlockTimeTimeStableV1, CurrencyAmountStableV1, CurrencyBalanceStableV1, CurrencyFeeStableV1,
    MinaNumbersGlobalSlotSinceGenesisMStableV1, MinaNumbersGlobalSlotSinceHardForkMStableV1,
    MinaNumbersGlobalSlotSpanStableV1, MinaStateBlockchainStateValueStableV2SignedAmount,
    SgnStableV1, SignedAmount, UnsignedExtendedUInt32StableV1,
    UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
};

impl From<CurrencyAmountStableV1> for Amount {
    fn from(value: CurrencyAmountStableV1) -> Self {
        Self(value.as_u64())
    }
}

impl From<CurrencyAmountStableV1> for Balance {
    fn from(value: CurrencyAmountStableV1) -> Self {
        Self(value.as_u64())
    }
}

impl From<Amount> for CurrencyAmountStableV1 {
    fn from(value: Amount) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&Balance> for CurrencyBalanceStableV1 {
    fn from(value: &Balance) -> Self {
        Self((*value).into())
    }
}

impl From<Balance> for CurrencyAmountStableV1 {
    fn from(value: Balance) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&SignedAmount> for Signed<Amount> {
    fn from(value: &SignedAmount) -> Self {
        Self {
            magnitude: Amount(value.magnitude.clone().as_u64()),
            sgn: value.sgn.clone().into(),
        }
    }
}

impl From<&Amount> for CurrencyAmountStableV1 {
    fn from(value: &Amount) -> Self {
        CurrencyAmountStableV1(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&Amount> for CurrencyFeeStableV1 {
    fn from(value: &Amount) -> Self {
        CurrencyFeeStableV1(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&Signed<Amount>> for SignedAmount {
    fn from(value: &Signed<Amount>) -> Self {
        Self {
            magnitude: (&value.magnitude).into(),
            sgn: (&value.sgn).into(),
        }
    }
}

impl From<&CurrencyFeeStableV1> for Fee {
    fn from(value: &CurrencyFeeStableV1) -> Self {
        Self(value.as_u64())
    }
}

impl From<&CurrencyAmountStableV1> for Fee {
    fn from(value: &CurrencyAmountStableV1) -> Self {
        Self(value.as_u64())
    }
}

impl From<&Nonce> for UnsignedExtendedUInt32StableV1 {
    fn from(value: &Nonce) -> Self {
        Self(value.as_u32().into())
    }
}

impl From<&UnsignedExtendedUInt32StableV1> for Nonce {
    fn from(value: &UnsignedExtendedUInt32StableV1) -> Self {
        Self::from_u32(value.as_u32())
    }
}

impl From<&UnsignedExtendedUInt32StableV1> for Slot {
    fn from(value: &UnsignedExtendedUInt32StableV1) -> Self {
        Self::from_u32(value.as_u32())
    }
}

impl From<&Slot> for UnsignedExtendedUInt32StableV1 {
    fn from(value: &Slot) -> Self {
        Self(value.as_u32().into())
    }
}

impl From<&UnsignedExtendedUInt32StableV1> for Length {
    fn from(value: &UnsignedExtendedUInt32StableV1) -> Self {
        Self::from_u32(value.0.as_u32())
    }
}

impl From<&Length> for UnsignedExtendedUInt32StableV1 {
    fn from(value: &Length) -> Self {
        Self(value.as_u32().into())
    }
}

impl From<SgnStableV1> for Sgn {
    fn from(value: SgnStableV1) -> Self {
        match value {
            SgnStableV1::Pos => Self::Pos,
            SgnStableV1::Neg => Self::Neg,
        }
    }
}

impl From<&SignedAmount> for Signed<Fee> {
    fn from(value: &SignedAmount) -> Self {
        Self {
            magnitude: (&value.magnitude).into(),
            sgn: value.sgn.clone().into(),
        }
    }
}

impl From<&Sgn> for SgnStableV1 {
    fn from(value: &Sgn) -> Self {
        match value {
            Sgn::Pos => Self::Pos,
            Sgn::Neg => Self::Neg,
        }
    }
}

impl From<&Fee> for CurrencyFeeStableV1 {
    fn from(value: &Fee) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&Fee> for CurrencyAmountStableV1 {
    fn from(value: &Fee) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&Signed<Fee>> for SignedAmount {
    fn from(value: &Signed<Fee>) -> Self {
        Self {
            magnitude: (&value.magnitude).into(),
            sgn: (&value.sgn).into(),
        }
    }
}

impl From<&MinaStateBlockchainStateValueStableV2SignedAmount> for Signed<Amount> {
    fn from(value: &MinaStateBlockchainStateValueStableV2SignedAmount) -> Self {
        let MinaStateBlockchainStateValueStableV2SignedAmount { magnitude, sgn } = value;

        Self {
            magnitude: (magnitude.clone()).into(),
            sgn: (sgn.clone()).into(),
        }
    }
}

impl From<&Signed<Amount>> for MinaStateBlockchainStateValueStableV2SignedAmount {
    fn from(value: &Signed<Amount>) -> Self {
        let Signed::<Amount> { magnitude, sgn } = value;

        Self {
            magnitude: (*magnitude).into(),
            sgn: sgn.into(),
        }
    }
}

impl From<&BlockTime> for BlockTimeTimeStableV1 {
    fn from(value: &BlockTime) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&MinaNumbersGlobalSlotSinceGenesisMStableV1> for Slot {
    fn from(value: &MinaNumbersGlobalSlotSinceGenesisMStableV1) -> Self {
        let MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(slot) = value;
        Self(slot.as_u32())
    }
}

impl From<&MinaNumbersGlobalSlotSinceHardForkMStableV1> for Slot {
    fn from(value: &MinaNumbersGlobalSlotSinceHardForkMStableV1) -> Self {
        let MinaNumbersGlobalSlotSinceHardForkMStableV1::SinceHardFork(slot) = value;
        Self(slot.as_u32())
    }
}

impl From<&MinaNumbersGlobalSlotSpanStableV1> for SlotSpan {
    fn from(value: &MinaNumbersGlobalSlotSpanStableV1) -> Self {
        let MinaNumbersGlobalSlotSpanStableV1::GlobalSlotSpan(slot) = value;
        Self(slot.as_u32())
    }
}

impl From<&Slot> for MinaNumbersGlobalSlotSinceGenesisMStableV1 {
    fn from(value: &Slot) -> Self {
        Self::SinceGenesis(value.as_u32().into())
    }
}

impl From<&Slot> for MinaNumbersGlobalSlotSinceHardForkMStableV1 {
    fn from(value: &Slot) -> Self {
        Self::SinceHardFork(value.as_u32().into())
    }
}

impl From<&SlotSpan> for MinaNumbersGlobalSlotSpanStableV1 {
    fn from(value: &SlotSpan) -> Self {
        Self::GlobalSlotSpan(value.as_u32().into())
    }
}

impl From<BlockTimeTimeStableV1> for BlockTime {
    fn from(bt: BlockTimeTimeStableV1) -> Self {
        BlockTime::from_u64(bt.0 .0 .0)
    }
}

// ============================================================================
// Generic number type N
// ============================================================================

/// A generic 64-bit number type for proof computations.
///
/// This type is used in various proof-related computations where a generic
/// 64-bit unsigned integer is needed. Unlike the specialized currency types,
/// `N` doesn't represent any specific unit.
#[derive(
    Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize,
)]
pub struct N(pub(crate) u64);

impl std::fmt::Debug for N {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("N({:?})", self.0))
    }
}

impl Magnitude for N {
    const NBITS: usize = 64;

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

    fn to_field<F: FieldLike>(&self) -> F {
        F::from(self.0)
    }

    fn of_field<F: FieldLike>(field: F) -> Self {
        use ark_ff::BigInteger256;
        let bigint: BigInteger256 = field.into();
        Self(bigint.0[0])
    }
}

impl MinMax for N {
    fn min() -> Self {
        Self(0)
    }
    fn max() -> Self {
        Self(u64::MAX)
    }
}

impl N {
    /// Number of bits in this type.
    pub const NBITS: usize = 64;

    /// Returns the inner value as u64.
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Creates from a u64 value.
    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }

    /// Multiplies by a scalar, returning None on overflow.
    pub const fn scale(&self, n: u64) -> Option<Self> {
        match self.0.checked_mul(n) {
            Some(n) => Some(Self(n)),
            None => None,
        }
    }
}

impl rand::distributions::Distribution<N> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> N {
        N(rng.next_u64())
    }
}

// ============================================================================
// Random generation implementations
// ============================================================================

/// Extension trait for random generation of [`Signed`] values.
///
/// This trait provides a convenient method for generating random signed values
/// for testing purposes.
pub trait SignedRandExt<T: Magnitude> {
    /// Generates a random signed value.
    fn gen() -> Signed<T>;
}

impl<T> SignedRandExt<T> for Signed<T>
where
    T: Magnitude + PartialOrd + Ord + Clone,
    rand::distributions::Standard: rand::distributions::Distribution<T>,
{
    fn gen() -> Signed<T> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let magnitude: T = rng.gen();
        let sgn = if rng.gen::<bool>() {
            Sgn::Pos
        } else {
            Sgn::Neg
        };

        Signed::create(magnitude, sgn)
    }
}

/// Extension trait for generating small random [`Slot`] values.
///
/// This trait provides a method for generating random slot values within
/// a small range, useful for testing scenarios.
pub trait SlotRandExt {
    /// Generates a random slot value in the range [0, 10000).
    fn gen_small() -> Slot;
}

impl SlotRandExt for Slot {
    fn gen_small() -> Slot {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Slot::from_u32(rng.gen::<u32>() % 10_000)
    }
}

macro_rules! impl_rand_distribution {
    (32: { $($name32:ident,)* }, 64: { $($name64:ident,)* },) => {
        $(
            impl rand::distributions::Distribution<$name32> for rand::distributions::Standard {
                fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> $name32 {
                    $name32::from_u32(rng.next_u32())
                }
            }
        )*
        $(
            impl rand::distributions::Distribution<$name64> for rand::distributions::Standard {
                fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> $name64 {
                    $name64::from_u64(rng.next_u64())
                }
            }
        )*
    };
}

impl_rand_distribution!(
    32: { Length, Slot, Nonce, Index, SlotSpan, TxnVersion, Epoch, },
    64: { Amount, Balance, Fee, BlockTime, BlockTimeSpan, },
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_mina_conversion() {
        let amount = Amount::of_mina(1).unwrap();
        assert_eq!(amount.0, 1_000_000_000);
        assert_eq!(amount.to_mina(), 1);
    }

    #[test]
    fn test_signed_add() {
        let a = Signed::of_unsigned(Amount(100));
        let b = Signed::create(Amount(30), Sgn::Neg);
        let result = a.add(&b).unwrap();
        assert_eq!(result.magnitude.0, 70);
        assert!(result.is_pos());
    }

    #[test]
    fn test_nonce_incr() {
        let n = Nonce(5);
        assert_eq!(n.incr().0, 6);
    }

    #[test]
    fn test_balance_arithmetic() {
        let balance = Balance(1000);
        let new_balance = balance.add_amount(Amount(500)).unwrap();
        assert_eq!(new_balance.0, 1500);

        let reduced = new_balance.sub_amount(Amount(200)).unwrap();
        assert_eq!(reduced.0, 1300);
    }

    #[test]
    fn test_to_bits() {
        let n = Nonce(5);
        let bits = n.to_bits();
        assert!(bits[0]); // 1
        assert!(!bits[1]); // 0
        assert!(bits[2]); // 1
        for bit in &bits[3..32] {
            assert!(!bit);
        }
    }
}
