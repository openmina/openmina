use ark_ec::AffineCurve;
use ark_ff::{BigInteger, BigInteger256, One, PrimeField, SquareRootField, Zero};
use ledger::AccountIndex;
// use keypair::Keypair;
use message::VrfMessage;
use mina_p2p_messages::bigint::BigInt as MinaBigInt;
use num::{BigInt, BigRational, ToPrimitive};
use output::VrfOutputHashInput;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use mina_curves::pasta::curves::pallas::Pallas as CurvePoint;
use mina_hasher::{create_kimchi, Hasher};
use mina_signer::Keypair;
use o1_utils::FieldHelpers;
use threshold::Threshold;

mod message;
mod output;
mod threshold;

// TODO: remove after finalization
pub use num::*;

type VrfResult<T> = std::result::Result<T, VrfError>;
type BaseField = <CurvePoint as AffineCurve>::BaseField;
type ScalarField = <CurvePoint as AffineCurve>::ScalarField;

#[derive(Error, Debug)]
pub enum VrfError {
    #[error("Failed to decode field from hex string: {0}")]
    HexDecodeError(#[from] hex::FromHexError),

    #[error("Failed to parse decimal big integer from string: {0}")]
    BigIntParseError(#[from] num::bigint::ParseBigIntError),

    // #[error("PubkeyError: {0}")]
    // PubKeyError(#[from] crate::pubkey::PubKeyError),
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
    pub vrf_output: String,
    pub vrf_output_bytes: Vec<u8>,
    pub vrf_fractional: f64,
    pub global_slot: u32,
    pub account_index: AccountIndex,
    pub vrf_hash: MinaBigInt,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VrfEvaluationOutput {
    SlotWon(VrfWonSlot),
    SlotLost(u32), // TODO(adonagy): create or use existing a type
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
    epoch_seed: String,
    account_pub_key: String,
    // TODO(adonagy): part of the delegatee table, pass the table when finished
    delegator_index: AccountIndex,
    delegated_stake: BigInt,
    total_currency: BigInt,
}

impl VrfEvaluationInput {
    pub fn new(
        producer_key: Keypair,
        epoch_seed: String,
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

// TODO(adonagy): inputs, outputs
/// Evaluate vrf with a specific input. Used by the block producer
pub fn evaluate_vrf(vrf_input: VrfEvaluationInput) -> VrfResult<VrfEvaluationOutput> {
    // TODO(adonagy): mocked, move to inputs
    // let producer_key = Keypair::from_bs58_string("EKEEpMELfQkMbJDt2fB4cFXKwSf1x4t7YD4twREy5yuJ84HBZtF9");
    // let global_slot = 518;
    // let epoch_seed_str = "2va9BGv9JrLTtrzZttiEMDYw1Zj6a6EHzXjmP9evHDTG3oEquURA";
    // let delegator_index = 2;
    // let total_currency: BigInt = BigInt::from_str("6000000000001000")?;
    // let delegated_stake: BigInt = BigInt::from_str("1000000000000000")?;

    let VrfEvaluationInput {
        producer_key,
        global_slot,
        epoch_seed,
        delegator_index,
        delegated_stake,
        total_currency,
        account_pub_key,
    } = vrf_input;

    let epoch_seed = seed_to_basefield(&epoch_seed);
    let vrf_message = VrfMessage::new(global_slot, epoch_seed, delegator_index.as_u64());

    let mut hasher = create_kimchi::<VrfMessage>(());
    let vrf_message_hash = hasher.update(&vrf_message).digest();
    let vrf_message_hash_group = to_group(vrf_message_hash)?;

    let vrf_message_hash_curve_point =
        CurvePoint::new(vrf_message_hash_group.0, vrf_message_hash_group.1, false);
    // let scaled_message_hash = vrf_message_hash_curve_point.mul(producer_key.secret.clone().into_scalar()).into_affine();
    let scaled_message_hash =
        producer_key.secret_multiply_with_curve_point(vrf_message_hash_curve_point);

    let vrf_output_hash_input = VrfOutputHashInput::new(vrf_message, scaled_message_hash);

    let mut hasher = create_kimchi::<VrfOutputHashInput>(());
    let vrf_output_hash = hasher.update(&vrf_output_hash_input).digest();

    let vrf_output_hash_bits = vrf_output_hash.to_bits();
    let vrf_output_hash_scalar_repr =
        BigInteger256::from_bits_le(&vrf_output_hash_bits[..vrf_output_hash_bits.len() - 3]);

    let vrf_output_hash_scalar = ScalarField::from_repr(vrf_output_hash_scalar_repr).unwrap();
    let slot_won =
        Threshold::new(delegated_stake, total_currency).threshold_met(vrf_output_hash_scalar_repr);

    let vrf_output_string = {
        // VRF output prefix
        let mut output_bytes = vec![0x15, 0x20];
        output_bytes.extend(vrf_output_hash_scalar.to_bytes());
        // checksum
        let checksum_hash = Sha256::digest(&Sha256::digest(&output_bytes[..])[..]);
        output_bytes.extend(&checksum_hash[..4]);

        bs58::encode(output_bytes).into_string()
    };

    if slot_won {
        Ok(VrfEvaluationOutput::SlotWon(VrfWonSlot {
            producer: producer_key.get_address(),
            vrf_output: vrf_output_string,
            vrf_output_bytes: vrf_output_hash_scalar.to_bytes(),
            winner_account: account_pub_key,
            vrf_fractional: get_fractional(vrf_output_hash_scalar_repr)
                .to_f64()
                .unwrap(),
            global_slot,
            account_index: delegator_index,
            vrf_hash: MinaBigInt::from(vrf_output_hash),
        }))
    } else {
        Ok(VrfEvaluationOutput::SlotLost(global_slot))
    }
}

// TODO(adonagy): unwraps
pub fn seed_to_basefield(seed: &str) -> BaseField {
    let bytes = bs58::decode(seed).into_vec().unwrap();
    let raw = &bytes[2..bytes.len() - 4];
    BaseField::from_bytes(raw).unwrap()
}

fn to_group(t: BaseField) -> VrfResult<(BaseField, BaseField)> {
    // helpers
    let two = BaseField::one() + BaseField::one();
    let three = two + BaseField::one();

    // params, according to ocaml
    let mut projection_point_z_bytes =
        hex::decode("1AF731EC3CA2D77CC5D13EDC8C9A0A77978CB5F4FBFCC470B5983F5B6336DB69")?;
    projection_point_z_bytes.reverse();
    let projection_point_z = BaseField::from_bytes(&projection_point_z_bytes)?;
    let projection_point_y = BaseField::one();
    let conic_c = three;
    let u_over_2 = BaseField::one();
    let u = two;

    // field to conic
    let ct = conic_c * t;
    let s = two * ((ct * projection_point_y) + projection_point_z) / ((ct * t) + BaseField::one());
    let conic_z = projection_point_z - s;
    let conic_y = projection_point_y - (s * t);

    // conic to s
    let v = (conic_z / conic_y) - u_over_2;
    let y = conic_y;

    // s to v
    let x1 = v;
    let x2 = -(u + v);
    let x3 = u + (y * y);

    let get_y = |x: BaseField| -> Option<BaseField> {
        let five = BaseField::one()
            + BaseField::one()
            + BaseField::one()
            + BaseField::one()
            + BaseField::one();
        let mut res = x;
        res *= &x; // x^2
        res += BaseField::zero(); // x^2 + A
        res *= &x; // x^3 + A x
        res += five; // x^3 + A x + B
        res.sqrt()
    };

    for x in [x1, x2, x3] {
        if let Some(y) = get_y(x) {
            return Ok((x, y));
        }
    }

    Err(VrfError::ToGroupError(t))
}

/// Converts an integer to a rational
pub fn get_fractional(vrf_out: BigInteger256) -> BigRational {
    // ocaml:   Bignum_bigint.(shift_left one length_in_bits))
    //          where: length_in_bits = Int.min 256 (Field.size_in_bits - 2)
    //                 Field.size_in_bits = 255
    let two_tpo_256 = BigInt::one() << 253u32;

    let vrf_out = BigInt::from_bytes_be(num::bigint::Sign::Plus, &vrf_out.to_bytes_be());

    BigRational::new(vrf_out, two_tpo_256)
}

pub fn keypair_from_bs58_string(str: &str) -> Keypair {
    let mut secret_hex_vec = bs58::decode(str).into_vec().unwrap();
    secret_hex_vec = secret_hex_vec[2..secret_hex_vec.len() - 4].to_vec();
    secret_hex_vec.reverse();
    let secret_hex = hex::encode(secret_hex_vec);
    Keypair::from_hex(&secret_hex).unwrap()
    // Self::from_hex(&secret_hex).unwrap()
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use ledger::AccountIndex;
    use mina_p2p_messages::bigint::BigInt as MinaBigInt;
    use num::BigInt;

    use crate::{keypair_from_bs58_string, VrfEvaluationInput, VrfEvaluationOutput, VrfWonSlot};

    use super::evaluate_vrf;

    #[test]
    fn test_evaluate_vrf_lost_slot() {
        let vrf_input = VrfEvaluationInput {
            producer_key: keypair_from_bs58_string(
                "EKEEpMELfQkMbJDt2fB4cFXKwSf1x4t7YD4twREy5yuJ84HBZtF9",
            ),
            epoch_seed: "2va9BGv9JrLTtrzZttiEMDYw1Zj6a6EHzXjmP9evHDTG3oEquURA".to_string(),
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
            epoch_seed: "2va9BGv9JrLTtrzZttiEMDYw1Zj6a6EHzXjmP9evHDTG3oEquURA".to_string(),
            global_slot: 6,
            delegator_index: AccountIndex(2),
            delegated_stake: BigInt::from_str("1000000000000000")
                .expect("Cannot convert to BigInt"),
            total_currency: BigInt::from_str("6000000000001000").expect("Cannot convert to BigInt"),
            account_pub_key: "Placeholder".to_string(),
        };

        let vrf_hash_bytes: Box<[u8; 32]> = Box::new([
            241, 165, 64, 215, 216, 29, 243, 42, 40, 116, 181, 119, 58, 116, 33, 22, 122, 56, 85,
            92, 240, 3, 250, 110, 105, 62, 18, 17, 96, 235, 110, 37,
        ]);

        let expected = VrfEvaluationOutput::SlotWon(VrfWonSlot {
            producer: "B62qrztYfPinaKqpXaYGY6QJ3SSW2NNKs7SajBLF1iFNXW9BoALN2Aq".to_string(),
            winner_account: "Placeholder".to_string(),
            vrf_output: "48HHFYbaz4d7XkJpWWJw5jN1vEBfPvU31nsX4Ljn74jDo3WyTojL".to_string(),
            vrf_output_bytes: vec![
                241, 165, 64, 215, 216, 29, 243, 42, 40, 116, 181, 119, 58, 116, 33, 22, 122, 56,
                85, 92, 240, 3, 250, 110, 105, 62, 18, 17, 96, 235, 110, 5,
            ],
            vrf_fractional: 0.16978997004532187,
            global_slot: vrf_input.global_slot,
            account_index: AccountIndex(2),
            vrf_hash: MinaBigInt::from(vrf_hash_bytes),
        });
        let evaluation_result = evaluate_vrf(vrf_input).expect("Failed to evaluate vrf");
        assert_eq!(expected, evaluation_result)
    }

    #[test]
    fn test_slot_calculation_time_big_producer() {
        let start = std::time::Instant::now();
        for i in 1..14403 {
            let vrf_input = VrfEvaluationInput {
                producer_key: keypair_from_bs58_string(
                    "EKEEpMELfQkMbJDt2fB4cFXKwSf1x4t7YD4twREy5yuJ84HBZtF9",
                ),
                epoch_seed: "2va9BGv9JrLTtrzZttiEMDYw1Zj6a6EHzXjmP9evHDTG3oEquURA".to_string(),
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
    fn test_first_winning_slot() {
        for i in 0..7000 {
            let vrf_input = VrfEvaluationInput {
                producer_key: keypair_from_bs58_string(
                    "EKEEpMELfQkMbJDt2fB4cFXKwSf1x4t7YD4twREy5yuJ84HBZtF9",
                ),
                epoch_seed: "2va9BGv9JrLTtrzZttiEMDYw1Zj6a6EHzXjmP9evHDTG3oEquURA".to_string(),
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
