use kimchi::proof::{PointEvaluations, ProofEvaluations};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::v2;

use crate::proofs::{
    field::FieldWitness, public_input::plonk_checks::derive_plonk, verification::make_scalars_env,
    BACKEND_TICK_ROUNDS_N,
};

use super::{
    public_input::{
        plonk_checks::{PlonkMinimal, ShiftedValue},
        prepared_statement::Plonk,
        scalar_challenge::ScalarChallenge,
    },
    to_field_elements::ToFieldElements,
    transaction::Check,
    util::u64_to_field,
    verification::prev_evals_from_p2p,
    witness::Witness,
    BACKEND_TOCK_ROUNDS_N,
};

pub mod ro {

    use mina_curves::pasta::Fq;
    use mina_hasher::Fp;

    use crate::proofs::{
        field::FieldWitness, public_input::scalar_challenge::ScalarChallenge,
        transaction::legacy_input::BitsIterator,
    };

    fn of_bits<F: FieldWitness>(bs: [bool; 255]) -> F {
        bs.iter().rev().fold(F::zero(), |acc, b| {
            let acc = acc + acc;
            if *b {
                acc + F::one()
            } else {
                acc
            }
        })
    }

    pub fn bits_random_oracle<const N: usize>(s: &str) -> [bool; N] {
        use blake2::digest::{Update, VariableOutput};
        use blake2::Blake2sVar;

        let mut hasher = Blake2sVar::new(32).unwrap();
        hasher.update(s.as_bytes());
        let hash = hasher.finalize_boxed();

        let mut bits = BitsIterator::<32> {
            index: 0,
            number: (&*hash).try_into().unwrap(),
        }
        .take(N);

        std::array::from_fn(|_| bits.next().unwrap())
    }

    fn ro<T, F, const N: usize>(n: usize, label: &str, fun: F) -> T
    where
        F: FnOnce([bool; N]) -> T,
    {
        let s = format!("{}_{}", label, n);
        fun(bits_random_oracle::<N>(&s))
    }

    pub fn tock(n: usize) -> Fq {
        ro(n, "fq", of_bits::<Fq>)
    }

    pub fn tick(n: usize) -> Fp {
        ro(n, "fq", of_bits::<Fp>)
    }

    pub fn chal(n: usize) -> ScalarChallenge {
        fn of_bits(bits: [bool; 128]) -> [u64; 2] {
            let mut limbs = bits.chunks(64).map(|chunk| {
                chunk.iter().enumerate().fold(
                    0u64,
                    |acc, (i, b)| {
                        if *b {
                            acc | (1 << i)
                        } else {
                            acc
                        }
                    },
                )
            });
            std::array::from_fn(|_| limbs.next().unwrap())
        }

        let [a, b] = ro(n, "chal", of_bits);
        ScalarChallenge::new(a, b)
    }
}

/// No `BranchData`
#[derive(Clone, Debug)]
pub struct DeferredValues {
    pub plonk: Plonk<Fq>,
    pub combined_inner_product: <Fq as FieldWitness>::Shifting,
    pub b: <Fq as FieldWitness>::Shifting,
    pub xi: [u64; 2],
    pub bulletproof_challenges: Vec<[u64; 2]>,
}

#[derive(Clone, Debug)]
pub struct Unfinalized {
    pub deferred_values: DeferredValues,
    pub should_finalize: bool,
    pub sponge_digest_before_evaluations: [u64; 4],
}

#[derive(Clone, Debug)]
pub struct EvalsWithPublicInput<F: FieldWitness> {
    pub evals: ProofEvaluations<[F; 2]>,
    pub public_input: (F, F),
}

#[derive(Clone, Debug)]
pub struct AllEvals<F: FieldWitness> {
    pub ft_eval1: F,
    pub evals: EvalsWithPublicInput<F>,
}

impl AllEvals<Fq> {
    /// Dummy.evals
    fn dummy_impl() -> Self {
        Self {
            ft_eval1: ro::tock(89),
            evals: EvalsWithPublicInput {
                evals: dummy_evals(),
                public_input: (ro::tock(88), ro::tock(87)),
            },
        }
    }

    /// Dummy.evals
    pub fn dummy() -> Self {
        cache_one! { AllEvals<Fq>, Self::dummy_impl() }
    }
}

impl<F: FieldWitness> From<&v2::PicklesProofProofsVerified2ReprStableV2PrevEvals> for AllEvals<F> {
    fn from(value: &v2::PicklesProofProofsVerified2ReprStableV2PrevEvals) -> Self {
        let v2::PicklesProofProofsVerified2ReprStableV2PrevEvals {
            evals:
                v2::PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals {
                    public_input: (p0, p1),
                    evals,
                },
            ft_eval1,
        } = value;

        Self {
            ft_eval1: ft_eval1.to_field(),
            evals: EvalsWithPublicInput {
                evals: prev_evals_from_p2p(evals).map(&|PointEvaluations { zeta, zeta_omega }| {
                    assert_eq!(zeta.len(), 1);
                    assert_eq!(zeta_omega.len(), 1);
                    [zeta[0], zeta_omega[0]]
                }),
                public_input: (p0.to_field(), p1.to_field()),
            },
        }
    }
}

