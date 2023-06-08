use std::array;

use ark_ff::{BigInteger256, Field};
use mina_p2p_messages::{
    bigint::BigInt, pseq::PaddedSeq,
    v2::PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A,
};

use crate::CurveAffine;

use super::public_input::scalar_challenge::ScalarChallenge;

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

pub fn u64_to_field<F, const N: usize>(v: &[u64; N]) -> F
where
    F: Field + From<BigInteger256>,
{
    let mut bigint: [u64; 4] = [0; 4];
    bigint[..N].copy_from_slice(v);

    let bigint = BigInteger256(bigint);
    F::from(bigint)
}
