use ark_ff::{BigInteger as _, BigInteger256, Field, FromBytes as _};
use mina_curves::pasta::Fp;

use crate::{Sponge, SpongeParamsForField};

enum Item {
    Bool(bool),
    U2(u8),
    U8(u8),
    U32(u32),
    U48([u8; 6]),
    U64(u64),
}

impl std::fmt::Debug for Item {
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

impl Item {
    fn nbits(&self) -> u32 {
        match self {
            Item::Bool(_) => 1,
            Item::U2(_) => 2,
            Item::U8(_) => 8,
            Item::U32(_) => 32,
            Item::U48(_) => 48,
            Item::U64(_) => 64,
        }
    }

    fn as_bigint(&self) -> BigInteger256 {
        match self {
            Item::Bool(v) => {
                if *v {
                    1.into()
                } else {
                    0.into()
                }
            }
            Item::U2(v) => (*v as u64).into(),
            Item::U8(v) => (*v as u64).into(),
            Item::U32(v) => (*v as u64).into(),
            Item::U48(v) => {
                let mut bytes = <[u8; 32]>::default();
                bytes[..6].copy_from_slice(&v[..]);
                BigInteger256::read(&bytes[..]).unwrap() // Never fail with only 6 bytes
            }
            Item::U64(v) => (*v).into(),
        }
    }
}

pub struct Inputs {
    fields: Vec<Fp>,
    packeds: Vec<Item>,
}

impl Default for Inputs {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Inputs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inputs")
            .field(&format!("fields[{:?}]", self.fields.len()), &self.fields)
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
        self.packeds.push(Item::Bool(value));
    }

    pub fn append_u2(&mut self, value: u8) {
        self.packeds.push(Item::U2(value));
    }

    pub fn append_u8(&mut self, value: u8) {
        self.packeds.push(Item::U8(value));
    }

    pub fn append_u32(&mut self, value: u32) {
        self.packeds.push(Item::U32(value));
    }

    pub fn append_u64(&mut self, value: u64) {
        self.packeds.push(Item::U64(value));
    }

    pub fn append_u48(&mut self, value: [u8; 6]) {
        self.packeds.push(Item::U48(value));
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

    // pub fn append<T>(&mut self, value: &T)
    // where
    //     T: ToInputs,
    // {
    //     value.to_inputs(self);
    // }

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
                self.fields.push(current.try_into().unwrap()); // Never fail
                current = item;
                nbits = item_nbits;
            }
        }

        if nbits > 0 {
            self.fields.push(current.try_into().unwrap()); // Never fail
        }

        self.fields
    }
}

fn param_to_field_impl(param: &str, default: &[u8; 32]) -> Fp {
    let param_bytes = param.as_bytes();
    let len = param_bytes.len();

    let mut fp = *default;
    fp[..len].copy_from_slice(param_bytes);

    Fp::read(&fp[..]).expect("fp read failed")
}

pub fn param_to_field(param: &str) -> Fp {
    const DEFAULT: &[u8; 32] = b"********************\0\0\0\0\0\0\0\0\0\0\0\0";

    if param.len() > 20 {
        panic!("must be 20 byte maximum");
    }

    param_to_field_impl(param, DEFAULT)
}

fn param_to_field_noinputs(param: &str) -> Fp {
    const DEFAULT: &[u8; 32] = &[0; 32];

    if param.len() > 32 {
        panic!("must be 32 byte maximum");
    }

    param_to_field_impl(param, DEFAULT)
}

pub fn hash_with_kimchi(param: &str, fields: &[Fp]) -> Fp {
    let mut sponge = Sponge::<Fp>::default();

    sponge.absorb(&[param_to_field(param)]);
    sponge.squeeze();

    sponge.absorb(fields);
    sponge.squeeze()
}

pub fn hash_fields<F: Field + SpongeParamsForField<F>>(fields: &[F]) -> F {
    let mut sponge = Sponge::<F>::default();

    sponge.absorb(fields);
    sponge.squeeze()
}

pub fn hash_noinputs(param: &str) -> Fp {
    let mut sponge = Sponge::<Fp>::default();
    // ArithmeticSponge::<Fp, PlonkSpongeConstantsKimchi>::new(pasta::fp_kimchi::static_params());

    sponge.absorb(&[param_to_field_noinputs(param)]);
    sponge.squeeze()
}
