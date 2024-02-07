#![cfg(feature = "hashing")]

use std::ops::Deref;

use ark_ff::{BigInteger, BigInteger256, FromBytes};
use mina_hasher::Fp;
use o1_utils::FieldHelpers;

use crate::{
    b58::Base58CheckOfBinProt, bigint::BigInt, number::{Int32, Int64, UInt32, UInt64}, pseq::PaddedSeq, string::ByteString
};

pub trait ToInput {
    fn to_input(&self, inputs: &mut Inputs);
}

impl ToInput for bool {
    fn to_input(&self, inputs: &mut Inputs) {
        inputs.append_bool(*self);
    }
}

impl ToInput for BigInt {
    fn to_input(&self, inputs: &mut Inputs) {
        inputs.append_field(self.to_field());
    }
}

impl ToInput for Int32 {
    fn to_input(&self, inputs: &mut Inputs) {
        inputs.append_u32(self.as_u32())
    }
}

impl ToInput for Int64 {
    fn to_input(&self, inputs: &mut Inputs) {
        inputs.append_u64(self.as_u64())
    }
}

impl ToInput for UInt32 {
    fn to_input(&self, inputs: &mut Inputs) {
        inputs.append_u32(self.as_u32())
    }
}

impl ToInput for UInt64 {
    fn to_input(&self, inputs: &mut Inputs) {
        inputs.append_u64(self.as_u64())
    }
}

impl ToInput for ByteString {
    fn to_input(&self, inputs: &mut Inputs) {
        inputs.append_bytes(self.as_ref())
    }
}

impl<T, D> ToInput for Vec<D>
where
    D: Deref<Target = T>,
    T: ToInput,
{
    fn to_input(&self, inputs: &mut Inputs) {
        self.iter().for_each(|v| v.to_input(inputs));
    }
}

impl<T> ToInput for (T, T)
where
    T: ToInput,
{
    fn to_input(&self, inputs: &mut Inputs) {
        self.0.to_input(inputs);
        self.1.to_input(inputs);
    }
}

impl<T> ToInput for Option<T>
where
    T: ToInput + Default,
{
    fn to_input(&self, inputs: &mut Inputs) {
        match self.as_ref() {
            Some(v) => v.to_input(inputs),
            None => T::default().to_input(inputs),
        }
    }
}

impl<T, const N: usize> ToInput for PaddedSeq<T, N>
where
    T: ToInput,
{
    fn to_input(&self, inputs: &mut Inputs) {
        for v in &self.0 {
            v.to_input(inputs);
        }
    }
}

impl<T, U, const V: u8> ToInput for Base58CheckOfBinProt<T, U, V>
where
    T: ToInput,
{
    fn to_input(&self, inputs: &mut Inputs) {
        self.inner().to_input(inputs);
    }
}

#[allow(unused)]
enum Packed {
    Bool(bool),
    U2(u8),
    U8(u8),
    U32(u32),
    U48([u8; 6]),
    U64(u64),
}

impl std::fmt::Debug for Packed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(arg0) => f.write_fmt(format_args!("{}_bool", i32::from(*arg0))),
            Self::U2(arg0) => f.write_fmt(format_args!("{}_u2", arg0)),
            Self::U8(arg0) => f.write_fmt(format_args!("{}_u8", arg0)),
            Self::U32(arg0) => f.write_fmt(format_args!("{}_u32", arg0)),
            Self::U48(arg0) => f.write_fmt(format_args!("{:?}_u48", arg0)),
            Self::U64(arg0) => f.write_fmt(format_args!("{}_u64", arg0)),
        }
    }
}

impl Packed {
    fn nbits(&self) -> u32 {
        match self {
            Packed::Bool(_) => 1,
            Packed::U2(_) => 2,
            Packed::U8(_) => 8,
            Packed::U32(_) => 32,
            Packed::U48(_) => 48,
            Packed::U64(_) => 64,
        }
    }

    fn as_bigint(&self) -> BigInteger256 {
        match self {
            Packed::Bool(v) => {
                if *v {
                    1.into()
                } else {
                    0.into()
                }
            }
            Packed::U2(v) => (*v as u64).into(),
            Packed::U8(v) => (*v as u64).into(),
            Packed::U32(v) => (*v as u64).into(),
            Packed::U48(v) => {
                let mut bytes = <[u8; 32]>::default();
                bytes[..6].copy_from_slice(&v[..]);
                BigInteger256::read(&bytes[..]).unwrap()
            }
            Packed::U64(v) => (*v).into(),
        }
    }
}

pub struct Inputs {
    fields: Vec<Fp>,
    packeds: Vec<Packed>,
}

impl Default for Inputs {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Inputs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inputs")
            .field(
                &format!("fields[{:?}]", self.fields.len()),
                &self.fields.iter().map(|f| f.to_hex()).collect::<Vec<_>>(),
            )
            .field(&format!("packeds[{:?}]", self.packeds.len()), &self.packeds)
            .finish()
    }
}

impl Inputs {
    pub fn new() -> Self {
        Self {
            fields: Vec::with_capacity(256),
            packeds: Vec::with_capacity(256),
        }
    }

    pub fn append_bool(&mut self, value: bool) {
        self.packeds.push(Packed::Bool(value));
    }

    #[allow(unused)]
    pub fn append_u2(&mut self, value: u8) {
        self.packeds.push(Packed::U2(value));
    }

    #[allow(unused)]
    pub fn append_u8(&mut self, value: u8) {
        self.packeds.push(Packed::U8(value));
    }

    pub fn append_u32(&mut self, value: u32) {
        self.packeds.push(Packed::U32(value));
    }

    pub fn append_u64(&mut self, value: u64) {
        self.packeds.push(Packed::U64(value));
    }

    #[allow(unused)]
    pub fn append_u48(&mut self, value: [u8; 6]) {
        self.packeds.push(Packed::U48(value));
    }

    pub fn append_field(&mut self, value: Fp) {
        self.fields.push(value);
    }

    pub fn append_bytes(&mut self, value: &[u8]) {
        const BITS: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

        self.packeds.reserve(value.len() * 8);

        for byte in value {
            for bit in BITS {
                self.append_bool(byte & bit != 0);
            }
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_fields(mut self) -> Vec<Fp> {
        let mut nbits = 0;
        let mut current: BigInteger256 = 0.into();

        for (item, item_nbits) in self.packeds.iter().map(|i| (i.as_bigint(), i.nbits())) {
            nbits += item_nbits;

            if nbits < 255 {
                current.muln(item_nbits);

                // Addition, but we use 'bitwise or' because we know bits of
                // `current` are zero (we just shift-left them)
                current = BigInteger256([
                    current.0[0] | item.0[0],
                    current.0[1] | item.0[1],
                    current.0[2] | item.0[2],
                    current.0[3] | item.0[3],
                ]);
            } else {
                self.fields.push(current.into());
                current = item;
                nbits = item_nbits;
            }
        }

        if nbits > 0 {
            self.fields.push(current.into());
        }

        self.fields
    }
}

#[cfg(test)]
mod tests {
    use o1_utils::FieldHelpers;

    use super::Inputs;

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
