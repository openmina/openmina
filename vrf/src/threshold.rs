use ark_ff::{BigInteger, BigInteger256, One, Zero};
use itertools::unfold;
use num::{BigInt, BigRational, FromPrimitive, Signed};

// #[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Threshold {
    pub total_currency: BigInt,
    pub delegated_stake: BigInt,
    pub threshold_rational: BigRational,
}

impl Threshold {
    const SIZE_IN_BITS: usize = 255;

    /// Creates a new Threshold struct
    pub fn new(delegated_stake: BigInt, total_currency: BigInt) -> Self {
        // 1. set up parameters to calculate threshold
        // Note: IMO all these parameters can be represented as constants. They do not change. The calculation is most likely in the
        //       code to adjust them in the future. We could create an utility that generates these params using f and log terms
        let f = BigRational::new(BigInt::from_u8(3).unwrap(), BigInt::from_u8(4).unwrap());

        let base = BigRational::one() - f;

        let abs_log_base = Self::log(100, base).abs();

        let (per_term_precission, terms_needed, _) = Self::bit_params(&abs_log_base);

        let terms_needed: i32 = terms_needed.try_into().unwrap();
        let mut linear_term_integer_part = BigInt::zero();

        let coefficients = (1..terms_needed).map(|x| {
            let c = abs_log_base.pow(x) / Self::factorial(x.into());
            let c_frac = if x == 1 {
                let c_whole = c.to_integer();
                let c_frac = c - bigint_to_bigrational(&c_whole);
                linear_term_integer_part = c_whole;
                c_frac
            } else {
                c
            };
            if x % 2 == 0 {
                -bigrational_as_fixed_point(c_frac, per_term_precission)
            } else {
                bigrational_as_fixed_point(c_frac, per_term_precission)
            }
        });

        let two_tpo_per_term_precission = BigInt::one() << per_term_precission;

        // one_minus_exp to calculate the threshold rational
        let numer = BigRational::new(
            &two_tpo_per_term_precission * &delegated_stake,
            total_currency.clone(),
        )
        .floor()
        .to_integer();
        let input = BigRational::new(numer, two_tpo_per_term_precission);

        let denom = BigInt::one() << per_term_precission;

        let (res, _) = coefficients.into_iter().fold(
            (BigRational::zero(), BigRational::one()),
            |(acc, x_i), coef| {
                let x_i = &input * &x_i;
                let c = BigRational::new(coef, denom.clone());
                (acc + (&x_i * &c), x_i)
            },
        );

        let threshold_rational = res + input * bigint_to_bigrational(&linear_term_integer_part);

        Self {
            delegated_stake,
            total_currency,
            threshold_rational,
        }
    }

    /// Compares the vrf output to the threshold. If vrf output <= threshold the vrf prover has the rights to
    /// produce a block at the desired slot
    pub fn threshold_met(&self, vrf_out: BigInteger256) -> bool {
        let vrf_out = get_fractional(vrf_out);
        vrf_out <= self.threshold_rational
    }

    fn terms_needed(log_base: &BigRational, bits_of_precission: u32) -> i32 {
        let two = BigInt::one() + BigInt::one();
        let lower_bound = bigint_to_bigrational(&two.pow(bits_of_precission));

        let mut n = 0;
        // let mut d = log_base.pow(1);

        loop {
            let d = log_base.pow(n + 1);
            if bigint_to_bigrational(&Self::factorial(n.into())) / d > lower_bound {
                return n;
            }
            n += 1;
        }
    }

    fn factorial(n: BigInt) -> BigInt {
        if n == BigInt::zero() {
            return BigInt::one();
        }
        let mut res = n.clone();
        let mut i = n - BigInt::one();
        while i != BigInt::zero() {
            res *= i.clone();
            i -= BigInt::one();
        }

        res
    }

    fn log(terms: usize, x: BigRational) -> BigRational {
        let two = BigInt::one() + BigInt::one();
        let a = x - BigRational::one();
        let i0 = BigRational::one();
        let seq = unfold((a.clone(), i0), |(ai, i)| {
            let t = ai.to_owned() / i.to_owned();
            let res = if &i.to_integer() % &two == BigInt::zero() {
                -t
            } else {
                t
            };

            *ai = ai.to_owned() * &a;
            *i = i.to_owned() + &BigRational::one();
            Some(res)
        });

        seq.take(terms).sum()
    }

    fn ciel_log2(n: &BigInt) -> BigInt {
        let two = BigInt::one() + BigInt::one();

        let mut i = 0;

        loop {
            if &two.pow(i) >= n {
                return i.into();
            }
            i += 1;
        }
    }

