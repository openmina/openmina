//! Currency types with ledger-specific implementations.
//!
//! Re-exports types from [`mina_tx_type::currency`] and adds implementations
//! for ledger-specific traits like [`ToFieldElements`] and [`Check`].
//!
//! Note: [`ToInputs`] implementations are provided by mina-tx-type.

use crate::proofs::{
    field::FieldWitness, to_field_elements::ToFieldElements, transaction::Check, witness::Witness,
};

// Re-export all types from mina-tx-type
pub use mina_tx_type::currency::{
    Amount, Balance, BlockTime, BlockTimeSpan, Epoch, Fee, FieldLike, Index, Length, Magnitude,
    MinMax, Nonce, Sgn, Signed, SignedRandExt, Slot, SlotRandExt, SlotSpan, ToChecked, TxnVersion,
    N,
};

// ============================================================================
// Ledger-specific trait implementations for N
// ============================================================================

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
// Macro for implementing ledger-specific traits on currency types
// ============================================================================

/// Macro to implement ToFieldElements and Check for currency types.
/// ToInputs implementations are provided by mina-tx-type.
macro_rules! impl_ledger_traits {
    (32: { $($name32:ident,)* }, 64: { $($name64:ident,)* },) => {
        $(impl_ledger_traits!({$name32, u32, as_u32},);)*
        $(impl_ledger_traits!({$name64, u64, as_u64},);)*
    };
    ($({ $name:ident, $inner:ty, $as_name:ident },)*) => ($(
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
