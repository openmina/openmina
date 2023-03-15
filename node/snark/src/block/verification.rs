use std::array;

use ark_ff::Field;
use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

use crate::utils::{extract_bulletproof, extract_polynomial_commitment, u64_to_field};
use kimchi::{
    circuits::polynomials::permutation::eval_zk_polynomial, error::VerifyError,
    mina_curves::pasta::Pallas, proof::ProofEvaluations,
};
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::{
    bigint::BigInt,
    v2::{
        CompositionTypesDigestConstantStableV1, MinaBlockHeaderStableV2,
        MinaStateProtocolStateValueStableV2, PicklesProofProofsVerified2ReprStableV2,
        PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof,
        PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof,
        PicklesProofProofsVerified2ReprStableV2StatementFp,
        PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues,
    },
};
use poly_commitment::{commitment::CommitmentCurve, PolyComm};

use super::{prover::make_prover, ProverProof, VerifierIndex};

use crate::public_input::{
    messages::{
        CurveAffine, MessagesForNextStepProof, MessagesForNextWrapProof, PlonkVerificationKeyEvals,
    },
    plonk_checks::{derive_plonk, PlonkMinimal, ScalarsEnv, ShiftedValue},
    prepared_statement::{DeferredValues, Plonk, PreparedStatement, ProofState},
    scalar_challenge::{endo_fp, endo_fq, ScalarChallenge},
};

use crate::public_input::plonk_checks::InCircuit;

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
    minimal: PlonkMinimal,
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
        lookup: None,
        generic_selector: to_fp(&evals.generic_selector),
        poseidon_selector: to_fp(&evals.poseidon_selector),
        coefficients: array::from_fn(|i| to_fp(&evals.coefficients[i])),
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

    DataForPublicInput { evals, minimal }
}

fn make_scalars_env(minimal: &PlonkMinimal) -> ScalarsEnv {
    let srs_length_log2: u64 = 16;
    let domain: Radix2EvaluationDomain<Fp> =
        Radix2EvaluationDomain::new(1 << srs_length_log2).unwrap();

    let zk_polynomial = eval_zk_polynomial(domain, minimal.zeta);
    let zeta_to_n_minus_1 = domain.evaluate_vanishing_polynomial(minimal.zeta);

    ScalarsEnv {
        zk_polynomial,
        zeta_to_n_minus_1,
        srs_length_log2,
    }
}

fn get_message_for_next_step_proof<'a>(
    messages_for_next_step_proof: &PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof,
    verifier_index: &VerifierIndex,
    protocol_state: &'a MinaStateProtocolStateValueStableV2,
) -> MessagesForNextStepProof<'a> {
    let msg_for_next_step_proof = &messages_for_next_step_proof;
    let challenge_polynomial_commitments: [CurveAffine<Fp>; 2] =
        extract_polynomial_commitment(&msg_for_next_step_proof.challenge_polynomial_commitments);
    let old_bulletproof_challenges: [[Fp; 16]; 2] = extract_bulletproof(
        &msg_for_next_step_proof.old_bulletproof_challenges,
        &endo_fp(),
    );

    let to_curve = |v: &PolyComm<Pallas>| {
        let v = v.unshifted[0];
        CurveAffine(v.x, v.y)
    };

    let dlog_plonk_index = PlonkVerificationKeyEvals {
        sigma: array::from_fn(|i| to_curve(&verifier_index.sigma_comm[i])),
        coefficients: array::from_fn(|i| to_curve(&verifier_index.coefficients_comm[i])),
        generic: to_curve(&verifier_index.generic_comm),
        psm: to_curve(&verifier_index.psm_comm),
        complete_add: to_curve(&verifier_index.complete_add_comm),
        mul: to_curve(&verifier_index.mul_comm),
        emul: to_curve(&verifier_index.emul_comm),
        endomul_scalar: to_curve(&verifier_index.endomul_scalar_comm),
    };

    MessagesForNextStepProof {
        protocol_state,
        dlog_plonk_index,
        challenge_polynomial_commitments,
        old_bulletproof_challenges,
    }
}

