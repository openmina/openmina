use ark_ff::{BigInteger256, Field, PrimeField};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::{
    bigint::BigInt, pseq::PaddedSeq,
    v2::PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A,
};
use mina_poseidon::poseidon::ArithmeticSpongeParams;
use std::array;

use crate::public_input::{messages::CurveAffine, scalar_challenge::ScalarChallenge};

pub trait FpExt {
    fn to_decimal(&self) -> String;
}

impl FpExt for Fp {
    fn to_decimal(&self) -> String {
        let r = self.into_repr();
        let bigint: num_bigint::BigUint = r.into();
        bigint.to_string()
    }
}

impl FpExt for Fq {
    fn to_decimal(&self) -> String {
        let r = self.into_repr();
        let bigint: num_bigint::BigUint = r.into();
        bigint.to_string()
    }
}

pub trait SpongeParamsForField<F: Field> {
    fn get_params() -> &'static ArithmeticSpongeParams<F>;
}

impl SpongeParamsForField<Fp> for Fp {
    fn get_params() -> &'static ArithmeticSpongeParams<Fp> {
        kimchi::mina_poseidon::pasta::fp_kimchi::static_params()
    }
}

impl SpongeParamsForField<Fq> for Fq {
    fn get_params() -> &'static ArithmeticSpongeParams<Fq> {
        kimchi::mina_poseidon::pasta::fq_kimchi::static_params()
    }
}

pub fn u64_to_field<F, const N: usize>(v: &[u64; N]) -> F
where
    F: Field + From<BigInteger256>,
{
    let mut bigint: [u64; 4] = [0; 4];
    bigint[..N].copy_from_slice(v);

    let bigint = BigInteger256(bigint);
    F::from(bigint)
}

pub fn extract_polynomial_commitment<F: Field, const N: usize>(
    curves: &[(BigInt, BigInt)],
) -> [CurveAffine<F>; N] {
    array::from_fn(|i| {
        let curve = &curves[i];
        CurveAffine(curve.0.to_field(), curve.1.to_field())
    })
}

pub fn extract_bulletproof<F: Field + From<i32>, const N: usize>(
    v: &[PaddedSeq<
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A,
        N,
    >],
    endo: &F,
) -> [[F; N]; 2] {
    array::from_fn(|i| {
        let old = &v[i];

        array::from_fn(|j| {
            let prechallenge = &old[j].prechallenge.inner;
            let prechallenge: [u64; 2] = array::from_fn(|k| prechallenge[k].as_u64());

            ScalarChallenge::from(prechallenge).to_field(endo)
        })
    })
}
