use std::{path::Path, str::FromStr};

use crate::{
    proofs::{
        prover::make_prover,
        public_input::{
            plonk_checks::ShiftingValue,
            prepared_statement::{DeferredValues, PreparedStatement, ProofState},
        },
        verifier_index::wrap_domains,
        wrap::{create_oracle, wrap_verifier, Domain, COMMON_MAX_DEGREE_WRAP_LOG2},
    },
    CurveAffine,
};
use ark_ec::short_weierstrass_jacobian::GroupAffine;
use ark_ff::{BigInteger256, One, Zero};
use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};
use kimchi::{
    proof::{PointEvaluations, ProverCommitments, ProverProof},
    verifier_index::VerifierIndex,
};
use mina_curves::pasta::Pallas;
use mina_curves::pasta::{Fq, PallasParameters};
use mina_hasher::Fp;
use mina_p2p_messages::v2;
use mina_poseidon::constants::PlonkSpongeConstantsKimchi as Constants;
use poly_commitment::evaluation_proof::OpeningProof;

use crate::{
    proofs::{
        public_input::{
            plonk_checks::{derive_plonk, InCircuit},
            prepared_statement::Plonk,
        },
        util::{challenge_polynomial, to_absorption_sequence2, u64_to_field},
        verification::{evals_from_p2p, make_scalars_env},
        witness::{endos, transaction_snark::assert_equal_local_state},
        wrap::{
            combined_inner_product2, evals_of_split_evals, CombinedInnerProductParams2,
            COMMON_MAX_DEGREE_STEP_LOG2,
        },
        BACKEND_TICK_ROUNDS_N, BACKEND_TOCK_ROUNDS_N,
    },
    scan_state::{
        fee_excess::FeeExcess,
        pending_coinbase,
        scan_state::transaction_snark::{
            validate_ledgers_at_merge_checked, SokDigest, SokMessage, Statement, StatementLedgers,
        },
    },
};

use self::step_verifier::{proof_verified_to_prefix, VerifyParams};

use super::{
    public_input::{messages::MessagesForNextWrapProof, plonk_checks::PlonkMinimal},
    to_field_elements::ToFieldElements,
    unfinalized::{AllEvals, EvalsWithPublicInput, Unfinalized},
    util::extract_bulletproof,
    witness::{
        make_group, scalar_challenge::to_field_checked, Boolean, Check, FieldWitness, InnerCurve,
        MessagesForNextStepProof, PlonkVerificationKeyEvals, Prover,
        ReducedMessagesForNextStepProof, Witness,
    },
    wrap::{CircuitVar, Domains},
};

fn read_witnesses() -> std::io::Result<Vec<Fp>> {
    let f = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("rampup4")
            .join("fps_merge.txt"),
    )?;
    // let f = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("fps.txt"))?;

    let fps = f
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| Fp::from_str(s).unwrap())
        .collect::<Vec<_>>();

    // TODO: Implement [0..652]
    // Ok(fps.split_off(652))
    Ok(fps)
}

fn merge_main(
    statement: Statement<SokDigest>,
    proofs: &(v2::LedgerProofProdStableV2, v2::LedgerProofProdStableV2),
    w: &mut Witness<Fp>,
) -> (Statement<SokDigest>, Statement<SokDigest>) {
    let (s1, s2) = w.exists({
        let (p1, p2) = proofs;
        let (s1, s2) = (&p1.0.statement, &p2.0.statement);
        let s1: Statement<SokDigest> = s1.into();
        let s2: Statement<SokDigest> = s2.into();
        (s1, s2)
    });

    let _fee_excess = FeeExcess::combine_checked(&s1.fee_excess, &s2.fee_excess, w);

    pending_coinbase::Stack::check_merge(
        (
            &s1.source.pending_coinbase_stack,
            &s1.target.pending_coinbase_stack,
        ),
        (
            &s2.source.pending_coinbase_stack,
            &s2.target.pending_coinbase_stack,
        ),
        w,
    );

    let _supply_increase = {
        let s1 = s1.supply_increase.to_checked::<Fp>();
        let s2 = s2.supply_increase.to_checked::<Fp>();

        // Made by `value` call in `add`
        w.exists_no_check(s1.value());
        w.exists_no_check(s2.value());

        s1.add(&s2, w)
    };

    assert_equal_local_state(&statement.source.local_state, &s1.source.local_state, w);
    assert_equal_local_state(&statement.target.local_state, &s2.target.local_state, w);

    let _valid_ledger = validate_ledgers_at_merge_checked(
        &StatementLedgers::of_statement(&s1),
        &StatementLedgers::of_statement(&s2),
        w,
    );

    {
        // Only `Statement.fee_excess`, not `fee_excess`
        let FeeExcess {
            fee_excess_l,
            fee_excess_r,
            ..
        } = statement.fee_excess;
        w.exists_no_check(fee_excess_l.to_checked::<Fp>().value());
        w.exists_no_check(fee_excess_r.to_checked::<Fp>().value());

        // Only `Statement.supply_increase`, not `supply_increase`
        let supply_increase = statement.supply_increase;
        w.exists_no_check(supply_increase.to_checked::<Fp>().value());
    }

    (s1, s2)
}

#[derive(Clone, Debug)]
struct PreviousProofStatement<'a> {
    public_input: &'a Statement<SokDigest>,
    proof: &'a v2::LedgerProofProdStableV2,
    proof_must_verify: Boolean,
}

struct InductiveRule<'a> {
    previous_proof_statements: [PreviousProofStatement<'a>; 2],
    public_output: (),
    auxiliary_output: (),
}

fn dlog_plonk_index(wrap_prover: &Prover<Fq>) -> PlonkVerificationKeyEvals<Fp> {
    // TODO: Dedup `crate::PlonkVerificationKeyEvals` and `PlonkVerificationKeyEvals`
    let v =
        crate::PlonkVerificationKeyEvals::from(wrap_prover.index.verifier_index.as_ref().unwrap());
    PlonkVerificationKeyEvals::from(v)
}

impl<F: FieldWitness>
    From<&v2::PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonk>
    for PlonkMinimal<F>
{
    fn from(
        value: &v2::PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonk,
    ) -> Self {
        let v2::PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonk {
            alpha,
            beta,
            gamma,
            zeta,
            joint_combiner,
            feature_flags: _, // TODO: Handle features flags
        } = value;

        let alpha_bytes = std::array::from_fn(|i| alpha.inner[i].as_u64());
        let beta_bytes = std::array::from_fn(|i| beta[i].as_u64());
        let gamma_bytes = std::array::from_fn(|i| gamma[i].as_u64());
        let zeta_bytes = std::array::from_fn(|i| zeta.inner[i].as_u64());

        assert!(joint_combiner.is_none());

        PlonkMinimal::<F, 2> {
            alpha: u64_to_field(&alpha_bytes),
            beta: u64_to_field(&beta_bytes),
            gamma: u64_to_field(&gamma_bytes),
            zeta: u64_to_field(&zeta_bytes),
            joint_combiner: None,
            alpha_bytes,
            beta_bytes,
            gamma_bytes,
            zeta_bytes,
        }
    }
}

fn to_bytes(f: Fp) -> [u64; 4] {
    let BigInteger256([a, b, c, d]): BigInteger256 = f.into();
    [a, b, c, d]
}

fn to_4limbs(v: [u64; 2]) -> [u64; 4] {
    [v[0], v[1], 0, 0]
}

pub fn expand_deferred(
    evals: &AllEvals<Fp>,
    old_bulletproof_challenges: &Vec<[Fp; 16]>,
    proof_state: &StatementProofState,
) -> DeferredValues<Fp> {
    use super::public_input::scalar_challenge::ScalarChallenge;

    let (_, endo) = endos::<Fq>();

    let plonk0 = &proof_state.deferred_values.plonk;

    let zeta = ScalarChallenge::from(plonk0.zeta_bytes).to_field(&endo);
    let alpha = ScalarChallenge::from(plonk0.alpha_bytes).to_field(&endo);
    let step_domain: u8 = proof_state.deferred_values.branch_data.domain_log2.as_u8();
    let domain: Radix2EvaluationDomain<Fp> =
        Radix2EvaluationDomain::new(1 << step_domain as u64).unwrap();
    let zetaw = zeta * domain.group_gen;

    let plonk_minimal = PlonkMinimal::<Fp, 4> {
        alpha,
        beta: plonk0.beta,
        gamma: plonk0.gamma,
        zeta,
        joint_combiner: None,
        alpha_bytes: to_bytes(alpha),
        beta_bytes: to_4limbs(plonk0.beta_bytes),
        gamma_bytes: to_4limbs(plonk0.gamma_bytes),
        zeta_bytes: to_bytes(zeta),
    };

    let es = evals.evals.evals.map_ref(&|[a, b]| PointEvaluations {
        zeta: vec![*a],
        zeta_omega: vec![*b],
    });
    let combined_evals = evals_of_split_evals(zeta, zetaw, &es, BACKEND_TICK_ROUNDS_N);

    let srs_length_log2 = COMMON_MAX_DEGREE_STEP_LOG2;
    let env = make_scalars_env(&plonk_minimal, step_domain, srs_length_log2);

    let plonk = {
        let InCircuit {
            alpha: _,
            beta: _,
            gamma: _,
            zeta: _,
            zeta_to_domain_size,
            zeta_to_srs_length,
            perm,
        } = derive_plonk(&env, &combined_evals, &plonk_minimal);

        Plonk {
            alpha: plonk0.alpha_bytes,
            beta: plonk0.beta_bytes,
            gamma: plonk0.gamma_bytes,
            zeta: plonk0.zeta_bytes,
            zeta_to_srs_length,
            zeta_to_domain_size,
            perm,
            lookup: (),
        }
    };

    let xs = to_absorption_sequence2(&evals.evals.evals);
    let (x1, x2) = &evals.evals.public_input;

    let old_bulletproof_challenges: Vec<_> = old_bulletproof_challenges
        .iter()
        .map(|v| std::array::from_fn(|i| ScalarChallenge::from(v[i]).to_field(&endo)))
        .collect();

    let challenges_digest = {
        use crate::Sponge;
        let mut sponge = crate::ArithmeticSponge::<Fp, crate::PlonkSpongeConstantsKimchi>::new(
            crate::static_params(),
        );
        for old_bulletproof_challenges in &old_bulletproof_challenges {
            sponge.absorb(old_bulletproof_challenges);
        }
        sponge.squeeze()
    };

    type SpongeParams = mina_poseidon::constants::PlonkSpongeConstantsKimchi;
    type EFqSponge =
        mina_poseidon::sponge::DefaultFqSponge<mina_curves::pasta::PallasParameters, SpongeParams>;
    use mina_poseidon::FqSponge;

    let mut sponge = EFqSponge::new(mina_poseidon::pasta::fp_kimchi::static_params());
    sponge.absorb_fq(&[u64_to_field(&proof_state.sponge_digest_before_evaluations)]);
    sponge.absorb_fq(&[challenges_digest]);
    sponge.absorb_fq(&[evals.ft_eval1]);
    sponge.absorb_fq(&[*x1, *x2]);
    xs.iter().for_each(|(x1, x2)| {
        sponge.absorb_fq(x1);
        sponge.absorb_fq(x2);
    });
    let xi_chal = sponge.squeeze_limbs(2);
    let r_chal = sponge.squeeze_limbs(2);

    let xi = ScalarChallenge::from(xi_chal.clone()).to_field(&endo);
    let r = ScalarChallenge::from(r_chal).to_field(&endo);

    let public_input = &evals.evals.public_input;
    let combined_inner_product_actual =
        combined_inner_product2(CombinedInnerProductParams2::<_, { Fp::NROUNDS }, 4> {
            env: &env,
            evals: &evals.evals.evals,
            public: [public_input.0, public_input.1],
            minimal: &plonk_minimal,
            ft_eval1: evals.ft_eval1,
            r,
            old_bulletproof_challenges: &old_bulletproof_challenges,
            xi,
            zetaw,
        });

    let bulletproof_challenges: Vec<_> = proof_state
        .deferred_values
        .bulletproof_challenges
        .iter()
        .map(|v| ScalarChallenge::from(*v).to_field(&endo))
        .collect();

    let b_actual = {
        let challenge_polys = challenge_polynomial(&bulletproof_challenges);
        challenge_polys(zeta) + (r * challenge_polys(zetaw))
    };

    let to_shifted = |f: Fp| <Fp as FieldWitness>::Shifting::of_field(f);

    DeferredValues {
        plonk,
        combined_inner_product: to_shifted(combined_inner_product_actual),
        b: to_shifted(b_actual),
        xi: xi_chal.try_into().unwrap(),
        bulletproof_challenges,
        branch_data: proof_state.deferred_values.branch_data.clone(),
    }
}

