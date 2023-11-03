#![allow(unused)]

use std::array;

use ark_ff::{BigInteger256, Field};
use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

use crate::{
    proofs::{
        accumulator_check,
        public_input::plonk_checks::ShiftingValue,
        util::{challenge_polynomial, to_absorption_sequence},
        verifier_index::make_zkapp_verifier_index,
        wrap::{combined_inner_product, CombinedInnerProductParams},
        BACKEND_TICK_ROUNDS_N,
    },
    scan_state::{
        scan_state::transaction_snark::{SokDigest, Statement},
        transaction_logic::zkapp_statement::ZkappStatement,
    },
    static_params, CurveAffine, PlonkVerificationKeyEvals, Sponge, VerificationKey,
};

use super::{
    to_field_elements::ToFieldElements,
    util::{extract_bulletproof, extract_polynomial_commitment, u64_to_field},
    witness::FieldWitness,
    wrap::evals_of_split_evals,
    VerifierSRS,
};
use kimchi::{
    circuits::polynomials::permutation::eval_zk_polynomial,
    error::VerifyError,
    mina_curves::pasta::Pallas,
    proof::{PointEvaluations, ProofEvaluations},
};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::{
    bigint::BigInt,
    pseq::PaddedSeq,
    v2::{
        CompositionTypesDigestConstantStableV1, MinaBlockHeaderStableV2,
        PicklesProofProofsVerified2ReprStableV2,
        PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof,
        PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof,
        PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
        PicklesProofProofsVerified2ReprStableV2StatementFp,
        PicklesProofProofsVerified2ReprStableV2StatementProofState,
        PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues,
        PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags,
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A,
        TransactionSnarkProofStableV2,
    },
};

use super::{prover::make_prover, ProverProof, VerifierIndex};

use super::public_input::{
    messages::{MessagesForNextStepProof, MessagesForNextWrapProof},
    plonk_checks::{derive_plonk, PlonkMinimal, ScalarsEnv, ShiftedValue},
    prepared_statement::{DeferredValues, Plonk, PreparedStatement, ProofState},
    scalar_challenge::{endo_fp, endo_fq, ScalarChallenge},
};

use super::public_input::plonk_checks::InCircuit;

#[cfg(target_family = "wasm")]
#[cfg(test)]
mod wasm {
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);
}

fn to_shifted_value<F>(
    value: &PicklesProofProofsVerified2ReprStableV2StatementFp,
) -> ShiftedValue<F>
where
    F: Field,
{
    let PicklesProofProofsVerified2ReprStableV2StatementFp::ShiftedValue(ref value) = value;

    let shifted: F = value.to_field();
    ShiftedValue { shifted }
}

struct DataForPublicInput {
    evals: ProofEvaluations<[Fp; 2]>,
    minimal: PlonkMinimal<Fp>,
    domain_log2: u8,
}

