#![cfg(feature = "hashing")]
use std::{fmt, io};

use ark_ff::FromBytes;
use binprot::BinProtWrite;
use generated::MinaStateBlockchainStateValueStableV2;
use mina_hasher::Fp;
use mina_poseidon::{
    constants::PlonkSpongeConstantsKimchi,
    pasta::fp_kimchi::static_params,
    poseidon::{ArithmeticSponge, Sponge},
};
use serde::{Deserialize, Serialize};
use sha2::{
    digest::{generic_array::GenericArray, typenum::U32},
    Digest, Sha256,
};

use crate::{
    bigint::BigInt,
    hash::MinaHash,
    hash_input::{Inputs, ToInput},
};

use super::{
    generated, Amount, ConsensusBodyReferenceStableV1, ConsensusGlobalSlotStableV1,
    ConsensusProofOfStakeDataConsensusStateValueStableV1,
    ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
    ConsensusVrfOutputTruncatedStableV1, MinaBaseEpochLedgerValueStableV1,
    MinaBaseFeeExcessStableV1, MinaBasePendingCoinbaseStackVersionedStableV1,
    MinaBasePendingCoinbaseStateStackStableV1, MinaBaseProtocolConstantsCheckedValueStableV1,
    MinaBaseStagedLedgerHashNonSnarkStableV1, MinaBaseStagedLedgerHashStableV1,
    MinaStateBlockchainStateValueStableV2LedgerProofStatement,
    MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
    MinaStateProtocolStateBodyValueStableV2, MinaStateProtocolStateValueStableV2,
    MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1,
    NonZeroCurvePointUncompressedStableV1, SgnStableV1, TokenFeeExcess,
};

impl generated::MinaBaseStagedLedgerHashNonSnarkStableV1 {
    pub fn sha256(&self) -> GenericArray<u8, U32> {
        let mut ledger_hash_bytes: [u8; 32] = [0; 32];

        ledger_hash_bytes.copy_from_slice(self.ledger_hash.as_ref());
        ledger_hash_bytes.reverse();

        let mut hasher = Sha256::new();
        hasher.update(ledger_hash_bytes);
        hasher.update(self.aux_hash.as_ref());
        hasher.update(self.pending_coinbase_aux.as_ref());

        hasher.finalize()
    }
}

impl generated::ConsensusVrfOutputTruncatedStableV1 {
    pub fn blake2b(&self) -> Vec<u8> {
        use blake2::{
            digest::{Update, VariableOutput},
            Blake2bVar,
        };
        let mut hasher = Blake2bVar::new(32).expect("Invalid Blake2bVar output size");
        hasher.update(&self.0);
        hasher.finalize_boxed().to_vec()
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct TransactionHash(Vec<u8>);

impl std::str::FromStr for TransactionHash {
    type Err = bs58::decode::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = bs58::decode(s).with_check(Some(0x1D)).into_vec()?[1..].to_vec();
        Ok(Self(bytes))
    }
}

impl fmt::Display for TransactionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        bs58::encode(&self.0)
            .with_check_version(0x1D)
            .into_string()
            .fmt(f)
    }
}

impl fmt::Debug for TransactionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
        // write!(f, "TransactionHash({})", self)
    }
}

impl Serialize for TransactionHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> serde::Deserialize<'de> for TransactionHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let b58: String = Deserialize::deserialize(deserializer)?;
            Ok(b58.parse().map_err(|err| serde::de::Error::custom(err))?)
        } else {
            Vec::deserialize(deserializer).map(|v| Self(v))
        }
    }
}

impl generated::MinaBaseUserCommandStableV2 {
    pub fn hash(&self) -> io::Result<TransactionHash> {
        match self {
            Self::SignedCommand(v) => v.hash(),
            Self::ZkappCommand(_) => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "zkapp tx hashing is not yet supported",
            )),
        }
    }
}

