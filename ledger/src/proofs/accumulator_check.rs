use ark_ff::fields::arithmetic::InvalidBigInt;
use mina_curves::pasta::Vesta;
use mina_hasher::Fp;
use mina_p2p_messages::{bigint::BigInt, v2::PicklesProofProofsVerified2ReprStableV2};
use poly_commitment::{commitment::CommitmentCurve, srs::SRS};

use super::public_input::scalar_challenge::ScalarChallenge;
use super::urs_utils;

pub fn accumulator_check(
    urs: &SRS<Vesta>,
    proof: &PicklesProofProofsVerified2ReprStableV2,
) -> Result<bool, InvalidBigInt> {
    // accumulator check
    // Note:
    // comms: statement.proof_state.messages_for_next_wrap_proof.challenge_polynomial_commitment
    // chals: statement.proof_state.deferred_values.bulletproof_challenges

    let deferred_values = &proof.statement.proof_state.deferred_values;
    let bulletproof_challenges = &deferred_values.bulletproof_challenges;
    let bulletproof_challenges: Vec<Fp> = bulletproof_challenges
        .iter()
        .map(|chal| {
            let prechallenge = &chal.prechallenge.inner;
            let prechallenge: [u64; 2] = prechallenge.each_ref().map(|c| c.as_u64());

            ScalarChallenge::limbs_to_field(&prechallenge)
        })
        .collect();

    let of_coord =
        |(x, y): &(BigInt, BigInt)| Ok(Vesta::of_coordinates(x.to_field()?, y.to_field()?));

    // statement.proof_state.messages_for_next_wrap_proof.challenge_polynomial_commitment
    let acc_comm = &proof
        .statement
        .proof_state
        .messages_for_next_wrap_proof
        .challenge_polynomial_commitment;
    let acc_comm: Vesta = of_coord(acc_comm)?;

    let acc_check =
        urs_utils::batch_dlog_accumulator_check(urs, &[acc_comm], &bulletproof_challenges);

    if !acc_check {
        println!("accumulator_check failed");
    }

    Ok(acc_check)
}