fn expand_proof(
    dlog_vk: &VerifierIndex<Pallas>,
    dlog_plonk_index: &PlonkVerificationKeyEvals<Fp>,
    app_state: &Statement<SokDigest>,
    t: &v2::LedgerProofProdStableV2,
    _tag: (),
    must_verify: Boolean,
) -> ExpandedProof {
    use super::public_input::scalar_challenge::ScalarChallenge;

    let t = &t.0.proof.0;

    let plonk0: PlonkMinimal<Fp> = (&t.statement.proof_state.deferred_values.plonk).into();

    let plonk = {
        let domain: u8 = t
            .statement
            .proof_state
            .deferred_values
            .branch_data
            .domain_log2
            .as_u8();

        let (_, endo) = endos::<Fq>();
        let alpha = ScalarChallenge::from(plonk0.alpha_bytes).to_field(&endo);
        let zeta = ScalarChallenge::from(plonk0.zeta_bytes).to_field(&endo);
        let w: Fp = Radix2EvaluationDomain::new(1 << dlog_vk.domain.log_size_of_group)
            .unwrap()
            .group_gen;
        let zetaw = zeta * w;

        let es = evals_from_p2p(&t.prev_evals.evals.evals);
        let combined_evals = evals_of_split_evals(zeta, zetaw, &es, BACKEND_TICK_ROUNDS_N);

        let plonk_minimal = PlonkMinimal::<Fp, 4> {
            alpha,
            beta: plonk0.beta,
            gamma: plonk0.gamma,
            zeta,
            joint_combiner: None,
            alpha_bytes: to_bytes(alpha),
            beta_bytes: to_4limbs(plonk0.beta_bytes),
            gamma_bytes: to_4limbs(plonk0.gamma_bytes),
            zeta_bytes: to_bytes(zeta),
        };

        let srs_length_log2 = COMMON_MAX_DEGREE_STEP_LOG2;
        let env = make_scalars_env(&plonk_minimal, domain, srs_length_log2);

        derive_plonk(&env, &combined_evals, &plonk_minimal)
    };

    let statement = &t.statement;

    let prev_challenges: Vec<[Fq; BACKEND_TOCK_ROUNDS_N]> = {
        let (_, endo) = endos::<Fp>();

        let old_bulletproof_challenges = &statement
            .proof_state
            .messages_for_next_wrap_proof
            .old_bulletproof_challenges;
        extract_bulletproof(
            &[
                old_bulletproof_challenges.0[0].0.clone(),
                old_bulletproof_challenges.0[1].0.clone(),
            ],
            &endo,
        )
    };

    let old_bulletproof_challenges: Vec<[Fp; 16]> = statement
        .messages_for_next_step_proof
        .old_bulletproof_challenges
        .iter()
        .map(|v| {
            v.0.clone()
                .map(|v| u64_to_field(&v.prechallenge.inner.0.map(|v| v.as_u64())))
        })
        .collect();

    let deferred_values_computed = {
        let evals: AllEvals<Fp> = (&t.prev_evals).into();
        let proof_state: StatementProofState = (&statement.proof_state).into();

        expand_deferred(&evals, &old_bulletproof_challenges, &proof_state)
    };

    let (_, endo) = endos::<Fq>();
    let old_bulletproof_challenges: Vec<_> = old_bulletproof_challenges
        .iter()
        .map(|v| std::array::from_fn(|i| ScalarChallenge::from(v[i]).to_field(&endo)))
        .collect();

    let messages_for_next_step_proof = MessagesForNextStepProof {
        app_state,
        dlog_plonk_index,
        challenge_polynomial_commitments: statement
            .messages_for_next_step_proof
            .challenge_polynomial_commitments
            .iter()
            .map(|(x, y)| InnerCurve::of_affine(make_group(x.to_field::<Fp>(), y.to_field())))
            .collect(),
        old_bulletproof_challenges: old_bulletproof_challenges.clone(),
    }
    .hash();

    let deferred_values = deferred_values_computed;
    let prev_statement_with_hashes = PreparedStatement {
        proof_state: ProofState {
            deferred_values: DeferredValues {
                plonk: Plonk {
                    alpha: plonk0.alpha_bytes,
                    beta: plonk0.beta_bytes,
                    gamma: plonk0.gamma_bytes,
                    zeta: plonk0.zeta_bytes,
                    zeta_to_srs_length: plonk.zeta_to_srs_length,
                    zeta_to_domain_size: plonk.zeta_to_domain_size,
                    perm: plonk.perm,
                    lookup: (),
                },
                combined_inner_product: deferred_values.combined_inner_product,
                b: deferred_values.b,
                xi: deferred_values.xi,
                bulletproof_challenges: statement
                    .proof_state
                    .deferred_values
                    .bulletproof_challenges
                    .iter()
                    .map(|v| {
                        u64_to_field::<_, 2>(&std::array::from_fn(|i| {
                            v.prechallenge.inner[i].as_u64()
                        }))
                    })
                    .collect(),
                branch_data: deferred_values.branch_data,
            },
            sponge_digest_before_evaluations: std::array::from_fn(|i| {
                statement.proof_state.sponge_digest_before_evaluations[i].as_u64()
            }),
            messages_for_next_wrap_proof: MessagesForNextWrapProof {
                old_bulletproof_challenges: prev_challenges.clone(),
                challenge_polynomial_commitment: {
                    let (x, y) = &statement
                        .proof_state
                        .messages_for_next_wrap_proof
                        .challenge_polynomial_commitment;
                    crate::CurveAffine(x.to_field(), y.to_field())
                },
            }
            .hash(),
        },
        messages_for_next_step_proof,
    };

    let mut proof = make_prover(t);
    let oracle = {
        let public_input = prev_statement_with_hashes.to_public_input(0);
        create_oracle(dlog_vk, &proof, &public_input)
    };

    let x_hat = (oracle.p_eval_1(), oracle.p_eval_2());

    let alpha = oracle.alpha();
    let beta = oracle.beta();
    let gamma = oracle.gamma();
    let zeta = oracle.zeta();

    let to_bytes = |f: Fq| {
        let BigInteger256([a, b, c, d]): BigInteger256 = f.into();
        assert_eq!([c, d], [0, 0]);
        [a, b]
    };

    let plonk0 = PlonkMinimal {
        alpha,
        beta,
        gamma,
        zeta,
        joint_combiner: None,
        alpha_bytes: to_bytes(alpha),
        beta_bytes: to_bytes(beta),
        gamma_bytes: to_bytes(gamma),
        zeta_bytes: to_bytes(zeta),
    };

    let xi = oracle.v();
    let r = oracle.u();
    let sponge_digest_before_evaluations = oracle.digest_before_evaluations;

    let (_, endo) = endos::<Fp>();
    let to_field = |bytes: [u64; 2]| -> Fq { ScalarChallenge::from(bytes).to_field(&endo) };

    let w = dlog_vk.domain.group_gen;

    let zetaw = {
        let zeta = to_field(plonk0.zeta_bytes);
        zeta * w
    };

    let (new_bulletproof_challenges, b) = {
        let chals = oracle
            .opening_prechallenges
            .iter()
            .map(|v| to_field(to_bytes(*v)))
            .collect::<Vec<_>>();

        let r = to_field(to_bytes(r.0));
        let zeta = to_field(plonk0.zeta_bytes);
        // TODO: Pass by value here
        let challenge_poly = challenge_polynomial(&chals);
        let b = challenge_poly(zeta) + (r * challenge_poly(zetaw));

        let prechals = oracle
            .opening_prechallenges
            .iter()
            .copied()
            .map(to_bytes)
            .collect::<Vec<_>>();

        (prechals, b)
    };

    let challenge_polynomial_commitment = match must_verify {
        Boolean::False => todo!(),
        Boolean::True => proof.proof.sg.clone(),
    };

    let witness = PerProofWitness {
        app_state: (),
        proof_state: prev_statement_with_hashes.proof_state.clone(),
        prev_proof_evals: (&t.prev_evals).into(),
        prev_challenge_polynomial_commitments: {
            // Or padding
            assert_eq!(
                t.statement
                    .messages_for_next_step_proof
                    .challenge_polynomial_commitments
                    .len(),
                2
            );
            t.statement
                .messages_for_next_step_proof
                .challenge_polynomial_commitments
                .iter()
                .map(|(x, y)| make_group::<Fp>(x.to_field(), y.to_field()))
                .collect()
        },
        prev_challenges: {
            // Or padding
            assert_eq!(old_bulletproof_challenges.len(), 2);
            old_bulletproof_challenges
        },
        wrap_proof: {
            proof.proof.sg = challenge_polynomial_commitment;
            proof.clone()
        },
    };

    let tock_combined_evals = {
        let zeta = to_field(plonk0.zeta_bytes);
        evals_of_split_evals(zeta, zetaw, &proof.evals, BACKEND_TOCK_ROUNDS_N)
    };

    let tock_plonk_minimal = {
        let alpha = to_field(plonk0.alpha_bytes);
        let zeta = to_field(plonk0.zeta_bytes);

        let to_bytes = |f: Fq| {
            let BigInteger256([a, b, c, d]): BigInteger256 = f.into();
            [a, b, c, d]
        };

        PlonkMinimal {
            alpha,
            beta,
            gamma,
            zeta,
            joint_combiner: None,
            alpha_bytes: to_bytes(alpha),
            beta_bytes: to_4limbs(plonk0.beta_bytes),
            gamma_bytes: to_4limbs(plonk0.gamma_bytes),
            zeta_bytes: to_bytes(alpha),
        }
    };

    let domain_log2 = dlog_vk.domain.log_size_of_group;
    let srs_length_log2 = COMMON_MAX_DEGREE_WRAP_LOG2;
    let tock_env = make_scalars_env(
        &tock_plonk_minimal,
        domain_log2 as u8,
        srs_length_log2 as u64,
    );

    let combined_inner_product = combined_inner_product2(CombinedInnerProductParams2 {
        env: &tock_env,
        evals: &tock_combined_evals,
        public: [x_hat.0, x_hat.1],
        minimal: &tock_plonk_minimal,
        ft_eval1: proof.ft_eval1,
        r: to_field(to_bytes(r.0)),
        old_bulletproof_challenges: &prev_challenges,
        xi: to_field(to_bytes(xi.0)),
        zetaw,
    });

    let plonk = derive_plonk(&tock_env, &tock_combined_evals, &tock_plonk_minimal);

    let shift = |f: Fq| <Fq as FieldWitness>::Shifting::of_field(f);

    let unfinalized = Unfinalized {
        deferred_values: crate::proofs::unfinalized::DeferredValues {
            plonk: Plonk {
                alpha: to_bytes(plonk0.alpha),
                beta: to_bytes(plonk0.beta),
                gamma: to_bytes(plonk0.gamma),
                zeta: to_bytes(plonk0.zeta),
                zeta_to_srs_length: plonk.zeta_to_srs_length,
                zeta_to_domain_size: plonk.zeta_to_domain_size,
                perm: plonk.perm,
                lookup: (),
            },
            combined_inner_product: shift(combined_inner_product),
            b: shift(b),
            xi: to_bytes(xi.0),
            bulletproof_challenges: new_bulletproof_challenges,
        },
        should_finalize: must_verify.as_bool(),
        sponge_digest_before_evaluations: {
            let BigInteger256([a, b, c, d]): BigInteger256 =
                sponge_digest_before_evaluations.into();
            [a, b, c, d]
        },
    };

    ExpandedProof {
        sg: challenge_polynomial_commitment,
        unfinalized,
        prev_statement_with_hashes,
        x_hat,
        witness,
        actual_wrap_domain: dlog_vk.domain.log_size_of_group,
    }
}