fn extract_data_for_public_input(
    proof: &PicklesProofProofsVerified2ReprStableV2,
) -> DataForPublicInput {
    let evals = &proof.prev_evals.evals.evals;

    let to_fp = |(a, b): &(Vec<BigInt>, Vec<BigInt>)| [a[0].to_field(), b[0].to_field()];

    let evals = ProofEvaluations::<[Fp; 2]> {
        w: array::from_fn(|i| to_fp(&evals.w[i])),
        z: to_fp(&evals.z),
        s: array::from_fn(|i| to_fp(&evals.s[i])),
        generic_selector: to_fp(&evals.generic_selector),
        poseidon_selector: to_fp(&evals.poseidon_selector),
        coefficients: array::from_fn(|i| to_fp(&evals.coefficients[i])),
        complete_add_selector: to_fp(&evals.complete_add_selector),
        mul_selector: to_fp(&evals.mul_selector),
        emul_selector: to_fp(&evals.emul_selector),
        endomul_scalar_selector: to_fp(&evals.endomul_scalar_selector),
        range_check0_selector: evals.range_check0_selector.as_ref().map(to_fp),
        range_check1_selector: evals.range_check1_selector.as_ref().map(to_fp),
        foreign_field_add_selector: evals.foreign_field_add_selector.as_ref().map(to_fp),
        foreign_field_mul_selector: evals.foreign_field_mul_selector.as_ref().map(to_fp),
        xor_selector: evals.xor_selector.as_ref().map(to_fp),
        rot_selector: evals.rot_selector.as_ref().map(to_fp),
        lookup_aggregation: evals.lookup_aggregation.as_ref().map(to_fp),
        lookup_table: evals.lookup_table.as_ref().map(to_fp),
        lookup_sorted: array::from_fn(|i| evals.lookup_sorted[i].as_ref().map(to_fp)),
        runtime_lookup_table: evals.runtime_lookup_table.as_ref().map(to_fp),
        runtime_lookup_table_selector: evals.runtime_lookup_table_selector.as_ref().map(to_fp),
        xor_lookup_selector: evals.xor_lookup_selector.as_ref().map(to_fp),
        lookup_gate_lookup_selector: evals.lookup_gate_lookup_selector.as_ref().map(to_fp),
        range_check_lookup_selector: evals.range_check_lookup_selector.as_ref().map(to_fp),
        foreign_field_mul_lookup_selector: evals
            .foreign_field_mul_lookup_selector
            .as_ref()
            .map(to_fp),
    };

    let plonk = &proof.statement.proof_state.deferred_values.plonk;

    let zeta_bytes: [u64; 2] = array::from_fn(|i| plonk.zeta.inner[i].as_u64());
    let alpha_bytes: [u64; 2] = array::from_fn(|i| plonk.alpha.inner[i].as_u64());
    let beta_bytes: [u64; 2] = array::from_fn(|i| plonk.beta[i].as_u64());
    let gamma_bytes: [u64; 2] = array::from_fn(|i| plonk.gamma[i].as_u64());

    let zeta: Fp = ScalarChallenge::from(zeta_bytes).to_field(&endo_fp());
    let alpha: Fp = ScalarChallenge::from(alpha_bytes).to_field(&endo_fp());
    let beta: Fp = u64_to_field(&beta_bytes);
    let gamma: Fp = u64_to_field(&gamma_bytes);

    let branch_data = &proof.statement.proof_state.deferred_values.branch_data;
    let domain_log2: u8 = branch_data.domain_log2.as_u8();

    let minimal = PlonkMinimal {
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

    DataForPublicInput {
        evals,
        minimal,
        domain_log2,
    }
}

fn validate_feature_flags(
    feature_flags: &PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags,
    evals: &PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
) -> bool {
    let PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
        w: _,
        coefficients: _,
        z: _,
        s: _,
        generic_selector: _,
        poseidon_selector: _,
        complete_add_selector: _,
        mul_selector: _,
        emul_selector: _,
        endomul_scalar_selector: _,
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

    fn enable_if<T>(x: &Option<T>, flag: bool) -> bool {
        x.is_some() == flag
    }

    let f = feature_flags;
    let range_check_lookup = f.range_check0 || f.range_check1 || f.rot;
    let lookups_per_row_4 = f.xor || range_check_lookup || f.foreign_field_mul;
    let lookups_per_row_3 = lookups_per_row_4 || f.lookup;
    let lookups_per_row_2 = lookups_per_row_3;

    [
        enable_if(range_check0_selector, f.range_check0),
        enable_if(range_check1_selector, f.range_check1),
        enable_if(foreign_field_add_selector, f.foreign_field_add),
        enable_if(foreign_field_mul_selector, f.foreign_field_mul),
        enable_if(xor_selector, f.xor),
        enable_if(rot_selector, f.rot),
        enable_if(lookup_aggregation, lookups_per_row_2),
        enable_if(lookup_table, lookups_per_row_2),
        lookup_sorted.iter().enumerate().fold(true, |acc, (i, x)| {
            let flag = match i {
                0 | 1 | 2 => lookups_per_row_2,
                3 => lookups_per_row_3,
                4 => lookups_per_row_4,
                _ => panic!(),
            };
            acc && enable_if(x, flag)
        }),
        enable_if(runtime_lookup_table, f.runtime_tables),
        enable_if(runtime_lookup_table_selector, f.runtime_tables),
        enable_if(xor_lookup_selector, f.xor),
        enable_if(lookup_gate_lookup_selector, f.lookup),
        enable_if(range_check_lookup_selector, range_check_lookup),
        enable_if(foreign_field_mul_lookup_selector, f.foreign_field_mul),
    ]
    .iter()
    .all(|b| *b == true)
}

pub fn prev_evals_from_p2p<F: FieldWitness>(
    evals: &PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
) -> ProofEvaluations<PointEvaluations<Vec<F>>> {
    let PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
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

    let of = |(zeta, zeta_omega): &(Vec<BigInt>, Vec<BigInt>)| -> PointEvaluations<Vec<F>> {
        PointEvaluations {
            zeta: zeta.iter().map(BigInt::to_field).collect(),
            zeta_omega: zeta_omega.iter().map(BigInt::to_field).collect(),
        }
    };

    let of_opt = |v: &Option<(Vec<BigInt>, Vec<BigInt>)>| v.as_ref().map(of);

    ProofEvaluations {
        w: array::from_fn(|i| of(&w[i])),
        z: of(z),
        s: array::from_fn(|i| of(&s[i])),
        coefficients: array::from_fn(|i| of(&coefficients[i])),
        generic_selector: of(generic_selector),
        poseidon_selector: of(poseidon_selector),
        complete_add_selector: of(complete_add_selector),
        mul_selector: of(mul_selector),
        emul_selector: of(emul_selector),
        endomul_scalar_selector: of(endomul_scalar_selector),
        range_check0_selector: of_opt(range_check0_selector),
        range_check1_selector: of_opt(range_check1_selector),
        foreign_field_add_selector: of_opt(foreign_field_add_selector),
        foreign_field_mul_selector: of_opt(foreign_field_mul_selector),
        xor_selector: of_opt(xor_selector),
        rot_selector: of_opt(rot_selector),
        lookup_aggregation: of_opt(lookup_aggregation),
        lookup_table: of_opt(lookup_table),
        lookup_sorted: array::from_fn(|i| of_opt(&lookup_sorted[i])),
        runtime_lookup_table: of_opt(runtime_lookup_table),
        runtime_lookup_table_selector: of_opt(runtime_lookup_table_selector),
        xor_lookup_selector: of_opt(xor_lookup_selector),
        lookup_gate_lookup_selector: of_opt(lookup_gate_lookup_selector),
        range_check_lookup_selector: of_opt(range_check_lookup_selector),
        foreign_field_mul_lookup_selector: of_opt(foreign_field_mul_lookup_selector),
    }
}

fn expand_deferred(
    env: &ScalarsEnv<Fp>,
    evals: &PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals,
    old_bulletproof_challenges: &[PaddedSeq<
        PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A,
        16,
    >],
    proof_state: &PicklesProofProofsVerified2ReprStableV2StatementProofState,
) {
    let old_bulletproof_challenges: Vec<[Fp; 16]> =
        extract_bulletproof(old_bulletproof_challenges, &endo_fp());

    // let
    let plonk0 = &proof_state.deferred_values.plonk;

    let zeta_bytes: [u64; 2] = array::from_fn(|i| plonk0.zeta.inner[i].as_u64());
    let alpha_bytes: [u64; 2] = array::from_fn(|i| plonk0.alpha.inner[i].as_u64());
    let beta_bytes: [u64; 2] = array::from_fn(|i| plonk0.beta[i].as_u64());
    let gamma_bytes: [u64; 2] = array::from_fn(|i| plonk0.gamma[i].as_u64());

    let zeta: Fp = ScalarChallenge::from(zeta_bytes).to_field(&endo_fp());
    let alpha: Fp = ScalarChallenge::from(alpha_bytes).to_field(&endo_fp());
    let beta: Fp = u64_to_field(&beta_bytes);
    let gamma: Fp = u64_to_field(&gamma_bytes);

    let branch_data = &proof_state.deferred_values.branch_data;
    let domain_log2: u8 = branch_data.domain_log2.as_u8();

    let w = env.domain.group_gen;
    let zetaw = zeta * w;

    let tick_plonk_minimal = PlonkMinimal {
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

    let evals = prev_evals_from_p2p(evals);
    evals_of_split_evals(zeta, zetaw, &evals, BACKEND_TICK_ROUNDS_N);

    let plonk = {
        // TODO: Dedup this
        let evals =
            evals.map_ref(&|PointEvaluations { zeta, zeta_omega }| [zeta[0], zeta_omega[0]]);
        let mut p = derive_plonk(env, &evals, &tick_plonk_minimal);
        p.zeta = u64_to_field(&zeta_bytes);
        p.alpha = u64_to_field(&alpha_bytes);
        p.beta = u64_to_field(&beta_bytes);
        p.gamma = u64_to_field(&gamma_bytes);
        p
    };
}

//   let absorb, squeeze =
//     let open Tick_field_sponge.Bits in
//     let sponge =
//       let s = create Tick_field_sponge.params in
//       absorb s
//         (Digest.Constant.to_tick_field
//            proof_state.sponge_digest_before_evaluations ) ;
//       s
//     in
//     let squeeze () =
//       let underlying =
//         Challenge.Constant.of_bits
//           (squeeze sponge ~length:Challenge.Constant.length)
//       in
//       Scalar_challenge.create underlying
//     in
//     (absorb sponge, squeeze)
//   in
//   let old_bulletproof_challenges =
//     Vector.map ~f:Ipa.Step.compute_challenges old_bulletproof_challenges
//   in
//   (let challenges_digest =
//      let open Tick_field_sponge.Field in
//      let sponge = create Tick_field_sponge.params in
//      Vector.iter old_bulletproof_challenges ~f:(Vector.iter ~f:(absorb sponge)) ;
//      squeeze sponge
//    in
//    absorb challenges_digest ;
//    absorb evals.ft_eval1 ;
//    let xs = Plonk_types.Evals.to_absorption_sequence evals.evals.evals in
//    let x1, x2 = evals.evals.public_input in
//    absorb x1 ;
//    absorb x2 ;
//    List.iter xs ~f:(fun (x1, x2) ->
//        Array.iter ~f:absorb x1 ; Array.iter ~f:absorb x2 ) ) ;
//   let xi_chal = squeeze () in
//   let xi = sc xi_chal in
//   let r_chal = squeeze () in
//   let r = sc r_chal in
//   Timer.clock __LOC__ ;
//   (* TODO: The deferred values "bulletproof_challenges" should get routed
//      into a "batch dlog Tick acc verifier" *)
//   let actual_proofs_verified = Vector.length old_bulletproof_challenges in
//   Timer.clock __LOC__ ;
//   let combined_inner_product_actual =
//     Wrap.combined_inner_product ~env:tick_env ~plonk:tick_plonk_minimal
//       ~domain:tick_domain ~ft_eval1:evals.ft_eval1
//       ~actual_proofs_verified:(Nat.Add.create actual_proofs_verified)
//       evals.evals ~old_bulletproof_challenges ~r ~xi ~zeta ~zetaw
//   in
//   Timer.clock __LOC__ ;
//   let bulletproof_challenges =
//     Ipa.Step.compute_challenges bulletproof_challenges
//   in
//   Timer.clock __LOC__ ;
//   let b_actual =
//     let challenge_poly =
//       unstage
//         (Wrap.challenge_polynomial (Vector.to_array bulletproof_challenges))
//     in
//     Tick.Field.(challenge_poly zeta + (r * challenge_poly zetaw))
//   in
//   let to_shifted =
//     Shifted_value.Type1.of_field (module Tick.Field) ~shift:Shifts.tick1
//   in
//   { xi = xi_chal
//   ; plonk
//   ; combined_inner_product = to_shifted combined_inner_product_actual
//   ; branch_data
//   ; bulletproof_challenges
//   ; b = to_shifted b_actual
//   }

// TODO: `domain_log2` and `srs_length_log2` might be the same here ? Remove one or the other
pub fn make_scalars_env<F: FieldWitness, const NLIMB: usize>(
    minimal: &PlonkMinimal<F, NLIMB>,
    domain_log2: u8,
    srs_length_log2: u64,
) -> ScalarsEnv<F> {
    let domain: Radix2EvaluationDomain<F> =
        Radix2EvaluationDomain::new(1 << domain_log2 as u64).unwrap();

    let zk_polynomial = eval_zk_polynomial(domain, minimal.zeta);
    let zeta_to_n_minus_1 = domain.evaluate_vanishing_polynomial(minimal.zeta);

    let (_w4, w3, _w2, _w1) = {
        let gen = domain.group_gen;
        let w1 = F::one() / gen;
        let w2 = w1.square();
        let w3 = w2 * w1;
        // let w4 = (); // unused for now
        // let w4 = w3 * w1;

        ((), w3, w2, w1)
    };

    ScalarsEnv {
        zk_polynomial,
        zeta_to_n_minus_1,
        srs_length_log2,
        domain,
        omega_to_minus_3: w3,
    }
}

fn get_message_for_next_step_proof<'a, AppState>(
    messages_for_next_step_proof: &PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof,
    commitments: &'a PlonkVerificationKeyEvals,
    app_state: &'a AppState,
) -> MessagesForNextStepProof<'a, AppState>
where
    AppState: ToFieldElements<Fp>,
{
    let PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof {
        app_state: _, // unused
        challenge_polynomial_commitments,
        old_bulletproof_challenges,
    } = messages_for_next_step_proof;

    let challenge_polynomial_commitments: Vec<CurveAffine<Fp>> =
        extract_polynomial_commitment(challenge_polynomial_commitments);
    let old_bulletproof_challenges: Vec<[Fp; 16]> =
        extract_bulletproof(old_bulletproof_challenges, &endo_fp());
    let dlog_plonk_index = commitments;

    MessagesForNextStepProof {
        app_state,
        dlog_plonk_index,
        challenge_polynomial_commitments,
        old_bulletproof_challenges,
    }
}

fn get_message_for_next_wrap_proof(
    PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof {
        challenge_polynomial_commitment,
        old_bulletproof_challenges,
    }: &PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof,
) -> MessagesForNextWrapProof {
    let challenge_polynomial_commitments: Vec<CurveAffine<Fq>> =
        extract_polynomial_commitment(&[challenge_polynomial_commitment.clone()]);

    let old_bulletproof_challenges: Vec<[Fq; 15]> = extract_bulletproof(
        &[
            old_bulletproof_challenges[0].0.clone(),
            old_bulletproof_challenges[1].0.clone(),
        ],
        &endo_fq(),
    );

    MessagesForNextWrapProof {
        challenge_polynomial_commitment: challenge_polynomial_commitments[0],
        old_bulletproof_challenges,
    }
}

fn get_prepared_statement<AppState>(
    message_for_next_step_proof: &MessagesForNextStepProof<AppState>,
    message_for_next_wrap_proof: &MessagesForNextWrapProof,
    plonk: InCircuit<Fp>,
    deferred_values: &PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues,
    sponge_digest_before_evaluations: &CompositionTypesDigestConstantStableV1,
    minimal: &PlonkMinimal<Fp>,
) -> PreparedStatement
where
    AppState: ToFieldElements<Fp>,
{
    let digest = sponge_digest_before_evaluations;
    let sponge_digest_before_evaluations: [u64; 4] = array::from_fn(|i| digest[i].as_u64());

    let plonk = Plonk {
        alpha: minimal.alpha_bytes,
        beta: minimal.beta_bytes,
        gamma: minimal.gamma_bytes,
        zeta: minimal.zeta_bytes,
        zeta_to_srs_length: plonk.zeta_to_srs_length,
        zeta_to_domain_size: plonk.zeta_to_domain_size,
        // vbmul: plonk.vbmul,
        // complete_add: plonk.complete_add,
        // endomul: plonk.endomul,
        // endomul_scalar: plonk.endomul_scalar,
        perm: plonk.perm,
        lookup: (),
    };

    // let xi: [u64; 2] = array::from_fn(|i| deferred_values.plonk.xi.inner[i].as_u64());
    // let b: ShiftedValue<Fp> = to_shifted_value(&deferred_values.b);
    // let combined_inner_product: ShiftedValue<Fp> =
    //     to_shifted_value(&deferred_values.combined_inner_product);

    let combined_inner_product = todo!();
    let b = todo!();
    let xi = todo!();

    let bulletproof_challenges = &deferred_values.bulletproof_challenges;
    let bulletproof_challenges: Vec<Fp> = bulletproof_challenges
        .iter()
        .map(|chal| {
            let inner = &chal.prechallenge.inner;
            u64_to_field(&[inner[0].as_u64(), inner[1].as_u64()])
        })
        .collect();

    let branch_data = deferred_values.branch_data.clone();

    PreparedStatement {
        proof_state: ProofState {
            deferred_values: DeferredValues {
                plonk,
                combined_inner_product,
                b,
                xi,
                bulletproof_challenges,
                branch_data,
            },
            sponge_digest_before_evaluations,
            messages_for_next_wrap_proof: message_for_next_wrap_proof.hash(),
        },
        messages_for_next_step_proof: message_for_next_step_proof.hash(),
    }
}

fn verify_with(
    verifier_index: &VerifierIndex,
    prover: &ProverProof,
    public_input: &[Fq],
) -> Result<(), VerifyError> {
    use kimchi::groupmap::GroupMap;
    use kimchi::mina_curves::pasta::PallasParameters;
    use mina_poseidon::sponge::{DefaultFqSponge, DefaultFrSponge};

    type SpongeParams = mina_poseidon::constants::PlonkSpongeConstantsKimchi;
    type EFqSponge = DefaultFqSponge<PallasParameters, SpongeParams>;
    type EFrSponge = DefaultFrSponge<Fq, SpongeParams>;

    let group_map = GroupMap::<Fp>::setup();

    kimchi::verifier::verify::<Pallas, EFqSponge, EFrSponge>(
        &group_map,
        verifier_index,
        prover,
        public_input,
    )
}

fn run_checks(
    env: &ScalarsEnv<Fp>,
    evals: &ProofEvaluations<[Fp; 2]>,
    minimal: &PlonkMinimal<Fp>,
    proof: &PicklesProofProofsVerified2ReprStableV2,
    verifier_index: &VerifierIndex,
) -> bool {
    type SpongeParams = mina_poseidon::constants::PlonkSpongeConstantsKimchi;
    type EFqSponge =
        mina_poseidon::sponge::DefaultFqSponge<mina_curves::pasta::PallasParameters, SpongeParams>;
    use mina_poseidon::FqSponge;

    let mut errors: Vec<String> = vec![];
    let mut checks = |condition: bool, s: &str| {
        if !condition {
            errors.push(s.to_string())
        }
    };

    let non_chunking = {
        let PicklesProofProofsVerified2ReprStableV2PrevEvalsEvalsEvals {
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
        } = &proof.prev_evals.evals.evals;

        let mut iter = w
            .iter()
            .chain(coefficients.iter())
            .chain([z])
            .chain(s.iter())
            .chain([
                generic_selector,
                poseidon_selector,
                complete_add_selector,
                mul_selector,
                emul_selector,
            ])
            .chain(range_check0_selector.iter())
            .chain(range_check1_selector.iter())
            .chain(foreign_field_add_selector.iter())
            .chain(foreign_field_mul_selector.iter())
            .chain(xor_selector.iter())
            .chain(rot_selector.iter())
            .chain(lookup_aggregation.iter())
            .chain(lookup_table.iter())
            .chain(lookup_sorted.iter().flatten())
            .chain(runtime_lookup_table.iter())
            .chain(runtime_lookup_table_selector.iter())
            .chain(xor_lookup_selector.iter())
            .chain(lookup_gate_lookup_selector.iter())
            .chain(range_check_lookup_selector.iter())
            .chain(foreign_field_mul_lookup_selector.iter());

        iter.all(|(a, b)| a.len() == 1 && b.len() == 1)
    };

    checks(non_chunking, "only uses single chunks");

    checks(
        validate_feature_flags(
            &proof
                .statement
                .proof_state
                .deferred_values
                .plonk
                .feature_flags,
            &proof.prev_evals.evals.evals,
        ),
        "feature flags are consistent with evaluations",
    );

    checks(
        env.domain.log_size_of_group as usize <= BACKEND_TICK_ROUNDS_N,
        "domain size is small enough",
    );

    let digest = &proof.statement.proof_state.sponge_digest_before_evaluations;
    let sponge_digest_before_evaluations: [u64; 4] = array::from_fn(|i| digest[i].as_u64());
    let sponge_digest_before_evaluations = BigInteger256(sponge_digest_before_evaluations);
    let sponge_digest_before_evaluations = Fp::from(sponge_digest_before_evaluations);

    let old_bulletproof_challenges = &proof
        .statement
        .messages_for_next_step_proof
        .old_bulletproof_challenges;
    let old_bulletproof_challenges: Vec<[Fp; 16]> =
        extract_bulletproof(old_bulletproof_challenges, &endo_fp());

    let challenges_digest = {
        let mut sponge =
            crate::ArithmeticSponge::<Fp, crate::PlonkSpongeConstantsKimchi>::new(static_params());
        for old_bulletproof_challenges in &old_bulletproof_challenges {
            sponge.absorb(old_bulletproof_challenges);
        }
        sponge.squeeze()
    };
    let endo_fp = endo_fp();
    let deferred_values = &proof.statement.proof_state.deferred_values;

    let xs = to_absorption_sequence(&proof.prev_evals.evals.evals);
    let (x1, x2) = &proof.prev_evals.evals.public_input;

    let mut sponge = EFqSponge::new(mina_poseidon::pasta::fp_kimchi::static_params());
    sponge.absorb_fq(&[sponge_digest_before_evaluations]);
    sponge.absorb_fq(&[challenges_digest]);
    sponge.absorb_fq(&[proof.prev_evals.ft_eval1.to_field()]);
    sponge.absorb_fq(&[x1.to_field(), x2.to_field()]);
    xs.iter().for_each(|(x1, x2)| {
        sponge.absorb_fq(x1);
        sponge.absorb_fq(x2);
    });
    let xi_actual = sponge.squeeze_limbs(2);
    let r_actual = sponge.squeeze_limbs(2);

    let xi_actual = ScalarChallenge::from(xi_actual).to_field(&endo_fp);
    let r_actual = ScalarChallenge::from(r_actual).to_field(&endo_fp);
    let zetaw = minimal.zeta * env.domain.group_gen;
    // let b: ShiftedValue<Fp> = to_shifted_value(&deferred_values.b);

    // let xi: [u64; 2] = array::from_fn(|i| deferred_values.xi.inner[i].as_u64());
    // let xi = ScalarChallenge::new(xi[0], xi[1]).to_field(&endo_fp);

    let old_bulletproof_challenges = &old_bulletproof_challenges;
    // let combined_inner_product_actual = combined_inner_product(CombinedInnerProductParams {
    //     env,
    //     evals,
    //     minimal,
    //     proof,
    //     r: r_actual,
    //     old_bulletproof_challenges,
    //     xi,
    //     zetaw,
    // });

    // let combined_inner_product: ShiftedValue<Fp> = to_shifted_value(
    //     &proof
    //         .statement
    //         .proof_state
    //         .deferred_values
    //         .combined_inner_product,
    // );

    let combined_inner_product_actual: Fp = todo!();
    let combined_inner_product: ShiftedValue<Fp> = todo!();

    let bulletproof_challenges = &deferred_values.bulletproof_challenges;
    let bulletproof_challenges: Vec<Fp> = bulletproof_challenges
        .iter()
        .map(|chal| {
            let prechallenge = &chal.prechallenge.inner;
            let prechallenge: [u64; 2] = array::from_fn(|k| prechallenge[k].as_u64());
            ScalarChallenge::from(prechallenge).to_field(&endo_fp)
        })
        .collect();

    let b_actual = {
        let challenge_polys = challenge_polynomial(&bulletproof_challenges);
        challenge_polys(minimal.zeta) + (r_actual * challenge_polys(zetaw))
    };

    {
        // TODO: Don't use hardcoded values
        let all_possible_domains = [13, 14, 15];
        let [greatest_wrap_domain, _, least_wrap_domain] = all_possible_domains;

        let actual_wrap_domain = verifier_index.domain.log_size_of_group;
        checks(
            actual_wrap_domain <= least_wrap_domain,
            "invalid actual_wrap_domain (least_wrap_domain)",
        );
        checks(
            actual_wrap_domain >= greatest_wrap_domain,
            "invalid actual_wrap_domain (greatest_wrap_domain)",
        );
    }

    checks(
        combined_inner_product.shifted_to_field() == combined_inner_product_actual,
        "different combined inner product",
    );
    // checks(b.shifted_to_field() == b_actual, "different b");
    // checks(xi == xi_actual, "different xi");

    for e in &errors {
        eprintln!("{:?}", e);
    }

    errors.is_empty()
}

/// https://github.com/MinaProtocol/mina/blob/4e0b324912017c3ff576704ee397ade3d9bda412/src/lib/pickles/verification_key.mli#L30
pub struct VK<'a> {
    pub commitments: PlonkVerificationKeyEvals,
    pub index: &'a VerifierIndex,
    pub data: (), // Unused in proof verification
}

