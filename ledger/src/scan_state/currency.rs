use ark_ff::{BigInteger256, Field};
use mina_p2p_messages::v2::BlockTimeTimeStableV1;
use rand::Rng;

use crate::proofs::{
    field::FieldWitness, to_field_elements::ToFieldElements, transaction::Check, witness::Witness,
};

// Re-export Amount, Fee, Signed, Magnitude, MinMax, and Sgn from mina-tx-type
pub use mina_tx_type::currency::{Amount, Fee, Magnitude, MagnitudeFieldExt, MinMax, Sgn, Signed};

/// Extension trait to provide `to_field` for Amount and Fee in the ledger crate.
///
/// This is needed because we can't implement `MagnitudeFieldExt<F>` directly due to
/// orphan rules - both the trait and the types are defined in mina-tx-type.
pub trait AmountFeeFieldExt<F: FieldWitness> {
    fn to_field(&self) -> F;
    fn of_field(field: F) -> Self;
}

impl<F: FieldWitness> AmountFeeFieldExt<F> for Amount {
    fn to_field(&self) -> F {
        let int = self.as_u64();
        F::from(int)
    }

    fn of_field(field: F) -> Self {
        let amount: BigInteger256 = field.into();
        let amount: u64 = amount.0[0];
        Self::from_u64(amount)
    }
}

impl<F: FieldWitness> AmountFeeFieldExt<F> for Fee {
    fn to_field(&self) -> F {
        let int = self.as_u64();
        F::from(int)
    }

    fn of_field(field: F) -> Self {
        let amount: BigInteger256 = field.into();
        let amount: u64 = amount.0[0];
        Self::from_u64(amount)
    }
}

// Ledger-specific trait implementations for Amount and Fee
impl crate::ToInputs for Amount {
    fn to_inputs(&self, inputs: &mut poseidon::hash::Inputs) {
        inputs.append_u64(self.0);
    }
}

impl crate::ToInputs for Fee {
    fn to_inputs(&self, inputs: &mut poseidon::hash::Inputs) {
        inputs.append_u64(self.0);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for Amount {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        fields.push(<Amount as AmountFeeFieldExt<F>>::to_field(self));
    }
}

impl<F: FieldWitness> ToFieldElements<F> for Fee {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        fields.push(<Fee as AmountFeeFieldExt<F>>::to_field(self));
    }
}

impl<F: FieldWitness> Check<F> for Amount {
    fn check(&self, witnesses: &mut Witness<F>) {
        use crate::proofs::transaction::scalar_challenge::to_field_checked_prime;

        const NBITS: usize = u64::BITS as usize;

        let number: u64 = self.as_u64();
        assert_eq!(NBITS, std::mem::size_of_val(&number) * 8);

        let number: F = number.into();
        to_field_checked_prime::<F, NBITS>(number, witnesses);
    }
}

impl<F: FieldWitness> Check<F> for Fee {
    fn check(&self, witnesses: &mut Witness<F>) {
        use crate::proofs::transaction::scalar_challenge::to_field_checked_prime;

        const NBITS: usize = u64::BITS as usize;

        let number: u64 = self.as_u64();
        assert_eq!(NBITS, std::mem::size_of_val(&number) * 8);

        let number: F = number.into();
        to_field_checked_prime::<F, NBITS>(number, witnesses);
    }
}

// Extension trait for Sgn with ledger-specific field conversion
pub trait SgnExt {
    fn to_field<F: FieldWitness>(&self) -> F;
}

impl SgnExt for Sgn {
    fn to_field<F: FieldWitness>(&self) -> F {
        match self {
            Sgn::Pos => F::one(),
            Sgn::Neg => F::one().neg(),
        }
    }
}

/// Extension trait for Signed with ledger-specific random generation.
pub trait SignedExt<T: Magnitude> {
    fn gen() -> Self;
}

impl<T> SignedExt<T> for Signed<T>
where
    T: Magnitude,
    rand::distributions::Standard: rand::distributions::Distribution<T>,
{
    fn gen() -> Self {
        Self {
            magnitude: rand::random(),
            sgn: if rand::random::<bool>() {
                Sgn::Pos
            } else {
                Sgn::Neg
            },
        }
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

    /// Passed amount gets multiplied by 1 billion to convert to nanomina.
    pub fn from_mina(amount: u64) -> Option<Self> {
        amount.checked_mul(1_000_000_000).map(Self::from_u64)
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

    /// low <= self <= high
    pub fn between(&self, low: &Self, high: &Self) -> bool {
        low <= self && self <= high
    }
}

impl BlockTime {
    pub fn add(&self, span: BlockTimeSpan) -> Self {
        Self(self.0.checked_add(span.0).unwrap())
    }

    pub fn sub(&self, span: BlockTimeSpan) -> Self {
        Self(self.0.checked_sub(span.0).unwrap())
    }

    pub fn diff(&self, other: Self) -> BlockTimeSpan {
        BlockTimeSpan(self.0 - other.0)
    }

    pub fn to_span_since_epoch(&self) -> BlockTimeSpan {
        let Self(ms) = self;
        BlockTimeSpan(*ms)
    }

    pub fn of_span_since_epoch(span: BlockTimeSpan) -> Self {
        let BlockTimeSpan(ms) = span;
        Self(ms)
    }
}

impl From<BlockTimeTimeStableV1> for BlockTime {
    fn from(bt: BlockTimeTimeStableV1) -> Self {
        Self(bt.0 .0 .0)
    }
}

impl BlockTimeSpan {
    pub fn of_ms(ms: u64) -> Self {
        Self(ms)
    }
    pub fn to_ms(&self) -> u64 {
        let Self(ms) = self;
        *ms
    }
}

impl Slot {
    // TODO: Not sure if OCaml wraps around here
    pub fn incr(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    pub fn add(&self, other: SlotSpan) -> Self {
        let SlotSpan(other) = other;
        Self(self.0.checked_add(other).unwrap())
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
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize)]
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
        }

        impl<F: FieldWitness> MagnitudeFieldExt<F> for $name {
            fn to_field(&self) -> F {
                <$name>::to_field::<F>(self)
            }

            fn of_field(field: F) -> Self {
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

            /// <https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/currency/currency.ml#L379>
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

            /// <https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/currency/currency.ml#L124>
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

            pub fn to_bits(&self) -> [bool; <$inner>::BITS as usize] {
                use crate::proofs::transaction::legacy_input::bits_iter;

                let mut iter = bits_iter::<$inner, { <$inner>::BITS as usize }>(self.0);
                std::array::from_fn(|_| iter.next().unwrap())
            }

            pub fn to_field<F: Field + From<BigInteger256>>(&self) -> F {
                let int = self.0 as u64;
                F::from(int)
            }
        }

        impl rand::distributions::Distribution<$name> for rand::distributions::Standard {
            fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> $name {
                $name(rng.$next_name())
            }
        }

        impl crate::ToInputs for $name {
            fn to_inputs(&self, inputs: &mut poseidon::hash::Inputs) {
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
    32: { Length, Slot, Nonce, Index, SlotSpan, TxnVersion, Epoch, },
    64: { Balance, BlockTime, BlockTimeSpan, N, },
);
