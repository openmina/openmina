use crate::{
    bigint::{BigInt, InvalidBigInt},
    list::List,
    number::{Int32, Int64, UInt32, UInt64},
    string::{ByteString, ZkAppUri},
};
use mina_hasher::ROInput;
use std::ops::Deref;

/// Difference with `ToInputs` in `ledger` is that it can fail here, due
/// to invalid bigints
pub trait FailableToInputs {
    fn to_input(&self, inputs: ROInput) -> Result<ROInput, InvalidBigInt>;
}

impl FailableToInputs for bool {
    fn to_input(&self, inputs: ROInput) -> Result<ROInput, InvalidBigInt> {
        Ok(inputs.append_bool(*self))
    }
}

impl FailableToInputs for BigInt {
    fn to_input(&self, inputs: ROInput) -> Result<ROInput, InvalidBigInt> {
        let field = self.to_field();
        Ok(inputs.append_field(field))
    }
}

impl FailableToInputs for Int32 {
    fn to_input(&self, inputs: ROInput) -> Result<ROInput, InvalidBigInt> {
        Ok(inputs.append_u32(self.as_u32()))
    }
}

impl FailableToInputs for Int64 {
    fn to_input(&self, inputs: ROInput) -> Result<ROInput, InvalidBigInt> {
        Ok(inputs.append_u64(self.as_u64()))
    }
}

impl FailableToInputs for UInt32 {
    fn to_input(&self, inputs: ROInput) -> Result<ROInput, InvalidBigInt> {
        Ok(inputs.append_u32(self.as_u32()))
    }
}

impl FailableToInputs for UInt64 {
    fn to_input(&self, inputs: ROInput) -> Result<ROInput, InvalidBigInt> {
        Ok(inputs.append_u64(self.as_u64()))
    }
}

impl FailableToInputs for ByteString {
    /// OCaml: <https://github.com/MinaProtocol/mina/blob/0063f0196d046d9d2fc8af0cea76ff30f51b49b7/src/lib/mina_base/signed_command_memo.ml#L132-L143>
    fn to_input(&self, mut inputs: ROInput) -> Result<ROInput, InvalidBigInt> {
        // Must match OCaml's reversed byte/bit processing
        for byte in self.as_ref().iter().rev() {
            for i in (0..8).rev() {
                inputs = inputs.append_bool(byte & (1 << i) != 0);
            }
        }
        Ok(inputs)
    }
}

impl FailableToInputs for ZkAppUri {
    /// OCaml: <https://github.com/MinaProtocol/mina/blob/8763656cb96a8ace65210b2d76d7f572222f28d3/src/lib/mina_base/zkapp_account.ml#L318-L325>
    fn to_input(&self, mut inputs: ROInput) -> Result<ROInput, InvalidBigInt> {
        // Must match OCaml's reversed byte/bit processing
        for byte in self.as_ref().iter().rev() {
            for i in (0..8).rev() {
                inputs = inputs.append_bool(byte & (1 << i) != 0);
            }
        }
        Ok(inputs)
    }
}

impl<T, D> FailableToInputs for List<D>
where
    D: Deref<Target = T>,
    T: FailableToInputs,
{
    fn to_input(&self, mut inputs: ROInput) -> Result<ROInput, InvalidBigInt> {
        for v in self.deref().iter().rev() {
            inputs = v.to_input(inputs)?;
        }
        Ok(inputs)
    }
}

#[cfg(test)]
mod tests {
    use super::ROInput;
    use o1_utils::FieldHelpers;

    trait ROInputTestExt {
        fn append_u2(self, value: u8) -> Self;
        fn append_u48(self, value: [u8; 6]) -> Self;
        fn append_bytes_be(self, value: &[u8]) -> Self;
        fn append_bytes_rev(self, value: &[u8]) -> Self;
    }

    impl ROInputTestExt for ROInput {
        fn append_u2(self, value: u8) -> Self {
            self.append_bool(value & 1 != 0).append_bool(value & 2 != 0)
        }

        fn append_u48(self, value: [u8; 6]) -> Self {
            self.append_bytes(&value)
        }

        fn append_bytes_be(mut self, value: &[u8]) -> Self {
            for byte in value.iter().rev() {
                for i in (0..8).rev() {
                    self = self.append_bool(byte & (1 << i) != 0);
                }
            }
            self
        }

        fn append_bytes_rev(mut self, value: &[u8]) -> Self {
            for byte in value.iter().rev() {
                self = self.append_bytes(&[*byte]);
            }
            self
        }
    }

    macro_rules! test_to_field {
        ($test:ident : $fun:ident ( $( $value:expr ),* $(,)? ) = $hex:expr ) => {
            #[test]
            fn $test() {
                let mut inputs = ROInput::new();
                let values = vec![ $( $value ),* ];
                for value in values.into_iter().rev() {
                    inputs = inputs.$fun(value);
                }
                let fields = inputs.to_fields();
                assert_eq!(fields.len(), 1);
                let hex = fields[0].to_hex();
                assert_eq!(&hex, $hex);
            }
        };
    }

    fn u48(n: u64) -> [u8; 6] {
        n.to_le_bytes()[..6].try_into().unwrap()
    }

    test_to_field!(to_field_bools_test: append_bool(true, false) = "0200000000000000000000000000000000000000000000000000000000000000");
    test_to_field!(to_field_u2_test: append_u2(0, 1, 3, 4) = "1c00000000000000000000000000000000000000000000000000000000000000");
    test_to_field!(to_field_u8_test: append_bytes_rev(&[u8::MIN, u8::MAX / 2, u8::MAX]) = "ff7f000000000000000000000000000000000000000000000000000000000000");
    test_to_field!(to_field_u32_test: append_u32(u32::MIN, u32::MAX/2, u32::MAX) = "ffffffffffffff7f000000000000000000000000000000000000000000000000");
    test_to_field!(to_field_u64_test: append_u64(u64::MIN, u64::MAX/2, u64::MAX) = "ffffffffffffffffffffffffffffff7f00000000000000000000000000000000");
    test_to_field!(to_field_u48_test: append_u48(u48(u64::MIN), u48(u64::MAX/2), u48(u64::MAX)) = "ffffffffffffffffffffffff0000000000000000000000000000000000000000");
    test_to_field!(to_field_bytes_test: append_bytes_be(&[0, 1, 255]) = "ff80000000000000000000000000000000000000000000000000000000000000");
}
