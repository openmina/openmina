//! Currency types with ledger-specific implementations.
//!
//! Re-exports types from [`mina_tx_type::currency`] and adds implementations
//! for ledger-specific traits like [`ToFieldElements`], [`Check`], and [`ToInputs`].

use rand::Rng;

use crate::proofs::{
    field::FieldWitness, to_field_elements::ToFieldElements, transaction::Check, witness::Witness,
};

// Re-export all types from mina-tx-type
pub use mina_tx_type::currency::{
    Amount, Balance, BlockTime, BlockTimeSpan, Epoch, Fee, FieldLike, Index, Length, Magnitude,
    MinMax, Nonce, Sgn, Signed, Slot, SlotSpan, TxnVersion,
};

// ============================================================================
// Extension traits for ledger-specific functionality
// ============================================================================

/// Extension trait for random generation of Signed values.
pub trait SignedRandExt<T: Magnitude> {
    fn gen() -> Signed<T>;
}

impl<T> SignedRandExt<T> for Signed<T>
where
    T: Magnitude + PartialOrd + Ord + Clone,
    rand::distributions::Standard: rand::distributions::Distribution<T>,
{
    fn gen() -> Signed<T> {
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

/// Extension trait for Slot random generation.
pub trait SlotRandExt {
    fn gen_small() -> Slot;
}

impl SlotRandExt for Slot {
    fn gen_small() -> Slot {
        let mut rng = rand::thread_rng();
        Slot::from_u32(rng.gen::<u32>() % 10_000)
    }
}

// ============================================================================
// Ledger-specific type: N (generic number)
// ============================================================================

/// A generic 64-bit number type for proof computations.
#[derive(
    Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize, serde::Serialize,
)]
pub struct N(pub(super) u64);

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
    pub const NBITS: usize = 64;

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }

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

impl crate::ToInputs for N {
    fn to_inputs(&self, inputs: &mut poseidon::hash::Inputs) {
        inputs.append_u64(self.0);
    }
}

impl<F: FieldWitness> ToFieldElements<F> for N {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        fields.push(self.to_field());
    }
}

impl<F: FieldWitness> Check<F> for N {
    fn check(&self, witnesses: &mut Witness<F>) {
        use crate::proofs::transaction::scalar_challenge::to_field_checked_prime;
        const NBITS: usize = 64;
        let number: u64 = self.as_u64();
        let number: F = number.into();
        to_field_checked_prime::<F, NBITS>(number, witnesses);
    }
}

// ============================================================================
// Macro for implementing ledger traits on currency types
// ============================================================================

macro_rules! impl_ledger_traits {
    (32: { $($name32:ident,)* }, 64: { $($name64:ident,)* },) => {
        $(impl_ledger_traits!({$name32, u32, as_u32, append_u32},);)*
        $(impl_ledger_traits!({$name64, u64, as_u64, append_u64},);)*
    };
    ($({ $name:ident, $inner:ty, $as_name:ident, $append_name:ident },)*) => ($(
        impl crate::ToInputs for $name {
            fn to_inputs(&self, inputs: &mut poseidon::hash::Inputs) {
                inputs.$append_name(self.$as_name());
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
    )*)
}

impl_ledger_traits!(
    32: { Length, Slot, Nonce, Index, SlotSpan, TxnVersion, Epoch, },
    64: { Amount, Balance, Fee, BlockTime, BlockTimeSpan, },
);
