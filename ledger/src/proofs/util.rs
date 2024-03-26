use ark_ff::{BigInteger256, Field};
use kimchi::proof::ProofEvaluations;
use mina_hasher::Fp;
use mina_p2p_messages::{
    bigint::BigInt, pseq::PaddedSeq,
    v2::PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A,
};

use crate::proofs::field::{field, Boolean, FieldWitness};

use super::{
    public_input::scalar_challenge::ScalarChallenge,
    step::{Opt, OptFlag},
    transaction::InnerCurve,
    witness::Witness,
};

pub fn extract_polynomial_commitment<
    'a,
    F: FieldWitness,
    I: IntoIterator<Item = &'a (BigInt, BigInt)>,
>(
    curves: I,
) -> Vec<InnerCurve<F>> {
    curves
        .into_iter()
        .map(|curve| InnerCurve::from((curve.0.to_field::<F>(), curve.1.to_field())))
        .collect()
}

pub fn extract_bulletproof<
    'a,
    F: FieldWitness,
    I: IntoIterator<
        Item = &'a PaddedSeq<
            PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A,
            N,
        >,
    >,
    const N: usize,
>(
    v: I,
) -> Vec<[F; N]> {
    v.into_iter()
        .map(|old| {
            old.each_ref().map(|old| {
                let prechallenge = old.prechallenge.inner.each_ref().map(|v| v.as_u64());
                ScalarChallenge::limbs_to_field(&prechallenge)
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
pub fn challenge_polynomial<F: FieldWitness>(chals: &[F]) -> impl Fn(F) -> F + '_ {
    |pt: F| {
        let k = chals.len();
        let pow_two_pows = {
            let mut res = vec![pt; k];
            for i in 1..k {
                let y = res[i - 1];
                res[i] = y * y;
            }
            res
        };
        fn prod<F: FieldWitness>(k: usize, fun: impl Fn(usize) -> F) -> F {
            let mut r = fun(0);
            for i in 1..k {
                r = fun(i) * r;
            }
            r
        }
        prod::<F>(k, |i| F::one() + (chals[i] * pow_two_pows[k - 1 - i]))
    }
}

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/pickles/wrap_verifier.ml#L16
pub fn challenge_polynomial_checked<F: FieldWitness>(
    chals: &[F],
) -> impl Fn(F, &mut Witness<F>) -> F + '_ {
    |pt: F, w: &mut Witness<F>| {
        let k = chals.len();
        let pow_two_pows = {
            let mut res = vec![pt; k];
            for i in 1..k {
                let y = res[i - 1];
                res[i] = field::mul(y, y, w);
            }
            res
        };
        fn prod<F: FieldWitness>(
            k: usize,
            fun: impl Fn(usize, &mut Witness<F>) -> F,
            w: &mut Witness<F>,
        ) -> F {
            let mut r = fun(0, w);
            for i in 1..k {
                r = field::mul(fun(i, w), r, w);
            }
            r
        }
        prod(
            k,
            |i, w| F::one() + field::mul(chals[i], pow_two_pows[k - 1 - i], w),
            w,
        )
    }
}

/// https://github.com/MinaProtocol/mina/blob/4af0c229548bc96d76678f11b6842999de5d3b0b/src/lib/pickles_types/plonk_types.ml#L611
pub fn proof_evaluation_to_list<F: FieldWitness>(e: &ProofEvaluations<[F; 2]>) -> Vec<[F; 2]> {
    let ProofEvaluations::<[F; 2]> {
        w,
        z,
        s,
        coefficients,
        generic_selector,
        poseidon_selector,
        complete_add_selector,
        mul_selector,
        emul_selector,
        endomul_scalar_selector,
        range_check0_selector,
        range_check1_selector,
        foreign_field_add_selector,
        foreign_field_mul_selector,
        xor_selector,
        rot_selector,
        lookup_aggregation,
        lookup_table,
        lookup_sorted,
        runtime_lookup_table,
        runtime_lookup_table_selector,
        xor_lookup_selector,
        lookup_gate_lookup_selector,
        range_check_lookup_selector,
        foreign_field_mul_lookup_selector,
    } = e;

    let mut list = vec![
        *z,
        *generic_selector,
        *poseidon_selector,
        *complete_add_selector,
        *mul_selector,
        *emul_selector,
        *endomul_scalar_selector,
    ];

    list.extend(w);
    list.extend(coefficients);
    list.extend(s);

    let optional_gates = [
        range_check0_selector,
        range_check1_selector,
        foreign_field_add_selector,
        foreign_field_mul_selector,
        xor_selector,
        rot_selector,
    ];

    list.extend(optional_gates.iter().filter_map(|v| **v));
    list.extend(lookup_sorted.iter().filter_map(|v| *v));
    list.extend(
        [
            lookup_aggregation,
            lookup_table,
            runtime_lookup_table,
            runtime_lookup_table_selector,
            xor_lookup_selector,
            lookup_gate_lookup_selector,
            range_check_lookup_selector,
            foreign_field_mul_lookup_selector,
        ]
        .iter()
        .filter_map(|v| **v),
    );

    list
}

/// https://github.com/MinaProtocol/mina/blob/4af0c229548bc96d76678f11b6842999de5d3b0b/src/lib/pickles_types/plonk_types.ml#L611
pub fn proof_evaluation_to_list_opt<F: FieldWitness>(
    e: &ProofEvaluations<[F; 2]>,
    hack_feature_flags: OptFlag,
) -> Vec<Opt<[F; 2]>> {
    let ProofEvaluations::<[F; 2]> {
        w,
        z,
        s,
        coefficients,
        generic_selector,
        poseidon_selector,
        complete_add_selector,
        mul_selector,
        emul_selector,
        endomul_scalar_selector,
        range_check0_selector,
        range_check1_selector,
        foreign_field_add_selector,
        foreign_field_mul_selector,
        xor_selector,
        rot_selector,
        lookup_aggregation,
        lookup_table,
        lookup_sorted,
        runtime_lookup_table,
        runtime_lookup_table_selector,
        xor_lookup_selector,
        lookup_gate_lookup_selector,
        range_check_lookup_selector,
        foreign_field_mul_lookup_selector,
    } = e;

    let mut list = vec![
        Opt::Some(*z),
        Opt::Some(*generic_selector),
        Opt::Some(*poseidon_selector),
        Opt::Some(*complete_add_selector),
        Opt::Some(*mul_selector),
        Opt::Some(*emul_selector),
        Opt::Some(*endomul_scalar_selector),
    ];

    list.extend(w.iter().copied().map(Opt::Some));
    list.extend(coefficients.iter().copied().map(Opt::Some));
    list.extend(s.iter().copied().map(Opt::Some));

    let zero = F::zero();
    let to_opt = |v: &Option<[F; 2]>| {
        if let OptFlag::Maybe = hack_feature_flags {
            match v {
                Some(v) => Opt::Maybe(Boolean::True, *v),
                None => Opt::Maybe(Boolean::False, [zero, zero]),
            }
        } else {
            match v {
                Some(v) => Opt::Some(*v),
                None => Opt::No,
            }
        }
    };

    let optional_gates = [
        range_check0_selector,
        range_check1_selector,
        foreign_field_add_selector,
        foreign_field_mul_selector,
        xor_selector,
        rot_selector,
    ];

    list.extend(optional_gates.into_iter().map(to_opt));
    list.extend(lookup_sorted.into_iter().map(to_opt));

    list.extend(
        [
            lookup_aggregation,
            lookup_table,
            runtime_lookup_table,
            runtime_lookup_table_selector,
            xor_lookup_selector,
            lookup_gate_lookup_selector,
            range_check_lookup_selector,
            foreign_field_mul_lookup_selector,
        ]
        .into_iter()
        .map(to_opt),
    );

    list
}

/// https://github.com/MinaProtocol/mina/blob/4af0c229548bc96d76678f11b6842999de5d3b0b/src/lib/pickles_types/plonk_types.ml#L459
pub fn to_absorption_sequence(
    evals: &mina_p2p_messages::v2::PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
) -> Vec<(Vec<Fp>, Vec<Fp>)> {
    let mina_p2p_messages::v2::PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
        w,
        coefficients,
        z,
        s,
        generic_selector,
        poseidon_selector,
        complete_add_selector,
        mul_selector,
        emul_selector,
        endomul_scalar_selector,
        range_check0_selector,
        range_check1_selector,
        foreign_field_add_selector,
        foreign_field_mul_selector,
        xor_selector,
        rot_selector,
        lookup_aggregation,
        lookup_table,
        lookup_sorted,
        runtime_lookup_table,
        runtime_lookup_table_selector,
        xor_lookup_selector,
        lookup_gate_lookup_selector,
        range_check_lookup_selector,
        foreign_field_mul_lookup_selector,
    } = evals;

    let mut list = vec![
        z,
        generic_selector,
        poseidon_selector,
        complete_add_selector,
        mul_selector,
        emul_selector,
        endomul_scalar_selector,
    ];

    list.extend(w.iter());
    list.extend(coefficients.iter());
    list.extend(s.iter());

    list.extend(
        [
            range_check0_selector,
            range_check1_selector,
            foreign_field_add_selector,
            foreign_field_mul_selector,
            xor_selector,
            rot_selector,
            lookup_aggregation,
            lookup_table,
        ]
        .iter()
        .filter_map(|v| v.as_ref()),
    );

    list.extend(lookup_sorted.iter().filter_map(|v| v.as_ref()));

    list.extend(
        [
            runtime_lookup_table,
            runtime_lookup_table_selector,
            xor_lookup_selector,
            lookup_gate_lookup_selector,
            range_check_lookup_selector,
            foreign_field_mul_lookup_selector,
        ]
        .iter()
        .filter_map(|v| v.as_ref()),
    );

    list.iter()
        .map(|(a, b)| {
            let a: Vec<_> = a.iter().map(Fp::from).collect();
            let b: Vec<_> = b.iter().map(Fp::from).collect();
            (a, b)
        })
        .collect()
}

// TODO: Dedup with above
pub fn to_absorption_sequence2<F: FieldWitness>(
    evals: &ProofEvaluations<[F; 2]>,
) -> Vec<(Vec<F>, Vec<F>)> {
    let ProofEvaluations {
        w,
        coefficients,
        z,
        s,
        generic_selector,
        poseidon_selector,
        complete_add_selector,
        mul_selector,
        emul_selector,
        endomul_scalar_selector,
        range_check0_selector,
        range_check1_selector,
        foreign_field_add_selector,
        foreign_field_mul_selector,
        xor_selector,
        rot_selector,
        lookup_aggregation,
        lookup_table,
        lookup_sorted,
        runtime_lookup_table,
        runtime_lookup_table_selector,
        xor_lookup_selector,
        lookup_gate_lookup_selector,
        range_check_lookup_selector,
        foreign_field_mul_lookup_selector,
    } = evals;

    let mut list = vec![
        z,
        generic_selector,
        poseidon_selector,
        complete_add_selector,
        mul_selector,
        emul_selector,
        endomul_scalar_selector,
    ];

    list.extend(w.iter());
    list.extend(coefficients.iter());
    list.extend(s.iter());

    list.extend(
        [
            range_check0_selector,
            range_check1_selector,
            foreign_field_add_selector,
            foreign_field_mul_selector,
            xor_selector,
            rot_selector,
            lookup_aggregation,
            lookup_table,
        ]
        .into_iter()
        .filter_map(|v| v.as_ref()),
    );

    list.extend(lookup_sorted.iter().filter_map(|v| v.as_ref()));

    list.extend(
        [
            runtime_lookup_table,
            runtime_lookup_table_selector,
            xor_lookup_selector,
            lookup_gate_lookup_selector,
            range_check_lookup_selector,
            foreign_field_mul_lookup_selector,
        ]
        .into_iter()
        .filter_map(|v| v.as_ref()),
    );

    list.iter().map(|[a, b]| (vec![*a], vec![*b])).collect()
}

/// https://github.com/MinaProtocol/mina/blob/4af0c229548bc96d76678f11b6842999de5d3b0b/src/lib/pickles_types/plonk_types.ml#L674
pub fn to_absorption_sequence_opt<F: FieldWitness>(
    evals: &ProofEvaluations<[F; 2]>,
    hack_feature_flags: OptFlag,
) -> Vec<Opt<[F; 2]>> {
    let ProofEvaluations {
        w,
        coefficients,
        z,
        s,
        generic_selector,
        poseidon_selector,
        complete_add_selector,
        mul_selector,
        emul_selector,
        endomul_scalar_selector,
        range_check0_selector,
        range_check1_selector,
        foreign_field_add_selector,
        foreign_field_mul_selector,
        xor_selector,
        rot_selector,
        lookup_aggregation,
        lookup_table,
        lookup_sorted,
        runtime_lookup_table,
        runtime_lookup_table_selector,
        xor_lookup_selector,
        lookup_gate_lookup_selector,
        range_check_lookup_selector,
        foreign_field_mul_lookup_selector,
    } = evals;

    let mut list = vec![
        Opt::Some(*z),
        Opt::Some(*generic_selector),
        Opt::Some(*poseidon_selector),
        Opt::Some(*complete_add_selector),
        Opt::Some(*mul_selector),
        Opt::Some(*emul_selector),
        Opt::Some(*endomul_scalar_selector),
    ];

    list.extend(w.iter().copied().map(Opt::Some));
    list.extend(coefficients.iter().copied().map(Opt::Some));
    list.extend(s.iter().copied().map(Opt::Some));

    let zero = F::zero();
    let to_opt = |v: &Option<[F; 2]>| {
        if let OptFlag::Maybe = hack_feature_flags {
            match v {
                Some(v) => Opt::Maybe(Boolean::True, *v),
                None => Opt::Maybe(Boolean::False, [zero, zero]),
            }
        } else {
            match v {
                Some(v) => Opt::Some(*v),
                None => Opt::No,
            }
        }
    };

    list.extend(
        [
            *range_check0_selector,
            *range_check1_selector,
            *foreign_field_add_selector,
            *foreign_field_mul_selector,
            *xor_selector,
            *rot_selector,
            *lookup_aggregation,
            *lookup_table,
        ]
        .iter()
        .map(to_opt),
    );

    list.extend(lookup_sorted.iter().map(to_opt));

    list.extend(
        [
            runtime_lookup_table,
            runtime_lookup_table_selector,
            xor_lookup_selector,
            lookup_gate_lookup_selector,
            range_check_lookup_selector,
            foreign_field_mul_lookup_selector,
        ]
        .into_iter()
        .map(to_opt),
    );

    list
}

pub fn sha256_sum(s: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(s);
    hex::encode(hasher.finalize())
}