impl generated::MinaBaseSignedCommandStableV2 {
    pub fn binprot_write_with_default_sig(&self) -> io::Result<Vec<u8>> {
        let default_signature = generated::MinaBaseSignatureStableV1(BigInt::one(), BigInt::one());

        let mut encoded = vec![];
        self.payload.binprot_write(&mut encoded)?;
        self.signer.binprot_write(&mut encoded)?;
        default_signature.binprot_write(&mut encoded)?;
        Ok(encoded)
    }

    pub fn hash(&self) -> io::Result<TransactionHash> {
        use blake2::{
            digest::{Update, VariableOutput},
            Blake2bVar,
        };
        let mut hasher = Blake2bVar::new(32).expect("Invalid Blake2bVar output size");

        hasher.update(&self.binprot_write_with_default_sig()?);
        let mut hash = vec![0; 33];
        hash[..1].copy_from_slice(&[32]);
        hash[1..].copy_from_slice(&hasher.finalize_boxed());

        Ok(TransactionHash(hash))
    }
}

#[cfg(test)]
mod tests {
    use super::super::manual;
    use super::*;
    use binprot::BinProtRead;

    fn pub_key(address: &str) -> manual::NonZeroCurvePoint {
        let key = mina_signer::PubKey::from_address(address)
            .unwrap()
            .into_compressed();
        let v = generated::NonZeroCurvePointUncompressedStableV1 {
            x: crate::bigint::BigInt::from(key.x),
            is_odd: key.is_odd,
        };
        v.into()
    }

    fn tx_hash(
        from: &str,
        to: &str,
        amount: u64,
        fee: u64,
        nonce: u32,
        valid_until: i32,
    ) -> String {
        use crate::number::Number;
        use crate::string::CharString;

        let from = pub_key(from);
        let to = pub_key(to);

        let v = Number(fee as i64);
        let v = generated::UnsignedExtendedUInt64Int64ForVersionTagsStableV1(v);
        let fee = generated::CurrencyFeeStableV1(v);

        let nonce = generated::UnsignedExtendedUInt32StableV1(Number(nonce as i32));

        let valid_until = generated::UnsignedExtendedUInt32StableV1(Number(valid_until));

        let memo = bs58::decode("E4Yks7aARFemZJqucP5eaARRYRthGdzaFjGfXqQRS3UeidsECRBvR")
            .with_check(Some(0x14))
            .into_vec()
            .unwrap()[1..]
            .to_vec();
        let v = CharString::from(&memo[..]);
        let memo = generated::MinaBaseSignedCommandMemoStableV1(v);

        let common = generated::MinaBaseSignedCommandPayloadCommonStableV2 {
            fee,
            fee_payer_pk: from.clone(),
            nonce,
            valid_until,
            memo,
        };

        let v = Number(amount as i64);
        let v = generated::UnsignedExtendedUInt64Int64ForVersionTagsStableV1(v);
        let amount = generated::CurrencyAmountStableV1(v);

        let v = generated::MinaBasePaymentPayloadStableV2 {
            source_pk: from.clone(),
            receiver_pk: to.clone(),
            amount,
        };
        let body = generated::MinaBaseSignedCommandPayloadBodyStableV2::Payment(v);

        let payload = generated::MinaBaseSignedCommandPayloadStableV2 { common, body };

        // Some random signature. hasher should ignore it and use default.
        let signature = generated::MinaBaseSignatureStableV1(
            BigInt::binprot_read(&mut &[122; 32][..]).unwrap(),
            BigInt::binprot_read(&mut &[123; 32][..]).unwrap(),
        );

        let v = generated::MinaBaseSignedCommandStableV2 {
            payload,
            signer: from.clone(),
            signature: signature.into(),
        };
        let v = generated::MinaBaseUserCommandStableV2::SignedCommand(v);
        dbg!(v.hash().unwrap()).to_string()
    }

    // #[test]
    // fn test_tx_hash() {
    //     let s = "5JuSRViCY1GbnnpExLhoYLkD96vwnA97ZrbE2UFTFzBk9SPLsAyE";
    //     let hash: TransactionHash = s.parse().unwrap();
    //     // let bytes = bs58::decode(s).with_check(Some(0x1D)).into_vec().unwrap()[1..].to_vec();
    //     dbg!(bs58::encode(&hash.0)
    //         .with_check_version(0x12)
    //         .into_string());
    //     panic!();
    // }