pub fn verify_block(
    header: &MinaBlockHeaderStableV2,
    verifier_index: &VerifierIndex,
    srs: &VerifierSRS,
) -> bool {
    let MinaBlockHeaderStableV2 {
        protocol_state,
        protocol_state_proof,
        ..
    } = &header;

    let vk = VK {
        commitments: PlonkVerificationKeyEvals::from(verifier_index),
        index: verifier_index,
        data: (),
    };

    let accum_check = accumulator_check::accumulator_check(srs, protocol_state_proof);
    let verified = verify_impl(protocol_state, protocol_state_proof, &vk);
    accum_check && verified
}

pub fn verify_transaction<'a>(
    proofs: impl IntoIterator<Item = (&'a Statement<SokDigest>, &'a TransactionSnarkProofStableV2)>,
    verifier_index: &VerifierIndex,
    srs: &VerifierSRS,
) -> bool {
    let vk = VK {
        commitments: PlonkVerificationKeyEvals::from(verifier_index),
        index: verifier_index,
        data: (),
    };

    proofs.into_iter().all(|(statement, transaction_proof)| {
        let accum_check = accumulator_check::accumulator_check(srs, transaction_proof);
        let verified = verify_impl(statement, transaction_proof, &vk);
        accum_check && verified
    })
}

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/crypto/kimchi_bindings/stubs/src/pasta_fq_plonk_proof.rs#L116
pub fn verify_zkapp(
    verification_key: &VerificationKey,
    zkapp_statement: ZkappStatement,
    sideloaded_proof: &PicklesProofProofsVerified2ReprStableV2,
    srs: &VerifierSRS,
) -> bool {
    let verifier_index = make_zkapp_verifier_index(verification_key);
    // https://github.com/MinaProtocol/mina/blob/4e0b324912017c3ff576704ee397ade3d9bda412/src/lib/pickles/pickles.ml#LL260C1-L274C18
    let vk = VK {
        commitments: verification_key.wrap_index.clone(),
        index: &verifier_index,
        data: (),
    };

    let accum_check = accumulator_check::accumulator_check(srs, sideloaded_proof);
    let verified = verify_impl(&zkapp_statement, sideloaded_proof, &vk);

    let ok = accum_check && verified;

    eprintln!("verify_zkapp OK={:?}", ok);

    ok
}

