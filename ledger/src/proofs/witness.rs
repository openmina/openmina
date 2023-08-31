use ark_ff::{BigInteger256, Field};

struct FieldBitsIterator {
    index: usize,
    bigint: BigInteger256,
}

impl Iterator for FieldBitsIterator {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;

        let limb_index = index / 64;
        let bit_index = index % 64;

        let limb = self.bigint.0.get(limb_index)?;
        Some(limb & (1 << bit_index) != 0)
    }
}

fn field_to_bits<F, const NBITS: usize>(field: F) -> Vec<bool>
where
    F: Field + Into<BigInteger256>,
{
    let bigint: BigInteger256 = field.into();
    FieldBitsIterator { index: 0, bigint }.take(NBITS).collect()
}

// TODO: This function is incomplete (compare to OCaml), here it only push witness values
// https://github.com/MinaProtocol/mina/blob/357144819e7ce5f61109d23d33da627be28024c7/src/lib/pickles/scalar_challenge.ml#L12
pub fn to_field_checked_prime<F, const NBITS: usize>(scalar: F, witnesses: &mut Vec<F>) -> (F, F, F)
where
    F: Field + Into<BigInteger256> + From<u64>,
{
    let mut push = |f: F| -> F {
        witnesses.push(f);
        f
    };

    let neg_one = F::one().neg();

    let a_func = |n: u64| match n {
        0 => F::zero(),
        1 => F::zero(),
        2 => neg_one,
        3 => F::one(),
        _ => panic!("invalid argument"),
    };

    let b_func = |n: u64| match n {
        0 => neg_one,
        1 => F::one(),
        2 => F::zero(),
        3 => F::zero(),
        _ => panic!("invalid argument"),
    };

    let bits_msb = {
        let mut bits = field_to_bits::<_, NBITS>(scalar);
        bits.reverse();
        bits
    };

    let nybbles_per_row = 8;
    let bits_per_row = 2 * nybbles_per_row;
    assert_eq!((NBITS % bits_per_row), 0);
    let rows = NBITS / bits_per_row;

    // TODO: Use arrays when const feature allows it
    // https://github.com/rust-lang/rust/issues/76560
    let nybbles_by_row: Vec<Vec<u64>> = (0..rows)
        .map(|i| {
            (0..nybbles_per_row)
                .map(|j| {
                    let bit = (bits_per_row * i) + (2 * j);
                    let b0 = bits_msb[bit + 1] as u64;
                    let b1 = bits_msb[bit] as u64;
                    b0 + (2 * b1)
                })
                .collect()
        })
        .collect();

    let two: F = 2u64.into();
    let mut a = two;
    let mut b = two;
    let mut n = F::zero();

    for i in 0..rows {
        let n0 = n;
        let a0 = a;
        let b0 = b;

        let xs: Vec<F> = (0..nybbles_per_row)
            .map(|j| push(F::from(nybbles_by_row[i][j])))
            .collect();

        let n8: F = push(xs.iter().fold(n0, |accum, x| accum.double().double() + x));

        let a8: F = push(
            nybbles_by_row[i]
                .iter()
                .fold(a0, |accum, x| accum.double() + a_func(*x)),
        );

        let b8: F = push(
            nybbles_by_row[i]
                .iter()
                .fold(b0, |accum, x| accum.double() + b_func(*x)),
        );

        n = n8;
        a = a8;
        b = b8;
    }

    (a, b, n)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use mina_hasher::Fp;
    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use super::*;

    #[test]
    fn test_to_field_checked() {
        let mut witness = Vec::with_capacity(32);
        let f = Fp::from_str("1866").unwrap();

        let res = to_field_checked_prime::<_, 32>(f, &mut witness);

        assert_eq!(res, (131085.into(), 65636.into(), 1866.into()));
        assert_eq!(
            witness,
            &[
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                512.into(),
                257.into(),
                0.into(),
                0.into(),
                1.into(),
                3.into(),
                1.into(),
                0.into(),
                2.into(),
                2.into(),
                1866.into(),
                131085.into(),
                65636.into(),
            ]
        );
    }
}