#[derive(Debug)]
struct ExpandedProof {
    sg: GroupAffine<PallasParameters>,
    unfinalized: Unfinalized,
    prev_statement_with_hashes: PreparedStatement,
    x_hat: (Fq, Fq),
    witness: PerProofWitness<()>,
    actual_wrap_domain: u32,
}

#[derive(Clone, Debug)]
struct PerProofWitness<AppState> {
    app_state: AppState,
    wrap_proof: ProverProof<GroupAffine<PallasParameters>>,
    proof_state: ProofState,
    prev_proof_evals: AllEvals<Fp>,
    prev_challenges: Vec<[Fp; 16]>,
    prev_challenge_polynomial_commitments: Vec<GroupAffine<PallasParameters>>,
}

impl<T> PerProofWitness<T> {
    fn with_app_state<T2>(self, app_state: T2) -> PerProofWitness<T2> {
        let Self {
            app_state: _,
            wrap_proof,
            proof_state,
            prev_proof_evals,
            prev_challenges,
            prev_challenge_polynomial_commitments,
        } = self;

        PerProofWitness::<T2> {
            app_state,
            wrap_proof,
            proof_state,
            prev_proof_evals,
            prev_challenges,
            prev_challenge_polynomial_commitments,
        }
    }
}

impl ToFieldElements<Fp> for PerProofWitness<()> {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            app_state: _,
            wrap_proof,
            proof_state,
            prev_proof_evals,
            prev_challenges,
            prev_challenge_polynomial_commitments,
        } = self;

        let push_affine = |g: GroupAffine<PallasParameters>, fields: &mut Vec<Fp>| {
            let GroupAffine { x, y, .. } = g;
            x.to_field_elements(fields);
            y.to_field_elements(fields);
        };

        let push_affines = |slice: &[GroupAffine<PallasParameters>], fields: &mut Vec<Fp>| {
            slice.iter().copied().for_each(|g| push_affine(g, fields))
        };

        let ProverProof {
            commitments:
                ProverCommitments {
                    w_comm,
                    z_comm,
                    t_comm,
                    lookup: _,
                },
            proof:
                OpeningProof {
                    lr,
                    delta,
                    z1,
                    z2,
                    sg,
                },
            evals: _,
            ft_eval1: _,
            prev_challenges: _,
        } = wrap_proof;

        for w in w_comm {
            push_affines(&w.unshifted, fields);
        }

        push_affines(&z_comm.unshifted, fields);
        push_affines(&t_comm.unshifted, fields);

        for (a, b) in lr {
            push_affine(*a, fields);
            push_affine(*b, fields);
        }

        let shift = |f: Fq| <Fq as FieldWitness>::Shifting::of_field(f);

        shift(*z1).to_field_elements(fields);
        shift(*z2).to_field_elements(fields);

        push_affines(&[*delta, *sg], fields);

        let ProofState {
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
                            lookup: _,
                        },
                    combined_inner_product,
                    b,
                    xi,
                    bulletproof_challenges,
                    branch_data,
                },
            sponge_digest_before_evaluations,
            messages_for_next_wrap_proof: _,
        } = proof_state;

        u64_to_field::<Fp, 2>(alpha).to_field_elements(fields);
        u64_to_field::<Fp, 2>(beta).to_field_elements(fields);
        u64_to_field::<Fp, 2>(gamma).to_field_elements(fields);
        u64_to_field::<Fp, 2>(zeta).to_field_elements(fields);

        zeta_to_srs_length.to_field_elements(fields);
        zeta_to_domain_size.to_field_elements(fields);
        perm.to_field_elements(fields);
        combined_inner_product.to_field_elements(fields);
        b.to_field_elements(fields);
        u64_to_field::<Fp, 2>(xi).to_field_elements(fields);
        bulletproof_challenges.to_field_elements(fields);

        // Index
        {
            let v2::CompositionTypesBranchDataStableV1 {
                proofs_verified,
                domain_log2,
            } = branch_data;
            // https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/pickles_base/proofs_verified.ml#L58
            let proofs_verified = match proofs_verified {
                v2::PicklesBaseProofsVerifiedStableV1::N0 => [Fp::zero(), Fp::zero()],
                v2::PicklesBaseProofsVerifiedStableV1::N1 => [Fp::zero(), Fp::one()],
                v2::PicklesBaseProofsVerifiedStableV1::N2 => [Fp::one(), Fp::one()],
            };
            let domain_log2: u64 = domain_log2.0.as_u8() as u64;

            proofs_verified.to_field_elements(fields);
            Fp::from(domain_log2).to_field_elements(fields);
        }

        u64_to_field::<Fp, 4>(sponge_digest_before_evaluations).to_field_elements(fields);

        let AllEvals {
            ft_eval1,
            evals:
                EvalsWithPublicInput {
                    evals,
                    public_input,
                },
        } = prev_proof_evals;

        public_input.to_field_elements(fields);
        evals.to_field_elements(fields);
        ft_eval1.to_field_elements(fields);

        prev_challenges.to_field_elements(fields);
        push_affines(prev_challenge_polynomial_commitments, fields);
    }
}

impl Check<Fp> for PerProofWitness<()> {
    fn check(&self, w: &mut Witness<Fp>) {
        let Self {
            app_state: _,
            wrap_proof,
            proof_state,
            prev_proof_evals: _,
            prev_challenges: _,
            prev_challenge_polynomial_commitments,
        } = self;

        let ProverProof {
            commitments:
                ProverCommitments {
                    w_comm,
                    z_comm,
                    t_comm,
                    lookup: _,
                },
            proof:
                OpeningProof {
                    lr,
                    delta,
                    z1,
                    z2,
                    sg,
                },
            evals: _,
            ft_eval1: _,
            prev_challenges: _,
        } = wrap_proof;

        for poly in w_comm {
            (&poly.unshifted).check(w);
        }
        (&z_comm.unshifted).check(w);
        (&t_comm.unshifted).check(w);
        lr.check(w);

        let shift = |f: Fq| <Fq as FieldWitness>::Shifting::of_field(f);

        shift(*z1).check(w);
        shift(*z2).check(w);

        delta.check(w);
        sg.check(w);

        let ProofState {
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
                            lookup: _,
                        },
                    combined_inner_product,
                    b,
                    xi,
                    bulletproof_challenges,
                    branch_data,
                },
            sponge_digest_before_evaluations: _,
            messages_for_next_wrap_proof: _,
        } = proof_state;

        zeta_to_srs_length.check(w);
        zeta_to_domain_size.check(w);
        perm.check(w);
        combined_inner_product.check(w);
        b.check(w);
        u64_to_field::<Fp, 2>(xi).check(w);
        bulletproof_challenges.check(w);

        {
            let v2::CompositionTypesBranchDataStableV1 {
                proofs_verified: _,
                domain_log2,
            } = branch_data;
            let domain_log2: u64 = domain_log2.0.as_u8() as u64;

            // Assert 16 bits
            const NBITS: usize = 16;
            let (_, endo) = endos::<Fq>();
            to_field_checked::<Fp, NBITS>(Fp::from(domain_log2), endo, w);
        }

        prev_challenge_polynomial_commitments.check(w);
    }
}