/// Equivalent of `to_kimchi` in OCaml
pub fn evals_from_p2p<F: FieldWitness>(
    e: &v2::PicklesWrapWireProofEvaluationsStableV1,
) -> ProofEvaluations<[F; 2]> {
    let v2::PicklesWrapWireProofEvaluationsStableV1 {
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
    } = e;

    use mina_p2p_messages::bigint::BigInt;

    let of = |(zeta, zeta_omega): &(BigInt, BigInt)| -> [F; 2] {
        [zeta.to_field(), zeta_omega.to_field()]
    };

    use std::array;
    ProofEvaluations {
        w: w.each_ref().map(of),
        z: of(z),
        s: s.each_ref().map(of),
        coefficients: coefficients.each_ref().map(of),
        generic_selector: of(generic_selector),
        poseidon_selector: of(poseidon_selector),
        complete_add_selector: of(complete_add_selector),
        mul_selector: of(mul_selector),
        emul_selector: of(emul_selector),
        endomul_scalar_selector: of(endomul_scalar_selector),
        range_check0_selector: None,
        range_check1_selector: None,
        foreign_field_add_selector: None,
        foreign_field_mul_selector: None,
        xor_selector: None,
        rot_selector: None,
        lookup_aggregation: None,
        lookup_table: None,
        lookup_sorted: array::from_fn(|_| None),
        runtime_lookup_table: None,
        runtime_lookup_table_selector: None,
        xor_lookup_selector: None,
        lookup_gate_lookup_selector: None,
        range_check_lookup_selector: None,
        foreign_field_mul_lookup_selector: None,
    }
}

fn dummy_evals() -> ProofEvaluations<[Fq; 2]> {
    type Evals = ProofEvaluations<[Fq; 2]>;
    cache_one! {
        Evals,
        {
            // TODO: Update this
            let mut n = 86;

            let mut iter = std::iter::from_fn(|| {
                let res = ro::tock(n);
                n = n.checked_sub(1)?;
                Some(res)
            });

            let mut next = || [iter.next().unwrap(), iter.next().unwrap()];

            ProofEvaluations::<[Fq; 2]> {
                w: std::array::from_fn(|_| next()),
                coefficients: std::array::from_fn(|_| next()),
                z: next(),
                s: std::array::from_fn(|_| next()),
                generic_selector: next(),
                poseidon_selector: next(),
                complete_add_selector: next(),
                mul_selector: next(),
                emul_selector: next(),
                endomul_scalar_selector: next(),
                range_check0_selector: None,
                range_check1_selector: None,
                foreign_field_add_selector: None,
                foreign_field_mul_selector: None,
                xor_selector: None,
                rot_selector: None,
                lookup_aggregation: None,
                lookup_table: None,
                lookup_sorted: std::array::from_fn(|_| None),
                runtime_lookup_table: None,
                runtime_lookup_table_selector: None,
                xor_lookup_selector: None,
                lookup_gate_lookup_selector: None,
                range_check_lookup_selector: None,
                foreign_field_mul_lookup_selector: None
            }
        }
    }
}

/// Value of `Dummy.Ipa.Wrap.challenges`
pub fn dummy_ipa_wrap_challenges() -> [[u64; 2]; BACKEND_TOCK_ROUNDS_N] {
    cache_one!([[u64; 2]; BACKEND_TOCK_ROUNDS_N], {
        std::array::from_fn(|i| ro::chal(15 - i).inner)
    })
}

/// Value of `Dummy.Ipa.Step.challenges`
pub fn dummy_ipa_step_challenges() -> [[u64; 2]; BACKEND_TICK_ROUNDS_N] {
    cache_one!([[u64; 2]; BACKEND_TICK_ROUNDS_N], {
        std::array::from_fn(|i| ro::chal(31 - i).inner)
    })
}

/// Dummy.Ipa.Step.challenges_computed
pub fn dummy_ipa_step_challenges_computed() -> [Fp; BACKEND_TICK_ROUNDS_N] {
    cache_one!([Fp; BACKEND_TICK_ROUNDS_N], {
        let challenges = dummy_ipa_step_challenges();
        challenges.each_ref().map(ScalarChallenge::limbs_to_field)
    })
}

