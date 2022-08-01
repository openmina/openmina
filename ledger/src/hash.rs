use ark_ff::{Zero, One};
use mina_hasher::Fp;

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
            Item::Bool(v) => if *v { Fp::one() } else { Fp::zero() },
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

// let pack_to_fields (type t) (module F : Field_intf with type t = t)
//     ~(pow2 : int -> t) { field_elements; packeds } =
//   let shift_left acc n = F.( * ) acc (pow2 n) in
//   let open F in
//   let packed_bits =
//     let xs, acc, acc_n =
//       Array.fold packeds ~init:([], zero, 0)
//         ~f:(fun (xs, acc, acc_n) (x, n) ->
//           let n' = Int.(n + acc_n) in
//           if Int.(n' < size_in_bits) then (xs, shift_left acc n + x, n')
//           else (acc :: xs, zero, 0) )
//     in
//     let xs = if acc_n > 0 then acc :: xs else xs in
//     Array.of_list_rev xs
//   in
//   Array.append field_elements packed_bits

impl Inputs {
    pub fn new() -> Self {
        Self { fields: Vec::with_capacity(10), packeds: Vec::with_capacity(10) }
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
        let two = 2u64;
        let init_state = (Vec::with_capacity(16), Fp::zero(), 0);

        let (mut fields, fp, acc_n) = self.packeds.iter().fold(init_state, |mut acc, item| {
            let item_nbits = item.nbits();
            let n_prime = acc.2 + item_nbits;

            if n_prime < 255 {
                let mult_by: Fp = two.pow(item_nbits).into();
                let item2: Fp = item.as_field();
                let fp = (acc.1 * mult_by) + item2;

                println!("ADDING={:?}={:?}", item, item2);

                (acc.0, fp, n_prime)
            } else {
                acc.0.push(acc.1);
                (acc.0, Fp::zero(), 0)
            }
        });

        if acc_n > 0 {
            fields.push(fp);
        }

        self.fields.iter().cloned().chain(fields).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