fn get_message_for_next_wrap_proof(
    messages_for_next_wrap_proof: PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof,
) -> MessagesForNextWrapProof {
    let challenge_polynomial_commitments: [CurveAffine<Fq>; 1] = extract_polynomial_commitment(&[
        messages_for_next_wrap_proof.challenge_polynomial_commitment,
    ]);

    let old_bulletproof_challenges = &messages_for_next_wrap_proof.old_bulletproof_challenges;
    let old_bulletproof_challenges: [[Fq; 15]; 2] = extract_bulletproof(
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

fn get_prepared_statement(
    message_for_next_step_proof: &MessagesForNextStepProof,
    message_for_next_wrap_proof: &MessagesForNextWrapProof,
    plonk: InCircuit,
    deferred_values: &PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues,
    sponge_digest_before_evaluations: &CompositionTypesDigestConstantStableV1,
    minimal: &PlonkMinimal,
) -> PreparedStatement {
    let digest = &sponge_digest_before_evaluations;
    let sponge_digest_before_evaluations: [u64; 4] = array::from_fn(|i| digest[i].as_u64());

    let plonk = Plonk {
        alpha: minimal.alpha_bytes,
        beta: minimal.beta_bytes,
        gamma: minimal.gamma_bytes,
        zeta: minimal.zeta_bytes,
        zeta_to_srs_length: plonk.zeta_to_srs_length,
        zeta_to_domain_size: plonk.zeta_to_domain_size,
        poseidon_selector: plonk.poseidon_selector,
        vbmul: plonk.vbmul,
        complete_add: plonk.complete_add,
        endomul: plonk.endomul,
        endomul_scalar: plonk.endomul_scalar,
        perm: plonk.perm,
        generic: plonk.generic,
        lookup: (),
    };

    let xi: [u64; 2] = array::from_fn(|i| deferred_values.xi.inner[i].as_u64());
    let b: ShiftedValue<Fp> = to_shifted_value(&deferred_values.b);
    let combined_inner_product: ShiftedValue<Fp> =
        to_shifted_value(&deferred_values.combined_inner_product);

    let bulletproof_challenges = &deferred_values.bulletproof_challenges;
    let bulletproof_challenges: Vec<[u64; 2]> = bulletproof_challenges
        .iter()
        .map(|chal| {
            let inner = &chal.prechallenge.inner;
            [inner[0].as_u64(), inner[1].as_u64()]
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

    type SpongeParams = mina_poseidon::constants::PlonkSpongeConstantsKimchi;
    type EFqSponge = mina_poseidon::sponge::DefaultFqSponge<
        kimchi::mina_curves::pasta::PallasParameters,
        SpongeParams,
    >;
    type EFrSponge = mina_poseidon::sponge::DefaultFrSponge<Fq, SpongeParams>;

    kimchi::verifier::verify::<Pallas, EFqSponge, EFrSponge>(
        &<Pallas as CommitmentCurve>::Map::setup(),
        verifier_index,
        prover,
        public_input,
    )
}

pub fn verify(header: &MinaBlockHeaderStableV2, verifier_index: &VerifierIndex) -> bool {
    let protocol_state = &header.protocol_state;
    let proof = &header.protocol_state_proof.0;

    let DataForPublicInput { evals, minimal } = extract_data_for_public_input(proof);

    let env = make_scalars_env(&minimal);
    let plonk = derive_plonk(&env, &evals, &minimal);

    let message_for_next_step_proof = get_message_for_next_step_proof(
        &proof.statement.messages_for_next_step_proof,
        verifier_index,
        protocol_state,
    );

    let message_for_next_wrap_proof = &proof.statement.proof_state.messages_for_next_wrap_proof;
    let message_for_next_wrap_proof =
        get_message_for_next_wrap_proof(message_for_next_wrap_proof.clone());

    let prepared_statement = get_prepared_statement(
        &message_for_next_step_proof,
        &message_for_next_wrap_proof,
        plonk,
        &proof.statement.proof_state.deferred_values,
        &proof.statement.proof_state.sponge_digest_before_evaluations,
        &minimal,
    );

    let public_inputs = prepared_statement.to_public_input();
    let prover = make_prover(proof);

    let result = verify_with(verifier_index, &prover, &public_inputs);

    if let Err(e) = result {
        println!("verify error={:?}", e);
    };

    result.is_ok()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use mina_curves::pasta::Vesta;
    use poly_commitment::srs::SRS;

    use crate::{
        block::caching::{
            srs_from_bytes, srs_to_bytes, verifier_index_from_bytes, verifier_index_to_bytes,
        },
        get_srs, get_verifier_index,
    };

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    #[test]
    fn test_verification() {
        let now = std::time::Instant::now();
        let verifier_index = get_verifier_index();
        println!("get_verifier_index={:?}", now.elapsed());

        let now = std::time::Instant::now();
        let srs = get_srs();
        println!("get_srs={:?}\n", now.elapsed());

        let now = std::time::Instant::now();
        let bytes = verifier_index_to_bytes(&verifier_index);
        println!("verifier_elapsed={:?}", now.elapsed());
        println!("verifier_length={:?}", bytes.len());
        assert_eq!(bytes.len(), 5675912);

        let now = std::time::Instant::now();
        let _verifier_index = verifier_index_from_bytes(&bytes);
        println!("verifier_deserialize_elapsed={:?}\n", now.elapsed());

        let now = std::time::Instant::now();
        let bytes = srs_to_bytes(&srs);
        println!("srs_elapsed={:?}", now.elapsed());
        println!("srs_length={:?}", bytes.len());
        assert_eq!(bytes.len(), 5308513);

        let now = std::time::Instant::now();
        let _srs: SRS<Vesta> = srs_from_bytes(&bytes);
        println!("deserialize_elapsed={:?}\n", now.elapsed());

        // TODO: Needs to update files with new blocks
        // let files = [
        //     include_bytes!("../data/2128.binprot"),
        //     include_bytes!("../data/2132.binprot"),
        //     include_bytes!("../data/2133.binprot"),
        // ];

        // for file in files {
        //     let header = MinaBlockHeaderStableV2::binprot_read(&mut file.as_slice()).unwrap();

        //     let now = std::time::Instant::now();
        //     let accum_check = accumulator_check(&srs, &header.protocol_state_proof.0);
        //     println!("accumulator_check={:?}", now.elapsed());

        //     let now = std::time::Instant::now();
        //     let verified = verify(&header, &verifier_index);
        //     println!("snark::verify={:?}", now.elapsed());

        //     assert!(accum_check && verified);
        // }
    }

    #[test]
    fn test_verifier_index_deterministic() {
        let mut nruns = 0;
        let nruns = &mut nruns;

        let mut hash_verifier_index = || {
            *nruns += 1;
            let verifier_index = get_verifier_index();
            let bytes = verifier_index_to_bytes(&verifier_index);

            let mut hasher = DefaultHasher::new();
            bytes.hash(&mut hasher);
            hasher.finish()
        };

        assert_eq!(hash_verifier_index(), hash_verifier_index());
        assert_eq!(*nruns, 2);
    }
}