impl Unfinalized {
    pub fn dummy() -> Unfinalized {
        // TODO: They might change between mina release/version ? Not sure
        let one_chal: [u64; 2] = [1, 1];
        let alpha_bytes: [u64; 2] = [746390447645740837, -5643124118675291918i64 as u64];
        let beta_bytes: [u64; 2] = [8345091427968288705, 8258453988658898844];
        let gamma_bytes: [u64; 2] = [8902445049614368905, -5479804816757020655i64 as u64];
        let zeta_bytes: [u64; 2] = [621834770194220300, -4327941673388439925i64 as u64];

        let zeta: Fq = ScalarChallenge::limbs_to_field(&zeta_bytes);
        let alpha: Fq = ScalarChallenge::limbs_to_field(&alpha_bytes);
        let beta: Fq = u64_to_field(&beta_bytes);
        let gamma: Fq = u64_to_field(&gamma_bytes);

        let chals = PlonkMinimal {
            alpha,
            beta,
            gamma,
            zeta,
            joint_combiner: None,
            alpha_bytes,
            beta_bytes,
            gamma_bytes,
            zeta_bytes,
        };

        let evals = dummy_evals();

        const DOMAIN_LOG2: u8 = 15;
        const SRS_LENGTH_LOG2: u64 = 15;
        let env = make_scalars_env(&chals, DOMAIN_LOG2, SRS_LENGTH_LOG2);
        let plonk = derive_plonk(&env, &evals, &chals);

        Unfinalized {
            deferred_values: DeferredValues {
                plonk: Plonk {
                    alpha: alpha_bytes,
                    beta: beta_bytes,
                    gamma: gamma_bytes,
                    zeta: zeta_bytes,
                    zeta_to_srs_length: plonk.zeta_to_srs_length,
                    zeta_to_domain_size: plonk.zeta_to_domain_size,
                    // vbmul: plonk.vbmul,
                    // complete_add: plonk.complete_add,
                    // endomul: plonk.endomul,
                    // endomul_scalar: plonk.endomul_scalar,
                    perm: plonk.perm,
                    lookup: (),
                },
                combined_inner_product: ShiftedValue::new(ro::tock(91)),
                b: ShiftedValue::new(ro::tock(90)),
                xi: one_chal,
                bulletproof_challenges: dummy_ipa_wrap_challenges().to_vec(),
            },
            should_finalize: false,
            // dummy digest
            sponge_digest_before_evaluations: [1, 1, 1, 1],
        }
    }
}

impl<F: FieldWitness> ToFieldElements<F> for Unfinalized {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        let Self {
            deferred_values:
                DeferredValues {
                    plonk:
                        Plonk {
                            alpha,
                            beta,
                            gamma,
                            zeta,
                            zeta_to_srs_length,
                            zeta_to_domain_size,
                            perm,
                            lookup: (),
                        },
                    combined_inner_product,
                    b,
                    xi,
                    bulletproof_challenges,
                },
            should_finalize,
            sponge_digest_before_evaluations,
        } = self;

        // Fq
        {
            combined_inner_product.shifted.to_field_elements(fields);
            b.shifted.to_field_elements(fields);
            zeta_to_srs_length.shifted.to_field_elements(fields);
            zeta_to_domain_size.shifted.to_field_elements(fields);
            perm.shifted.to_field_elements(fields);
        }

        // Digest
        {
            fields.push(u64_to_field(sponge_digest_before_evaluations));
        }

        // Challenge
        {
            fields.push(u64_to_field(beta));
            fields.push(u64_to_field(gamma));
        }

        // Scalar challenge
        {
            fields.push(u64_to_field(alpha));
            fields.push(u64_to_field(zeta));
            fields.push(u64_to_field(xi));
        }

        fields.extend(bulletproof_challenges.iter().map(u64_to_field::<F, 2>));

        // Bool
        {
            fields.push(F::from(*should_finalize));
        }
    }
}

impl Check<Fp> for Unfinalized {
    fn check(&self, w: &mut Witness<Fp>) {
        let Self {
            deferred_values:
                DeferredValues {
                    plonk:
                        Plonk {
                            alpha: _,
                            beta: _,
                            gamma: _,
                            zeta: _,
                            zeta_to_srs_length,
                            zeta_to_domain_size,
                            perm,
                            lookup: (),
                        },
                    combined_inner_product,
                    b,
                    xi: _,
                    bulletproof_challenges: _,
                },
            should_finalize: _,
            sponge_digest_before_evaluations: _,
        } = self;

        combined_inner_product.check(w);
        b.check(w);
        zeta_to_srs_length.check(w);
        zeta_to_domain_size.check(w);
        perm.check(w);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unfinalized() {
        let dummy: Vec<Fp> = Unfinalized::dummy().to_field_elements_owned();
        dbg!(&dummy);
        dbg!(&dummy.len());
    }
}
