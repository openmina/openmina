// REVIEW(dw): STATUS: DONE, with commecnts
use ark_ff::{fields::arithmetic::InvalidBigInt, BigInteger256, Field};
use kimchi::proof::{PointEvaluations, ProofEvaluations};
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

// REVIEW(dw): This converts two affine coordinates into a projective coordinate..
// Code is
// Self::of_affine(make_group(x, y))
// There should be a check it is in the prime subgroup.
// Mot of the code in transaction.rs could reuse some code from arkworkds, no?
// It is dangerous to reimplement everything from scratch.
pub fn extract_polynomial_commitment<
    'a,
    F: FieldWitness,
    I: IntoIterator<Item = &'a (BigInt, BigInt)>,
>(
    curves: I,
) -> Result<Vec<InnerCurve<F>>, InvalidBigInt> {
    curves
        .into_iter()
        .map(|curve| {
            Ok(InnerCurve::from((
                curve.0.to_field::<F>()?,
                curve.1.to_field()?,
            )))
        })
        .collect()
}

// REVIEW(dw): move into proof-systems?
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

// REVIEW(dw): I would add some size chek + unit tests
// REVIEW(dw): doc should mention if it is from montgomery repr or decimal repr
pub fn four_u64_to_field<F>(v: &[u64; 4]) -> Result<F, InvalidBigInt>
where
    F: Field + TryFrom<BigInteger256, Error = InvalidBigInt>,
{
    let mut bigint: [u64; 4] = [0; 4];
    bigint[..4].copy_from_slice(v);

    let bigint = BigInteger256(bigint);
    F::try_from(bigint)
}

pub fn two_u64_to_field<F>(v: &[u64; 2]) -> F
where
    F: Field + TryFrom<BigInteger256, Error = InvalidBigInt>,
{
    let mut bigint: [u64; 4] = [0; 4];
    bigint[..2].copy_from_slice(v);

    let bigint = BigInteger256(bigint);
    F::try_from(bigint).unwrap() // Never fail with 2 limbs
}

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/pickles/wrap_verifier.ml#L16
// REVIEW(dw): would be nice to have simple unit tests to check with Caml code
// REVIEW(dw): there might be somth already in proof-systems for this
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
// REVIEW(dw): would be nice to have simple unit tests to check with Caml code
// REVIEW(dw): there might be somth already in proof-systems for this
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

/// Note: Outdated URL
/// Note: Different than `to_absorption_sequence`
/// https://github.com/MinaProtocol/mina/blob/4af0c229548bc96d76678f11b6842999de5d3b0b/src/lib/pickles_types/plonk_types.ml#L611
// REVIEW(dw): order checked with to_list in Plonk_types. OK
pub fn proof_evaluation_to_list<F: FieldWitness>(
    e: &ProofEvaluations<PointEvaluations<Vec<F>>>,
) -> Vec<&PointEvaluations<Vec<F>>> {
    let ProofEvaluations::<PointEvaluations<Vec<F>>> {
        public: _,
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
        z,
        generic_selector,
        poseidon_selector,
        complete_add_selector,
        mul_selector,
        emul_selector,
        endomul_scalar_selector,
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

    list.extend(optional_gates.iter().filter_map(|v| (*v).as_ref()));
    list.extend(lookup_sorted.iter().filter_map(|v| v.as_ref()));
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
        .filter_map(|v| (*v).as_ref()),
    );

    list
}

pub fn proof_evaluation_to_absorption_sequence<F: FieldWitness>(
    e: &ProofEvaluations<PointEvaluations<Vec<F>>>,
) -> Vec<&PointEvaluations<Vec<F>>> {
    let ProofEvaluations {
        public: _,
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
    } = e;

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

    #[allow(clippy::iter_cloned_collect)]
    list.iter().cloned().collect()
}

/// https://github.com/MinaProtocol/mina/blob/4af0c229548bc96d76678f11b6842999de5d3b0b/src/lib/pickles_types/plonk_types.ml#L611
pub fn proof_evaluation_to_list_opt<F: FieldWitness>(
    e: &ProofEvaluations<PointEvaluations<Vec<F>>>,
    hack_feature_flags: OptFlag,
) -> Vec<Opt<PointEvaluations<Vec<F>>>> {
    let ProofEvaluations {
        public: _,
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
        Opt::Some(z.clone()),
        Opt::Some(generic_selector.clone()),
        Opt::Some(poseidon_selector.clone()),
        Opt::Some(complete_add_selector.clone()),
        Opt::Some(mul_selector.clone()),
        Opt::Some(emul_selector.clone()),
        Opt::Some(endomul_scalar_selector.clone()),
    ];

    list.extend(w.iter().cloned().map(Opt::Some));
    list.extend(coefficients.iter().cloned().map(Opt::Some));
    list.extend(s.iter().cloned().map(Opt::Some));

    let zero = || PointEvaluations {
        zeta: vec![F::zero()],
        zeta_omega: vec![F::zero()],
    };
    // REVIEW(dw): check this twice. I don't remember where it is linked to
    let to_opt = |v: &Option<PointEvaluations<Vec<F>>>| {
        if let OptFlag::Maybe = hack_feature_flags {
            match v {
                Some(v) => Opt::Maybe(Boolean::True, v.clone()),
                None => Opt::Maybe(Boolean::False, zero()),
            }
        } else {
            match v {
                Some(v) => Opt::Some(v.clone()),
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
    list.extend(lookup_sorted.iter().map(to_opt));

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

/// https://github.com/MinaProtocol/mina/blob/4af0c229548bc96d76678f11b6842999de5d3b0b/src/lib/pickles_types/plonk_types.ml#L674
// REVIEW(dw): Ok
pub fn to_absorption_sequence_opt<F: FieldWitness>(
    evals: &ProofEvaluations<PointEvaluations<Vec<F>>>,
    hack_feature_flags: OptFlag,
) -> Vec<Opt<PointEvaluations<Vec<F>>>> {
    let ProofEvaluations {
        public: _,
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
        Opt::Some(z.clone()),
        Opt::Some(generic_selector.clone()),
        Opt::Some(poseidon_selector.clone()),
        Opt::Some(complete_add_selector.clone()),
        Opt::Some(mul_selector.clone()),
        Opt::Some(emul_selector.clone()),
        Opt::Some(endomul_scalar_selector.clone()),
    ];

    list.extend(w.iter().cloned().map(Opt::Some));
    list.extend(coefficients.iter().cloned().map(Opt::Some));
    list.extend(s.iter().cloned().map(Opt::Some));

    let zero = || PointEvaluations {
        zeta: vec![F::zero()],
        zeta_omega: vec![F::zero()],
    };
    let to_opt = |v: &Option<PointEvaluations<Vec<F>>>| {
        if let OptFlag::Maybe = hack_feature_flags {
            match v {
                Some(v) => Opt::Maybe(Boolean::True, v.clone()),
                None => Opt::Maybe(Boolean::False, zero()),
            }
        } else {
            match v {
                Some(v) => Opt::Some(v.clone()),
                None => Opt::No,
            }
        }
    };

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
        .map(|e| to_opt(e)),
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

// REVIEW(dw): test vectors?
pub fn sha256_sum(s: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(s);
    hex::encode(hasher.finalize())
}

pub fn field_of_bits<F: FieldWitness, const N: usize>(bs: &[bool; N]) -> F {
    bs.iter().rev().fold(F::zero(), |acc, b| {
        let acc = acc + acc;
        if *b {
            acc + F::one()
        } else {
            acc
        }
    })
}