fn verify_impl<AppState>(
    app_state: &AppState,
    proof: &PicklesProofProofsVerified2ReprStableV2,
    vk: &VK,
) -> bool
where
    AppState: ToFieldElements<Fp>,
{
    let DataForPublicInput {
        evals,
        minimal,
        domain_log2,
    } = extract_data_for_public_input(proof);

    let env = make_scalars_env(&minimal, domain_log2, 16);
    let plonk = derive_plonk(&env, &evals, &minimal);

    let checks = run_checks(&env, &evals, &minimal, proof, vk.index);

    let message_for_next_step_proof = get_message_for_next_step_proof(
        &proof.statement.messages_for_next_step_proof,
        &vk.commitments,
        app_state,
    );

    let message_for_next_wrap_proof =
        get_message_for_next_wrap_proof(&proof.statement.proof_state.messages_for_next_wrap_proof);

    let prepared_statement = get_prepared_statement(
        &message_for_next_step_proof,
        &message_for_next_wrap_proof,
        plonk,
        &proof.statement.proof_state.deferred_values,
        &proof.statement.proof_state.sponge_digest_before_evaluations,
        &minimal,
    );

    let npublic_input = vk.index.public;
    let public_inputs = prepared_statement.to_public_input(npublic_input);
    let prover = make_prover(proof);

    let result = verify_with(vk.index, &prover, &public_inputs);

    if let Err(e) = result {
        eprintln!("verify error={:?}", e);
    };

    result.is_ok() && checks
}