pub struct StatementDeferredValues {
    pub plonk: PlonkMinimal<Fp, 2>,
    pub bulletproof_challenges: [[u64; 2]; 16],
    pub branch_data: v2::CompositionTypesBranchDataStableV1,
}

pub struct StatementProofState {
    pub deferred_values: StatementDeferredValues,
    pub sponge_digest_before_evaluations: [u64; 4],
    pub messages_for_next_wrap_proof: MessagesForNextWrapProof,
}

impl From<&v2::PicklesProofProofsVerified2ReprStableV2StatementProofState> for StatementProofState {
    fn from(value: &v2::PicklesProofProofsVerified2ReprStableV2StatementProofState) -> Self {
        let v2::PicklesProofProofsVerified2ReprStableV2StatementProofState {
            deferred_values:
                v2::PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues {
                    plonk,
                    bulletproof_challenges,
                    branch_data,
                },
            sponge_digest_before_evaluations,
            messages_for_next_wrap_proof:
                v2::PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof {
                    challenge_polynomial_commitment: (c0, c1),
                    old_bulletproof_challenges,
                },
        } = value;

        Self {
            deferred_values: StatementDeferredValues {
                plonk: plonk.into(),
                bulletproof_challenges: std::array::from_fn(|i| {
                    std::array::from_fn(|j| {
                        bulletproof_challenges[i].prechallenge.inner[j].as_u64()
                    })
                }),
                branch_data: branch_data.clone(),
            },
            sponge_digest_before_evaluations: std::array::from_fn(|i| {
                sponge_digest_before_evaluations[i].as_u64()
            }),
            messages_for_next_wrap_proof: MessagesForNextWrapProof {
                challenge_polynomial_commitment: crate::CurveAffine(c0.to_field(), c1.to_field()),
                old_bulletproof_challenges: old_bulletproof_challenges
                    .iter()
                    .map(|v| {
                        std::array::from_fn(|i| {
                            u64_to_field::<_, 2>(&std::array::from_fn(|j| {
                                v.0[i].prechallenge.inner[j].as_u64()
                            }))
                        })
                    })
                    .collect(),
            },
        }
    }
}

mod step_verifier {
    use std::ops::Neg;

    use super::*;
    use crate::proofs::{
        opt_sponge::OptSponge,
        public_input::plonk_checks::{self, ft_eval0_checked},
        unfinalized,
        util::{
            challenge_polynomial_checked, proof_evaluation_to_list_opt, to_absorption_sequence_opt,
        },
        witness::{
            field, poseidon::Sponge, scalar_challenge, ReducedMessagesForNextStepProof, ToBoolean,
        },
        wrap::{
            make_scalars_env_checked,
            pcs_batch::PcsBatch,
            pseudo::{self, PseudoDomain},
            wrap_verifier::{actual_evaluation, lowest_128_bits, split_commitments::Point, Advice},
            CircuitVar, PERMUTS_MINUS_1_ADD_N1,
        },
    };
    use itertools::Itertools;
    use mina_poseidon::constants::PlonkSpongeConstantsKimchi as Constants;
    use poly_commitment::{srs::SRS, PolyComm};

    fn squeeze_challenge(s: &mut Sponge<Fp, Constants>, w: &mut Witness<Fp>) -> Fp {
        lowest_128_bits(s.squeeze(w), true, w)
    }

    fn squeeze_scalar(s: &mut Sponge<Fp, Constants>, w: &mut Witness<Fp>) -> Fp {
        lowest_128_bits(s.squeeze(w), false, w)
    }

    fn absorb_curve(c: &Pallas, sponge: &mut Sponge<Fp, Constants>, w: &mut Witness<Fp>) {
        let GroupAffine { x, y, .. } = c;
        sponge.absorb2(&[*x, *y], w);
    }

    fn domain_for_compiled(
        domains: &[Domains],
        branch_data: &v2::CompositionTypesBranchDataStableV1,
        w: &mut Witness<Fp>,
    ) -> PseudoDomain<Fp> {
        let unique_domains = domains
            .iter()
            .map(|d| d.h)
            .sorted()
            .dedup()
            .collect::<Vec<_>>();
        let mut which_log2 = unique_domains
            .iter()
            .rev()
            .map(|Domain::Pow2RootsOfUnity(d)| {
                let d = Fp::from(*d);
                let domain_log2 = Fp::from(branch_data.domain_log2.as_u8() as u64);
                field::equal(d, domain_log2, w)
            })
            .collect::<Vec<_>>();

        which_log2.reverse();

        pseudo::to_domain::<Fp>(&which_log2, &unique_domains)
    }

    pub fn proof_verified_to_prefix(p: &v2::PicklesBaseProofsVerifiedStableV1) -> [Boolean; 2] {
        use v2::PicklesBaseProofsVerifiedStableV1::*;

        match p {
            N0 => [Boolean::False, Boolean::False],
            N1 => [Boolean::False, Boolean::True],
            N2 => [Boolean::True, Boolean::True],
        }
    }

