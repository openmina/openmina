#![allow(unused)]

use std::{ops::Neg, str::FromStr};

use ark_ec::short_weierstrass_jacobian::GroupAffine;
use ark_ff::{BigInteger256, One, Zero};
use ark_poly::{
    univariate::DensePolynomial, EvaluationDomain, Radix2EvaluationDomain, UVPolynomial,
};
use itertools::Itertools;
use kimchi::{
    circuits::{scalars::RandomOracles, wires::COLUMNS},
    oracles::OraclesResult,
    proof::{PointEvaluations, ProofEvaluations, ProverProof},
};
use mina_curves::pasta::{Fq, PallasParameters, Vesta};
use mina_hasher::Fp;
use mina_p2p_messages::v2::{
    CompositionTypesBranchDataDomainLog2StableV1, CompositionTypesBranchDataStableV1,
    PicklesBaseProofsVerifiedStableV1, PicklesProofProofsVerified2ReprStableV2,
};
use mina_poseidon::{sponge::ScalarChallenge, FqSponge};
use mina_signer::CurvePoint;
use poly_commitment::{
    commitment::{b_poly_coefficients, CommitmentCurve},
    PolyComm,
};

use crate::{
    proofs::{
        opt_sponge::OptSponge,
        public_input::{
            plonk_checks::{derive_plonk, ft_eval0, ShiftingValue},
            prepared_statement::{DeferredValues, Plonk, PreparedStatement, ProofState},
        },
        unfinalized::{dummy_ipa_wrap_challenges, Unfinalized},
        util::{challenge_polynomial, proof_evaluation_to_list},
        verification::make_scalars_env,
        witness::{
            endos, field, make_group, Boolean, FieldWitness, InnerCurve, StepStatementWithHash,
            ToBoolean,
        },
        BACKEND_TICK_ROUNDS_N,
    },
    scan_state::scan_state::transaction_snark::{SokDigest, Statement},
    verifier::SRS_PALLAS,
    CurveAffine,
};

use self::pseudo::PseudoDomain;

use super::{
    public_input::{
        messages::{dummy_ipa_step_sg, MessagesForNextWrapProof},
        plonk_checks::{PlonkMinimal, ScalarsEnv, ShiftedValue},
    },
    to_field_elements::ToFieldElements,
    unfinalized::AllEvals,
    util::u64_to_field,
    witness::{
        plonk_curve_ops::scale_fast, Check, PlonkVerificationKeyEvals,
        ReducedMessagesForNextStepProof, StepProofState, StepStatement, Witness,
    },
};

/// Common.Max_degree.wrap_log2
pub const COMMON_MAX_DEGREE_WRAP_LOG2: usize = 15;

pub struct CombinedInnerProductParams<'a> {
    pub env: &'a ScalarsEnv<Fp>,
    pub evals: &'a ProofEvaluations<[Fp; 2]>,
    pub minimal: &'a PlonkMinimal<Fp>,
    pub proof: &'a PicklesProofProofsVerified2ReprStableV2,
    pub r: Fp,
    pub old_bulletproof_challenges: &'a [[Fp; 16]],
    pub xi: Fp,
    pub zetaw: Fp,
}

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/pickles/wrap.ml#L37
pub fn combined_inner_product(params: CombinedInnerProductParams) -> Fp {
    let CombinedInnerProductParams {
        env,
        old_bulletproof_challenges,
        evals,
        minimal,
        proof,
        r,
        xi,
        zetaw,
    } = params;

    let ft_eval0 = ft_eval0(
        env,
        evals,
        minimal,
        proof.prev_evals.evals.public_input.0.to_field(),
    );

    let challenge_polys: Vec<_> = old_bulletproof_challenges
        .iter()
        .map(|v| challenge_polynomial(v))
        .collect();

    let a = proof_evaluation_to_list(evals);
    let ft_eval1: Fp = proof.prev_evals.ft_eval1.to_field();

    enum WhichEval {
        First,
        Second,
    }

    let combine = |which_eval: WhichEval, ft: Fp, pt: Fp| {
        let f = |[x, y]: &[Fp; 2]| match which_eval {
            WhichEval::First => *x,
            WhichEval::Second => *y,
        };
        let a: Vec<_> = a.iter().map(f).collect();
        let public_input = &proof.prev_evals.evals.public_input;
        let public_input: [Fp; 2] = [public_input.0.to_field(), public_input.1.to_field()];

        let mut v: Vec<_> = challenge_polys
            .iter()
            .map(|f| f(pt))
            .chain([f(&public_input), ft])
            .chain(a)
            .collect();

        v.reverse();
        let (init, rest) = v.split_at(1);
        rest.iter().fold(init[0], |acc, fx| *fx + (xi * acc))
    };

    combine(WhichEval::First, ft_eval0, minimal.zeta)
        + (r * combine(WhichEval::Second, ft_eval1, zetaw))
}

// TODO: De-duplicate with CombinedInnerProductParams
pub struct CombinedInnerProductParams2<
    'a,
    F: FieldWitness,
    const NROUNDS: usize,
    const NLIMB: usize = 2,
> {
    pub env: &'a ScalarsEnv<F>,
    pub evals: &'a ProofEvaluations<[F; 2]>,
    pub public: [F; 2],
    pub minimal: &'a PlonkMinimal<F, NLIMB>,
    pub ft_eval1: F,
    pub r: F,
    pub old_bulletproof_challenges: &'a [[F; NROUNDS]],
    pub xi: F,
    pub zetaw: F,
}

// TODO: De-duplicate with combined_inner_product
/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/pickles/wrap.ml#L37
pub fn combined_inner_product2<F: FieldWitness, const NROUNDS: usize, const NLIMB: usize>(
    params: CombinedInnerProductParams2<F, NROUNDS, NLIMB>,
) -> F {
    let CombinedInnerProductParams2 {
        env,
        old_bulletproof_challenges,
        evals,
        minimal,
        r,
        xi,
        zetaw,
        public,
        ft_eval1,
    } = params;

    let ft_eval0 = ft_eval0::<F, NLIMB>(env, evals, minimal, public[0]);

    let challenge_polys: Vec<_> = old_bulletproof_challenges
        .iter()
        .map(|v| challenge_polynomial(v))
        .collect();

    let a = proof_evaluation_to_list(evals);

    enum WhichEval {
        First,
        Second,
    }

    let combine = |which_eval: WhichEval, ft: F, pt: F| {
        let f = |[x, y]: &[F; 2]| match which_eval {
            WhichEval::First => *x,
            WhichEval::Second => *y,
        };

        challenge_polys
            .iter()
            .map(|f| f(pt))
            .chain([f(&public), ft])
            .chain(a.iter().map(f))
            .rev()
            .reduce(|acc, fx| fx + (xi * acc))
            .unwrap()
    };

    combine(WhichEval::First, ft_eval0, minimal.zeta)
        + (r * combine(WhichEval::Second, ft_eval1, zetaw))
}

pub struct Oracles<F: FieldWitness> {
    pub o: RandomOracles<F>,
    pub p_eval: (F, F),
    pub opening_prechallenges: Vec<F>,
    pub digest_before_evaluations: F,
}

impl<F: FieldWitness> Oracles<F> {
    pub fn alpha(&self) -> F {
        self.o.alpha_chal.0
    }

    pub fn beta(&self) -> F {
        self.o.beta
    }

    pub fn gamma(&self) -> F {
        self.o.gamma
    }

    pub fn zeta(&self) -> F {
        self.o.zeta_chal.0
    }

    pub fn v(&self) -> ScalarChallenge<F> {
        self.o.v_chal.clone()
    }

    pub fn u(&self) -> ScalarChallenge<F> {
        self.o.u_chal.clone()
    }

    pub fn p_eval_1(&self) -> F {
        self.p_eval.0
    }

    pub fn p_eval_2(&self) -> F {
        self.p_eval.1
    }
}

pub fn create_oracle<F: FieldWitness>(
    verifier_index: &kimchi::verifier_index::VerifierIndex<F::OtherCurve>,
    proof: &kimchi::proof::ProverProof<F::OtherCurve>,
    public: &[F],
) -> Oracles<F> {
    use mina_curves::pasta::VestaParameters;
    use mina_poseidon::constants::PlonkSpongeConstantsKimchi;
    use mina_poseidon::sponge::DefaultFqSponge;
    use mina_poseidon::sponge::DefaultFrSponge;
    use poly_commitment::commitment::shift_scalar;

    // TODO: Don't clone the SRS here
    let mut srs = (**verifier_index.srs.get().unwrap()).clone();
    let log_size_of_group = verifier_index.domain.log_size_of_group;
    let lgr_comm = make_lagrange::<F>(&mut srs, log_size_of_group);

    let lgr_comm: Vec<PolyComm<F::OtherCurve>> = lgr_comm.into_iter().take(public.len()).collect();
    let lgr_comm_refs: Vec<_> = lgr_comm.iter().collect();

    let p_comm = PolyComm::<F::OtherCurve>::multi_scalar_mul(
        &lgr_comm_refs,
        &public.iter().map(|s| -*s).collect::<Vec<_>>(),
    );

    let p_comm = {
        verifier_index
            .srs()
            .mask_custom(p_comm.clone(), &p_comm.map(|_| F::one()))
            .unwrap()
            .commitment
    };

    type EFrSponge<F> = DefaultFrSponge<F, PlonkSpongeConstantsKimchi>;
    let oracles_result = proof
        .oracles::<F::FqSponge, EFrSponge<F>>(&verifier_index, &p_comm, public)
        .unwrap();

    let OraclesResult {
        digest,
        oracles,
        combined_inner_product,
        fq_sponge: mut sponge,
        public_evals: p_eval,
        all_alphas: _,
        powers_of_eval_points_for_chunks: _,
        polys: _,
        zeta1: _,
        ft_eval0: _,
    } = oracles_result;

    sponge.absorb_fr(&[shift_scalar::<F::OtherCurve>(combined_inner_product)]);

    let opening_prechallenges: Vec<_> = proof
        .proof
        .prechallenges(&mut sponge)
        .into_iter()
        .map(|f| f.0)
        .collect();

    Oracles {
        o: oracles,
        p_eval: (p_eval[0][0], p_eval[1][0]),
        opening_prechallenges,
        digest_before_evaluations: digest,
    }
}

pub fn make_lagrange<F: FieldWitness>(
    srs: &mut poly_commitment::srs::SRS<F::OtherCurve>,
    domain_log2: u32,
) -> Vec<PolyComm<F::OtherCurve>> {
    let domain_size = 2u64.pow(domain_log2) as usize;

    dbg!(domain_log2, domain_size);

    let x_domain = EvaluationDomain::<F>::new(domain_size).expect("invalid argument");

    srs.add_lagrange_basis(x_domain);

    let lagrange_bases = &srs.lagrange_bases[&x_domain.size()];
    lagrange_bases[..domain_size].to_vec()
}

/// Defined in `plonk_checks.ml`
fn actual_evaluation<F: FieldWitness>(pt: F, e: &[F], rounds: usize) -> F {
    let [e, es @ ..] = e else {
        return F::zero();
    };

    let pt_n = (0..rounds).fold(pt, |acc, _| acc * acc);
    es.iter().fold(*e, |acc, fx| *fx + (pt_n * acc))
}

pub fn evals_of_split_evals<F: FieldWitness>(
    zeta: F,
    zetaw: F,
    es: &ProofEvaluations<PointEvaluations<Vec<F>>>,
    rounds: usize,
) -> ProofEvaluations<[F; 2]> {
    es.map_ref(&|PointEvaluations {
                     zeta: x1,
                     zeta_omega: x2,
                 }| {
        let zeta = actual_evaluation(zeta, x1, rounds);
        let zeta_omega = actual_evaluation(zetaw, x2, rounds);
        [zeta, zeta_omega]
        // PointEvaluations {
        //     zeta: actual_evaluation(zeta, x1, rounds),
        //     zeta_omega: actual_evaluation(zetaw, x2, rounds),
        // }
    })
}