    #[test]
    fn test_payment_hash_1() {
        let expected_hash = "5JthQdVqzEJRLBLALeuwPdbnGhFmCow2bVnkfHGH6vZ7R6fiMf2o";
        let expected_tx_hash: TransactionHash = expected_hash.parse().unwrap();
        dbg!(expected_tx_hash);

        assert_eq!(
            tx_hash(
                "B62qp3B9VW1ir5qL1MWRwr6ecjC2NZbGr8vysGeme9vXGcFXTMNXb2t",
                "B62qoieQNrsNKCNTZ6R4D6cib3NxVbkwZaAtRVbfS3ndrb2MkFJ1UVJ",
                1089541195,
                89541195,
                26100,
                -1
            ),
            expected_hash
        )
    }
}

impl MinaHash for MinaStateProtocolStateBodyValueStableV2 {
    fn hash(&self) -> mina_hasher::Fp {
        let mut inputs = Inputs::new();
        self.to_input(&mut inputs);
        hash_with_kimchi("MinaProtoStateBody", &inputs.to_fields())
    }
}

impl MinaHash for MinaStateProtocolStateValueStableV2 {
    fn hash(&self) -> mina_hasher::Fp {
        let mut inputs = Inputs::new();
        inputs.append_field(self.previous_state_hash.to_field());
        inputs.append_field(self.body.hash());
        hash_with_kimchi("MinaProtoState", &inputs.to_fields())
    }
}

fn param_to_field_impl(param: &str, default: &[u8; 32]) -> Fp {
    let param_bytes = param.as_bytes();
    let len = param_bytes.len();

    let mut fp = *default;
    fp[..len].copy_from_slice(param_bytes);

    Fp::read(&fp[..]).expect("fp read failed")
}

fn param_to_field(param: &str) -> Fp {
    const DEFAULT: [u8; 32] = [
        b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*', b'*',
        b'*', b'*', b'*', b'*', b'*', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    if param.len() > 20 {
        panic!("must be 20 byte maximum");
    }

    param_to_field_impl(param, &DEFAULT)
}

pub fn hash_with_kimchi(param: &str, fields: &[Fp]) -> Fp {
    let mut sponge = ArithmeticSponge::<Fp, PlonkSpongeConstantsKimchi>::new(static_params());

    sponge.absorb(&[param_to_field(param)]);
    sponge.squeeze();

    sponge.absorb(fields);
    sponge.squeeze()
}

macro_rules! to_input_fields {
    ( $inputs:expr, $( $field:ident ),* $(,)?) => {
        $(
            $field.to_input($inputs);
        )*
    };
}

impl ToInput for MinaStateProtocolStateBodyValueStableV2 {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaStateProtocolStateBodyValueStableV2 {
            genesis_state_hash,
            blockchain_state,
            consensus_state,
            constants,
        } = self;

        to_input_fields!(
            inputs,
            constants,
            genesis_state_hash,
            blockchain_state,
            consensus_state
        );
        //     constants.to_input(inputs);
        //     genesis_state_hash.to_input(inputs);
        //     blockchain_state.to_input(inputs);
        //     consensus_state.to_input(inputs);
    }
}

impl ToInput for MinaBaseProtocolConstantsCheckedValueStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaBaseProtocolConstantsCheckedValueStableV1 {
            k,
            slots_per_epoch,
            slots_per_sub_window,
            delta,
            genesis_state_timestamp,
        } = self;

        k.to_input(inputs);
        delta.to_input(inputs);
        slots_per_epoch.to_input(inputs);
        slots_per_sub_window.to_input(inputs);
        genesis_state_timestamp.to_input(inputs);
    }
}

impl ToInput for MinaStateBlockchainStateValueStableV2 {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaStateBlockchainStateValueStableV2 {
            staged_ledger_hash,
            genesis_ledger_hash,
            ledger_proof_statement,
            timestamp,
            body_reference,
        } = self;

