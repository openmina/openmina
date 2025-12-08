//! Hash computation types and traits for the ledger.
//!
//! This module provides the [`ToInputs`] trait for converting types to hash
//! inputs, enabling Poseidon hash computation for transactions and other
//! protocol data structures.

use mina_curves::pasta::Fp;
use mina_signer::CompressedPubKey;

use crate::{proofs::witness::Witness, scan_state::currency};
use poseidon::hash::{hash_with_kimchi, Inputs, LazyParam};

/// Trait for types that can be converted to hash inputs.
///
/// This trait is the foundation for computing Poseidon hashes of protocol
/// data structures. Types implementing this trait can be hashed using
/// the Mina-compatible Kimchi hash function.
pub trait ToInputs {
    /// Appends the hash inputs for this value to the given input buffer.
    fn to_inputs(&self, inputs: &mut Inputs);

    /// Creates a new input buffer containing this value's hash inputs.
    fn to_inputs_owned(&self) -> Inputs {
        let mut inputs = Inputs::new();
        self.to_inputs(&mut inputs);
        inputs
    }

    /// Computes the Poseidon hash of this value with the given parameter.
    fn hash_with_param(&self, param: &LazyParam) -> Fp {
        let mut inputs = Inputs::new();
        self.to_inputs(&mut inputs);
        hash_with_kimchi(param, &inputs.to_fields())
    }

    /// Computes the Poseidon hash with circuit witness generation.
    fn checked_hash_with_param(&self, param: &LazyParam, w: &mut Witness<Fp>) -> Fp {
        use crate::proofs::transaction::transaction_snark::checked_hash;

        let inputs = self.to_inputs_owned();
        checked_hash(param, &inputs.to_fields(), w)
    }
}

impl ToInputs for Fp {
    fn to_inputs(&self, inputs: &mut Inputs) {
        inputs.append_field(*self);
    }
}

impl<const N: usize> ToInputs for [Fp; N] {
    fn to_inputs(&self, inputs: &mut Inputs) {
        for field in self {
            inputs.append(field);
        }
    }
}

impl ToInputs for CompressedPubKey {
    fn to_inputs(&self, inputs: &mut Inputs) {
        inputs.append_field(self.x);
        inputs.append_bool(self.is_odd);
    }
}

impl ToInputs for crate::TokenId {
    fn to_inputs(&self, inputs: &mut Inputs) {
        inputs.append_field(self.0);
    }
}

impl ToInputs for bool {
    fn to_inputs(&self, inputs: &mut Inputs) {
        inputs.append_bool(*self);
    }
}

impl<T> ToInputs for currency::Signed<T>
where
    T: currency::Magnitude,
    T: ToInputs,
{
    /// <https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/currency/currency.ml#L453>
    fn to_inputs(&self, inputs: &mut Inputs) {
        self.magnitude.to_inputs(inputs);
        let sgn = matches!(self.sgn, currency::Sgn::Pos);
        inputs.append_bool(sgn);
    }
}

/// Extension trait for appending [`ToInputs`] values to an input buffer.
pub trait AppendToInputs {
    /// Appends a value implementing [`ToInputs`] to this input buffer.
    fn append<T>(&mut self, value: &T)
    where
        T: ToInputs;
}

impl AppendToInputs for Inputs {
    fn append<T>(&mut self, value: &T)
    where
        T: ToInputs,
    {
        value.to_inputs(self);
    }
}

// ============================================================================
// ToInputs implementations for currency types
// ============================================================================

macro_rules! impl_to_inputs {
    (32: { $($name32:ident,)* }, 64: { $($name64:ident,)* },) => {
        $(
            impl ToInputs for currency::$name32 {
                fn to_inputs(&self, inputs: &mut Inputs) {
                    inputs.append_u32(self.as_u32());
                }
            }
        )*
        $(
            impl ToInputs for currency::$name64 {
                fn to_inputs(&self, inputs: &mut Inputs) {
                    inputs.append_u64(self.as_u64());
                }
            }
        )*
    };
}

impl_to_inputs!(
    32: { Length, Slot, Nonce, Index, SlotSpan, TxnVersion, Epoch, },
    64: { Amount, Balance, Fee, BlockTime, BlockTimeSpan, N, },
);

#[cfg(test)]
mod tests {
    use poseidon::hash::Inputs;

    #[test]
    fn test_inputs() {
        let mut inputs = Inputs::new();

        inputs.append_bool(true);
        inputs.append_u64(0); // initial_minimum_balance
        inputs.append_u32(0); // cliff_time
        inputs.append_u64(0); // cliff_amount
        inputs.append_u32(1); // vesting_period
        inputs.append_u64(0); // vesting_increment

        elog!("INPUTS={:?}", inputs);
        elog!("FIELDS={:?}", inputs.to_fields());
    }
}