    pub fn finalize_other_proof(
        max_proof_verified: usize,
        _feature_flags: &FeatureFlags<OptFlag>,
        step_domains: &ForStepKind<Vec<Domains>>,
        mut sponge: Sponge<Fp, Constants>,
        prev_challenges: &[[Fp; Fp::NROUNDS]],
        deferred_values: &DeferredValues<Fp>,
        evals: &AllEvals<Fp>,
        w: &mut Witness<Fp>,
    ) -> (Boolean, Vec<Fp>) {
        let DeferredValues {
            plonk,
            combined_inner_product,
            b,
            xi,
            bulletproof_challenges,
            branch_data,
        } = deferred_values;

        let AllEvals { ft_eval1, evals } = evals;

        let actual_width_mask = &branch_data.proofs_verified;
        let actual_width_mask = proof_verified_to_prefix(actual_width_mask);

        let (_, endo) = endos::<Fq>();
        let scalar = |b: &[u64; 2], w: &mut Witness<Fp>| {
            let scalar = u64_to_field(b);
            to_field_checked::<Fp, 128>(scalar, endo, w)
        };

        let plonk = {
            let Plonk {
                alpha,
                beta,
                gamma,
                zeta,
                zeta_to_srs_length,
                zeta_to_domain_size,
                perm,
                lookup: (),
            } = plonk;

            dbg!(zeta, alpha);
            // We decompose this way because of OCaml evaluation order
            let zeta = scalar(zeta, w);
            let alpha = scalar(alpha, w);

            InCircuit {
                alpha,
                beta: u64_to_field(beta),
                gamma: u64_to_field(gamma),
                zeta,
                zeta_to_domain_size: zeta_to_domain_size.clone(),
                zeta_to_srs_length: zeta_to_srs_length.clone(),
                perm: perm.clone(),
            }
        };

        let domain = match step_domains {
            ForStepKind::Known(ds) => domain_for_compiled(ds, branch_data, w),
            ForStepKind::SideLoaded => todo!(),
        };

        let zetaw = field::mul(plonk.zeta, domain.domain.group_gen, w);

        let sg_olds = prev_challenges
            .iter()
            .map(|chals| challenge_polynomial_checked(chals))
            .collect::<Vec<_>>();

        let (sg_evals1, sg_evals2) = {
            let sg_evals = |pt: Fp, w: &mut Witness<Fp>| {
                let ntrim_front = 2 - max_proof_verified;

                let mut sg = sg_olds
                    .iter()
                    .zip(&actual_width_mask[ntrim_front..])
                    .rev()
                    .map(|(f, keep)| (*keep, f(pt, w)))
                    .collect::<Vec<_>>();
                sg.reverse();
                sg
            };

            // We decompose this way because of OCaml evaluation order
            let sg_evals2 = sg_evals(zetaw, w);
            let sg_evals1 = sg_evals(plonk.zeta, w);
            (sg_evals1, sg_evals2)
        };

        let _sponge_state = {
            let challenge_digest = {
                let ntrim_front = 2 - max_proof_verified;

                let mut sponge = OptSponge::create();
                prev_challenges
                    .iter()
                    .zip(&actual_width_mask[ntrim_front..])
                    .for_each(|(chals, keep)| {
                        let keep = CircuitVar::Var(*keep);
                        for chal in chals {
                            sponge.absorb((keep, *chal));
                        }
                    });
                sponge.squeeze(w)
            };

            sponge.absorb2(&[challenge_digest], w);
            sponge.absorb(&[*ft_eval1], w);
            sponge.absorb(&[evals.public_input.0], w);
            sponge.absorb(&[evals.public_input.1], w);

            for eval in &to_absorption_sequence_opt(&evals.evals) {
                // TODO: Support sequences with Maybe
                if let Some([x1, x2]) = eval.as_ref().copied() {
                    sponge.absorb(&[x1, x2], w);
                }
            }
        };

        let xi_actual = lowest_128_bits(sponge.squeeze(w), true, w);
        let r_actual = lowest_128_bits(sponge.squeeze(w), true, w);

        let xi_correct = field::equal(xi_actual, u64_to_field(xi), w);

        let xi = scalar(xi, w);
        let r = to_field_checked::<Fp, 128>(r_actual, endo, w);

        let to_bytes = |f: Fp| {
            let BigInteger256([a, b, c, d]) = f.into();
            [a, b, c, d]
        };

        let plonk_mininal = PlonkMinimal::<Fp, 4> {
            alpha: plonk.alpha,
            beta: plonk.beta,
            gamma: plonk.gamma,
            zeta: plonk.zeta,
            joint_combiner: None,
            alpha_bytes: to_bytes(plonk.alpha),
            beta_bytes: to_bytes(plonk.beta),
            gamma_bytes: to_bytes(plonk.gamma),
            zeta_bytes: to_bytes(plonk.zeta),
        };

        let combined_evals = {
            let mut pow2pow =
                |f: Fp| (0..COMMON_MAX_DEGREE_STEP_LOG2).fold(f, |acc, _| field::square(acc, w));

            let zeta_n = pow2pow(plonk.zeta);
            let zetaw_n = pow2pow(zetaw);

            evals.evals.map_ref(&|[x0, x1]| {
                let a = actual_evaluation(&[*x0], zeta_n);
                let b = actual_evaluation(&[*x1], zetaw_n);
                [a, b]
            })
        };

        let srs_length_log2 = COMMON_MAX_DEGREE_STEP_LOG2 as u64;
        let env = make_scalars_env_checked(&plonk_mininal, &domain, srs_length_log2, w);

        let combined_inner_product_correct = {
            let p_eval0 = evals.public_input.0;
            let ft_eval0 = ft_eval0_checked(&env, &combined_evals, &plonk_mininal, p_eval0, w);
            let a = proof_evaluation_to_list_opt(&evals.evals)
                .into_iter()
                .filter_map(|v| match v {
                    Some(v) => Some(Opt::Some(v)),
                    None => None,
                })
                .collect::<Vec<_>>();

            let actual_combined_inner_product = {
                enum WhichEval {
                    First,
                    Second,
                }

                let combine = |which_eval: WhichEval,
                               sg_evals: &[(Boolean, Fp)],
                               ft_eval: Fp,
                               x_hat: Fp,
                               w: &mut Witness<Fp>| {
                    let f = |v: &Opt<[Fp; 2]>| match which_eval {
                        WhichEval::First => v.map(|v| v[0]),
                        WhichEval::Second => v.map(|v| v[1]),
                    };
                    let v = sg_evals
                        .iter()
                        .copied()
                        .map(|(b, v)| Opt::Maybe(b, v))
                        .chain([Opt::Some(x_hat)])
                        .chain([Opt::Some(ft_eval)])
                        .chain(a.iter().map(f))
                        .rev()
                        .collect::<Vec<_>>();

                    let (init, rest) = v.split_at(1);

                    let init = match init[0] {
                        Opt::Some(x) => x,
                        Opt::No => Fp::zero(),
                        Opt::Maybe(b, x) => field::mul(b.to_field(), x, w),
                    };
                    rest.iter().fold(init, |acc: Fp, fx: &Opt<Fp>| match fx {
                        Opt::No => acc,
                        Opt::Some(fx) => *fx + field::mul(xi, acc, w),
                        Opt::Maybe(b, fx) => {
                            let v = match b {
                                Boolean::True => *fx + field::mul(xi, acc, w),
                                Boolean::False => acc,
                            };
                            w.exists_no_check(v)
                        }
                    })
                };

                // We decompose this way because of OCaml evaluation order
                let b = combine(
                    WhichEval::Second,
                    &sg_evals2,
                    *ft_eval1,
                    evals.public_input.1,
                    w,
                );
                let b = field::mul(b, r, w);
                let a = combine(
                    WhichEval::First,
                    &sg_evals1,
                    ft_eval0,
                    evals.public_input.0,
                    w,
                );
                a + b
            };

            let combined_inner_product =
                ShiftingValue::<Fp>::shifted_to_field(combined_inner_product);
            field::equal(combined_inner_product, actual_combined_inner_product, w)
        };

        let mut bulletproof_challenges = bulletproof_challenges
            .iter()
            .rev()
            .map(|f| to_field_checked::<Fp, 128>(*f, endo, w))
            .collect::<Vec<_>>();
        bulletproof_challenges.reverse();

        let b_correct = {
            let challenge_poly = challenge_polynomial_checked(&bulletproof_challenges);

            // We decompose this way because of OCaml evaluation order
            let r_zetaw = field::mul(r, challenge_poly(zetaw, w), w);
            let b_actual = challenge_poly(plonk.zeta, w) + r_zetaw;

            field::equal(b.shifted_to_field(), b_actual, w)
        };

        let plonk = wrap_verifier::PlonkWithField {
            alpha: plonk.alpha,
            beta: plonk.beta,
            gamma: plonk.gamma,
            zeta: plonk.zeta,
            zeta_to_srs_length: plonk.zeta_to_srs_length,
            zeta_to_domain_size: plonk.zeta_to_domain_size,
            perm: plonk.perm,
            lookup: (),
        };
        let plonk_checks_passed = plonk_checks::checked(&env, &combined_evals, &plonk, w);

        let finalized = Boolean::all(
            &[
                xi_correct,
                b_correct,
                combined_inner_product_correct,
                plonk_checks_passed,
            ],
            w,
        );

        (finalized, bulletproof_challenges)
    }

    pub fn sponge_after_index(
        index: &PlonkVerificationKeyEvals<Fp>,
        w: &mut Witness<Fp>,
    ) -> Sponge<Fp, Constants> {
        use mina_poseidon::pasta::fp_kimchi::static_params;

        let mut sponge = Sponge::<Fp, Constants>::new(static_params());
        let fields = index.to_field_elements_owned();
        sponge.absorb2(&fields, w);
        sponge
    }

    pub fn hash_messages_for_next_step_proof_opt(
        msg: ReducedMessagesForNextStepProof<&Statement<SokDigest>>,
        sponge: Sponge<Fp, Constants>,
        _widths: &ForStepKind<Vec<Fp>>,
        _max_width: usize,
        proofs_verified_mask: &[Boolean],
        w: &mut Witness<Fp>,
    ) -> Fp {
        enum MaybeOpt<T, T2> {
            Opt(Boolean, T),
            NotOpt(T2),
        }

        // TODO: Refactor/clean this, it's a mess

        let ReducedMessagesForNextStepProof {
            app_state,
            challenge_polynomial_commitments,
            old_bulletproof_challenges,
        } = msg;

        let old_bulletproof_challenges = proofs_verified_mask
            .iter()
            .zip(old_bulletproof_challenges)
            .map(|(b, v)| {
                let b = *b;
                v.map(|v| MaybeOpt::Opt(b, v))
            });

        let challenge_polynomial_commitments = proofs_verified_mask
            .iter()
            .zip(challenge_polynomial_commitments)
            .map(|(b, v)| {
                let b = *b;
                let CurveAffine(x, y) = v;
                [MaybeOpt::Opt(b, x), MaybeOpt::Opt(b, y)]
            });

        let app_state = app_state
            .to_field_elements_owned()
            .into_iter()
            .map(|v| MaybeOpt::NotOpt(v));

        let both = challenge_polynomial_commitments
            .zip(old_bulletproof_challenges)
            .map(|(c, o)| c.into_iter().chain(o.into_iter()));

        let res = app_state
            .chain(both.flatten())
            .fold(MaybeOpt::NotOpt(sponge), |acc, v| match (acc, v) {
                (MaybeOpt::NotOpt(mut sponge), MaybeOpt::NotOpt(v)) => {
                    sponge.absorb(&[v], w);
                    MaybeOpt::NotOpt(sponge)
                }
                (MaybeOpt::NotOpt(sponge), MaybeOpt::Opt(b, v)) => {
                    let mut sponge = OptSponge::of_sponge(sponge, w);
                    sponge.absorb((CircuitVar::Var(b), v));
                    MaybeOpt::Opt(Boolean::True, sponge)
                }
                (MaybeOpt::Opt(_, mut sponge), MaybeOpt::Opt(b, v)) => {
                    sponge.absorb((CircuitVar::Var(b), v));
                    MaybeOpt::Opt(Boolean::True, sponge)
                }
                (MaybeOpt::Opt(_, _), MaybeOpt::NotOpt(_)) => panic!(),
            });

        match res {
            MaybeOpt::NotOpt(mut sponge) => sponge.squeeze(w),
            MaybeOpt::Opt(_, mut sponge) => sponge.squeeze(w),
        }
    }

    // TODO: Dedup with the one in `wrap_verifier`
    fn scale_fast2<F: FieldWitness>(
        g: CircuitVar<GroupAffine<F::Parameters>>,
        (s_div_2, s_odd): (F, Boolean),
        num_bits: usize,
        w: &mut Witness<F>,
    ) -> GroupAffine<F::Parameters> {
        use crate::proofs::witness::plonk_curve_ops::scale_fast_unpack;
        use wrap_verifier::*;

        let s_div_2_bits = num_bits - 1;
        let chunks_needed = chunks_needed(s_div_2_bits);
        let actual_bits_used = chunks_needed * OPS_BITS_PER_CHUNK;

        let g_value = g.value().clone();
        let shifted = F::Shifting::of_raw(s_div_2);
        let h = match actual_bits_used {
            255 => scale_fast_unpack::<F, F, 255>(g_value, shifted, w).0,
            130 => scale_fast_unpack::<F, F, 130>(g_value, shifted, w).0,
            10 => scale_fast_unpack::<F, F, 10>(g_value, shifted, w).0,
            n => todo!("{:?} param_num_bits={:?}", n, num_bits),
        };

        let on_false = {
            let g_neg = g.value().neg();
            if let CircuitVar::Var(_) = g {
                w.exists(g_neg.y);
            };
            w.add_fast(h, g_neg)
        };

        w.exists_no_check(match s_odd {
            Boolean::True => h,
            Boolean::False => on_false,
        })
    }

    // TODO: Dedup with the one above and in `wrap_verifier`
    fn scale_fast22<F: FieldWitness>(
        g: CircuitVar<GroupAffine<F::Parameters>>,
        (s_div_2, s_odd): (F, Boolean),
        num_bits: usize,
        w: &mut Witness<F>,
    ) -> GroupAffine<F::Parameters> {
        use crate::proofs::witness::plonk_curve_ops::scale_fast_unpack;
        use wrap_verifier::*;

        let s_div_2_bits = num_bits - 1;
        let chunks_needed = chunks_needed(s_div_2_bits);
        let actual_bits_used = chunks_needed * OPS_BITS_PER_CHUNK;

        let g_value = g.value().clone();
        let shifted = F::Shifting::of_raw(s_div_2);
        let h = match actual_bits_used {
            255 => scale_fast_unpack::<F, F, 255>(g_value, shifted, w).0,
            130 => scale_fast_unpack::<F, F, 130>(g_value, shifted, w).0,
            10 => scale_fast_unpack::<F, F, 10>(g_value, shifted, w).0,
            n => todo!("{:?} param_num_bits={:?}", n, num_bits),
        };

        let on_false = {
            let g_neg = w.exists_no_check(g.value().neg());
            w.add_fast(h, g_neg)
        };

        w.exists_no_check(match s_odd {
            Boolean::True => h,
            Boolean::False => on_false,
        })
    }