        to_input_fields!(
            inputs,
            staged_ledger_hash,
            genesis_ledger_hash,
            ledger_proof_statement,
            timestamp,
            body_reference
        );
    }
}

impl ToInput for ConsensusProofOfStakeDataConsensusStateValueStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        let ConsensusProofOfStakeDataConsensusStateValueStableV1 {
            blockchain_length,
            epoch_count,
            min_window_density,
            sub_window_densities,
            last_vrf_output,
            total_currency,
            curr_global_slot,
            global_slot_since_genesis,
            staking_epoch_data,
            next_epoch_data,
            has_ancestor_in_same_checkpoint_window,
            block_stake_winner,
            block_creator,
            coinbase_receiver,
            supercharge_coinbase,
        } = self;
        to_input_fields!(
            inputs,
            blockchain_length,
            epoch_count,
            min_window_density,
            sub_window_densities,
            last_vrf_output,
            total_currency,
            curr_global_slot,
            global_slot_since_genesis,
            has_ancestor_in_same_checkpoint_window,
            supercharge_coinbase,
            staking_epoch_data,
            next_epoch_data,
            block_stake_winner,
            block_creator,
            coinbase_receiver,
        );
    }
}

impl ToInput for MinaBaseStagedLedgerHashStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaBaseStagedLedgerHashStableV1 {
            non_snark,
            pending_coinbase_hash,
        } = self;
        to_input_fields!(inputs, non_snark, pending_coinbase_hash);
    }
}

impl ToInput for MinaBaseStagedLedgerHashNonSnarkStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        inputs.append_bytes(self.sha256().as_ref());
    }
}

impl ToInput for MinaStateBlockchainStateValueStableV2LedgerProofStatement {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaStateBlockchainStateValueStableV2LedgerProofStatement {
            source,
            target,
            connecting_ledger_left,
            connecting_ledger_right,
            supply_increase,
            fee_excess,
            sok_digest: _,
        } = self;
        to_input_fields!(
            inputs,
            source,
            target,
            connecting_ledger_left,
            connecting_ledger_right,
            supply_increase,
            fee_excess
        );
    }
}

impl ToInput for ConsensusBodyReferenceStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        inputs.append_bytes(self.as_ref());
    }
}

impl ToInput for ConsensusVrfOutputTruncatedStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        let vrf: &[u8] = self.as_ref();
        inputs.append_bytes(&vrf[..31]);
        // Ignore the last 3 bits
        let last_byte = vrf[31];
        for bit in [1, 2, 4, 8, 16] {
            inputs.append_bool(last_byte & bit != 0);
        }
    }
}

impl ToInput for ConsensusGlobalSlotStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        let ConsensusGlobalSlotStableV1 {
            slot_number,
            slots_per_epoch,
        } = self;
        to_input_fields!(inputs, slot_number, slots_per_epoch);
    }
}

impl ToInput for ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        let ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 {
            ledger,
            seed,
            start_checkpoint,
            lock_checkpoint,
            epoch_length,
        } = self;
        to_input_fields!(
            inputs,
            seed,
            start_checkpoint,
            epoch_length,
            ledger,
            lock_checkpoint
        );
    }
}

impl ToInput for ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        let ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 {
            ledger,
            seed,
            start_checkpoint,
            lock_checkpoint,
            epoch_length,
        } = self;
        to_input_fields!(
            inputs,
            seed,
            start_checkpoint,
            epoch_length,
            ledger,
            lock_checkpoint
        );
    }
}

impl ToInput for NonZeroCurvePointUncompressedStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        let NonZeroCurvePointUncompressedStableV1 { x, is_odd } = self;
        to_input_fields!(inputs, x, is_odd);
    }
}

impl ToInput for MinaStateBlockchainStateValueStableV2LedgerProofStatementSource {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaStateBlockchainStateValueStableV2LedgerProofStatementSource {
            first_pass_ledger,
            second_pass_ledger,
            pending_coinbase_stack,
            local_state,
        } = self;
        to_input_fields!(
            inputs,
            first_pass_ledger,
            second_pass_ledger,
            pending_coinbase_stack,
            local_state
        );
    }
}

