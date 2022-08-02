use std::{
    borrow::Cow,
    io::{Cursor, Write},
};

use ark_ff::{One, Zero};
use mina_hasher::Fp;
use o1_utils::FieldHelpers;
use oracle::{
    constants::PlonkSpongeConstantsKimchi,
    pasta,
    poseidon::{ArithmeticSponge, Sponge},
};

#[derive(Debug)]
enum Item {
    Bool(bool),
    U8(u8),
    U32(u32),
    U64(u64),
}

impl Item {
    fn nbits(&self) -> u32 {
        match self {
            Item::Bool(_) => 1,
            Item::U8(_) => 8,
            Item::U32(_) => 32,
            Item::U64(_) => 64,
        }
    }

    fn as_field(&self) -> Fp {
        match self {
            Item::Bool(v) => {
                if *v {
                    Fp::one()
                } else {
                    Fp::zero()
                }
            }
            Item::U8(v) => (*v).into(),
            Item::U32(v) => (*v).into(),
            Item::U64(v) => (*v).into(),
        }
    }
}

#[derive(Debug)]
struct Inputs {
    fields: Vec<Fp>,
    packeds: Vec<Item>,
}

impl Inputs {
    pub fn new() -> Self {
        Self {
            fields: Vec::with_capacity(16),
            packeds: Vec::with_capacity(16),
        }
    }

    pub fn append_bool(&mut self, value: bool) {
        self.packeds.push(Item::Bool(value));
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

    pub fn append_fields(&mut self, value: Fp) {
        self.fields.push(value);
    }

    fn to_fields(&self) -> Vec<Fp> {
        let mut nbits = 0;
        let mut current_field = Fp::zero();
        let mut fields = Vec::with_capacity(16);

        for (item, item_nbits) in self.packeds.iter().map(|i| (i.as_field(), i.nbits())) {
            nbits += item_nbits;

            if nbits < 255 {
                let multiply_by: Fp = 2u64.pow(item_nbits).into();
                current_field = (current_field * multiply_by) + item;
            } else {
                fields.push(current_field);
                current_field = item;
                nbits = item_nbits;
            }
        }

        if nbits > 0 {
            fields.push(current_field);
        }

        self.fields.iter().cloned().chain(fields).collect()
    }
}

fn param_to_field(param: Cow<str>) -> Fp {
    if param.len() > 20 {
        panic!("must be 20 byte maximum");
    }

    let param_bytes = param.as_ref().as_bytes();

    let mut fp = <[u8; 32]>::default();
    let mut cursor = Cursor::new(&mut fp[..]);

    cursor.write(param_bytes).expect("write failed");

    for _ in param_bytes.len()..20 {
        cursor.write("*".as_bytes()).expect("write failed");
    }

    Fp::from_bytes(&fp).expect("Fp::from_bytes failed")
}

fn hash_with_kimchi(param: Cow<str>, inputs: Inputs) -> Fp {
    let mut sponge =
        ArithmeticSponge::<Fp, PlonkSpongeConstantsKimchi>::new(pasta::fp_kimchi::static_params());

    sponge.absorb(&[param_to_field(param)]);
    sponge.squeeze();

    sponge.absorb(&inputs.to_fields());
    sponge.squeeze()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param() {
        for (s, hex) in [
            (
                "",
                "2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a000000000000000000000000",
            ),
            (
                "hello",
                "68656c6c6f2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a000000000000000000000000",
            ),
            (
                "aaaaaaaaaaaaaaaaaaaa",
                "6161616161616161616161616161616161616161000000000000000000000000",
            ),
        ] {
            let field = param_to_field(Cow::Borrowed(s));
            assert_eq!(field.to_hex(), hex);
        }
    }

    #[test]
    fn test_inputs() {
        let mut inputs = Inputs::new();

        inputs.append_bool(true);
        inputs.append_u64(0); // initial_minimum_balance
        inputs.append_u32(0); // cliff_time
        inputs.append_u64(0); // cliff_amount
        inputs.append_u32(1); // vesting_period
        inputs.append_u64(0); // vesting_increment

        println!("INPUTS={:?}", inputs);
        println!("FIELDS={:?}", inputs.to_fields());

        // // Self::timing
        // match self.timing {
        //     Timing::Untimed => {
        //         roi.append_bool(false);
        //         roi.append_u64(0); // initial_minimum_balance
        //         roi.append_u32(0); // cliff_time
        //         roi.append_u64(0); // cliff_amount
        //         roi.append_u32(1); // vesting_period
        //         roi.append_u64(0); // vesting_increment
        //     }
        //     Timing::Timed {
        //         initial_minimum_balance,
        //         cliff_time,
        //         cliff_amount,
        //         vesting_period,
        //         vesting_increment,
        //     } => {
        //         roi.append_bool(true);
        //         roi.append_u64(initial_minimum_balance);
        //         roi.append_u32(cliff_time);
        //         roi.append_u64(cliff_amount);
        //         roi.append_u32(vesting_period);
        //         roi.append_u64(vesting_increment);
        //     }
        // }
    }
}
