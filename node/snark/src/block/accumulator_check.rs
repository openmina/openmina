use std::array;

use commitment_dlog::{commitment::CommitmentCurve, srs::SRS};
use kimchi::curve::KimchiCurve;
use mina_curves::pasta::{Pallas, Vesta};
use mina_hasher::Fp;
use mina_p2p_messages::{bigint::BigInt, v2::PicklesProofProofsVerified2ReprStableV2};

use super::urs_utils;
use crate::public_input::scalar_challenge::{endo_fp, ScalarChallenge};

const OTHER_URS_LENGTH: usize = 65536;

pub fn get_srs() -> super::VerifierSRS {
    // We need an URS with 65536 points (should be in the other verfifier index - step?)
    SRS::<<Pallas as KimchiCurve>::OtherCurve>::create(OTHER_URS_LENGTH)
}

pub fn accumulator_check(
    urs: &super::VerifierSRS,
    proof: &PicklesProofProofsVerified2ReprStableV2,
) -> bool {
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
            let prechallenge: [u64; 2] = array::from_fn(|k| prechallenge[k].as_u64());

            ScalarChallenge::from(prechallenge).to_field(&endo_fp())
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
