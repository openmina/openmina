use std::str::FromStr;

use kimchi::{
    poly_commitment::PolyComm,
    proof::{PointEvaluations, ProofEvaluations, ProverCommitments, RecursionChallenge},
};
use mina_curves::pasta::Pallas;
use mina_hasher::Fp;
use once_cell::sync::Lazy;
use poly_commitment::{commitment::CommitmentCurve, evaluation_proof::OpeningProof};

use super::public_input::scalar_challenge::endo_fq;
use super::util::extract_bulletproof;
use mina_curves::pasta::Fq;
use mina_p2p_messages::{bigint::BigInt, v2::PicklesProofProofsVerified2ReprStableV2};

use super::ProverProof;

fn get_challenge_polynomial_commitments_padding() -> (BigInt, BigInt) {
    static PADDING: Lazy<(BigInt, BigInt)> = Lazy::new(|| {
        let first = Fp::from_str(
            "8063668238751197448664615329057427953229339439010717262869116690340613895496",
        )
        .unwrap();
        let second = Fp::from_str(
            "2694491010813221541025626495812026140144933943906714931997499229912601205355",
        )
        .unwrap();

        (first.into(), second.into())
    });

    PADDING.clone()
}

pub fn make_prover(
    PicklesProofProofsVerified2ReprStableV2 {
        statement: _,
        prev_evals: _, // unused
        proof: _,
    }: &PicklesProofProofsVerified2ReprStableV2,
) -> ProverProof {
    todo!()
    // let of_coord = |(a, b): &(BigInt, BigInt)| Pallas::of_coordinates(a.to_field(), b.to_field());

    // let make_poly = |poly: &(BigInt, BigInt)| {
    //     let unshifted = of_coord(poly);
    //     PolyComm {
    //         unshifted: vec![unshifted],
    //         shifted: None,
    //     }
    // };

    // let _w_comm: [PolyComm<Pallas>; 15] = array::from_fn(|i| make_poly(&proof.commitments.w_comm[i]));
    // let _z_comm: PolyComm<Pallas> = make_poly(&proof.commitments.z_comm);
    // let _t_comm: PolyComm<Pallas> = todo!();

    // let bulletproof = &proof.bulletproof;

    // let lr = &bulletproof.lr;
    // let lr: Vec<(Pallas, Pallas)> = lr.iter().map(|(a, b)| (of_coord(a), of_coord(b))).collect();

    // let delta: Pallas = of_coord(&bulletproof.delta);
    // let z1: Fq = bulletproof.z_1.to_field();
    // let z2: Fq = bulletproof.z_2.to_field();

    // let sg: Pallas = of_coord(&bulletproof.challenge_polynomial_commitment);

    // let evals = &proof.evaluations;

    // let to_pt_eval = |(first, second): &(BigInt, BigInt)| PointEvaluations {
    //     zeta: vec![first.to_field()],
    //     zeta_omega: vec![second.to_field()],
    // };

    // let evals: ProofEvaluations<PointEvaluations<Vec<Fq>>> = ProofEvaluations {
    //     w: array::from_fn(|i| to_pt_eval(&evals.w[i])),
    //     z: to_pt_eval(&evals.z),
    //     s: array::from_fn(|i| to_pt_eval(&evals.s[i])),
    //     generic_selector: to_pt_eval(&evals.generic_selector),
    //     poseidon_selector: to_pt_eval(&evals.poseidon_selector),
    //     coefficients: array::from_fn(|i| to_pt_eval(&evals.coefficients[i])),
    //     complete_add_selector: to_pt_eval(&evals.complete_add_selector),
    //     mul_selector: to_pt_eval(&evals.mul_selector),
    //     emul_selector: to_pt_eval(&evals.emul_selector),
    //     endomul_scalar_selector: to_pt_eval(&evals.endomul_scalar_selector),
    //     range_check0_selector: None,
    //     range_check1_selector: None,
    //     foreign_field_add_selector: None,
    //     foreign_field_mul_selector: None,
    //     xor_selector: None,
    //     rot_selector: None,
    //     lookup_aggregation: None,
    //     lookup_table: None,
    //     lookup_sorted: [None, None, None, None, None],
    //     runtime_lookup_table: None,
    //     runtime_lookup_table_selector: None,
    //     xor_lookup_selector: None,
    //     lookup_gate_lookup_selector: None,
    //     range_check_lookup_selector: None,
    //     foreign_field_mul_lookup_selector: None,
    // };

    // let ft_eval1: Fq = proof.ft_eval1.to_field();

    // let old_bulletproof_challenges = &statement
    //     .proof_state
    //     .messages_for_next_wrap_proof
    //     .old_bulletproof_challenges;
    // let old_bulletproof_challenges: Vec<[Fq; 15]> = extract_bulletproof(
    //     &[
    //         old_bulletproof_challenges.0[0].0.clone(),
    //         old_bulletproof_challenges.0[1].0.clone(),
    //     ],
    //     &endo_fq(),
    // );

    // let make_poly = |poly: &(BigInt, BigInt)| {
    //     let point = of_coord(poly);
    //     PolyComm {
    //         unshifted: vec![point],
    //         shifted: None,
    //     }
    // };

    // let mut challenge_polynomial_commitments = Cow::Borrowed(
    //     &statement
    //         .messages_for_next_step_proof
    //         .challenge_polynomial_commitments,
    // );

    // // Prepend padding:
    // // https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/pickles/verify.ml#L361C1-L364C51
    // while challenge_polynomial_commitments.len() < 2 {
    //     let padding = get_challenge_polynomial_commitments_padding();
    //     challenge_polynomial_commitments.to_mut().insert(0, padding);
    // }

    // let challenge_polynomial_commitments: Vec<PolyComm<Pallas>> = challenge_polynomial_commitments
    //     .iter()
    //     .map(make_poly)
    //     .collect();

    // let prev_challenges: Vec<RecursionChallenge<Pallas>> = old_bulletproof_challenges
    //     .iter()
    //     .zip(challenge_polynomial_commitments)
    //     .map(|(chals, comm)| RecursionChallenge::new(chals.to_vec(), comm))
    //     .collect();

    // ProverProof {
    //     commitments: ProverCommitments {
    //         w_comm,
    //         z_comm,
    //         t_comm,
    //         lookup: None,
    //     },
    //     proof: OpeningProof {
    //         lr,
    //         delta,
    //         z1,
    //         z2,
    //         sg,
    //     },
    //     evals,
    //     ft_eval1,
    //     prev_challenges,
    // }
}
