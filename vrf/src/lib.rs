use ark_ec::AffineCurve;
use ark_ff::PrimeField;
use ledger::AccountIndex;
use message::VrfMessage;
use mina_p2p_messages::{
    bigint::BigInt as MinaBigInt,
    v2::{EpochSeed, MinaBaseEpochSeedStableV1},
};
use num::BigInt;
use output::VrfOutput;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use mina_curves::pasta::curves::pallas::Pallas as CurvePoint;
use mina_signer::Keypair;
use threshold::Threshold;

mod message;
pub mod output;
mod serialize;
mod threshold;

type VrfResult<T> = std::result::Result<T, VrfError>;
type BaseField = <CurvePoint as AffineCurve>::BaseField;
type ScalarField = <CurvePoint as AffineCurve>::ScalarField;

#[derive(Error, Debug)]
pub enum VrfError {
    #[error("Failed to decode field from hex string: {0}")]
    HexDecodeError(#[from] hex::FromHexError),

    #[error("Failed to parse decimal big integer from string: {0}")]
    BigIntParseError(#[from] num::bigint::ParseBigIntError),

    #[error("Field conversion error: {0}")]
    FieldHelpersError(#[from] o1_utils::field_helpers::FieldHelpersError),

    #[error("Failed to decode the base58 string: {0}")]
    Base58DecodeError(#[from] bs58::decode::Error),

    #[error("Scalar field does not exists from repr: {0:?}")]
    ScalarFieldFromReprError(BaseField),

    #[error("Cannot convert rational to f64")]
    RationalToF64,

    #[error("Cannot find a curve point for {0}")]
    ToGroupError(BaseField),

    #[error("The vrf_output is missing from the witness")]
    NoOutputError,

    #[error("The witness is invalid")]
    IvalidWitness,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VrfWonSlot {
    pub producer: String,
    pub winner_account: String,
    pub global_slot: u32,
    pub account_index: AccountIndex,
    pub vrf_output: VrfOutput,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VrfEvaluationOutput {
    SlotWon(VrfWonSlot),
    SlotLost(u32),
}

impl VrfEvaluationOutput {
    pub fn global_slot(&self) -> u32 {
        match self {
            Self::SlotWon(won_slot) => won_slot.global_slot,
            Self::SlotLost(global_slot) => *global_slot,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VrfEvaluationInput {
    producer_key: Keypair,
    global_slot: u32,
    epoch_seed: EpochSeed,
    account_pub_key: String,
    delegator_index: AccountIndex,
    delegated_stake: BigInt,
    total_currency: BigInt,
}

impl VrfEvaluationInput {
    pub fn new(
        producer_key: Keypair,
        epoch_seed: EpochSeed,
        account_pub_key: String,
        global_slot: u32,
        delegator_index: AccountIndex,
        delegated_stake: BigInt,
        total_currency: BigInt,
    ) -> Self {
        Self {
            producer_key,
            global_slot,
            epoch_seed,
            delegator_index,
            delegated_stake,
            total_currency,
            account_pub_key,
        }
    }
}

/// Generates the VRF output for the genesis block
pub fn genesis_vrf() -> VrfResult<VrfOutput> {
    let genesis_keypair =
        keypair_from_bs58_string("EKFKgDtU3rcuFTVSEpmpXSkukjmX4cKefYREi6Sdsk7E7wsT7KRw");

    let epoch_seed = EpochSeed::from(MinaBaseEpochSeedStableV1(MinaBigInt::zero()));

    calculate_vrf(&genesis_keypair, epoch_seed, 0, &AccountIndex(0))
}

/// Calculates the VRF output
fn calculate_vrf(
    producer_key: &Keypair,
    epoch_seed: EpochSeed,
    global_slot: u32,
    delegator_index: &AccountIndex,
) -> VrfResult<VrfOutput> {
    let vrf_message = VrfMessage::new(global_slot, epoch_seed, delegator_index.as_u64());

    let vrf_message_hash_curve_point = vrf_message.to_group()?;

    let scaled_message_hash =
        producer_key.secret_multiply_with_curve_point(vrf_message_hash_curve_point);

    Ok(VrfOutput::new(vrf_message, scaled_message_hash))
}

/// Evaluate vrf with a specific input. Used by the block producer
pub fn evaluate_vrf(vrf_input: VrfEvaluationInput) -> VrfResult<VrfEvaluationOutput> {
    let VrfEvaluationInput {
        producer_key,
        global_slot,
        epoch_seed,
        delegator_index,
        delegated_stake,
        total_currency,
        account_pub_key,
    } = vrf_input;

    let vrf_output = calculate_vrf(&producer_key, epoch_seed, global_slot, &delegator_index)?;

    let slot_won = Threshold::new(delegated_stake, total_currency)
        .threshold_met(vrf_output.truncated().into_repr());

    if slot_won {
        Ok(VrfEvaluationOutput::SlotWon(VrfWonSlot {
            producer: producer_key.get_address(),
            vrf_output,
            winner_account: account_pub_key,
            global_slot,
            account_index: delegator_index,
        }))
    } else {
        Ok(VrfEvaluationOutput::SlotLost(global_slot))
    }
}

fn keypair_from_bs58_string(str: &str) -> Keypair {
    let mut secret_hex_vec = bs58::decode(str).into_vec().unwrap();
    secret_hex_vec = secret_hex_vec[2..secret_hex_vec.len() - 4].to_vec();
    secret_hex_vec.reverse();
    let secret_hex = hex::encode(secret_hex_vec);
    Keypair::from_hex(&secret_hex).unwrap()
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use ledger::AccountIndex;
    use mina_p2p_messages::v2::EpochSeed;
    use num::BigInt;

    use crate::{genesis_vrf, keypair_from_bs58_string, VrfEvaluationInput, VrfEvaluationOutput};

    use super::evaluate_vrf;

    #[test]
    fn test_genesis_vrf() {
        let out = genesis_vrf().unwrap();
        let expected = "48H9Qk4D6RzS9kAJQX9HCDjiJ5qLiopxgxaS6xbDCWNaKQMQ9Y4C";
        assert_eq!(expected, out.to_string());
    }

    #[test]
    fn test_evaluate_vrf_lost_slot() {
        let vrf_input = VrfEvaluationInput {
            producer_key: keypair_from_bs58_string(
                "EKEEpMELfQkMbJDt2fB4cFXKwSf1x4t7YD4twREy5yuJ84HBZtF9",
            ),
            epoch_seed: EpochSeed::from_str("2va9BGv9JrLTtrzZttiEMDYw1Zj6a6EHzXjmP9evHDTG3oEquURA")
                .unwrap(),
            global_slot: 518,
            delegator_index: AccountIndex(2),
            delegated_stake: BigInt::from_str("1000000000000000")
                .expect("Cannot convert to BigInt"),
            total_currency: BigInt::from_str("6000000000001000").expect("Cannot convert to BigInt"),
            account_pub_key: "Placeholder".to_string(),
        };
        let evaluation_result = evaluate_vrf(vrf_input.clone()).expect("Failed to evaluate vrf");
        assert_eq!(
            evaluation_result,
            VrfEvaluationOutput::SlotLost(vrf_input.global_slot)
        )
    }

    #[test]
    fn test_evaluate_vrf_won_slot() {
        let vrf_input = VrfEvaluationInput {
            producer_key: keypair_from_bs58_string(
                "EKEEpMELfQkMbJDt2fB4cFXKwSf1x4t7YD4twREy5yuJ84HBZtF9",
            ),
            epoch_seed: EpochSeed::from_str("2va9BGv9JrLTtrzZttiEMDYw1Zj6a6EHzXjmP9evHDTG3oEquURA")
                .unwrap(),
            global_slot: 6,
            delegator_index: AccountIndex(2),
            delegated_stake: BigInt::from_str("1000000000000000")
                .expect("Cannot convert to BigInt"),
            total_currency: BigInt::from_str("6000000000001000").expect("Cannot convert to BigInt"),
            account_pub_key: "Placeholder".to_string(),
        };

        let evaluation_result = evaluate_vrf(vrf_input).expect("Failed to evaluate vrf");

        if let VrfEvaluationOutput::SlotWon(won_slot) = evaluation_result {
            assert_eq!(
                "48HHFYbaz4d7XkJpWWJw5jN1vEBfPvU31nsX4Ljn74jDo3WyTojL",
                won_slot.vrf_output.to_string()
            );
            assert_eq!(0.16978997004532187, won_slot.vrf_output.fractional());
            assert_eq!(
                "B62qrztYfPinaKqpXaYGY6QJ3SSW2NNKs7SajBLF1iFNXW9BoALN2Aq",
                won_slot.producer
            );
        } else {
            panic!("Slot should have been won!")
        }

        // assert_eq!(expected, evaluation_result)
    }

    #[test]
    #[ignore]
    fn test_slot_calculation_time_big_producer() {
        let start = std::time::Instant::now();
        for i in 1..14403 {
            let vrf_input = VrfEvaluationInput {
                producer_key: keypair_from_bs58_string(
                    "EKEEpMELfQkMbJDt2fB4cFXKwSf1x4t7YD4twREy5yuJ84HBZtF9",
                ),
                epoch_seed: EpochSeed::from_str(
                    "2va9BGv9JrLTtrzZttiEMDYw1Zj6a6EHzXjmP9evHDTG3oEquURA",
                )
                .unwrap(),
                global_slot: 6,
                delegator_index: AccountIndex(i),
                delegated_stake: BigInt::from_str("1000000000000000")
                    .expect("Cannot convert to BigInt"),
                total_currency: BigInt::from_str("6000000000001000")
                    .expect("Cannot convert to BigInt"),
                account_pub_key: "Placeholder".to_string(),
            };
            let _ = evaluate_vrf(vrf_input).expect("Failed to evaluate VRF");
            if i % 100 == 0 {
                println!("Delegators evaluated: {}", i);
            }
        }
        let elapsed = start.elapsed();
        println!("Duration: {}", elapsed.as_secs());
    }

    #[test]
    #[ignore]
    fn test_first_winning_slot() {
        for i in 0..7000 {
            let vrf_input = VrfEvaluationInput {
                producer_key: keypair_from_bs58_string(
                    "EKEEpMELfQkMbJDt2fB4cFXKwSf1x4t7YD4twREy5yuJ84HBZtF9",
                ),
                epoch_seed: EpochSeed::from_str(
                    "2va9BGv9JrLTtrzZttiEMDYw1Zj6a6EHzXjmP9evHDTG3oEquURA",
                )
                .unwrap(),
                global_slot: i,
                delegator_index: AccountIndex(2),
                delegated_stake: BigInt::from_str("1000000000000000")
                    .expect("Cannot convert to BigInt"),
                total_currency: BigInt::from_str("6000000000001000")
                    .expect("Cannot convert to BigInt"),
                account_pub_key: "Placeholder".to_string(),
            };
            let evaluation_result =
                evaluate_vrf(vrf_input.clone()).expect("Failed to evaluate vrf");
            if evaluation_result != VrfEvaluationOutput::SlotLost(vrf_input.global_slot) {
                println!("{:?}", evaluation_result);
            }
        }
    }
}
