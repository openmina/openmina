use std::{path::Path, str::FromStr};

use crate::proofs::public_input::plonk_checks::ShiftingValue;
use ark_ff::BigInteger256;
use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};
use kimchi::{proof::PointEvaluations, verifier_index::VerifierIndex};
use mina_curves::pasta::{Fq, Vesta};
use mina_hasher::Fp;
use mina_p2p_messages::v2::{self, CompositionTypesBranchDataStableV1};

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

use super::{
    public_input::{messages::MessagesForNextWrapProof, plonk_checks::PlonkMinimal},
    unfinalized::AllEvals,
    util::extract_bulletproof,
    witness::{
        make_group, Boolean, FieldWitness, InnerCurve, MessagesForNextStepProof,
        PlonkVerificationKeyEvals, Prover, Witness,
    },
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
    let combined_inner_product_actual = combined_inner_product2(CombinedInnerProductParams2::<4> {
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

#[derive(Clone, Debug)]
pub struct DeferredValues<F: FieldWitness> {
    pub plonk: Plonk<F>,
    pub combined_inner_product: F::Shifting,
    pub b: F::Shifting,
    pub xi: [u64; 2],
    pub bulletproof_challenges: Vec<Fp>,
    pub branch_data: CompositionTypesBranchDataStableV1,
}

fn expand_proof(
    dlog_vk: &VerifierIndex<Vesta>,
    dlog_plonk_index: &PlonkVerificationKeyEvals<Fp>,
    app_state: &Statement<SokDigest>,
    t: &v2::LedgerProofProdStableV2,
    tag: (),
    must_verify: Boolean,
) {
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
        let zetaw = zeta * dlog_vk.domain.group_gen;

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

    let hash = MessagesForNextStepProof {
        app_state,
        dlog_plonk_index,
        challenge_polynomial_commitments: statement
            .messages_for_next_step_proof
            .challenge_polynomial_commitments
            .iter()
            .map(|(x, y)| InnerCurve::of_affine(make_group(x.to_field::<Fp>(), y.to_field())))
            .collect(),
        old_bulletproof_challenges,
    }
    .hash();
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

pub fn generate_merge_proof(
    statement: &v2::MinaStateBlockchainStateValueStableV2LedgerProofStatement,
    proofs: &(v2::LedgerProofProdStableV2, v2::LedgerProofProdStableV2),
    message: &SokMessage,
    step_prover: &Prover<Fp>,
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
    let verifier_index = step_prover.index.verifier_index.as_ref().unwrap();

    let res = rule
        .previous_proof_statements
        .iter()
        .map(
            |PreviousProofStatement {
                 public_input,
                 proof,
                 proof_must_verify,
             }| {
                expand_proof(
                    verifier_index,
                    &dlog_plonk_index,
                    public_input,
                    proof,
                    (),
                    *proof_must_verify,
                )
            },
        )
        .collect::<Vec<_>>();

    dbg!(w.aux.len() + w.primary.capacity());
    dbg!(w.ocaml_aux.len());
}
