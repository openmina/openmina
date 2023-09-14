use std::array;

use ark_ff::{BigInteger256, Field, One};
use mina_hasher::Fp;
use mina_p2p_messages::{
    bigint::BigInt, pseq::PaddedSeq,
    v2::PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A,
};

use crate::CurveAffine;

use super::public_input::scalar_challenge::ScalarChallenge;

pub fn extract_polynomial_commitment<F: Field>(curves: &[(BigInt, BigInt)]) -> Vec<CurveAffine<F>> {
    curves
        .iter()
        .map(|curve| CurveAffine(curve.0.to_field(), curve.1.to_field()))
        .collect()
}

pub fn extract_bulletproof<F: Field + From<i32>, const N: usize>(
    v: &[PaddedSeq<
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A,
        N,
    >],
    endo: &F,
) -> Vec<[F; N]> {
    v.iter()
        .map(|old| {
            array::from_fn(|j| {
                let prechallenge = &old[j].prechallenge.inner;
                let prechallenge: [u64; 2] = array::from_fn(|k| prechallenge[k].as_u64());
                ScalarChallenge::from(prechallenge).to_field(endo)
            })
        })
        .collect()
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

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/pickles/wrap_verifier.ml#L16
pub fn challenge_polynomial(chals: &[Fp]) -> Box<dyn Fn(Fp) -> Fp> {
    let chals = chals.to_vec();
    Box::new(move |pt: Fp| {
        let k = chals.len();
        let pow_two_pows = {
            let mut res = vec![pt; k];
            for i in 1..k {
                let y = res[i - 1];
                res[i] = y * y;
            }
            res
        };
        fn prod(k: usize, fun: impl Fn(usize) -> Fp) -> Fp {
            let mut r = fun(0);
            for i in 1..k {
                r = fun(i) * r;
            }
            r
        }
        prod(k, |i| Fp::one() + (chals[i] * pow_two_pows[k - 1 - i]))
    })
}

// /// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/pickles_types/plonk_types.ml#L588
// pub fn proof_evaluation_to_list(e: &ProofEvaluations<[Fp; 2]>) -> Vec<[Fp; 2]> {
//     let ProofEvaluations::<[Fp; 2]> {
//         w,
//         z,
//         s,
//         coefficients,
//         // lookup,
//         generic_selector,
//         poseidon_selector,
//     } = e;

//     let mut list = vec![*z, *generic_selector, *poseidon_selector];

//     list.extend(w);
//     list.extend(coefficients);
//     list.extend(s);

//     if let Some(lookup) = lookup {
//         list.extend(lookup.sorted.clone());
//         list.push(lookup.aggreg);
//         list.push(lookup.table);
//         list.extend(lookup.runtime);
//     }

//     list
// }

// /// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/pickles_types/plonk_types.ml#L437
// pub fn to_absorption_sequence(
//     evals: &mina_p2p_messages::v2::PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
// ) -> Vec<(Vec<Fp>, Vec<Fp>)> {
//     let mina_p2p_messages::v2::PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
//         w,
//         coefficients,
//         z,
//         s,
//         generic_selector,
//         poseidon_selector,
//         lookup,
//     } = evals;

//     let mut list = vec![
//         z.clone(),
//         generic_selector.clone(),
//         poseidon_selector.clone(),
//     ];

//     list.extend(w.to_vec());
//     list.extend(coefficients.to_vec());
//     list.extend(s.to_vec());

//     if let Some(lookup) = lookup {
//         let PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvalsLookupA {
//             sorted,
//             aggreg,
//             table,
//             runtime,
//         } = lookup;

//         list.push(aggreg.clone());
//         list.push(table.clone());
//         list.extend(sorted.clone());
//         list.extend(runtime.clone());
//     };

//     list.iter()
//         .map(|(a, b)| {
//             let a: Vec<_> = a.iter().map(Fp::from).collect();
//             let b: Vec<_> = b.iter().map(Fp::from).collect();
//             (a, b)
//         })
//         .collect()
// }
