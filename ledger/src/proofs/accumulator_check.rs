use ark_ff::fields::arithmetic::InvalidBigInt;
use mina_curves::pasta::Vesta;
use mina_hasher::Fp;
use mina_p2p_messages::{bigint::BigInt, v2::PicklesProofProofsVerified2ReprStableV2};
use poly_commitment::{commitment::CommitmentCurve, srs::SRS};

use super::public_input::scalar_challenge::ScalarChallenge;
use super::urs_utils;

pub fn accumulator_check(
    urs: &SRS<Vesta>,
    proofs: &[&PicklesProofProofsVerified2ReprStableV2],
) -> Result<bool, InvalidBigInt> {
    // accumulator check
    // https://github.com/MinaProtocol/mina/blob/fb1c3c0a408c344810140bdbcedacc532a11be91/src/lib/pickles/common.ml#L191-L204
    // Note:
    // comms: statement.proof_state.messages_for_next_wrap_proof.challenge_polynomial_commitment
    //        Array.of_list_map comm_chals ~f:(fun (comm, _) -> Or_infinity.Finite comm )
    // chals: statement.proof_state.deferred_values.bulletproof_challenges
    //        Array.concat @@ List.map comm_chals ~f:(fun (_, chals) -> Vector.to_array chals)

    let mut comms = Vec::with_capacity(proofs.len());
    let mut bulletproof_challenges = vec![];

    for proof in proofs {
        let chals = &proof
            .statement
            .proof_state
            .deferred_values
            .bulletproof_challenges;
        let mut chals: Vec<Fp> = chals
            .iter()
            .map(|chal| {
                let prechallenge = &chal.prechallenge.inner;
                let prechallenge: [u64; 2] = prechallenge.each_ref().map(|c| c.as_u64());

                ScalarChallenge::limbs_to_field(&prechallenge)
            })
            .collect();

        bulletproof_challenges.append(&mut chals);

        let of_coord =
            |(x, y): &(BigInt, BigInt)| Ok(Vesta::of_coordinates(x.to_field()?, y.to_field()?));

        // statement.proof_state.messages_for_next_wrap_proof.challenge_polynomial_commitment
        let acc_comm = &proof
            .statement
            .proof_state
            .messages_for_next_wrap_proof
            .challenge_polynomial_commitment;
        let acc_comm: Vesta = of_coord(acc_comm)?;

        comms.push(acc_comm);
    }

    let acc_check = urs_utils::batch_dlog_accumulator_check(urs, &comms, &bulletproof_challenges);

    if !acc_check {
        println!("accumulator_check failed");
    }

    Ok(acc_check)
}