/// Value of `Common.Max_degree.step_log2`
pub const COMMON_MAX_DEGREE_STEP_LOG2: u64 = 16;

fn deferred_values(
    _sgs: Vec<crate::CurveAffine<Fp>>,
    _prev_challenges: Vec<Fp>,
    // step_vk: &VerifierIndex,
    public_input: &[Fp],
    proof: &kimchi::proof::ProverProof<Vesta>,
    actual_proofs_verified: usize,
    prover_index: &kimchi::prover_index::ProverIndex<Vesta>,
) -> DeferredValuesAndHints {
    let step_vk = prover_index.verifier_index();
    let log_size_of_group = step_vk.domain.log_size_of_group;

    let oracle = create_oracle(&step_vk, &proof, public_input);
    let x_hat = [oracle.p_eval.0, oracle.p_eval.1];

    let alpha = oracle.alpha();
    let beta = oracle.beta();
    let gamma = oracle.gamma();
    let zeta = oracle.zeta();

    let to_bytes = |f: Fp| {
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

    let r = oracle.u();
    let xi = oracle.v();

    let (_, endo) = endos::<Fq>();
    let scalar_to_field = |bytes: [u64; 2]| -> Fp {
        use crate::proofs::public_input::scalar_challenge::ScalarChallenge;
        ScalarChallenge::from(bytes).to_field(&endo)
    };

    let _domain = step_vk.domain.log_size_of_group;
    let zetaw = {
        let zeta = scalar_to_field(plonk0.zeta_bytes);
        step_vk.domain.group_gen * zeta
    };

    let tick_plonk_minimal = PlonkMinimal {
        zeta: scalar_to_field(plonk0.zeta_bytes),
        alpha: scalar_to_field(plonk0.alpha_bytes),
        ..plonk0.clone()
    };
    let tick_combined_evals =
        evals_of_split_evals(zeta, zetaw, &proof.evals, BACKEND_TICK_ROUNDS_N);

    let domain_log2: u8 = log_size_of_group.try_into().unwrap();
    let tick_env = make_scalars_env(
        &tick_plonk_minimal,
        domain_log2,
        COMMON_MAX_DEGREE_STEP_LOG2,
    );
    let plonk = derive_plonk(&tick_env, &tick_combined_evals, &tick_plonk_minimal);

    let (new_bulletproof_challenges, b) = {
        let chals = oracle
            .opening_prechallenges
            .iter()
            .copied()
            .map(|v| scalar_to_field(to_bytes(v)))
            .collect::<Vec<_>>();

        let r = scalar_to_field(to_bytes(r.0));
        let zeta = scalar_to_field(plonk0.zeta_bytes);
        // TODO: Pass by value here
        let challenge_poly = challenge_polynomial(&chals);
        let b = challenge_poly(zeta) + (r * challenge_poly(zetaw));

        let prechals = oracle
            .opening_prechallenges
            .iter()
            .copied()
            // .map(to_bytes)
            .collect::<Vec<_>>();
        (prechals, b)
    };

    let f = |s: &str| Fp::from_str(s).unwrap();
    // assert_eq!(
    //     b,
    //     f("5767724647428070642927793768409372780130584519014792248744668603804331622261")
    // );

    let evals = proof
        .evals
        .map_ref(&|PointEvaluations { zeta, zeta_omega }| {
            assert_eq!(zeta.len(), 1);
            assert_eq!(zeta_omega.len(), 1);
            [zeta[0], zeta_omega[0]]
        });

    let combined_inner_product =
        combined_inner_product2(CombinedInnerProductParams2::<_, { Fp::NROUNDS }, 2> {
            env: &tick_env,
            evals: &evals,
            minimal: &tick_plonk_minimal,
            r: scalar_to_field(to_bytes(r.0)),
            old_bulletproof_challenges: &[],
            xi: scalar_to_field(to_bytes(xi.0)),
            zetaw,
            public: x_hat,
            ft_eval1: proof.ft_eval1,
        });

    // assert_eq!(
    //     combined_inner_product,
    //     f("1794066667028988946470104780887317784726228530187228813826020823032902910060")
    // );

    let shift = |f: Fp| <Fp as FieldWitness>::Shifting::of_field(f);

    DeferredValuesAndHints {
        deferred_values: DeferredValues {
            plonk: Plonk {
                alpha: plonk0.alpha_bytes,
                beta: plonk0.beta_bytes,
                gamma: plonk0.gamma_bytes,
                zeta: plonk0.zeta_bytes,
                zeta_to_srs_length: plonk.zeta_to_srs_length,
                zeta_to_domain_size: plonk.zeta_to_domain_size,
                // vbmul: plonk.vbmul,
                // complete_add: plonk.complete_add,
                // endomul: plonk.endomul,
                // endomul_scalar: plonk.endomul_scalar,
                perm: plonk.perm,
                lookup: (),
            },
            combined_inner_product: shift(combined_inner_product),
            b: shift(b),
            xi: to_bytes(xi.0),
            bulletproof_challenges: {
                assert_eq!(new_bulletproof_challenges.len(), BACKEND_TICK_ROUNDS_N);
                new_bulletproof_challenges
            },
            branch_data: CompositionTypesBranchDataStableV1 {
                proofs_verified: match actual_proofs_verified {
                    0 => PicklesBaseProofsVerifiedStableV1::N0,
                    1 => PicklesBaseProofsVerifiedStableV1::N1,
                    2 => PicklesBaseProofsVerifiedStableV1::N2,
                    _ => panic!(),
                },
                domain_log2: CompositionTypesBranchDataDomainLog2StableV1(
                    (log_size_of_group as u8).into(),
                ),
            },
        },
        sponge_digest_before_evaluations: oracle.digest_before_evaluations,
        x_hat_evals: x_hat,
    }
}

struct DeferredValuesAndHints {
    deferred_values: DeferredValues<Fp>,
    sponge_digest_before_evaluations: Fp,
    x_hat_evals: [Fp; 2],
}

fn pad_messages_for_next_wrap_proof(
    mut msgs: Vec<MessagesForNextWrapProof>,
) -> Vec<MessagesForNextWrapProof> {
    const N_MSGS: usize = 2;
    const N_CHALS: usize = 2;

    while msgs.len() < N_MSGS {
        let msg = MessagesForNextWrapProof {
            challenge_polynomial_commitment: crate::CurveAffine::new(dummy_ipa_step_sg()),
            old_bulletproof_challenges: vec![MessagesForNextWrapProof::dummy_padding(); N_CHALS],
        };
        // TODO: Not sure if it prepend or append
        msgs.insert(0, msg);
    }
    msgs
}

fn make_public_input(
    step_statement: &StepStatement,
    messages_for_next_step_proof_hash: [u64; 4],
    messages_for_next_wrap_proof_hash: &[[u64; 4]],
) -> Vec<Fp> {
    let to_fp = |v: [u64; 4]| Fp::from(BigInteger256(v));
    let mut fields = Vec::with_capacity(135);

    for unfinalized_proofs in &step_statement.proof_state.unfinalized_proofs {
        unfinalized_proofs.to_field_elements(&mut fields);
    }

    to_fp(messages_for_next_step_proof_hash).to_field_elements(&mut fields);

    for msg in messages_for_next_wrap_proof_hash.iter().copied().map(to_fp) {
        msg.to_field_elements(&mut fields);
    }

    fields
}

#[derive(Clone, Debug)]
pub struct WrapProofState {
    pub deferred_values: DeferredValues<Fp>,
    pub sponge_digest_before_evaluations: Fp,
    pub messages_for_next_wrap_proof: MessagesForNextWrapProof,
}

#[derive(Clone, Debug)]
pub struct WrapStatement {
    pub proof_state: WrapProofState,
    pub messages_for_next_step_proof: ReducedMessagesForNextStepProof<Statement<SokDigest>>,
}

fn exists_prev_statement(
    step_statement: &StepStatement,
    messages_for_next_step_proof_hash: [u64; 4],
    w: &mut Witness<Fq>,
) {
    for unfinalized in &step_statement.proof_state.unfinalized_proofs {
        w.exists_no_check(unfinalized);
    }
    w.exists(u64_to_field::<Fq, 4>(&messages_for_next_step_proof_hash));
}

/// Dummy.Ipa.Wrap.sg
fn dummy_ipa_wrap_sg() -> GroupAffine<PallasParameters> {
    type G = GroupAffine<PallasParameters>;

    cache_one!(G, {
        use crate::proofs::public_input::scalar_challenge::ScalarChallenge;
        let (_, endo) = endos::<Fp>();

        let dummy = dummy_ipa_wrap_challenges();
        let dummy = dummy
            .iter()
            .map(|c| ScalarChallenge::from(*c).to_field(&endo))
            .collect::<Vec<_>>();

        let coeffs = b_poly_coefficients(&dummy);
        let p = DensePolynomial::from_coefficients_vec(coeffs);

        let comm = {
            let srs = SRS_PALLAS.lock().unwrap();
            srs.commit_non_hiding(&p, None)
        };
        comm.unshifted[0]
    })
}

pub struct ChallengePolynomial {
    pub commitment: CurveAffine<Fp>,
    pub challenges: [Fq; 15],
}

pub fn wrap(
    statement_with_sok: &Statement<SokDigest>,
    proof: &kimchi::proof::ProverProof<Vesta>,
    step_statement: StepStatement,
    prev_evals: &[AllEvals<Fq>],
    dlog_plonk_index: &PlonkVerificationKeyEvals<Fp>,
    prover_index: &kimchi::prover_index::ProverIndex<Vesta>,
    w: &mut Witness<Fq>,
) -> Vec<ChallengePolynomial> {
    let messages_for_next_step_proof_hash = crate::proofs::witness::MessagesForNextStepProof {
        app_state: &statement_with_sok,
        challenge_polynomial_commitments: vec![],
        old_bulletproof_challenges: vec![],
        dlog_plonk_index,
    }
    .hash();
    let messages_for_next_wrap_proof_padded =
        pad_messages_for_next_wrap_proof(step_statement.messages_for_next_wrap_proof.clone());
    let messages_for_next_wrap_proof_hash = messages_for_next_wrap_proof_padded
        .iter()
        .map(MessagesForNextWrapProof::hash)
        .collect::<Vec<_>>();

    // assert_eq!(
    //     messages_for_next_step_proof_hash,
    //     [
    //         12928032459193155768,
    //         823333255794445397,
    //         14777852695581800947,
    //         354023456053555014
    //     ]
    // );

    let public_input = make_public_input(
        &step_statement,
        messages_for_next_step_proof_hash,
        &messages_for_next_wrap_proof_hash,
    );

    let actual_proofs_verified = 0; // TODO

    let DeferredValuesAndHints {
        deferred_values,
        sponge_digest_before_evaluations,
        x_hat_evals: _,
    } = deferred_values(
        vec![],
        vec![],
        &public_input,
        proof,
        actual_proofs_verified,
        prover_index,
    );

    let to_fq = |[a, b]: [u64; 2]| Fq::from(BigInteger256([a, b, 0, 0]));
    let to_fqs = |v: &[[u64; 2]]| v.iter().copied().map(to_fq).collect::<Vec<_>>();

    let messages_for_next_wrap_proof = MessagesForNextWrapProof {
        challenge_polynomial_commitment: {
            let GroupAffine { x, y, .. } = proof.proof.sg;
            CurveAffine(x, y)
        },
        old_bulletproof_challenges: step_statement
            .proof_state
            .unfinalized_proofs
            .iter()
            .map(|a| {
                to_fqs(&a.deferred_values.bulletproof_challenges)
                    .try_into()
                    .unwrap()
            })
            .collect(),
    };

    let messages_for_next_wrap_proof_prepared = {
        use crate::proofs::public_input::scalar_challenge::ScalarChallenge;

        let MessagesForNextWrapProof {
            challenge_polynomial_commitment,
            old_bulletproof_challenges,
        } = &messages_for_next_wrap_proof;

        let (_, endo) = endos::<Fp>();

        MessagesForNextWrapProof {
            challenge_polynomial_commitment: challenge_polynomial_commitment.clone(),
            old_bulletproof_challenges: old_bulletproof_challenges
                .iter()
                .map(|c| c.map(|c| ScalarChallenge::from(c).to_field(&endo)))
                .collect(),
        }
    };

    let next_statement = WrapStatement {
        proof_state: WrapProofState {
            deferred_values: deferred_values.clone(),
            sponge_digest_before_evaluations,
            messages_for_next_wrap_proof,
        },
        messages_for_next_step_proof: step_statement
            .proof_state
            .messages_for_next_step_proof
            .clone(),
    };

    next_statement.check(w);

    let next_accumulator = {
        let mut vec = step_statement
            .proof_state
            .messages_for_next_step_proof
            .challenge_polynomial_commitments
            .clone();
        while vec.len() < MAX_PROOFS_VERIFIED_N as usize {
            let GroupAffine { x, y, .. } = dummy_ipa_wrap_sg();
            vec.insert(0, CurveAffine(x, y));
        }

        let old = &messages_for_next_wrap_proof_prepared.old_bulletproof_challenges;

        vec.into_iter()
            .zip(old)
            .map(|(sg, chals)| ChallengePolynomial {
                commitment: sg,
                challenges: *chals,
            })
            .collect::<Vec<_>>()
    };

    // public input
    w.primary = PreparedStatement {
        proof_state: ProofState {
            deferred_values,
            sponge_digest_before_evaluations: {
                let bigint: BigInteger256 = next_statement
                    .proof_state
                    .sponge_digest_before_evaluations
                    .into();
                bigint.0
            },
            messages_for_next_wrap_proof: messages_for_next_wrap_proof_prepared.hash(),
        },
        messages_for_next_step_proof: messages_for_next_step_proof_hash,
    }
    .to_public_input(40);

    // TODO: Those are variables
    let which_index = 0;
    let pi_branches = 5;
    let step_widths = [0, 2, 0, 0, 1];
    let step_domains = [
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
    ];

    let main_params = WrapMainParams {
        step_statement,
        next_statement,
        messages_for_next_wrap_proof_padded,
        which_index,
        pi_branches,
        step_widths,
        step_domains,
        messages_for_next_step_proof_hash,
        prev_evals,
        proof,
        prover_index,
    };

    wrap_main(&main_params, w);

    next_accumulator
}

// TODO: Compute those values instead of hardcoded
const FORBIDDEN_SHIFTED_VALUES: &[Fq; 2] = &[
    ark_ff::field_new!(Fq, "91120631062839412180561524743370440705"),
    ark_ff::field_new!(Fq, "91120631062839412180561524743370440706"),
];

impl Check<Fq> for ShiftedValue<Fp> {
    fn check(&self, w: &mut Witness<Fq>) {
        let bools = FORBIDDEN_SHIFTED_VALUES.map(|forbidden| {
            let shifted: Fq = {
                let ShiftedValue { shifted } = self.clone();
                let f: BigInteger256 = shifted.into();
                f.into()
            };
            field::equal(shifted, forbidden, w)
        });
        Boolean::any(&bools, w);
    }
}

impl Check<Fp> for ShiftedValue<Fq> {
    fn check(&self, w: &mut Witness<Fp>) {
        // TODO: Compute those values instead of hardcoded
        #[rustfmt::skip]
        const FORBIDDEN_SHIFTED_VALUES: &[(Fp, Boolean); 4] = &[
            (ark_ff::field_new!(Fp, "45560315531506369815346746415080538112"), Boolean::False),
            (ark_ff::field_new!(Fp, "45560315531506369815346746415080538113"), Boolean::False),
            (ark_ff::field_new!(Fp, "14474011154664524427946373126085988481727088556502330059655218120611762012161"), Boolean::True),
            (ark_ff::field_new!(Fp, "14474011154664524427946373126085988481727088556502330059655218120611762012161"), Boolean::True),
        ];

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
        // `Fq` is larger than `Fp` so we have to split the field (low & high bits)
        // See:
        // https://github.com/MinaProtocol/mina/blob/e85cf6969e42060f69d305fb63df9b8d7215d3d7/src/lib/pickles/impls.ml#L94C1-L105C45

        let to_high_low = |fq: Fq| {
            let [low, high @ ..] = crate::proofs::witness::field_to_bits::<Fq, 255>(fq);
            (of_bits::<Fp>(&high), low.to_boolean())
        };

        let bools = FORBIDDEN_SHIFTED_VALUES.map(|(x2, b2)| {
            let (x1, b1) = to_high_low(self.shifted);
            let x_eq = field::equal(x1, x2, w);
            let b_eq = match b2 {
                Boolean::True => b1,
                Boolean::False => b1.neg(),
            };
            x_eq.and(&b_eq, w)
        });
        Boolean::any(&bools, w);
    }
}

impl<F: FieldWitness> Check<F> for ShiftedValue<F> {
    fn check(&self, w: &mut Witness<F>) {
        // Same field, no check
    }
}

impl Check<Fq> for WrapStatement {
    fn check(&self, w: &mut Witness<Fq>) {
        let WrapStatement {
            proof_state:
                WrapProofState {
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
                                    // vbmul,
                                    // complete_add,
                                    // endomul,
                                    // endomul_scalar,
                                    perm,
                                    lookup: _,
                                },
                            combined_inner_product,
                            b,
                            xi: _,
                            bulletproof_challenges: _,
                            branch_data: _,
                        },
                    sponge_digest_before_evaluations: _,
                    messages_for_next_wrap_proof: _,
                },
            messages_for_next_step_proof: _,
        } = self;

        perm.check(w);
        zeta_to_domain_size.check(w);
        zeta_to_srs_length.check(w);
        b.check(w);
        combined_inner_product.check(w);
    }
}