// #[cfg(test)]
// mod tests {
//     use std::{
//         collections::hash_map::DefaultHasher,
//         hash::{Hash, Hasher},
//     };

//     use binprot::BinProtRead;
//     use mina_curves::pasta::Vesta;
//     use mina_p2p_messages::v2::MinaBlockHeaderStableV2;
//     use poly_commitment::srs::SRS;

//     use crate::{
//         block::caching::{
//             srs_from_bytes, srs_to_bytes, verifier_index_from_bytes, verifier_index_to_bytes,
//         },
//         get_srs, get_verifier_index,
//     };

//     #[cfg(target_family = "wasm")]
//     use wasm_bindgen_test::wasm_bindgen_test as test;

//     #[test]
//     fn test_verification() {
//         let now = std::time::Instant::now();
//         let verifier_index = get_verifier_index();
//         println!("get_verifier_index={:?}", now.elapsed());

//         let now = std::time::Instant::now();
//         let srs = get_srs();
//         println!("get_srs={:?}\n", now.elapsed());

//         let now = std::time::Instant::now();
//         let bytes = verifier_index_to_bytes(&verifier_index);
//         println!("verifier_elapsed={:?}", now.elapsed());
//         println!("verifier_length={:?}", bytes.len());
//         assert_eq!(bytes.len(), 5622520);

