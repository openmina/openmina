use std::{array::IntoIter, str::FromStr};

use mina_hasher::Fp;

#[derive(Clone, Debug)]
struct ScalarChallenge {
    pub inner: [u64; 2],
}

struct ScalarChallengeBitsIterator {
    inner: IntoIter<bool, 128>,
}

impl Iterator for ScalarChallengeBitsIterator {
    type Item = (bool, bool);

    fn next(&mut self) -> Option<Self::Item> {
        let second = self.inner.next()?;
        let first = self.inner.next()?;
        Some((first, second))
    }
}

pub fn endo() -> Fp {
    // That's a constant but it seems to be computed somewhere.
    // TODO: Find where it's computed
    Fp::from_str("8503465768106391777493614032514048814691664078728891710322960303815233784505")
        .unwrap()
}

impl ScalarChallenge {
    fn iter_bits(&self) -> ScalarChallengeBitsIterator {
        let a: u128 = self.inner[0].reverse_bits() as u128;
        let b: u128 = self.inner[1].reverse_bits() as u128;
        let num: u128 = (a << 64) | b;

        let mut bits = [false; 128];
        for (index, bit) in bits.iter_mut().enumerate() {
            *bit = ((num >> index) & 1) != 0;
        }

        ScalarChallengeBitsIterator {
            inner: bits.into_iter(),
        }
    }

    /// Implemention of `to_field_constant`
    /// https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles/scalar_challenge.ml#L139
    pub fn to_field(&self, endo: &Fp) -> Fp {
        let mut a: Fp = 2.into();
        let mut b: Fp = 2.into();
        let one: Fp = 1.into();
        let neg_one: Fp = -one;

        for (first, second) in self.iter_bits() {
            let s = if first { one } else { neg_one };

            a += a;
            b += b;

            if second {
                a += s;
            } else {
                b += s;
            }
        }

        (a * endo) + b
    }
}

#[cfg(test)]
mod tests {
    use crate::FpExt;

    use super::*;

    #[test]
    fn test_challenges() {
        let scalar_challenge = ScalarChallenge {
            inner: [
                -6073566958339139610i64 as u64,
                -2081966045668129095i64 as u64,
            ],
        };

        // let rev_bits: String = scalar_challenge.iter_bits().rev().map(|bit| if bit { '1' } else { '0' }).collect();
        // let bits: String = scalar_challenge.iter_bits().map(|bit| if bit { '1' } else { '0' }).collect();

        // println!("BITS={:?}", bits);
        // println!("REV_BITS={:?}", rev_bits);

        // assert_eq!(bits, "01100111111010101010010100001011111000001101101001101101110101011001110101100011011000011010000110010111011110101101100011000111");
        // assert_eq!(bits, rev_bits.chars().rev().collect::<String>());

        let endo = endo();
        println!("ENDO={:?}", endo.to_decimal());

        let scalar_challenge = scalar_challenge.to_field(&endo);
        println!("RES={:?}", scalar_challenge.to_decimal());

        assert_eq!(
            scalar_challenge.to_decimal(),
            "11088960946452242729814251490831984807138805895197664788816609458265399565988"
        );

        // let a: [u8; 3] = [1,2,3];
        // let i = a.into_iter();

        // for bit in scalar_challenge.iter_bits() {
        //     println!("bit=")
        // }

        // to_field_constant2=-6073566958339139610,-2081966045668129095
        // BITS LENGTH=128 VALUES=01100111111010101010010100001011111000001101101001101101110101011001110101100011011000011010000110010111011110101101100011000111

        // let a: u64 = 7895667244538374865;
        // let b: u64 = -7599583809327744882i64 as u64;
    }
}