    // TODO: Dedup with the one in `wrap_verifier`
    pub fn scale_fast2_prime<F: FieldWitness, F2: FieldWitness>(
        g: CircuitVar<GroupAffine<F::Parameters>>,
        s: F2,
        num_bits: usize,
        w: &mut Witness<F>,
    ) -> GroupAffine<F::Parameters> {
        let s_parts = w.exists({
            // TODO: Here `s` is a `F` but needs to be read as a `F::Scalar`
            let bigint: BigInteger256 = s.into();
            let s_odd = bigint.0[0] & 1 != 0;
            let v = if s_odd { s - F2::one() } else { s };
            // TODO: Remove this ugly hack
            let v: BigInteger256 = (v / F2::from(2u64)).into();
            (F::from(v), s_odd.to_boolean())
        });

        scale_fast2(g, s_parts, num_bits, w)
    }

    // TODO: Dedup with the one in `wrap_verifier`
    pub fn ft_comm<F: FieldWitness, Scale>(
        plonk: &Plonk<F::Scalar>,
        t_comm: &PolyComm<GroupAffine<F::Parameters>>,
        verification_key: &PlonkVerificationKeyEvals<F>,
        scale: Scale,
        w: &mut Witness<F>,
    ) -> GroupAffine<F::Parameters>
    where
        Scale: Fn(
            GroupAffine<F::Parameters>,
            <F::Scalar as FieldWitness>::Shifting,
            &mut Witness<F>,
        ) -> GroupAffine<F::Parameters>,
    {
        let m = verification_key;
        let [sigma_comm_last] = &m.sigma[PERMUTS_MINUS_1_ADD_N1..] else {
            panic!()
        };

        // We decompose this way because of OCaml evaluation order (reversed)
        let f_comm = [scale(sigma_comm_last.to_affine(), plonk.perm.clone(), w)]
            .into_iter()
            .rev()
            .reduce(|acc, v| w.add_fast(acc, v))
            .unwrap();

        let chunked_t_comm = t_comm
            .unshifted
            .iter()
            .rev()
            .copied()
            .reduce(|acc, v| {
                let scaled = scale(acc, plonk.zeta_to_srs_length.clone(), w);
                w.add_fast(v, scaled)
            })
            .unwrap();

        // We decompose this way because of OCaml evaluation order
        let scaled = scale(chunked_t_comm, plonk.zeta_to_domain_size.clone(), w).neg();
        let v = w.add_fast(f_comm, chunked_t_comm);

        // Because of `neg()` above
        w.exists_no_check(scaled.y);

        w.add_fast(v, scaled)
    }

    fn multiscale_known(
        ts: &[(&Packed, InnerCurve<Fp>)],
        w: &mut Witness<Fp>,
    ) -> GroupAffine<PallasParameters> {
        let pow2pow = |x: InnerCurve<Fp>, n: usize| (0..n).fold(x, |acc, _| acc.clone() + acc);

        let (constant_part, non_constant_part): (Vec<_>, Vec<_>) =
            ts.into_iter().partition_map(|(t, g)| {
                use itertools::Either::{Left, Right};
                use CircuitVar::Constant;
                use Packed::{Field, PackedBits};

                match t {
                    Field(Constant(c)) | PackedBits(Constant(c), _) => Left(if c.is_zero() {
                        None
                    } else if c.is_one() {
                        Some(g)
                    } else {
                        todo!()
                    }),
                    Field(x) => Right((Field(*x), g)),
                    PackedBits(x, n) => Right((PackedBits(*x, *n), g)),
                }
            });

        let add_opt = |xo: Option<InnerCurve<Fp>>, y: &InnerCurve<Fp>| match xo {
            Some(x) => x + y.clone(),
            None => y.clone(),
        };

        let constant_part = constant_part
            .iter()
            .filter_map(|c| c.as_ref())
            .fold(None, |acc, x| Some(add_opt(acc, x)));

        let (correction, acc) = {
            non_constant_part
                .iter()
                .map(|(s, x)| {
                    let (rr, n) = match s {
                        Packed::PackedBits(s, n) => {
                            let x = CircuitVar::Constant(x.to_affine());
                            let c = scale_fast2_prime(x, s.as_field(), *n, w);
                            (c, *n)
                        }
                        Packed::Field(s) => {
                            let x = CircuitVar::Constant(x.to_affine());
                            let c = scale_fast2_prime(x, s.as_field(), 255, w);
                            (c, 255)
                        }
                    };

                    let n = wrap_verifier::OPS_BITS_PER_CHUNK * wrap_verifier::chunks_needed(n - 1);
                    let cc = pow2pow((*x).clone(), n);
                    (cc, rr)
                })
                .collect::<Vec<_>>() // We need to collect because `w` is borrowed :(
                .into_iter()
                .reduce(|(a1, b1), (a2, b2)| ((a1 + a2), w.add_fast(b1, b2)))
                .unwrap()
        };

        let correction = InnerCurve::of_affine(correction.to_affine().neg());
        w.add_fast(acc, add_opt(constant_part, &correction).to_affine())
    }

    pub fn lagrange_commitment<F: FieldWitness>(
        srs: &mut SRS<GroupAffine<F::Parameters>>,
        domain: &Domain,
        i: usize,
    ) -> InnerCurve<F> {
        let d = domain.size();
        let unshifted = wrap_verifier::lagrange_commitment::<F>(srs, d, i).unshifted;

        assert_eq!(unshifted.len(), 1);
        InnerCurve::of_affine(unshifted[0])
    }

    fn to_high_low(f: Fq) -> (Fp, Boolean) {
        fn of_bits<F: FieldWitness>(bs: &[bool; 254]) -> F {
            bs.iter().rev().fold(F::zero(), |acc, b| {
                let acc = acc + acc;
                if *b {
                    acc + F::one()
                } else {
                    acc
                }
            })
        }
        let to_high_low = |fq: Fq| {
            let [low, high @ ..] = crate::proofs::witness::field_to_bits::<Fq, 255>(fq);
            (of_bits::<Fp>(&high), low.to_boolean())
        };
        to_high_low(f)
    }

    fn scale_for_ft_comm(
        g: GroupAffine<PallasParameters>,
        f: <Fq as FieldWitness>::Shifting,
        w: &mut Witness<Fp>,
    ) -> GroupAffine<PallasParameters> {
        let scalar = to_high_low(f.shifted_raw());
        scale_fast2(CircuitVar::Var(g), scalar, 255, w)
    }