//         let now = std::time::Instant::now();
//         let verifier_index = verifier_index_from_bytes(&bytes);
//         println!("verifier_deserialize_elapsed={:?}\n", now.elapsed());

//         let now = std::time::Instant::now();
//         let bytes = srs_to_bytes(&srs);
//         println!("srs_elapsed={:?}", now.elapsed());
//         println!("srs_length={:?}", bytes.len());
//         assert_eq!(bytes.len(), 5308513);

//         let now = std::time::Instant::now();
//         let srs: SRS<Vesta> = srs_from_bytes(&bytes);
//         println!("deserialize_elapsed={:?}\n", now.elapsed());

//         // Few blocks headers from berkeleynet
//         let files = [
//             include_bytes!("../data/rampup.binprot"),
//             include_bytes!("../data/5573.binprot"),
//             include_bytes!("../data/5574.binprot"),
//             include_bytes!("../data/5575.binprot"),
//             include_bytes!("../data/5576.binprot"),
//             include_bytes!("../data/5577.binprot"),
//             include_bytes!("../data/5578.binprot"),
//             include_bytes!("../data/5579.binprot"),
//             include_bytes!("../data/5580.binprot"),
//         ];

//         for file in files {
//             let header = MinaBlockHeaderStableV2::binprot_read(&mut file.as_slice()).unwrap();

//             let now = std::time::Instant::now();
//             let accum_check = crate::accumulator_check(&srs, &header.protocol_state_proof.0);
//             println!("accumulator_check={:?}", now.elapsed());

//             let now = std::time::Instant::now();
//             let verified = crate::verify(&header, &verifier_index);
//             println!("snark::verify={:?}", now.elapsed());

//             assert!(accum_check);
//             assert!(verified);
//         }
//     }

//     #[test]
//     fn test_verifier_index_deterministic() {
//         let mut nruns = 0;
//         let nruns = &mut nruns;

//         let mut hash_verifier_index = || {
//             *nruns += 1;
//             let verifier_index = get_verifier_index();
//             let bytes = verifier_index_to_bytes(&verifier_index);

//             let mut hasher = DefaultHasher::new();
//             bytes.hash(&mut hasher);
//             hasher.finish()
//         };

//         assert_eq!(hash_verifier_index(), hash_verifier_index());
//         assert_eq!(*nruns, 2);
//     }
// }