pub mod pseudo {
    use ark_poly::Radix2EvaluationDomain;

    use super::*;

    pub struct PseudoDomain<F: FieldWitness> {
        pub domain: Radix2EvaluationDomain<F>,
        pub max_log2: u64,
        pub which_branch: Box<[Boolean]>,
        pub all_possible_domain: Box<[Domain]>,
    }

    impl<F: FieldWitness> PseudoDomain<F> {
        pub fn vanishing_polynomial(&self, x: F, w: &mut Witness<F>) -> F {
            let max_log2 = self.max_log2 as usize;

            let pow2_pows = {
                let mut res = vec![x; max_log2 + 1];
                for i in 1..res.len() {
                    res[i] = field::square(res[i - 1], w);
                }
                res
            };

            let which = &self.which_branch;
            let ws = self
                .all_possible_domain
                .iter()
                .map(|d| pow2_pows[d.log2_size() as usize])
                .collect::<Vec<_>>();

            let res = choose_checked(which, &ws, w);
            w.exists(res - F::one())
        }
    }

    fn mask(bits: &[Boolean], xs: &[u64]) -> Fq {
        let xs = xs.iter().copied().map(Fq::from);
        let bits = bits.iter().copied().map(Boolean::to_field::<Fq>);

        bits.zip(xs).map(|(b, x)| b * x).sum()
    }

    pub fn choose(bits: &[Boolean], xs: &[u64]) -> Fq {
        mask(bits, xs)
    }

    fn mask_checked<F: FieldWitness>(bits: &[Boolean], xs: &[F], w: &mut Witness<F>) -> F {
        let bits = bits.iter().copied().map(Boolean::to_field::<F>);

        bits.zip(xs).rev().map(|(b, x)| field::mul(b, *x, w)).sum()
    }

    pub fn choose_checked<F: FieldWitness>(bits: &[Boolean], xs: &[F], w: &mut Witness<F>) -> F {
        mask_checked(bits, xs, w)
    }

    pub fn to_domain<F: FieldWitness>(
        which_branch: &[Boolean],
        all_possible_domains: &[Domain],
    ) -> PseudoDomain<F> {
        assert_eq!(which_branch.len(), all_possible_domains.len());

        // TODO: Not sure if that implementation is correct, OCaml does some weird stuff
        let which = which_branch.iter().position(Boolean::as_bool).unwrap();
        let domain = &all_possible_domains[which];
        let domain = Radix2EvaluationDomain::new(domain.size() as usize).unwrap();
        let max_log2 = {
            let all = all_possible_domains.iter().map(Domain::log2_size);
            Iterator::max(all).unwrap() // `rust-analyzer` is confused if we use `all.max()`
        };

        PseudoDomain {
            domain,
            max_log2,
            which_branch: Box::from(which_branch),
            all_possible_domain: Box::from(all_possible_domains),
        }
    }
}

fn ones_vector(first_zero: Fq, n: u64, w: &mut Witness<Fq>) -> Vec<Boolean> {
    let mut value = Boolean::True;

    let mut vector = (0..n)
        .map(|i| {
            let eq = field::equal(first_zero, Fq::from(i), w);
            value = if i == 0 {
                value.const_and(&eq.neg())
            } else {
                value.and(&eq.neg(), w)
            };
            value
        })
        .collect::<Vec<_>>();
    vector.reverse();
    vector
}

/// Max_proofs_verified.n
pub const MAX_PROOFS_VERIFIED_N: u64 = 2;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Domain {
    Pow2RootsOfUnity(u64),
}

impl Domain {
    pub fn log2_size(&self) -> u64 {
        let Self::Pow2RootsOfUnity(k) = self;
        *k
    }

    pub fn size(&self) -> u64 {
        1 << self.log2_size()
    }
}

#[derive(Debug)]
pub struct Domains {
    pub h: Domain,
}

pub fn make_scalars_env_checked<F: FieldWitness>(
    minimal: &PlonkMinimal<F, 4>,
    domain: &PseudoDomain<F>,
    srs_length_log2: u64,
    w: &mut Witness<F>,
) -> ScalarsEnv<F> {
    let PlonkMinimal {
        alpha,
        beta,
        gamma,
        zeta,
        joint_combiner,
        ..
    } = minimal;

    let alpha_pows = {
        let alpha = *alpha;
        let mut alphas = Box::new([F::one(); 71]);
        alphas[1] = alpha;
        for i in 2..alphas.len() {
            alphas[i] = field::mul(alpha, alphas[i - 1], w);
        }
        alphas
    };

    let (_w4, w3, w2, w1) = {
        let gen = domain.domain.group_gen;
        let w1 = field::div(F::one(), gen, w);
        let w2 = field::square(w1, w);
        let w3 = field::mul(w2, w1, w);
        // let w4 = (); // unused for now
        // let w4 = w3 * w1;

        ((), w3, w2, w1)
    };

    let zeta = *zeta;
    let zk_polynomial = {
        let a = zeta - w1;
        let b = zeta - w2;
        let c = zeta - w3;

        let res = field::mul(a, b, w);
        field::mul(res, c, w)
    };

    dbg!(zeta);
    let zeta_to_n_minus_1 = domain.vanishing_polynomial(zeta, w);

    ScalarsEnv {
        zk_polynomial,
        zeta_to_n_minus_1,
        srs_length_log2,
        domain: domain.domain.clone(),
        omega_to_minus_3: w3,
    }
}

/// Permuts_minus_1.add Nat.N1.n
const PERMUTS_MINUS_1_ADD_N1: usize = 6;

/// Other_field.Packed.Constant.size_in_bits
const OTHER_FIELD_PACKED_CONSTANT_SIZE_IN_BITS: usize = 255;