    struct CheckBulletProofParams<'a> {
        pcs_batch: PcsBatch,
        sponge: Sponge<Fp, mina_poseidon::constants::PlonkSpongeConstantsKimchi>,
        xi: [u64; 2],
        advice: &'a Advice<Fp>,
        openings_proof: &'a OpeningProof<GroupAffine<PallasParameters>>,
        srs: &'a SRS<GroupAffine<PallasParameters>>,
        polynomials: (Vec<GroupAffine<PallasParameters>>, Vec<()>),
    }

    fn check_bulletproof(
        params: CheckBulletProofParams,
        w: &mut Witness<Fp>,
    ) -> (Boolean, Vec<Fp>) {
        let CheckBulletProofParams {
            pcs_batch: _,
            mut sponge,
            xi,
            advice,
            openings_proof:
                OpeningProof {
                    lr,
                    delta,
                    z1,
                    z2,
                    sg,
                },
            srs,
            polynomials,
        } = params;

        let (without_degree_bound, _with_degree_bound) = polynomials;

        let (high, low) = to_high_low(advice.combined_inner_product.shifted_raw());
        sponge.absorb2(&[high, low.to_field()], w);

        let u = {
            let t = sponge.squeeze(w);
            wrap_verifier::group_map(t, w)
        };

        let xi_field: Fq = u64_to_field(&xi);

        let point: Point<Fp> = PcsBatch::combine_split_commitments::<Fp, _, _, _, _>(
            |p, _w| p.clone(),
            |acc, _xi, p, w| match acc {
                Point::MaybeFinite(acc_is_finite, acc) => match p {
                    Point::MaybeFinite(p_is_finite, p) => {
                        let is_finite = p_is_finite.or(&acc_is_finite, w);
                        let xi_acc = scalar_challenge::endo::<Fp, _, 128>(acc, xi_field, w);

                        let p = *p;
                        let on_true = w.add_fast(p, xi_acc);

                        let v = match acc_is_finite.as_boolean() {
                            Boolean::True => on_true,
                            Boolean::False => p,
                        };
                        if let CircuitVar::Var(_) = acc_is_finite {
                            w.exists_no_check(&v);
                        };
                        Point::<Fp>::MaybeFinite(is_finite, v)
                    }
                    Point::Finite(p) => {
                        let xi_acc = scalar_challenge::endo::<Fp, _, 128>(acc, xi_field, w);
                        let p = *p;
                        let on_true = w.add_fast(p, xi_acc);

                        let v = match acc_is_finite.as_boolean() {
                            Boolean::True => on_true,
                            Boolean::False => p,
                        };
                        if let CircuitVar::Var(_) = acc_is_finite {
                            w.exists_no_check(&v);
                        };
                        Point::<Fp>::Finite(v)
                    }
                },
                Point::Finite(acc) => {
                    let xi_acc = scalar_challenge::endo::<Fp, _, 128>(acc, xi_field, w);

                    let v = match p {
                        Point::Finite(p) => w.add_fast(*p, xi_acc),
                        Point::MaybeFinite(p_is_finite, p) => {
                            let p = *p;
                            let on_true = w.add_fast(p, xi_acc);

                            let v = match p_is_finite.as_boolean() {
                                Boolean::True => on_true,
                                Boolean::False => xi_acc,
                            };
                            if let CircuitVar::Var(_) = p_is_finite {
                                w.exists_no_check(&v);
                            };
                            v
                        }
                    };

                    Point::Finite(v)
                }
            },
            xi,
            &without_degree_bound
                .into_iter()
                .map(|v| Point::Finite(v))
                .collect::<Vec<_>>(),
            &[],
            w,
        );

        let Point::Finite(combined_polynomial) = point else {
            panic!("invalid state");
        };

        let (lr_prod, challenges) = wrap_verifier::bullet_reduce(&mut sponge, lr, w);

        let p_prime = {
            w.exists_no_check(&u); // Made by `plonk_curve_ops.seal` in `scale_fast`
            let combined_inner_product = to_high_low(advice.combined_inner_product.shifted_raw());
            let uc = scale_fast22(CircuitVar::Var(u), combined_inner_product, 255, w);
            w.add_fast(combined_polynomial, uc)
        };

        let q = w.add_fast(p_prime, lr_prod);
        absorb_curve(delta, &mut sponge, w);
        let c = squeeze_scalar(&mut sponge, w);

        let lhs = {
            let cq = scalar_challenge::endo::<Fp, Fp, 128>(q, c, w);
            w.add_fast(cq, *delta)
        };

        let rhs = {
            let b_u = {
                w.exists_no_check(&u); // Made by `plonk_curve_ops.seal` in `scale_fast`
                let b = to_high_low(advice.b.shifted_raw());
                scale_fast22(CircuitVar::Var(u), b, 255, w)
            };

            let z_1_g_plus_b_u = {
                use plonk_checks::ShiftedValue;

                let z1 = to_high_low(ShiftedValue::<Fq>::of_field(*z1).shifted);
                scale_fast2(CircuitVar::Var(w.add_fast(*sg, b_u)), z1, 255, w)
            };

            let z2_h = {
                use plonk_checks::ShiftedValue;

                let z2 = to_high_low(ShiftedValue::<Fq>::of_field(*z2).shifted);
                scale_fast2(CircuitVar::Constant(srs.h), z2, 255, w)
            };

            w.add_fast(z_1_g_plus_b_u, z2_h)
        };

        (wrap_verifier::equal_g(lhs, rhs, w), challenges)
    }

    struct IncrementallyVerifyProofParams<'a> {
        pub proofs_verified: usize,
        pub srs: &'a mut poly_commitment::srs::SRS<Pallas>,
        pub wrap_domain: &'a ForStepKind<Domain>,
        pub sponge: Sponge<Fp, Constants>,
        pub sponge_after_index: Sponge<Fp, Constants>,
        pub wrap_verification_key: &'a PlonkVerificationKeyEvals<Fp>,
        pub xi: [u64; 2],
        pub public_input: Vec<Packed>,
        pub sg_old: &'a Vec<GroupAffine<PallasParameters>>,
        pub advice: &'a Advice<Fp>,
        pub proof: &'a ProverProof<GroupAffine<PallasParameters>>,
        pub plonk: &'a Plonk<Fq>,
    }

    fn incrementally_verify_proof(
        params: IncrementallyVerifyProofParams,
        w: &mut Witness<Fp>,
    ) -> (Fp, (Boolean, Vec<Fp>)) {
        let IncrementallyVerifyProofParams {
            proofs_verified: _,
            srs,
            wrap_domain,
            mut sponge,
            sponge_after_index,
            wrap_verification_key,
            xi,
            public_input,
            sg_old,
            advice,
            proof,
            plonk,
        } = params;

        let ProverProof {
            commitments: messages,
            proof: openings_proof,
            ..
            // evals,
            // ft_eval1,
            // prev_challenges
        } = proof;

        let sample = squeeze_challenge;
        let sample_scalar = squeeze_scalar;

        let index_digest = {
            let mut index_sponge = sponge_after_index.clone();
            index_sponge.squeeze(w)
        };

        sponge.absorb2(&[index_digest], w);

        // Or padding
        assert_eq!(sg_old.len(), 2);
        for v in sg_old {
            absorb_curve(v, &mut sponge, w);
        }

        let x_hat = match wrap_domain {
            ForStepKind::Known(domain) => {
                let ts = public_input
                    .iter()
                    .enumerate()
                    .map(|(i, x)| (x, lagrange_commitment::<Fp>(srs, domain, i)))
                    .collect::<Vec<_>>();
                multiscale_known(&ts, w).neg()
            }
            ForStepKind::SideLoaded => todo!(),
        };

        let x_hat = {
            w.exists(x_hat.y); // Because of `.neg()` above
            w.add_fast(x_hat, srs.h)
        };

        absorb_curve(&x_hat, &mut sponge, w);

        let w_comm = &messages.w_comm;
        for g in w_comm.iter().flat_map(|w| &w.unshifted) {
            absorb_curve(g, &mut sponge, w);
        }

        let _beta = sample(&mut sponge, w);
        let _gamma = sample(&mut sponge, w);

        let z_comm = &messages.z_comm;
        for z in z_comm.unshifted.iter() {
            absorb_curve(z, &mut sponge, w);
        }

        let _alpha = sample_scalar(&mut sponge, w);

        let t_comm = &messages.t_comm;
        for t in t_comm.unshifted.iter() {
            absorb_curve(t, &mut sponge, w);
        }

        let _zeta = sample_scalar(&mut sponge, w);

        let sponge_before_evaluations = sponge.clone();
        let sponge_digest_before_evaluations = sponge.squeeze(w);

        let sigma_comm_init = &wrap_verification_key.sigma[..PERMUTS_MINUS_1_ADD_N1];

        eprintln!("AAAAA");

        let ft_comm = ft_comm(plonk, t_comm, wrap_verification_key, scale_for_ft_comm, w);

        let bulletproof_challenges = {
            /// Wrap_hack.Padded_length
            const WRAP_HACK_PADDED_LENGTH: usize = 2;
            const NUM_COMMITMENTS_WITHOUT_DEGREE_BOUND: usize = 45;

            let without_degree_bound = {
                let sg_old = sg_old.iter().copied();
                let rest = [x_hat, ft_comm]
                    .into_iter()
                    .chain(z_comm.unshifted.iter().cloned())
                    .chain([
                        wrap_verification_key.generic.to_affine(),
                        wrap_verification_key.psm.to_affine(),
                        wrap_verification_key.complete_add.to_affine(),
                        wrap_verification_key.mul.to_affine(),
                        wrap_verification_key.emul.to_affine(),
                        wrap_verification_key.endomul_scalar.to_affine(),
                    ])
                    .chain(w_comm.iter().flat_map(|w| w.unshifted.iter().cloned()))
                    .chain(
                        wrap_verification_key
                            .coefficients
                            .iter()
                            .map(|v| v.to_affine()),
                    )
                    .chain(sigma_comm_init.iter().map(|v| v.to_affine()));
                sg_old.chain(rest).collect::<Vec<_>>()
            };

            check_bulletproof(
                CheckBulletProofParams {
                    pcs_batch: PcsBatch::create(
                        WRAP_HACK_PADDED_LENGTH + NUM_COMMITMENTS_WITHOUT_DEGREE_BOUND,
                    ),
                    sponge: sponge_before_evaluations,
                    xi,
                    advice,
                    openings_proof,
                    srs,
                    polynomials: (without_degree_bound, vec![]),
                },
                w,
            )
        };

        (sponge_digest_before_evaluations, bulletproof_challenges)
    }

    pub struct VerifyParams<'a> {
        pub srs: &'a mut poly_commitment::srs::SRS<Pallas>,
        pub feature_flags: &'a FeatureFlags<OptFlag>,
        pub lookup_parameters: (),
        pub proofs_verified: usize,
        pub wrap_domain: &'a ForStepKind<Domain>,
        pub is_base_case: Boolean,
        pub sponge_after_index: Sponge<Fp, Constants>,
        pub sg_old: &'a Vec<GroupAffine<PallasParameters>>,
        pub proof: &'a ProverProof<GroupAffine<PallasParameters>>,
        pub wrap_verification_key: &'a PlonkVerificationKeyEvals<Fp>,
        pub statement: &'a PreparedStatement,
        pub unfinalized: &'a Unfinalized,
    }

    pub fn verify(params: VerifyParams, w: &mut Witness<Fp>) -> Boolean {
        let VerifyParams {
            srs,
            feature_flags: _,
            lookup_parameters: _,
            proofs_verified,
            wrap_domain,
            is_base_case: _,
            sponge_after_index,
            sg_old,
            proof,
            wrap_verification_key,
            statement,
            unfinalized,
        } = params;

        let public_input = {
            let mut public_input = statement.to_public_input_cvar(39);
            // TODO: See how padding works
            public_input.push(Packed::PackedBits(CircuitVar::Constant(Fq::zero()), 128));
            public_input
        };

        let unfinalized::DeferredValues {
            plonk,
            combined_inner_product,
            b,
            xi,
            bulletproof_challenges: _,
        } = &unfinalized.deferred_values;

        let b = b.clone();
        let combined_inner_product = combined_inner_product.clone();
        let sponge = Sponge::create();

        let (
            _sponge_digest_before_evaluations_actual,
            (bulletproof_success, _bulletproof_challenges_actual),
        ) = incrementally_verify_proof(
            IncrementallyVerifyProofParams {
                proofs_verified,
                srs,
                wrap_domain,
                sponge,
                sponge_after_index,
                wrap_verification_key,
                xi: *xi,
                public_input,
                sg_old,
                advice: &Advice {
                    b,
                    combined_inner_product,
                },
                proof,
                plonk,
            },
            w,
        );

        bulletproof_success
    }
}

