use crate::{
    bigint::{BigInt, InvalidBigInt},
    list::List,
    number::{Int32, Int64, UInt32, UInt64},
    string::{ByteString, ZkAppUri},
};
use poseidon::hash::Inputs;
use std::ops::Deref;

/// Difference with `ToInputs` in `ledger` is that it can fail here, due
/// to invalid bigints
pub trait FailableToInputs {
    fn to_input(&self, inputs: &mut Inputs) -> Result<(), InvalidBigInt>;
}

impl FailableToInputs for bool {
    fn to_input(&self, inputs: &mut Inputs) -> Result<(), InvalidBigInt> {
        inputs.append_bool(*self);
        Ok(())
    }
}

impl FailableToInputs for BigInt {
    fn to_input(&self, inputs: &mut Inputs) -> Result<(), InvalidBigInt> {
        let field = self.to_field()?;
        inputs.append_field(field);
        Ok(())
    }
}

impl FailableToInputs for Int32 {
    fn to_input(&self, inputs: &mut Inputs) -> Result<(), InvalidBigInt> {
        inputs.append_u32(self.as_u32());
        Ok(())
    }
}

impl FailableToInputs for Int64 {
    fn to_input(&self, inputs: &mut Inputs) -> Result<(), InvalidBigInt> {
        inputs.append_u64(self.as_u64());
        Ok(())
    }
}

impl FailableToInputs for UInt32 {
    fn to_input(&self, inputs: &mut Inputs) -> Result<(), InvalidBigInt> {
        inputs.append_u32(self.as_u32());
        Ok(())
    }
}

impl FailableToInputs for UInt64 {
    fn to_input(&self, inputs: &mut Inputs) -> Result<(), InvalidBigInt> {
        inputs.append_u64(self.as_u64());
        Ok(())
    }
}

impl FailableToInputs for ByteString {
    fn to_input(&self, inputs: &mut Inputs) -> Result<(), InvalidBigInt> {
        inputs.append_bytes(self.as_ref());
        Ok(())
    }
}

impl FailableToInputs for ZkAppUri {
    fn to_input(&self, inputs: &mut Inputs) -> Result<(), InvalidBigInt> {
        inputs.append_bytes(self.as_ref());
        Ok(())
    }
}

impl<T, D> FailableToInputs for List<D>
where
    D: Deref<Target = T>,
    T: FailableToInputs,
{
    fn to_input(&self, inputs: &mut Inputs) -> Result<(), InvalidBigInt> {
        for v in self.deref().iter() {
            v.to_input(inputs)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Inputs;
    use o1_utils::FieldHelpers;

    macro_rules! test_to_field {
        ($test:ident : $fun:ident ( $( $value:expr ),* $(,)? ) = $hex:expr ) => {
            #[test]
            fn $test() {
                let mut inputs = Inputs::new();
                $(
                    inputs.$fun($value);
                )*
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
    test_to_field!(to_field_u8_test: append_u8(u8::MIN, u8::MAX / 2, u8::MAX) = "ff7f000000000000000000000000000000000000000000000000000000000000");
    test_to_field!(to_field_u32_test: append_u32(u32::MIN, u32::MAX/2, u32::MAX) = "ffffffffffffff7f000000000000000000000000000000000000000000000000");
    test_to_field!(to_field_u64_test: append_u64(u64::MIN, u64::MAX/2, u64::MAX) = "ffffffffffffffffffffffffffffff7f00000000000000000000000000000000");
    test_to_field!(to_field_u48_test: append_u48(u48(u64::MIN), u48(u64::MAX/2), u48(u64::MAX)) = "ffffffffffffffffffffffff0000000000000000000000000000000000000000");
    test_to_field!(to_field_bytes_test: append_bytes(&[0, 1, 255]) = "ff80000000000000000000000000000000000000000000000000000000000000");
}