impl ToInput for Amount {
    fn to_input(&self, inputs: &mut Inputs) {
        let Amount { magnitude, sgn } = self;
        to_input_fields!(inputs, magnitude, sgn);
    }
}

impl ToInput for MinaBaseFeeExcessStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaBaseFeeExcessStableV1(left, right) = self;
        to_input_fields!(inputs, left, right);
    }
}

impl ToInput for TokenFeeExcess {
    fn to_input(&self, inputs: &mut Inputs) {
        let TokenFeeExcess { token, amount } = self;
        to_input_fields!(inputs, token, amount);
    }
}

impl ToInput for MinaBaseEpochLedgerValueStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaBaseEpochLedgerValueStableV1 {
            hash,
            total_currency,
        } = self;
        to_input_fields!(inputs, hash, total_currency);
    }
}

impl ToInput for MinaBasePendingCoinbaseStackVersionedStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaBasePendingCoinbaseStackVersionedStableV1 { data, state } = self;
        to_input_fields!(inputs, data, state);
    }
}

impl ToInput for MinaBasePendingCoinbaseStateStackStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaBasePendingCoinbaseStateStackStableV1 { init, curr } = self;
        to_input_fields!(inputs, init, curr);
    }
}

impl ToInput for MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        let MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 {
            stack_frame,
            call_stack,
            transaction_commitment,
            full_transaction_commitment,
            token_id,
            excess,
            supply_increase,
            ledger,
            success,
            account_update_index,
            failure_status_tbl: _,
            will_succeed,
        } = self;
        to_input_fields!(
            inputs,
            stack_frame,
            call_stack,
            transaction_commitment,
            full_transaction_commitment,
            token_id,
            excess,
            supply_increase,
            ledger,
            account_update_index,
            success,
            will_succeed,
        );
    }
}

impl ToInput for SgnStableV1 {
    fn to_input(&self, inputs: &mut Inputs) {
        inputs.append_bool(self == &SgnStableV1::Pos);
    }
}

// impl ToInput for  {
//     fn to_input(&self, inputs: &mut Inputs) {
//         todo!()
//     }
// }

// impl ToInput for  {
//     fn to_input(&self, inputs: &mut Inputs) {
//         todo!()
//     }
// }

// impl ToInput for  {
//     fn to_input(&self, inputs: &mut Inputs) {
//         todo!()
//     }
// }

// impl ToInput for  {
//     fn to_input(&self, inputs: &mut Inputs) {
//         todo!()
//     }
// }

// impl ToInput for  {
//     fn to_input(&self, inputs: &mut Inputs) {
//         todo!()
//     }
// }

// impl ToInput for  {
//     fn to_input(&self, inputs: &mut Inputs) {
//         todo!()
//     }
// }

// impl ToInput for  {
//     fn to_input(&self, inputs: &mut Inputs) {
//         todo!()
//     }
// }

#[cfg(test)]
mod hash_tests {
    use crate::{
        hash::MinaHash,
        v2::{DataHashLibStateHashStableV1, MinaStateProtocolStateValueStableV2, StateHash},
    };

    #[test]
    fn state_hash() {
        const HASH: &str = "3NKpXp2SXWGC3XHnAJYjGtNcbq8tzossqj6kK4eGr6mSyJoFmpxR";
        const JSON: &str = include_str!("../../tests/files/v2/state/617-3NKpXp2SXWGC3XHnAJYjGtNcbq8tzossqj6kK4eGr6mSyJoFmpxR.json");

        let state: MinaStateProtocolStateValueStableV2 = serde_json::from_str(JSON).unwrap();
        let hash = StateHash::from(DataHashLibStateHashStableV1(state.hash().into()));
        let expected_hash = serde_json::from_value(serde_json::json!(HASH)).unwrap();
        assert_eq!(hash, expected_hash)
    }
}