fn ft_comm(
    alpha: Fq,
    plonk: &Plonk<Fp>,
    t_comm: &PolyComm<Vesta>,
    verification_key: &PlonkVerificationKeyEvals<Fq>,
    w: &mut Witness<Fq>,
) -> Vesta {
    let m = verification_key;
    let [sigma_comm_last] = &m.sigma[PERMUTS_MINUS_1_ADD_N1..] else {
        panic!()
    };

    let scale = scale_fast::<Fq, Fp, OTHER_FIELD_PACKED_CONSTANT_SIZE_IN_BITS>;

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

mod pcs_batch {
    use super::{
        wrap_verifier::split_commitments::{CurveOpt, Point},
        *,
    };

    pub struct PcsBatch {
        without_degree_bound: usize,
        with_degree_bound: Vec<()>,
    }

    impl PcsBatch {
        pub fn create(without_degree_bound: usize) -> Self {
            Self {
                without_degree_bound,
                with_degree_bound: vec![],
            }
        }

        pub fn combine_split_commitments<F, Init, Scale>(
            mut init: Init,
            mut scale_and_add: Scale,
            xi: [u64; 2],
            without_degree_bound: &[(CircuitVar<Boolean>, Point<F>)],
            with_degree_bound: &[()],
            w: &mut Witness<F>,
        ) -> CurveOpt<F>
        where
            F: FieldWitness,
            Init: FnMut(&(CircuitVar<Boolean>, Point<F>), &mut Witness<F>) -> CurveOpt<F>,
            Scale: FnMut(
                CurveOpt<F>,
                [u64; 2],
                &(CircuitVar<Boolean>, Point<F>),
                &mut Witness<F>,
            ) -> CurveOpt<F>,
        {
            // TODO: Handle non-empty
            assert!(with_degree_bound.is_empty());

            let (last, comms) = without_degree_bound
                .split_last()
                .expect("combine_split_commitments: empty");

            comms
                .iter()
                .rev()
                .fold(init(last, w), |acc, p| scale_and_add(acc, xi, p, w))
        }
    }
}

pub mod wrap_verifier {
    use std::{convert::identity, ops::Neg, sync::Arc};

    use ark_poly::Radix2EvaluationDomain;
    use itertools::Itertools;
    use kimchi::prover_index::ProverIndex;
    use mina_curves::pasta::VestaParameters;
    use poly_commitment::{evaluation_proof::OpeningProof, srs::SRS};

    use crate::proofs::{
        public_input::plonk_checks::{self, ft_eval0_checked, ShiftFq},
        unfinalized,
        util::{challenge_polynomial_checked, to_absorption_sequence_opt},
        verifier_index::wrap_domains,
        witness::{
            add_fast,
            scalar_challenge::{self, to_field_checked},
            InnerCurve,
        },
        wrap::pcs_batch::PcsBatch,
    };

    use super::{pseudo::PseudoDomain, *};

    // TODO: Here we pick the verifier index directly from the prover index
    //       but OCaml does it differently
    pub fn choose_key(
        prover_index: &ProverIndex<Vesta>,
        w: &mut Witness<Fq>,
    ) -> PlonkVerificationKeyEvals<Fq> {
        let vk = prover_index.verifier_index.as_ref().unwrap();

        let to_curve = |v: &PolyComm<Vesta>| {
            let v = v.unshifted[0];
            InnerCurve::<Fq>::of_affine(make_group(v.x, v.y))
        };

        let plonk_index = PlonkVerificationKeyEvals {
            sigma: std::array::from_fn(|i| to_curve(&vk.sigma_comm[i])),
            coefficients: std::array::from_fn(|i| to_curve(&vk.coefficients_comm[i])),
            generic: to_curve(&vk.generic_comm),
            psm: to_curve(&vk.psm_comm),
            complete_add: to_curve(&vk.complete_add_comm),
            mul: to_curve(&vk.mul_comm),
            emul: to_curve(&vk.emul_comm),
            endomul_scalar: to_curve(&vk.endomul_scalar_comm),
        };

        let mut exists = |c: &InnerCurve<Fq>| {
            let GroupAffine { x, y, .. } = c.to_affine();
            w.exists_no_check([y, x]); // Note: `y` first
        };

        exists(&plonk_index.endomul_scalar);
        exists(&plonk_index.emul);
        exists(&plonk_index.mul);
        exists(&plonk_index.complete_add);
        exists(&plonk_index.psm);
        exists(&plonk_index.generic);
        plonk_index.coefficients.iter().rev().for_each(&mut exists);
        plonk_index.sigma.iter().rev().for_each(&mut exists);

        plonk_index
    }

    pub const NUM_POSSIBLE_DOMAINS: usize = 3;

    pub fn all_possible_domains() -> [Domain; NUM_POSSIBLE_DOMAINS] {
        [0, 1, 2].map(|proofs_verified| wrap_domains(proofs_verified).h)
    }

    use crate::proofs::witness::poseidon::Sponge;
    use mina_poseidon::constants::PlonkSpongeConstantsKimchi as Constants;

    #[derive(Clone, Debug)]
    pub struct PlonkWithField<F: FieldWitness> {
        pub alpha: F,
        pub beta: F,
        pub gamma: F,
        pub zeta: F,
        pub zeta_to_srs_length: ShiftedValue<F>,
        pub zeta_to_domain_size: ShiftedValue<F>,
        // pub vbmul: ShiftedValue<F>,
        // pub complete_add: ShiftedValue<F>,
        // pub endomul: ShiftedValue<F>,
        // pub endomul_scalar: ShiftedValue<F>,
        pub perm: ShiftedValue<F>,
        pub lookup: (),
    }

    fn map_plonk_to_field(plonk: &Plonk<Fq>, w: &mut Witness<Fq>) -> PlonkWithField<Fq> {
        let Plonk {
            alpha,
            beta,
            gamma,
            zeta,
            zeta_to_srs_length,
            zeta_to_domain_size,
            // vbmul,
            // complete_add,
            // endomul,
            // endomul_scalar,
            perm,
            lookup,
        } = plonk;

        let (_, endo) = endos::<Fp>();

        let mut scalar = |v: &[u64; 2]| to_field_checked::<Fq, 128>(u64_to_field(v), endo, w);

        let zeta = scalar(zeta);
        let gamma: Fq = u64_to_field(gamma);
        let beta: Fq = u64_to_field(beta);
        let alpha = scalar(alpha);

        PlonkWithField {
            alpha,
            beta,
            gamma,
            zeta,
            zeta_to_srs_length: zeta_to_srs_length.clone(),
            zeta_to_domain_size: zeta_to_domain_size.clone(),
            // vbmul: vbmul.clone(),
            // complete_add: complete_add.clone(),
            // endomul: endomul.clone(),
            // endomul_scalar: endomul_scalar.clone(),
            perm: perm.clone(),
            lookup: (),
        }
    }

    pub fn lowest_128_bits<F: FieldWitness>(f: F, assert_low_bits: bool, w: &mut Witness<F>) -> F {
        let (_, endo) = endos::<F::Scalar>();

        let (lo, hi): (F, F) = w.exists({
            let BigInteger256([a, b, c, d]) = f.into();
            (u64_to_field(&[a, b]), u64_to_field(&[c, d]))
        });

        to_field_checked::<_, 128>(hi, endo, w);
        if assert_low_bits {
            to_field_checked::<_, 128>(lo, endo, w);
        }
        lo
    }

    pub fn actual_evaluation<F: FieldWitness>(e: &[F], pt_to_n: F) -> F {
        let (last, rest) = e.split_last().expect("empty list");

        rest.iter().rev().fold(*last, |acc, y| {
            // TODO: So far only 1 item
            todo!()
        })
    }

    pub fn finalize_other_proof(
        domain: &PseudoDomain<Fq>,
        mut sponge: Sponge<Fq, Constants>,
        old_bulletproof_challenges: &[[Fq; 15]],
        deferred_values: &unfinalized::DeferredValues,
        evals: &AllEvals<Fq>,
        w: &mut Witness<Fq>,
    ) -> (Boolean, Vec<Fq>) {
        let unfinalized::DeferredValues {
            plonk,
            combined_inner_product,
            b,
            xi,
            bulletproof_challenges,
        } = deferred_values;

        let AllEvals { ft_eval1, evals } = evals;

        let plonk = map_plonk_to_field(plonk, w);
        let zetaw = field::mul(domain.domain.group_gen, plonk.zeta, w);

        let (sg_evals1, sg_evals2) = {
            let sg_olds = old_bulletproof_challenges
                .iter()
                .map(|chals| challenge_polynomial_checked(chals))
                .collect::<Vec<_>>();

            let mut sg_evals = |pt: Fq| sg_olds.iter().map(|f| f(pt, w)).collect::<Vec<_>>();

            // We decompose this way because of OCaml evaluation order
            let sg_evals2 = sg_evals(zetaw);
            let sg_evals1 = sg_evals(plonk.zeta);
            (sg_evals1, sg_evals2)
        };

        let sponge_state = {
            use crate::proofs::witness::poseidon::Sponge;
            use mina_poseidon::pasta::fq_kimchi::static_params;

            let challenge_digest = {
                let mut sponge = Sponge::<Fq, Constants>::new(static_params());
                old_bulletproof_challenges.iter().for_each(|v| {
                    sponge.absorb2(v, w);
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

        let xi_actual = lowest_128_bits(sponge.squeeze(w), false, w);
        let r_actual = lowest_128_bits(sponge.squeeze(w), true, w);

        let xi_correct = field::equal(xi_actual, u64_to_field(xi), w);

        let (_, endo) = endos::<Fp>();

        let xi = to_field_checked::<Fq, 128>(u64_to_field(xi), endo, w);
        let r = to_field_checked::<Fq, 128>(r_actual, endo, w);

        let to_bytes = |f: Fq| {
            let BigInteger256([a, b, c, d]) = f.into();
            [a, b, c, d]
        };

        let plonk_mininal = PlonkMinimal::<Fq, 4> {
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
                |f: Fq| (0..COMMON_MAX_DEGREE_WRAP_LOG2).fold(f, |acc, _| field::square(acc, w));

            let zeta_n = pow2pow(plonk.zeta);
            let zetaw_n = pow2pow(zetaw);

            evals.evals.map_ref(&|[x0, x1]| {
                let a = actual_evaluation(&[*x0], zeta_n);
                let b = actual_evaluation(&[*x1], zetaw_n);
                [a, b]
            })
        };

        let srs_length_log2 = COMMON_MAX_DEGREE_WRAP_LOG2 as u64;
        let env = make_scalars_env_checked(&plonk_mininal, domain, srs_length_log2, w);

        let combined_inner_product_correct = {
            let p_eval0 = evals.public_input.0;
            let ft_eval0 = ft_eval0_checked(&env, &combined_evals, &plonk_mininal, p_eval0, w);
            let a = proof_evaluation_to_list(&evals.evals);
            // assert_eq!(
            //     ft_eval0,
            //     Fq::from_str(
            //         "185615279260584823497562138911626835196500046105133768509329205349731009215"
            //     )
            //     .unwrap()
            // );

            let actual_combined_inner_product = {
                enum WhichEval {
                    First,
                    Second,
                }

                let combine = |which_eval: WhichEval,
                               sg_evals: &[Fq],
                               ft_eval: Fq,
                               x_hat: Fq,
                               w: &mut Witness<Fq>| {
                    let f = |[x, y]: &[Fq; 2]| match which_eval {
                        WhichEval::First => *x,
                        WhichEval::Second => *y,
                    };
                    sg_evals
                        .iter()
                        .copied()
                        .chain([x_hat])
                        .chain([ft_eval])
                        .chain(a.iter().map(f))
                        .rev()
                        .reduce(|acc, fx| fx + field::mul(xi, acc, w))
                        // OCaml panics too
                        .expect("combine_split_evaluations: empty")
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

            // assert_eq!(
            //     actual_combined_inner_product,
            //     Fq::from_str(
            //         "8504630526995452578694872162766067838423804679362527237174610151794554026782"
            //     )
            //     .unwrap()
            // );

            let combined_inner_product =
                ShiftingValue::<Fq>::shifted_to_field(combined_inner_product);
            field::equal(combined_inner_product, actual_combined_inner_product, w)
        };

        let mut bulletproof_challenges = bulletproof_challenges
            .iter()
            .rev()
            .map(|bytes| to_field_checked::<Fq, 128>(u64_to_field(bytes), endo, w))
            .collect::<Vec<_>>();
        bulletproof_challenges.reverse();

        let b_correct = {
            let challenge_poly = challenge_polynomial_checked(&bulletproof_challenges);

            // We decompose this way because of OCaml evaluation order
            let r_zetaw = field::mul(r, challenge_poly(zetaw, w), w);
            let b_actual = challenge_poly(plonk.zeta, w) + r_zetaw;

            field::equal(b.shifted_to_field(), b_actual, w)
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

    fn lagrange_commitment(
        srs: &mut SRS<Vesta>,
        d: u64,
        i: usize,
    ) -> PolyComm<GroupAffine<VestaParameters>> {
        let d = d as usize;
        let x_domain = EvaluationDomain::<Fp>::new(d).expect("invalid argument");

        srs.add_lagrange_basis(x_domain);

        let lagrange_bases = &srs.lagrange_bases[&x_domain.size()];
        lagrange_bases[i].clone()
    }

    fn lagrange(domain: (&[Boolean], &[Domains; 5]), srs: &mut SRS<Vesta>, i: usize) -> (Fq, Fq) {
        let (which_branch, domains) = domain;

        domains
            .iter()
            .map(|d| {
                let d = 2u64.pow(d.h.log2_size() as u32);
                match lagrange_commitment(srs, d, i).unshifted.as_slice() {
                    &[GroupAffine { x, y, .. }] => (x, y),
                    _ => unreachable!(),
                }
            })
            .zip(which_branch)
            .map(|((x, y), b)| {
                let b = b.to_field::<Fq>();
                (b * x, b * y)
            })
            .reduce(|mut acc, v| {
                acc.0 += v.0;
                acc.1 += v.1;
                acc
            })
            .unwrap()
    }

    const OPS_BITS_PER_CHUNK: usize = 5;

    fn chunks_needed(num_bits: usize) -> usize {
        (num_bits + (OPS_BITS_PER_CHUNK - 1)) / OPS_BITS_PER_CHUNK
    }

    fn lagrange_with_correction(
        input_length: usize,
        domain: (&[Boolean], &[Domains; 5]),
        srs: &mut SRS<Vesta>,
        i: usize,
        w: &mut Witness<Fq>,
    ) -> (InnerCurve<Fq>, InnerCurve<Fq>) {
        let (which_branch, domains) = domain;

        let actual_shift = { OPS_BITS_PER_CHUNK * chunks_needed(input_length) };
        let pow2pow = |x: InnerCurve<Fq>, n: usize| (0..n).fold(x, |acc, _| acc.clone() + acc);

        let mut base_and_correction = |h: Domain| {
            let d = 2u64.pow(h.log2_size() as u32);
            match lagrange_commitment(srs, d, i).unshifted.as_slice() {
                &[g] => {
                    let g = InnerCurve::of_affine(g);
                    let b = pow2pow(g.clone(), actual_shift).neg();
                    (g, b)
                }
                _ => unreachable!(),
            }
        };

        let [d, ds @ ..] = domains;

        if ds.iter().all(|d2| d.h == d2.h) {
            base_and_correction(d.h)
        } else {
            let (x, y) = domains
                .iter()
                .map(|ds| base_and_correction(ds.h))
                .zip(which_branch)
                .map(|((x, y), b)| {
                    let b = b.to_field::<Fq>();
                    let x = {
                        let GroupAffine { x, y, .. } = x.to_affine();
                        make_group(b * x, b * y)
                    };
                    let y = {
                        let GroupAffine { x, y, .. } = y.to_affine();
                        make_group(b * x, b * y)
                    };
                    (x, y)
                })
                .reduce(|mut acc, v| {
                    acc.0 += &v.0;
                    acc.1 += &v.1;
                    acc
                })
                .unwrap();

            w.exists([y.y, y.x]);
            w.exists([x.y, x.x]);

            (InnerCurve::of_affine(x), InnerCurve::of_affine(y))
        }
    }

    // TODO: We might have to use F::Scalar here
    fn scale_fast2<F: FieldWitness>(
        g: GroupAffine<F::Parameters>,
        (s_div_2, s_odd): (F, Boolean),
        num_bits: usize,
        w: &mut Witness<F>,
    ) -> GroupAffine<F::Parameters> {
        use crate::proofs::witness::plonk_curve_ops::scale_fast_unpack;

        let s_div_2_bits = num_bits - 1;
        let chunks_needed = chunks_needed(s_div_2_bits);
        let actual_bits_used = chunks_needed * OPS_BITS_PER_CHUNK;

        let h = match actual_bits_used {
            255 => scale_fast_unpack::<_, _, 255>(g, ShiftedValue { shifted: s_div_2 }, w).0,
            130 => scale_fast_unpack::<_, _, 130>(g, ShiftedValue { shifted: s_div_2 }, w).0,
            n => todo!("{:?}", n),
        };

        let on_false = {
            let g_neg = g.neg();
            w.exists(g_neg.y);
            w.add_fast(h, g_neg)
        };

        w.exists_no_check(match s_odd {
            Boolean::True => h,
            Boolean::False => on_false,
        })
    }

    // TODO: We might have to use F::Scalar here
    fn scale_fast2_prime<F: FieldWitness>(
        g: GroupAffine<F::Parameters>,
        s: F,
        num_bits: usize,
        w: &mut Witness<F>,
    ) -> GroupAffine<F::Parameters> {
        let s_parts = w.exists({
            // TODO: Here `s` is a `F` but needs to be read as a `F::Scalar`
            let bigint: BigInteger256 = s.into();
            let s_odd = bigint.0[0] & 1 != 0;
            let v = if s_odd { s - F::one() } else { s };
            (v / F::from(2u64), s_odd.to_boolean())
        });

        scale_fast2(g, s_parts, num_bits, w)
    }

    fn group_map<F: FieldWitness>(x: F, w: &mut Witness<F>) -> GroupAffine<F::Parameters> {
        use crate::proofs::group_map;

        let params = group_map::bw19::Params::<F>::create();
        let (x, y) = group_map::wrap(x, &params, w);
        make_group(x, y)
    }

    pub mod split_commitments {
        use crate::proofs::witness::scalar_challenge;

        use super::*;

        #[derive(Debug)]
        pub enum Point<F: FieldWitness> {
            Finite(GroupAffine<F::Parameters>),
            MaybeFinite(CircuitVar<Boolean>, GroupAffine<F::Parameters>),
        }

        impl<F: FieldWitness> Point<F> {
            fn finite(&self) -> CircuitVar<Boolean> {
                match self {
                    Point::Finite(_) => CircuitVar::Constant(Boolean::True),
                    Point::MaybeFinite(b, _) => b.clone(),
                }
            }

            fn add(
                &self,
                q: GroupAffine<F::Parameters>,
                w: &mut Witness<F>,
            ) -> GroupAffine<F::Parameters> {
                match self {
                    Point::Finite(p) => w.add_fast(*p, q),
                    Point::MaybeFinite(_, _) => todo!(),
                }
            }

            fn underlying(&self) -> GroupAffine<F::Parameters> {
                match self {
                    Point::Finite(p) => p.clone(),
                    Point::MaybeFinite(_, p) => p.clone(),
                }
            }
        }

        #[derive(Debug)]
        pub struct CurveOpt<F: FieldWitness> {
            pub point: GroupAffine<F::Parameters>,
            pub non_zero: CircuitVar<Boolean>,
        }

        pub fn combine<F: FieldWitness>(
            batch: PcsBatch,
            xi: [u64; 2],
            without_bound: &[(CircuitVar<Boolean>, Point<F>)],
            with_bound: &[()],
            w: &mut Witness<F>,
        ) -> GroupAffine<F::Parameters> {
            let CurveOpt { point, non_zero } = PcsBatch::combine_split_commitments::<F, _, _>(
                |(keep, p), w| CurveOpt {
                    non_zero: keep.and(&p.finite(), w),
                    point: p.underlying(),
                },
                |acc, xi, (keep, p), w| {
                    let on_acc_non_zero = {
                        let xi: F = u64_to_field(&xi);
                        p.add(scalar_challenge::endo::<F, F, 128>(acc.point, xi, w), w)
                    };

                    let point = match keep.as_boolean() {
                        Boolean::True => match acc.non_zero.as_boolean() {
                            Boolean::True => on_acc_non_zero,
                            Boolean::False => p.underlying(),
                        },
                        Boolean::False => acc.point,
                    };

                    if let CircuitVar::Var(_) = keep {
                        w.exists_no_check(point);
                    }

                    let non_zero = {
                        let v = p.finite().or(&acc.non_zero, w);
                        keep.and(&v, w)
                    };

                    CurveOpt { point, non_zero }
                },
                xi,
                without_bound,
                with_bound,
                w,
            );
            point
        }
    }

    fn bullet_reduce(
        sponge: &mut Sponge<Fq, mina_poseidon::constants::PlonkSpongeConstantsKimchi>,
        gammas: &[(GroupAffine<VestaParameters>, GroupAffine<VestaParameters>)],
        w: &mut Witness<Fq>,
    ) -> (GroupAffine<VestaParameters>, Vec<Fq>) {
        type S = Sponge<Fq, mina_poseidon::constants::PlonkSpongeConstantsKimchi>;

        let absorb_curve =
            |c: &GroupAffine<VestaParameters>, sponge: &mut S, w: &mut Witness<Fq>| {
                let GroupAffine { x, y, .. } = c;
                sponge.absorb(&[*x, *y], w);
            };

        let prechallenges = gammas
            .iter()
            .map(|gamma_i| {
                absorb_curve(&gamma_i.0, sponge, w);
                absorb_curve(&gamma_i.1, sponge, w);
                lowest_128_bits(sponge.squeeze(w), false, w)
            })
            .collect::<Vec<_>>();

        let mut term_and_challenge =
            |(l, r): &(GroupAffine<VestaParameters>, GroupAffine<VestaParameters>), pre: Fq| {
                let left_term = scalar_challenge::endo_inv::<Fq, Fq, 128>(*l, pre, w);
                let right_term = scalar_challenge::endo::<Fq, Fq, 128>(*r, pre, w);
                (w.add_fast(left_term, right_term), pre)
            };

        let (terms, challenges): (Vec<_>, Vec<_>) = gammas
            .iter()
            .zip(prechallenges)
            .map(|(c, pre)| term_and_challenge(c, pre))
            .unzip();

        (
            terms
                .into_iter()
                .reduce(|acc, v| w.add_fast(acc, v))
                .unwrap(),
            challenges,
        )
    }

    fn equal_g<F: FieldWitness>(
        g1: GroupAffine<F::Parameters>,
        g2: GroupAffine<F::Parameters>,
        w: &mut Witness<F>,
    ) -> Boolean {
        let g1: Vec<F> = g1.to_field_elements_owned();
        let g2: Vec<F> = g2.to_field_elements_owned();

        let equals = g1
            .into_iter()
            .zip(g2)
            .map(|(f1, f2)| field::equal(f1, f2, w))
            .collect::<Vec<_>>();
        Boolean::all(&equals, w)
    }

    struct CheckBulletProofParams<'a> {
        pcs_batch: PcsBatch,
        sponge: Sponge<Fq, mina_poseidon::constants::PlonkSpongeConstantsKimchi>,
        xi: [u64; 2],
        advice: &'a Advice<Fq>,
        openings_proof: &'a OpeningProof<Vesta>,
        srs: &'a SRS<GroupAffine<VestaParameters>>,
        polynomials: (
            Vec<(CircuitVar<Boolean>, split_commitments::Point<Fq>)>,
            Vec<()>,
        ),
    }

    fn check_bulletproof(
        params: CheckBulletProofParams,
        w: &mut Witness<Fq>,
    ) -> (Boolean, Vec<Fq>) {
        let scale = scale_fast::<Fq, Fp, OTHER_FIELD_PACKED_CONSTANT_SIZE_IN_BITS>;

        let CheckBulletProofParams {
            pcs_batch,
            mut sponge,
            xi,
            advice,
            openings_proof,
            srs,
            polynomials,
        } = params;

        let OpeningProof {
            lr,
            delta,
            z1,
            z2,
            sg,
        } = openings_proof;

        let combined_inner_product: Fq = {
            let bigint: BigInteger256 = advice.combined_inner_product.shifted.into();
            bigint.into()
        };
        sponge.absorb(&[combined_inner_product], w);

        let u = {
            let t = sponge.squeeze(w);
            group_map(t, w)
        };

        let combined_polynomial = {
            let (without_degree_bound, with_degree_bound) = &polynomials;
            split_commitments::combine(pcs_batch, xi, without_degree_bound, with_degree_bound, w)
        };

        let (lr_prod, challenges) = bullet_reduce(&mut sponge, lr, w);

        let p_prime = {
            w.exists_no_check(u); // Made by `plonk_curve_ops.seal` in `scale_fast`
            let uc = scale(u, advice.combined_inner_product.clone(), w);
            w.add_fast(combined_polynomial, uc)
        };

        type S = Sponge<Fq, mina_poseidon::constants::PlonkSpongeConstantsKimchi>;
        let absorb_curve =
            |c: &GroupAffine<VestaParameters>, sponge: &mut S, w: &mut Witness<Fq>| {
                let GroupAffine { x, y, .. } = c;
                sponge.absorb(&[*x, *y], w);
            };

        let q = w.add_fast(p_prime, lr_prod);

        absorb_curve(delta, &mut sponge, w);
        let c = lowest_128_bits(sponge.squeeze(w), false, w);

        let lhs = {
            let cq = scalar_challenge::endo::<Fq, Fq, 128>(q, c, w);
            w.add_fast(cq, *delta)
        };

        let rhs = {
            let b_u = {
                w.exists_no_check(u); // Made by `plonk_curve_ops.seal` in `scale_fast`
                scale(u, advice.b.clone(), w)
            };
            let z_1_g_plus_b_u = scale(w.add_fast(*sg, b_u), ShiftedValue::of_field(*z1), w);
            let z2_h = scale(srs.h, ShiftedValue::of_field(*z2), w);
            w.add_fast(z_1_g_plus_b_u, z2_h)
        };

        (equal_g(lhs, rhs, w), challenges)
    }

    #[derive(Debug)]
    pub struct Advice<F: FieldWitness> {
        pub b: ShiftedValue<F::Scalar>,
        pub combined_inner_product: ShiftedValue<F::Scalar>,
    }

    pub struct IncrementallyVerifyProofParams<'a> {
        pub actual_proofs_verified_mask: Vec<Boolean>,
        pub step_domains: &'a [Domains; 5],
        pub verification_key: &'a PlonkVerificationKeyEvals<Fq>,
        pub srs: Arc<SRS<Vesta>>,
        pub xi: &'a [u64; 2],
        pub sponge: OptSponge<Fq>,
        pub public_input: Vec<Packed<Boolean>>,
        pub sg_old: Vec<InnerCurve<Fq>>,
        pub advice: Advice<Fq>,
        pub messages: &'a kimchi::proof::ProverCommitments<Vesta>,
        pub which_branch: Vec<Boolean>,
        pub openings_proof: &'a OpeningProof<Vesta>,
        pub plonk: &'a Plonk<Fp>,
    }

    pub fn incrementally_verify_proof(
        params: IncrementallyVerifyProofParams,
        w: &mut Witness<Fq>,
    ) -> (Fq, (Boolean, Vec<Fq>)) {
        let IncrementallyVerifyProofParams {
            actual_proofs_verified_mask,
            step_domains,
            verification_key,
            srs,
            xi,
            mut sponge,
            public_input,
            sg_old,
            advice,
            messages,
            which_branch,
            openings_proof,
            plonk,
        } = params;

        let challenge =
            |s: &mut OptSponge<Fq>, w: &mut Witness<Fq>| lowest_128_bits(s.squeeze(w), true, w);
        let scalar_challenge =
            |s: &mut OptSponge<Fq>, w: &mut Witness<Fq>| lowest_128_bits(s.squeeze(w), false, w);

        let mut absorb_curve =
            |b: &CircuitVar<Boolean>, c: &InnerCurve<Fq>, sponge: &mut OptSponge<Fq>| {
                let GroupAffine { x, y, .. } = c.to_affine();
                sponge.absorb((*b, x));
                sponge.absorb((*b, y));
            };

        dbg!(&actual_proofs_verified_mask);

        let mut srs = (*srs).clone();
        let sg_old = actual_proofs_verified_mask
            .iter()
            .map(|b| CircuitVar::Var(*b))
            .zip(&sg_old)
            .collect::<Vec<_>>();

        let sample = challenge;
        let sample_scalar = scalar_challenge;

        let index_digest = {
            use crate::proofs::witness::poseidon::Sponge;
            use mina_poseidon::pasta::fq_kimchi::static_params;

            let mut sponge = Sponge::<Fq, Constants>::new(static_params());
            let fields = verification_key.to_field_elements_owned();
            sponge.absorb2(&fields, w);
            sponge.squeeze(w)
        };

        sponge.absorb((CircuitVar::Constant(Boolean::True), index_digest));

        for (b, v) in &sg_old {
            absorb_curve(&b, *v, &mut sponge);
        }

        let x_hat = {
            let domain = (which_branch.as_slice(), step_domains);
            let public_input = public_input.iter().flat_map(|v| {
                // TODO: Do not use `vec!` here
                match v {
                    Packed::Field((x, b)) => vec![
                        Packed::Field((*x, 255)),
                        Packed::Field((CircuitVar::Var(b.to_field::<Fq>()), 1)),
                    ],
                    Packed::PackedBits(x, n) => vec![Packed::Field((*x, *n))],
                }
            });

            let (constant_part, non_constant_part): (Vec<_>, Vec<_>) =
                public_input.enumerate().partition_map(|(i, t)| {
                    use itertools::Either::{Left, Right};
                    match t {
                        Packed::Field((CircuitVar::Constant(c), _)) => Left(if c.is_zero() {
                            None
                        } else if c.is_one() {
                            Some(lagrange(domain, &mut srs, i))
                        } else {
                            todo!()
                        }),
                        Packed::Field(x) => Right((i, x)),
                        _ => unreachable!(),
                    }
                });

            #[derive(Debug)]
            enum CondOrAdd {
                CondAdd(Boolean, (Fq, Fq)),
                AddWithCorrection((CircuitVar<Fq>, usize), (InnerCurve<Fq>, InnerCurve<Fq>)),
            }

            let terms = non_constant_part
                .into_iter()
                .map(|(i, x)| match x {
                    (b, 1) => CondOrAdd::CondAdd(
                        Boolean::of_field(b.as_field()),
                        lagrange(domain, &mut srs, i),
                    ),
                    (x, n) => CondOrAdd::AddWithCorrection(
                        (x, n),
                        (lagrange_with_correction(n, domain, &mut srs, i, w)),
                    ),
                })
                .collect::<Vec<_>>();

            let correction = terms
                .iter()
                .filter_map(|term| match term {
                    CondOrAdd::CondAdd(_, _) => None,
                    CondOrAdd::AddWithCorrection(_, (_, corr)) => Some(corr.to_affine()),
                })
                .reduce(|acc, v| w.add_fast(acc, v))
                .unwrap();

            let init = constant_part
                .into_iter()
                .filter_map(identity)
                .fold(correction, |acc, (x, y)| w.add_fast(acc, make_group(x, y)));

            terms
                .into_iter()
                .enumerate()
                .fold(init, |acc, (i, term)| match term {
                    CondOrAdd::CondAdd(b, (x, y)) => {
                        let g = w.exists_no_check(make_group(x, y));
                        let on_true = w.add_fast(g, acc);

                        w.exists_no_check(match b {
                            Boolean::True => on_true,
                            Boolean::False => acc,
                        })
                    }
                    CondOrAdd::AddWithCorrection((x, num_bits), (g, _)) => {
                        let v = scale_fast2_prime(g.to_affine(), x.as_field(), num_bits, w);
                        w.add_fast(acc, v)
                    }
                })
                .neg()
        };

        let x_hat = {
            w.exists(x_hat.y); // Because of `.neg()` above
            w.add_fast(x_hat, srs.h)
        };

        absorb_curve(
            &CircuitVar::Constant(Boolean::True),
            &InnerCurve::of_affine(x_hat),
            &mut sponge,
        );

        let w_comm = &messages.w_comm;

        for w in w_comm.iter().flat_map(|w| &w.unshifted) {
            absorb_curve(
                &CircuitVar::Constant(Boolean::True),
                &InnerCurve::of_affine(w.clone()),
                &mut sponge,
            );
        }

        println!("START SAMPLES\n");
        let beta = sample(&mut sponge, w);
        let gamma = sample(&mut sponge, w);

        dbg!(beta, gamma);

        let z_comm = &messages.z_comm;
        for z in z_comm.unshifted.iter() {
            absorb_curve(
                &CircuitVar::Constant(Boolean::True),
                &InnerCurve::of_affine(z.clone()),
                &mut sponge,
            );
        }

        eprintln!("BEFORE ALPHA");
        let alpha = sample_scalar(&mut sponge, w);
        dbg!(alpha);

        let t_comm = &messages.t_comm;
        for t in t_comm.unshifted.iter() {
            absorb_curve(
                &CircuitVar::Constant(Boolean::True),
                &InnerCurve::of_affine(t.clone()),
                &mut sponge,
            );
        }

        eprintln!("BEFORE ZETA");
        let zeta = sample_scalar(&mut sponge, w);
        dbg!(zeta);

        let mut sponge = {
            use crate::proofs::opt_sponge::SpongeState as OptSpongeState;
            use crate::proofs::witness::poseidon::Sponge;
            use mina_poseidon::pasta::fq_kimchi::static_params;
            use mina_poseidon::poseidon::SpongeState;

            let OptSpongeState::Squeezed(n_squeezed) = sponge.sponge_state else {
                // We just called `sample_scalar`
                panic!("OCaml panics too")
            };
            let mut sponge = Sponge::<Fq, Constants>::new_with_state(sponge.state, static_params());
            sponge.sponge_state = SpongeState::Squeezed(n_squeezed);
            sponge
        };

        let sponge_before_evaluations = sponge.clone();
        let sponge_digest_before_evaluations = sponge.squeeze(w);

        dbg!(sponge_digest_before_evaluations);

        let sigma_comm_init = &verification_key.sigma[..PERMUTS_MINUS_1_ADD_N1];
        let ft_comm = ft_comm(alpha, plonk, t_comm, verification_key, w);

        let bulletproof_challenges = {
            const NUM_COMMITMENTS_WITHOUT_DEGREE_BOUND: usize = 45;

            let without_degree_bound = {
                let sg_old = sg_old.iter().map(|(b, v)| (*b, v.to_affine()));
                let rest = [x_hat, ft_comm]
                    .into_iter()
                    .chain(z_comm.unshifted.iter().cloned())
                    .chain([
                        verification_key.generic.to_affine(),
                        verification_key.psm.to_affine(),
                        verification_key.complete_add.to_affine(),
                        verification_key.mul.to_affine(),
                        verification_key.emul.to_affine(),
                        verification_key.endomul_scalar.to_affine(),
                    ])
                    .chain(w_comm.iter().flat_map(|w| w.unshifted.iter().cloned()))
                    .chain(verification_key.coefficients.iter().map(|v| v.to_affine()))
                    .chain(sigma_comm_init.iter().map(|v| v.to_affine()))
                    .map(|v| (CircuitVar::Constant(Boolean::True), v));
                sg_old.chain(rest)
            };

            use split_commitments::Point;

            let polynomials = without_degree_bound
                .map(|(keep, x)| (keep, Point::Finite(x)))
                .collect::<Vec<_>>();

            let pcs_batch = PcsBatch::create(
                MAX_PROOFS_VERIFIED_N as usize + NUM_COMMITMENTS_WITHOUT_DEGREE_BOUND,
            );
            let xi = *xi;
            let advice = &advice;
            let srs = &srs;

            check_bulletproof(
                CheckBulletProofParams {
                    pcs_batch,
                    sponge,
                    xi,
                    advice,
                    openings_proof,
                    srs,
                    polynomials: (polynomials, vec![]),
                },
                w,
            )
        };

        (sponge_digest_before_evaluations, bulletproof_challenges)
    }
}

fn wrap_domain_indices() -> [Fq; 2] {
    // TODO
    [Fq::one(), Fq::one()]
}

mod one_hot_vector {
    use super::*;

    pub fn of_index(i: Fq, length: u64, w: &mut Witness<Fq>) -> Vec<Boolean> {
        let mut v = (0..length)
            .rev()
            .map(|j| field::equal(Fq::from(j), i, w))
            .collect::<Vec<_>>();
        Boolean::assert_any(&v, w);
        v.reverse();
        v
    }
}

impl Check<Fq> for poly_commitment::evaluation_proof::OpeningProof<Vesta> {
    fn check(&self, w: &mut Witness<Fq>) {
        let Self {
            lr,
            delta,
            z1,
            z2,
            sg,
        } = self;

        let to_curve = |c: &Vesta| InnerCurve::<Fq>::of_affine(c.clone());
        let shift = |f: Fp| <Fp as FieldWitness>::Shifting::of_field(f);

        lr.iter().for_each(|(a, b)| {
            to_curve(a).check(w);
            to_curve(b).check(w);
        });
        shift(*z1).check(w);
        shift(*z2).check(w);
        to_curve(delta).check(w);
        to_curve(sg).check(w);
    }
}

impl ToFieldElements<Fq> for poly_commitment::evaluation_proof::OpeningProof<Vesta> {
    fn to_field_elements(&self, fields: &mut Vec<Fq>) {
        let Self {
            lr,
            delta,
            z1,
            z2,
            sg,
        } = self;

        let push = |c: &Vesta, fields: &mut Vec<Fq>| {
            let GroupAffine { x, y, .. } = c;
            x.to_field_elements(fields);
            y.to_field_elements(fields);
        };
        let shift = |f: Fp| <Fp as FieldWitness>::Shifting::of_field(f);

        lr.iter().for_each(|(a, b)| {
            push(a, fields);
            push(b, fields);
        });
        shift(*z1).shifted.to_field_elements(fields);
        shift(*z2).shifted.to_field_elements(fields);
        push(delta, fields);
        push(sg, fields);
    }
}

struct CommitmentLengths {
    w: [Fq; COLUMNS],
    z: Fq,
    t: Fq,
}

impl CommitmentLengths {
    fn create() -> Self {
        Self {
            w: [Fq::one(); COLUMNS],
            z: Fq::one(),
            t: 7u64.into(),
        }
    }
}

impl ToFieldElements<Fq> for kimchi::proof::ProverCommitments<Vesta> {
    fn to_field_elements(&self, fields: &mut Vec<Fq>) {
        let Self {
            w_comm,
            z_comm,
            t_comm,
            lookup,
        } = self;

        let mut push_poly = |poly: &PolyComm<Vesta>| {
            let PolyComm { unshifted, shifted } = poly;
            for GroupAffine { x, y, .. } in unshifted {
                x.to_field_elements(fields);
                y.to_field_elements(fields);
            }
            assert!(shifted.is_none());
        };
        for poly in w_comm.iter().chain([z_comm, t_comm]) {
            push_poly(poly);
        }
        assert!(lookup.is_none());
    }
}

impl Check<Fq> for kimchi::proof::ProverCommitments<Vesta> {
    fn check(&self, w: &mut Witness<Fq>) {
        let Self {
            w_comm,
            z_comm,
            t_comm,
            lookup,
        } = self;

        let mut check_poly = |poly: &PolyComm<Vesta>| {
            let PolyComm { unshifted, shifted } = poly;
            for affine in unshifted {
                InnerCurve::of_affine(affine.clone()).check(w);
            }
        };

        for poly in w_comm.iter().chain([z_comm, t_comm]) {
            check_poly(poly);
        }
    }
}

pub enum Packed<T> {
    Field((CircuitVar<Fq>, T)),
    PackedBits(CircuitVar<Fq>, usize),
}

impl<T: std::fmt::Debug> std::fmt::Debug for Packed<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field((a, b)) => f.write_fmt(format_args!("Field({:?}, {:?})", a, b)),
            Self::PackedBits(a, b) => f.write_fmt(format_args!("PackedBits({:?}, {:?})", a, b)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CircuitVar<F> {
    Var(F),
    Constant(F),
}

impl<F: FieldWitness> CircuitVar<F> {
    pub fn as_field(&self) -> F {
        match self {
            CircuitVar::Var(f) => *f,
            CircuitVar::Constant(f) => *f,
        }
    }

    fn scale(x: CircuitVar<F>, s: F) -> CircuitVar<F> {
        use CircuitVar::*;

        if s.is_zero() {
            Constant(F::zero())
        } else if s.is_one() {
            x
        } else {
            match x {
                Constant(x) => Constant(x * s),
                Var(x) => Var(x * s),
            }
        }
    }

    fn mul(&self, other: &Self, w: &mut Witness<F>) -> CircuitVar<F> {
        use CircuitVar::*;

        let (x, y) = (*self, *other);
        match (x, y) {
            (Constant(x), Constant(y)) => Constant(x * y),
            (Constant(x), _) => Self::scale(y, x),
            (_, Constant(y)) => Self::scale(x, y),
            (Var(x), Var(y)) => Var(w.exists(x * y)),
        }
    }

    // pub fn mul(&self, other: &Self, w: &mut Witness<F>) -> Self {
    //     use CircuitVar::*;

    //     match (self, other) {
    //         (Constant(x), Constant(y)) => Constant(*x * *y),
    //         (Var(x), Var(y)) => Var(field::mul(*x, *y, w)),
    //         (Var(x), Constant(y)) => Var(*x * *y),
    //         (Constant(x), Var(y)) => Var(*x * *y),
    //     }
    // }

    fn equal(x: &Self, y: &Self, w: &mut Witness<F>) -> CircuitVar<Boolean> {
        match (x, y) {
            (CircuitVar::Constant(x), CircuitVar::Constant(y)) => {
                let eq = if x == y {
                    Boolean::True
                } else {
                    Boolean::False
                };
                CircuitVar::Constant(eq)
            }
            _ => {
                let x = x.as_field();
                let y = y.as_field();
                CircuitVar::Var(field::equal(x, y, w))
            }
        }
    }
}

impl<T> CircuitVar<T> {
    pub fn is_const(&self) -> bool {
        match self {
            CircuitVar::Var(_) => false,
            CircuitVar::Constant(_) => true,
        }
    }

    pub fn map<Fun, V>(&self, fun: Fun) -> CircuitVar<V>
    where
        Fun: Fn(&T) -> V,
    {
        match self {
            CircuitVar::Var(v) => CircuitVar::Var(fun(v)),
            CircuitVar::Constant(v) => CircuitVar::Constant(fun(v)),
        }
    }
}

impl CircuitVar<Boolean> {
    pub fn as_boolean(&self) -> Boolean {
        match self {
            CircuitVar::Var(b) => *b,
            CircuitVar::Constant(b) => *b,
        }
    }

    fn as_bool(&self) -> bool {
        match self {
            CircuitVar::Var(b) => b.as_bool(),
            CircuitVar::Constant(b) => b.as_bool(),
        }
    }

    fn as_cvar<F: FieldWitness>(&self) -> CircuitVar<F> {
        match self {
            CircuitVar::Var(b) => CircuitVar::Var(b.to_field::<F>()),
            CircuitVar::Constant(b) => CircuitVar::Constant(b.to_field::<F>()),
        }
    }

    pub fn as_field<F: FieldWitness>(&self) -> F {
        todo!()
    }

    pub fn and<F: FieldWitness>(&self, other: &Self, w: &mut Witness<F>) -> Self {
        self.as_cvar().mul(&other.as_cvar(), w).map(|v| {
            // TODO: Should we check for `is_one` or `is_zero` here ? To match OCaml behavior
            if v.is_one() {
                Boolean::True
            } else {
                Boolean::False
            }
        })
    }

    fn boolean_sum<F: FieldWitness>(x: &[Self]) -> CircuitVar<F> {
        let sum = x.iter().fold(0u64, |acc, b| {
            acc + match b.as_boolean() {
                Boolean::True => 1,
                Boolean::False => 0,
            }
        });
        if x.iter().all(|x| matches!(x, CircuitVar::Constant(_))) {
            CircuitVar::Constant(F::from(sum))
        } else {
            CircuitVar::Var(F::from(sum))
        }
    }

    pub fn any<F: FieldWitness>(x: &[Self], w: &mut Witness<F>) -> CircuitVar<Boolean> {
        match x {
            [] => CircuitVar::Constant(Boolean::False),
            [b1] => *b1,
            [b1, b2] => b1.or(b2, w),
            bs => {
                let sum = Self::boolean_sum(bs);
                // let sum = bs.iter().fold(0u64, |acc, b| {
                //     acc + match b.as_boolean() {
                //         Boolean::True => 1,
                //         Boolean::False => 0,
                //     }
                // });
                CircuitVar::equal(&sum, &CircuitVar::Constant(F::zero()), w).map(Boolean::neg)
            }
        }
    }

    pub fn all<F: FieldWitness>(x: &[Self], w: &mut Witness<F>) -> CircuitVar<Boolean> {
        eprintln!("all={:?}", x);
        match x {
            [] => CircuitVar::Constant(Boolean::True),
            [b1] => *b1,
            [b1, b2] => b1.and(b2, w),
            bs => {
                let sum = Self::boolean_sum(bs);
                // eprintln!("all bs={:?}", bs);
                let len = F::from(bs.len() as u64);
                // let sum = bs.iter().fold(0u64, |acc, b| {
                //     acc + match b.as_boolean() {
                //         Boolean::True => 1,
                //         Boolean::False => 0,
                //     }
                // });
                CircuitVar::equal(&CircuitVar::Constant(len), &sum, w)
            }
        }
    }

    pub fn neg(&self) -> Self {
        self.map(Boolean::neg)
    }

    pub fn or<F: FieldWitness>(&self, other: &Self, w: &mut Witness<F>) -> Self {
        use CircuitVar::{Constant, Var};

        eprintln!("or self={:?} other={:?}", self, other);

        let both_false = self.neg().and(&other.neg(), w);
        both_false.neg()

        // self.neg().and(&other.neg(), w).neg()

        // todo!()

        // match (self, other) {
        //     (Constant(a), Constant(b)) => {
        //         Constant(a.const_or(b))
        //     }
        //     (Var(a), Var(b)) => Var(a.or(b, w)),
        //     (Constant(a), b) => Constant(a.const_or(&b.as_boolean())),
        //     (Constant(a), Var(b)) => Constant(a.const_or(&b)),
        //     (Var(a), Constant(b)) => Var(a.const_or(&b)),
        //     // (a, b) => CircuitVar::Constant(a.as_boolean().const_or(&b.as_boolean())),
        // }
    }

    // let ( && ) (x : var) (y : var) : var Checked.t =
    //   Printf.eprintf "[snarky.utils] this &&\n%!" ;
    //   let res = Checked.map ~f:create (mul (x :> Cvar.t) (y :> Cvar.t)) in
    //   Printf.eprintf "[snarky.utils] this && DONE\n%!" ;
    //   res
    //   [@@inline never]

    // let ( &&& ) = ( && )

    // let ( || ) x y =
    //   let open Let_syntax in
    //   let%map both_false = (not x) && not y in
    //   let res = not both_false in
    //   Printf.eprintf
    //     !"[snarky.utils] or x=%{sexp: string option} y=%{sexp: string option} \
    //       both_false=%{sexp: string option} res=%{sexp: string option}\n\
    //       %!"
    //     (my_eval_cvar Obj.(magic @@ repr x))
    //     (my_eval_cvar Obj.(magic @@ repr y))
    //     (my_eval_cvar Obj.(magic @@ repr both_false))
    //     (my_eval_cvar Obj.(magic @@ repr res)) ;
    //   res

    // or x=("Add(0)") y=("Constant(1)") both_false=("Constant(0)") res=("Constant(1)")

    pub fn lxor<F: FieldWitness>(&self, other: &Self, w: &mut Witness<F>) -> Self {
        match (self, other) {
            (CircuitVar::Var(a), CircuitVar::Var(b)) => CircuitVar::Var(a.lxor(b, w)),
            (CircuitVar::Constant(a), CircuitVar::Constant(b)) => {
                CircuitVar::Constant(a.const_lxor(b))
            }
            (a, b) => CircuitVar::Var(a.as_boolean().const_lxor(&b.as_boolean())),
        }
    }
}

fn pack_statement(
    statement: &StepStatementWithHash,
    messages_for_next_step_proof_hash: &[u64; 4],
    w: &mut Witness<Fq>,
) -> Vec<Packed<Boolean>> {
    let StepStatementWithHash {
        proof_state:
            StepProofState {
                unfinalized_proofs,
                messages_for_next_step_proof,
            },
        messages_for_next_wrap_proof,
    } = statement;

    let mut packed = Vec::<Packed<_>>::with_capacity(300);

    let var = CircuitVar::Var;
    // let cons = CircuitVar::Constant;

    let mut split = |f: Fq| {
        let (f, b) = split_field(f, w);
        (CircuitVar::Var(f), b)
    };

    for unfinalized in unfinalized_proofs {
        let Unfinalized {
            deferred_values:
                super::unfinalized::DeferredValues {
                    plonk:
                        Plonk {
                            alpha,
                            beta,
                            gamma,
                            zeta,
                            zeta_to_srs_length,
                            zeta_to_domain_size,
                            // vbmul,
                            // complete_add,
                            // endomul,
                            // endomul_scalar,
                            perm,
                            lookup,
                        },
                    combined_inner_product,
                    b,
                    xi,
                    bulletproof_challenges,
                },
            should_finalize,
            sponge_digest_before_evaluations,
        } = unfinalized;

        // Fq
        {
            packed.push(Packed::Field(split(combined_inner_product.shifted)));
            packed.push(Packed::Field(split(b.shifted)));
            packed.push(Packed::Field(split(zeta_to_srs_length.shifted)));
            packed.push(Packed::Field(split(zeta_to_domain_size.shifted)));
            // packed.push(Packed::Field(split(vbmul.shifted)));
            // packed.push(Packed::Field(split(complete_add.shifted)));
            // packed.push(Packed::Field(split(endomul.shifted)));
            // packed.push(Packed::Field(split(endomul_scalar.shifted)));
            packed.push(Packed::Field(split(perm.shifted)));
        }

        // Digest
        {
            packed.push(Packed::PackedBits(
                var(u64_to_field(sponge_digest_before_evaluations)),
                255,
            ));
        }

        // Challenge
        {
            packed.push(Packed::PackedBits(var(u64_to_field(beta)), 128));
            packed.push(Packed::PackedBits(var(u64_to_field(gamma)), 128));
        }

        // Scalar challenge
        {
            packed.push(Packed::PackedBits(var(u64_to_field(alpha)), 128));
            packed.push(Packed::PackedBits(var(u64_to_field(zeta)), 128));
            packed.push(Packed::PackedBits(var(u64_to_field(xi)), 128));
        }

        packed.extend(
            bulletproof_challenges
                .iter()
                .map(|v| Packed::PackedBits(var(u64_to_field::<Fq, 2>(v)), 128)),
        );

        // Bool
        {
            packed.push(Packed::PackedBits(var(Fq::from(*should_finalize)), 1));
        }

        // TODO: Check how that padding works
        // (0..9).for_each(|_| {
        //     packed.push(Packed::PackedBits(cons(Fq::zero()), 1));
        // });
        // packed.push(Packed::PackedBits(cons(Fq::zero()), 128));
        // (0..8).for_each(|i| {
        //     dbg!(i);
        //     packed.push(Packed::Field(split(Fq::zero())));
        // });
    }

    packed.push(Packed::PackedBits(
        var(u64_to_field(messages_for_next_step_proof_hash)),
        255,
    ));

    for msg in messages_for_next_wrap_proof {
        packed.push(Packed::PackedBits(var(u64_to_field(msg)), 255));
    }

    packed
}

fn split_field(x: Fq, w: &mut Witness<Fq>) -> (Fq, Boolean) {
    let n: BigInteger256 = x.into();

    let is_odd = n.0[0] & 1 != 0;

    let y = if is_odd { x - Fq::one() } else { x };
    let y = y / Fq::from(2u64);

    w.exists((y, is_odd.to_boolean()))
}

pub struct WrapMainParams<'a> {
    pub step_statement: StepStatement,
    pub next_statement: WrapStatement,
    pub messages_for_next_wrap_proof_padded: Vec<MessagesForNextWrapProof>,
    pub which_index: u64,
    pub pi_branches: u64,
    pub step_widths: [u64; 5],
    pub step_domains: [Domains; 5],
    pub messages_for_next_step_proof_hash: [u64; 4],
    pub prev_evals: &'a [AllEvals<Fq>],
    pub proof: &'a ProverProof<Vesta>,
    pub prover_index: &'a kimchi::prover_index::ProverIndex<Vesta>,
}

fn wrap_main(params: &WrapMainParams, w: &mut Witness<Fq>) {
    let WrapMainParams {
        step_statement,
        next_statement,
        messages_for_next_wrap_proof_padded,
        which_index,
        pi_branches,
        step_widths,
        step_domains,
        messages_for_next_step_proof_hash,
        prev_evals,
        proof,
        prover_index,
    } = params;

    let which_branch = w.exists(Fq::from(*which_index));

    let branches = pi_branches;

    let which_branch = one_hot_vector::of_index(which_branch, *branches, w);

    // let which_branch = {
    //     let mut v = (0..branches)
    //         .rev()
    //         .map(|j| field::equal(Fq::from(j), which_branch, w))
    //         .collect::<Vec<_>>();
    //     Boolean::assert_any(&v, w);
    //     v.reverse();
    //     v
    // };

    let first_zero = pseudo::choose(&which_branch, &step_widths[..]);

    let actual_proofs_verified_mask = {
        let mut vector = ones_vector(first_zero, MAX_PROOFS_VERIFIED_N, w);
        vector.reverse();
        vector
    };

    let domain_log2 = pseudo::choose(
        &which_branch,
        &step_domains
            .iter()
            .map(|ds| ds.h.log2_size())
            .collect::<Vec<_>>(),
    );

    exists_prev_statement(step_statement, *messages_for_next_step_proof_hash, w);

    let step_plonk_index = wrap_verifier::choose_key(prover_index, w);

    let prev_step_accs = w.exists({
        let to_inner_curve = |m: &MessagesForNextWrapProof| {
            let CurveAffine(x, y) = m.challenge_polynomial_commitment.clone();
            InnerCurve::<Fq>::of_affine(make_group(x, y))
        };
        messages_for_next_wrap_proof_padded
            .iter()
            .map(to_inner_curve)
            .collect::<Vec<_>>()
    });

    let old_bp_chals = w.exists({
        messages_for_next_wrap_proof_padded
            .iter()
            .map(|m| m.old_bulletproof_challenges.clone())
            .collect::<Vec<_>>()
    });

    let new_bulletproof_challenges = {
        let evals = w.exists(*prev_evals);

        let chals = {
            let wrap_domains = {
                let all_possible_domains = wrap_verifier::all_possible_domains();
                let wrap_domain_indices = w.exists(wrap_domain_indices());

                wrap_domain_indices.map(|index| {
                    let which_branch = one_hot_vector::of_index(
                        index,
                        wrap_verifier::NUM_POSSIBLE_DOMAINS as u64,
                        w,
                    );
                    pseudo::to_domain(&which_branch, &all_possible_domains)
                })
            };

            let unfinalized_proofs = &step_statement.proof_state.unfinalized_proofs;

            dbg!(unfinalized_proofs.len());
            dbg!(old_bp_chals.len());
            dbg!(evals.len());
            dbg!(wrap_domains.len());

            unfinalized_proofs
                .iter()
                .zip(&old_bp_chals)
                .zip(evals)
                .zip(&wrap_domains)
                .map(
                    |(((unfinalized, old_bulletproof_challenges), evals), wrap_domain)| {
                        let Unfinalized {
                            deferred_values,
                            should_finalize,
                            sponge_digest_before_evaluations,
                        } = unfinalized;

                        use mina_poseidon::constants::PlonkSpongeConstantsKimchi as Constants;
                        use mina_poseidon::pasta::fq_kimchi::static_params;

                        let mut sponge =
                            crate::proofs::witness::poseidon::Sponge::<Fq, Constants>::new(
                                static_params(),
                            );
                        sponge.absorb2(&[u64_to_field(sponge_digest_before_evaluations)], w);

                        // sponge
                        // Or `Wrap_hack.Checked.pad_challenges` needs to be used
                        assert_eq!(old_bulletproof_challenges.len(), 2);

                        let (finalized, chals) = wrap_verifier::finalize_other_proof(
                            wrap_domain,
                            sponge,
                            old_bulletproof_challenges,
                            deferred_values,
                            evals,
                            w,
                        );
                        Boolean::assert_any(&[finalized, should_finalize.to_boolean().neg()], w);
                        chals
                    },
                )
                .collect::<Vec<_>>()
        };
        chals
    };

    let prev_statement = {
        // Note: We might have to use `Iterator::rev` here
        let prev_messages_for_next_wrap_proof = prev_step_accs
            .iter()
            .zip(old_bp_chals)
            .map(|(sacc, chals)| {
                MessagesForNextWrapProof {
                    challenge_polynomial_commitment: {
                        let GroupAffine { x, y, .. } = sacc.to_affine();
                        CurveAffine(x, y)
                    },
                    old_bulletproof_challenges: chals,
                }
                .hash_checked(w)
            })
            .collect::<Vec<_>>();

        StepStatementWithHash {
            proof_state: step_statement.proof_state.clone(),
            messages_for_next_wrap_proof: prev_messages_for_next_wrap_proof,
        }
    };

    let openings_proof = w.exists(&proof.proof);
    let messages = w.exists(&proof.commitments);

    let public_input = pack_statement(&prev_statement, messages_for_next_step_proof_hash, w);

    let DeferredValues {
        plonk,
        combined_inner_product,
        b,
        xi,
        bulletproof_challenges,
        branch_data,
    } = &next_statement.proof_state.deferred_values;

    let sponge = OptSponge::create();
    let params = wrap_verifier::IncrementallyVerifyProofParams {
        actual_proofs_verified_mask,
        step_domains,
        verification_key: &step_plonk_index,
        srs: prover_index.srs.clone(),
        xi,
        sponge,
        public_input,
        sg_old: prev_step_accs,
        advice: wrap_verifier::Advice {
            b: b.clone(),
            combined_inner_product: combined_inner_product.clone(),
        },
        messages,
        which_branch,
        openings_proof,
        plonk,
    };
    wrap_verifier::incrementally_verify_proof(params, w);

    MessagesForNextWrapProof {
        challenge_polynomial_commitment: {
            let GroupAffine { x, y, .. } = &openings_proof.sg;
            CurveAffine(*x, *y)
        },
        old_bulletproof_challenges: new_bulletproof_challenges
            .into_iter()
            .map(|v| v.try_into().unwrap())
            .collect(),
    }
    .hash_checked3(w);
}
