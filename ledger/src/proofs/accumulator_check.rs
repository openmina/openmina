use std::array;

use mina_curves::pasta::{Fq, Vesta};
use mina_hasher::Fp;
use mina_p2p_messages::{bigint::BigInt, v2::PicklesProofProofsVerified2ReprStableV2};
use poly_commitment::{commitment::CommitmentCurve, srs::SRS};

use super::public_input::scalar_challenge::ScalarChallenge;
use super::transaction::endos;
use super::urs_utils;

pub fn accumulator_check(
    urs: &SRS<Vesta>,
    proof: &PicklesProofProofsVerified2ReprStableV2,
) -> bool {
    // accumulator check
    // Note:
    // comms: statement.proof_state.messages_for_next_wrap_proof.challenge_polynomial_commitment
    // chals: statement.proof_state.deferred_values.bulletproof_challenges

    let (_, endo) = endos::<Fq>();

    let deferred_values = &proof.statement.proof_state.deferred_values;
    let bulletproof_challenges = &deferred_values.bulletproof_challenges;
    let bulletproof_challenges: Vec<Fp> = bulletproof_challenges
        .iter()
        .map(|chal| {
            let prechallenge = &chal.prechallenge.inner;
            let prechallenge: [u64; 2] = array::from_fn(|k| prechallenge[k].as_u64());

            ScalarChallenge::from(prechallenge).to_field(&endo)
        })
        .collect();

    let of_coord = |x: &(BigInt, BigInt)| Vesta::of_coordinates(x.0.to_field(), x.1.to_field());

    // statement.proof_state.messages_for_next_wrap_proof.challenge_polynomial_commitment
    let acc_comm = &proof
        .statement
        .proof_state
        .messages_for_next_wrap_proof
        .challenge_polynomial_commitment;
    let acc_comm: Vesta = of_coord(acc_comm);

    let acc_check =
        urs_utils::batch_dlog_accumulator_check(urs, &[acc_comm], &bulletproof_challenges);

    if !acc_check {
        println!("accumulator_check failed");
    }

    acc_check
}