    fn bit_params(log_base: &BigRational) -> (usize, BigInt, BigInt) {
        let mut k = 0;

        let greatest = |k| -> Option<(usize, BigInt, BigInt)> {
            let mut n: BigInt = Self::terms_needed(log_base, k).into();
            n += BigInt::one();

            let per_term_precision = Self::ciel_log2(&n) + k;
            // println!("[k = {k}] terms_needed = {n} per_term_precision = {}", per_term_precision);

            if (&n * &per_term_precision) + &per_term_precision < Self::SIZE_IN_BITS.into() {
                Some((per_term_precision.try_into().unwrap(), n, k.into()))
            } else {
                None
            }
        };

        let mut best = (0, BigInt::zero(), BigInt::zero());
        while let Some(better) = greatest(k) {
            best = better;
            k += 1;
        }

        best
    }
}

/// Converts an integer to a rational
pub fn get_fractional(vrf_out: BigInteger256) -> BigRational {
    // ocaml:   Bignum_bigint.(shift_left one length_in_bits))
    //          where: length_in_bits = Int.min 256 (Field.size_in_bits - 2)
    //                 Field.size_in_bits = 255
    let two_tpo_256 = BigInt::one() << 253u32;

    let vrf_out = BigInt::from_bytes_be(num::bigint::Sign::Plus, &vrf_out.to_bytes_be());

    BigRational::new(vrf_out, two_tpo_256)
}

// TODO: is there a fn like this?
pub fn bigint_to_bigrational(x: &BigInt) -> BigRational {
    BigRational::new(x.clone(), BigInt::one())
}

pub fn bigrational_as_fixed_point(c: BigRational, per_term_precission: usize) -> BigInt {
    let numer = c.numer();
    let denom = c.denom();

    (numer << per_term_precission) / denom
}

// pub fn can_produce_block(
//     vrf_out: BigInteger256,
//     delegated_stake: BigInt,
//     total_currency: BigInt,
// ) -> bool {
//     Threshold::new(delegated_stake, total_currency).threshold_met(vrf_out)
// }

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use ark_ff::{One, Zero};
    use num::{BigInt, BigRational, ToPrimitive};

    use super::*;

    // TODO: move to regular fns, rework step
    fn first_non_zero(stake: BigInt, total_currency: BigInt, step: BigInt) -> BigInt {
        let ten = BigInt::from_str("10").unwrap();
        let mut stake = stake;
        if step == BigInt::zero() {
            stake + BigInt::one()
        } else {
            loop {
                let thrs = Threshold::new(stake.clone(), total_currency.clone());

                if thrs.threshold_rational != BigRational::zero() {
                    println!("stake: {stake} nanoMINA");
                    return first_non_zero(stake - step.clone(), total_currency, step / ten);
                }
                stake += step.clone();
            }
        }
    }

    #[test]
    #[ignore]
    fn test_threshold_nonzero() {
        // let total_currency = BigInt::from_str("1157953132840039233").unwrap();
        // let initial_stake = BigInt::zero();
        // let initial_step = BigInt::from_str("10000000000000000000").unwrap();

        let total_currency = BigInt::from_str("1025422352000001000").unwrap();
        let initial_stake = BigInt::zero();
        let initial_step = BigInt::from_str("10000000000000000000").unwrap();

        let first_non_zero_nanomina =
            first_non_zero(initial_stake, total_currency.clone(), initial_step);

        let last_zero = first_non_zero_nanomina.clone() - BigInt::one();

        let thrs_zero = Threshold::new(last_zero, total_currency.clone());
        assert_eq!(thrs_zero.threshold_rational, BigRational::zero());

        let thrs_first = Threshold::new(first_non_zero_nanomina.clone(), total_currency);
        assert!(thrs_first.threshold_rational > BigRational::zero());

        let first_non_zero_mina = first_non_zero_nanomina.to_f64().unwrap() / 1_000_000_000.0;

        println!("First non zero stake: {first_non_zero_mina} MINA");
        println!(
            "First non zero threshold: {}",
            thrs_first.threshold_rational.to_f64().unwrap()
        );
    }

    #[test]
    #[ignore]
    fn test_threshold_increase() {
        // let total_currency = BigInt::from_str("1157953132840039233").unwrap();
        // let mut stake_nanomina = BigInt::from_str("1104310162392").unwrap();
        // let mut step = BigInt::from_str("1000000000000").unwrap();

        let total_currency = BigInt::from_str("1025422352000001000").unwrap();
        let mut stake_nanomina = BigInt::from_str("2000000000000000").unwrap();
        let mut step = BigInt::from_str("1000000000000").unwrap();

        loop {
            if stake_nanomina > total_currency {
                break;
            }
            let thrs = Threshold::new(stake_nanomina.clone(), total_currency.clone());
            let stake_mina = stake_nanomina.to_f64().unwrap() / 1_000_000_000.0;
            println!(
                "stake: {stake_mina} MINA - threshold: {}",
                thrs.threshold_rational.to_f64().unwrap()
            );

            stake_nanomina += step.clone();
            step *= 2;
        }
    }
}
