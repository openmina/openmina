use std::array;

use kimchi::{
    poly_commitment::PolyComm,
    proof::{PointEvaluations, ProofEvaluations, ProverCommitments, RecursionChallenge},
};
use mina_curves::pasta::Pallas;
use poly_commitment::{commitment::CommitmentCurve, evaluation_proof::OpeningProof};

use crate::{public_input::scalar_challenge::endo_fq, utils::extract_bulletproof};
use mina_curves::pasta::Fq;
use mina_p2p_messages::{bigint::BigInt, v2::PicklesProofProofsVerified2ReprStableV2};

use super::ProverProof;

pub fn make_prover(proof: &PicklesProofProofsVerified2ReprStableV2) -> ProverProof {
    let statement = &proof.statement;
    let proof = &proof.proof;

    let of_coord = |(a, b): &(BigInt, BigInt)| Pallas::of_coordinates(a.to_field(), b.to_field());

    let make_poly = |poly: &[(BigInt, BigInt)]| {
        let unshifted = poly.iter().map(of_coord).collect();
        PolyComm {
            unshifted,
            shifted: None,
        }
    };

    let w_comm: [PolyComm<Pallas>; 15] = array::from_fn(|i| make_poly(&proof.messages.w_comm[i]));
    let z_comm: PolyComm<Pallas> = make_poly(&proof.messages.z_comm);
    let t_comm: PolyComm<Pallas> = make_poly(&proof.messages.t_comm);

    let openings = &proof.openings;

    let lr = &openings.proof.lr;
    let lr: Vec<(Pallas, Pallas)> = lr.iter().map(|(a, b)| (of_coord(a), of_coord(b))).collect();

    let delta: Pallas = of_coord(&openings.proof.delta);
    let z1: Fq = openings.proof.z_1.to_field();
    let z2: Fq = openings.proof.z_2.to_field();

    let sg: Pallas = of_coord(&openings.proof.challenge_polynomial_commitment);

    let evals = &openings.evals;
    let to_fields = |x: &Vec<BigInt>| x.iter().map(BigInt::to_field).collect();

    let to_pt_eval = |(first, second): &(Vec<BigInt>, Vec<BigInt>)| PointEvaluations {
        zeta: to_fields(first),
        zeta_omega: to_fields(second),
    };

    let evals: ProofEvaluations<PointEvaluations<Vec<Fq>>> = ProofEvaluations {
        w: array::from_fn(|i| to_pt_eval(&evals.w[i])),
        z: to_pt_eval(&evals.z),
        s: array::from_fn(|i| to_pt_eval(&evals.s[i])),
        lookup: None,
        generic_selector: to_pt_eval(&evals.generic_selector),
        poseidon_selector: to_pt_eval(&evals.poseidon_selector),
        coefficients: array::from_fn(|i| to_pt_eval(&evals.coefficients[i])),
    };

    let ft_eval1: Fq = openings.ft_eval1.to_field();

    let old_bulletproof_challenges = &statement
        .proof_state
        .messages_for_next_wrap_proof
        .old_bulletproof_challenges;
    let old_bulletproof_challenges: [[Fq; 15]; 2] = extract_bulletproof(
        &[
            old_bulletproof_challenges.0[0].0.clone(),
            old_bulletproof_challenges.0[1].0.clone(),
        ],
        &endo_fq(),
    );

    let make_poly = |poly: &(BigInt, BigInt)| {
        let point = of_coord(poly);
        PolyComm {
            unshifted: vec![point],
            shifted: None,
        }
    };

    let comms = &statement
        .messages_for_next_step_proof
        .challenge_polynomial_commitments;
    let commitments: Vec<PolyComm<Pallas>> = comms.iter().map(make_poly).collect();

    let prev_challenges: Vec<RecursionChallenge<Pallas>> = old_bulletproof_challenges
        .iter()
        .zip(commitments)
        .map(|(chals, comm)| RecursionChallenge::new(chals.to_vec(), comm))
        .collect();

    ProverProof {
        commitments: ProverCommitments {
            w_comm,
            z_comm,
            t_comm,
            lookup: None,
        },
        proof: OpeningProof {
            lr,
            delta,
            z1,
            z2,
            sg,
        },
        evals,
        ft_eval1,
        prev_challenges,
    }
}