fn verify_one(
    srs: &mut poly_commitment::srs::SRS<Pallas>,
    proof: &PerProofWitness<Statement<SokDigest>>,
    data: &ForStep,
    messages_for_next_wrap_proof: Fp,
    unfinalized: &Unfinalized,
    should_verify: Boolean,
    w: &mut Witness<Fp>,
) -> (Vec<Fp>, Boolean) {
    let PerProofWitness {
        app_state,
        wrap_proof,
        proof_state,
        prev_proof_evals,
        prev_challenges,
        prev_challenge_polynomial_commitments,
    } = proof;

    let deferred_values = &proof_state.deferred_values;

    let (finalized, chals) = {
        let sponge_digest = proof_state.sponge_digest_before_evaluations;

        let sponge = {
            use mina_poseidon::pasta::fp_kimchi::static_params;

            let mut sponge =
                crate::proofs::witness::poseidon::Sponge::<Fp, Constants>::new(static_params());
            sponge.absorb2(&[u64_to_field(&sponge_digest)], w);
            sponge
        };

        step_verifier::finalize_other_proof(
            data.max_proofs_verified,
            &data.feature_flags,
            &data.step_domains,
            sponge,
            prev_challenges,
            deferred_values,
            prev_proof_evals,
            w,
        )
    };

    let branch_data = &deferred_values.branch_data;

    let sponge_after_index = step_verifier::sponge_after_index(&data.wrap_key, w);

    let statement = {
        let msg = ReducedMessagesForNextStepProof {
            app_state,
            challenge_polynomial_commitments: prev_challenge_polynomial_commitments
                .iter()
                .map(|g| {
                    let GroupAffine { x, y, .. } = g;
                    CurveAffine(*x, *y)
                })
                .collect(),
            old_bulletproof_challenges: prev_challenges.clone(),
        };

        let ntrim_front = 2 - prev_challenge_polynomial_commitments.len();
        let proofs_verified_mask = {
            let proofs_verified_mask = &branch_data.proofs_verified;
            proof_verified_to_prefix(proofs_verified_mask)
        };
        let proofs_verified_mask = &proofs_verified_mask[ntrim_front..];

        let prev_messages_for_next_step_proof =
            step_verifier::hash_messages_for_next_step_proof_opt(
                msg,
                sponge_after_index.clone(),
                &data.proof_verifieds,
                data.max_proofs_verified,
                proofs_verified_mask,
                w,
            );

        PreparedStatement {
            proof_state: ProofState {
                messages_for_next_wrap_proof: { to_bytes(messages_for_next_wrap_proof) },
                ..proof_state.clone()
            },
            messages_for_next_step_proof: { to_bytes(prev_messages_for_next_step_proof) },
        }
    };

    let verified = step_verifier::verify(
        VerifyParams {
            srs,
            feature_flags: &data.feature_flags,
            lookup_parameters: (),
            proofs_verified: data.max_proofs_verified,
            wrap_domain: &data.wrap_domain,
            is_base_case: should_verify.neg(),
            sponge_after_index,
            sg_old: prev_challenge_polynomial_commitments,
            proof: wrap_proof,
            wrap_verification_key: &data.wrap_key,
            statement: &statement,
            unfinalized,
        },
        w,
    );

    let a = verified.and(&finalized, w);
    let b = a.const_or(&should_verify);

    (chals, b)
}

pub enum Packed {
    Field(CircuitVar<Fq>),
    PackedBits(CircuitVar<Fq>, usize),
}

impl std::fmt::Debug for Packed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field(x) => f.write_fmt(format_args!("Field({:?})", x)),
            Self::PackedBits(a, b) => f.write_fmt(format_args!("PackedBits({:?}, {:?})", a, b)),
        }
    }
}

pub fn generate_merge_proof(
    statement: &v2::MinaStateBlockchainStateValueStableV2LedgerProofStatement,
    proofs: &(v2::LedgerProofProdStableV2, v2::LedgerProofProdStableV2),
    message: &SokMessage,
    _step_prover: &Prover<Fp>,
    wrap_prover: &Prover<Fq>,
    w: &mut Witness<Fp>,
) {
    w.ocaml_aux = read_witnesses().unwrap();

    let statement: Statement<()> = statement.into();
    let sok_digest = message.digest();
    let statement_with_sok = statement.with_digest(sok_digest);

    w.exists(&statement_with_sok);

    let (s1, s2) = merge_main(statement_with_sok, proofs, w);
    let (p1, p2) = proofs;

    let rule = InductiveRule {
        previous_proof_statements: [
            PreviousProofStatement {
                public_input: &s1,
                proof: p1,
                proof_must_verify: Boolean::True,
            },
            PreviousProofStatement {
                public_input: &s2,
                proof: p2,
                proof_must_verify: Boolean::True,
            },
        ],
        public_output: (),
        auxiliary_output: (),
    };

    let dlog_plonk_index = w.exists(dlog_plonk_index(wrap_prover));
    let verifier_index = wrap_prover.index.verifier_index.as_ref().unwrap();

    let expanded_proofs: [ExpandedProof; 2] = rule
        .previous_proof_statements
        .iter()
        .map(|statement| {
            let PreviousProofStatement {
                public_input,
                proof,
                proof_must_verify,
            } = statement;
            expand_proof(
                verifier_index,
                &dlog_plonk_index,
                public_input,
                proof,
                (),
                *proof_must_verify,
            )
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    eprintln!("AAA");

    let fst = &expanded_proofs[0];
    let snd = &expanded_proofs[1];

    let prevs = w.exists([&fst.witness, &snd.witness]);
    let unfinalized_proofs_unextended = w.exists([&fst.unfinalized, &snd.unfinalized]);

    // panic!("unfinalized={:#?}", unfinalized_proofs_unextended);

    let f = u64_to_field::<Fp, 4>;
    let messages_for_next_wrap_proof = w.exists([
        f(&fst
            .prev_statement_with_hashes
            .proof_state
            .messages_for_next_wrap_proof),
        f(&snd
            .prev_statement_with_hashes
            .proof_state
            .messages_for_next_wrap_proof),
    ]);

    let actual_wrap_domains = {
        let all_possible_domains = wrap_verifier::all_possible_domains();

        [fst.actual_wrap_domain, snd.actual_wrap_domain].map(|domain_size| {
            let domain_size = domain_size as u64;
            all_possible_domains
                .iter()
                .position(|Domain::Pow2RootsOfUnity(d)| *d == domain_size)
                .unwrap_or(0)
        })
    };

    dbg!(fst.actual_wrap_domain);
    dbg!(snd.actual_wrap_domain);

    let prevs: [PerProofWitness<_>; 2] = rule
        .previous_proof_statements
        .iter()
        .zip(prevs)
        .map(|(stmt, proof)| proof.clone().with_app_state(stmt.public_input.clone()))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let basic = Basic {
        proof_verifieds: vec![0, 2, 0, 0, 1],
        wrap_domain: Domains {
            h: Domain::Pow2RootsOfUnity(14),
        },
        step_domains: vec![
            Domains {
                h: Domain::Pow2RootsOfUnity(15),
            },
            Domains {
                h: Domain::Pow2RootsOfUnity(15),
            },
            Domains {
                h: Domain::Pow2RootsOfUnity(15),
            },
            Domains {
                h: Domain::Pow2RootsOfUnity(14),
            },
            Domains {
                h: Domain::Pow2RootsOfUnity(15),
            },
        ],
        feature_flags: FeatureFlags {
            range_check0: OptFlag::No,
            range_check1: OptFlag::No,
            foreign_field_add: OptFlag::No,
            foreign_field_mul: OptFlag::No,
            xor: OptFlag::No,
            rot: OptFlag::No,
            lookup: OptFlag::No,
            runtime_tables: OptFlag::No,
        },
    };

    let self_branches = 5;
    let max_proofs_verified = 2;
    let self_data = ForStep {
        branches: self_branches,
        max_proofs_verified,
        proof_verifieds: ForStepKind::Known(
            basic
                .proof_verifieds
                .iter()
                .copied()
                .map(Fp::from)
                .collect(),
        ),
        public_input: (),
        wrap_key: dlog_plonk_index,
        wrap_domain: ForStepKind::Known(basic.wrap_domain.h),
        step_domains: ForStepKind::Known(basic.step_domains),
        feature_flags: basic.feature_flags,
    };

    let srs = <Fq as FieldWitness>::get_srs();
    let mut srs = srs.lock().unwrap();

    for ((((proof, msg_for_next_wrap_proof), unfinalized), stmt), actual_wrap_domain) in prevs
        .iter()
        .zip(messages_for_next_wrap_proof)
        .zip(unfinalized_proofs_unextended)
        .zip(&rule.previous_proof_statements)
        .zip(actual_wrap_domains)
    // .take(1)
    {
        let PreviousProofStatement {
            proof_must_verify: should_verify,
            ..
        } = stmt;

        match self_data.wrap_domain {
            ForStepKind::SideLoaded => (),
            ForStepKind::Known(wrap_domain) => {
                let actual_wrap_domain = wrap_domains(actual_wrap_domain);
                assert_eq!(actual_wrap_domain.h, wrap_domain);
            }
        }

        verify_one(
            &mut srs,
            proof,
            &self_data,
            msg_for_next_wrap_proof,
            unfinalized,
            *should_verify,
            w,
        );
    }

    dbg!(w.aux.len() + w.primary.capacity());
    dbg!(w.ocaml_aux.len());
}

#[derive(Debug)]
enum OptFlag {
    Yes,
    No,
    Maybe,
}

#[derive(Debug)]
enum Opt<T> {
    Some(T),
    No,
    Maybe(Boolean, T),
}

impl<T> Opt<T> {
    fn map<V>(&self, fun: impl Fn(&T) -> V) -> Opt<V> {
        match self {
            Opt::Some(v) => Opt::Some(fun(v)),
            Opt::No => Opt::No,
            Opt::Maybe(b, v) => Opt::Maybe(*b, fun(v)),
        }
    }
}

#[derive(Debug)]
struct FeatureFlags<Bool> {
    range_check0: Bool,
    range_check1: Bool,
    foreign_field_add: Bool,
    foreign_field_mul: Bool,
    xor: Bool,
    rot: Bool,
    lookup: Bool,
    runtime_tables: Bool,
}

#[derive(Debug)]
struct Basic {
    proof_verifieds: Vec<u64>,
    // branches: u64,
    wrap_domain: Domains,
    step_domains: Vec<Domains>,
    feature_flags: FeatureFlags<OptFlag>,
}

#[derive(Debug)]
enum ForStepKind<T> {
    Known(T),
    SideLoaded,
}

#[derive(Debug)]
struct ForStep {
    branches: usize,
    max_proofs_verified: usize,
    proof_verifieds: ForStepKind<Vec<Fp>>,
    public_input: (), // Typ
    wrap_key: PlonkVerificationKeyEvals<Fp>,
    wrap_domain: ForStepKind<Domain>,
    step_domains: ForStepKind<Vec<Domains>>,
    feature_flags: FeatureFlags<OptFlag>,
}
