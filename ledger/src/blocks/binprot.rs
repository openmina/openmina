use std::{fmt::Display, marker::PhantomData, str::FromStr};

use ark_ff::One;
use mina_hasher::Fp;
use mina_signer::CompressedPubKey;
use serde::{de::Visitor, Deserialize, Serialize};

use crate::{
    BigInt, CurveAffine, MessagesForNextStepProof, MinaBaseVerificationKeyWireStableV1WrapIndex,
    PlonkVerificationKeyEvals,
};

/// String of bytes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteString(Vec<u8>);

impl AsRef<[u8]> for ByteString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for ByteString {
    fn from(source: Vec<u8>) -> Self {
        Self(source)
    }
}

impl From<&str> for ByteString {
    fn from(source: &str) -> Self {
        Self(source.as_bytes().to_vec())
    }
}

impl Serialize for ByteString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if !serializer.is_human_readable() {
            return self.0.serialize(serializer);
        }
        serializer.serialize_str(&hex::encode(&self.0))
    }
}

impl<'de> Deserialize<'de> for ByteString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if !deserializer.is_human_readable() {
            return Vec::<u8>::deserialize(deserializer).map(Self);
        }
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Vec<u8>;

            fn expecting(
                &self,
                formatter: &mut serde::__private::fmt::Formatter,
            ) -> serde::__private::fmt::Result {
                formatter.write_str("hex string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                hex::decode(v)
                    .map_err(|_| serde::de::Error::custom(format!("failed to decode hex str")))
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                println!("BUF={:?}", v);
                Ok(v)
            }
        }
        deserializer.deserialize_byte_buf(V).map(Self)
        // deserializer.deserialize_str(V).map(Self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataHashLibStateHashStableV1(pub BigInt);

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseLedgerHash0StableV1(pub BigInt);

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseStagedLedgerHashAuxHashStableV1(pub ByteString);

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1(pub ByteString);

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseStagedLedgerHashNonSnarkStableV1 {
    pub ledger_hash: MinaBaseLedgerHash0StableV1,
    pub aux_hash: MinaBaseStagedLedgerHashAuxHashStableV1,
    pub pending_coinbase_aux: MinaBaseStagedLedgerHashPendingCoinbaseAuxStableV1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaStateProtocolStateValueStableV2 {
    pub previous_state_hash: DataHashLibStateHashStableV1,
    pub body: MinaStateProtocolStateBodyValueStableV2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBasePendingCoinbaseHashBuilderStableV1(pub BigInt);

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBasePendingCoinbaseHashVersionedStableV1(
    pub MinaBasePendingCoinbaseHashBuilderStableV1,
);

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseStagedLedgerHashStableV1 {
    pub non_snark: MinaBaseStagedLedgerHashNonSnarkStableV1,
    pub pending_coinbase_hash: MinaBasePendingCoinbaseHashVersionedStableV1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnsignedExtendedUInt64StableV1(pub i64);
// pub struct UnsignedExtendedUInt64StableV1(pub Int64);

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrencyMakeStrAmountMakeStrStableV1(pub i64);
// pub struct CurrencyMakeStrAmountMakeStrStableV1(pub UnsignedExtendedUInt64StableV1);

#[derive(Debug, Serialize, Deserialize)]
pub enum SgnStableV1 {
    Pos,
    Neg,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaTransactionLogicPartiesLogicLocalStateValueStableV1Excess {
    pub magnitude: CurrencyMakeStrAmountMakeStrStableV1,
    pub sgn: SgnStableV1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseStackFrameDigestStableV1(pub BigInt);

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseCallStackDigestStableV1(pub BigInt);

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseAccountIdMakeStrDigestStableV1(pub BigInt);

#[derive(Debug, Serialize, Deserialize)]
pub struct UnsignedExtendedUInt32StableV1(pub i32);
// pub struct UnsignedExtendedUInt32StableV1(pub Int32);

#[derive(Debug, Serialize, Deserialize)]
pub enum MinaBaseTransactionStatusFailureStableV2 {
    Predicate,
    SourceNotPresent,
    ReceiverNotPresent,
    AmountInsufficientToCreateAccount,
    CannotPayCreationFeeInToken,
    SourceInsufficientBalance,
    SourceMinimumBalanceViolation,
    ReceiverAlreadyExists,
    TokenOwnerNotCaller,
    Overflow,
    GlobalExcessOverflow,
    LocalExcessOverflow,
    SignedCommandOnZkappAccount,
    ZkappAccountNotPresent,
    UpdateNotPermittedBalance,
    UpdateNotPermittedTimingExistingAccount,
    UpdateNotPermittedDelegate,
    UpdateNotPermittedAppState,
    UpdateNotPermittedVerificationKey,
    UpdateNotPermittedSequenceState,
    UpdateNotPermittedZkappUri,
    UpdateNotPermittedTokenSymbol,
    UpdateNotPermittedPermissions,
    UpdateNotPermittedNonce,
    UpdateNotPermittedVotingFor,
    PartiesReplayCheckFailed,
    FeePayerNonceMustIncrease,
    FeePayerMustBeSigned,
    AccountBalancePreconditionUnsatisfied,
    AccountNoncePreconditionUnsatisfied,
    AccountReceiptChainHashPreconditionUnsatisfied,
    AccountDelegatePreconditionUnsatisfied,
    AccountSequenceStatePreconditionUnsatisfied,
    AccountAppStatePreconditionUnsatisfied(i32),
    AccountProvedStatePreconditionUnsatisfied,
    AccountIsNewPreconditionUnsatisfied,
    ProtocolStatePreconditionUnsatisfied,
    IncorrectNonce,
    InvalidFeeExcess,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseTransactionStatusFailureCollectionStableV1(
    pub Vec<Vec<MinaBaseTransactionStatusFailureStableV2>>,
);

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaTransactionLogicPartiesLogicLocalStateValueStableV1 {
    pub stack_frame: MinaBaseStackFrameDigestStableV1,
    pub call_stack: MinaBaseCallStackDigestStableV1,
    pub transaction_commitment: BigInt,
    pub full_transaction_commitment: BigInt,
    pub token_id: MinaBaseAccountIdMakeStrDigestStableV1,
    pub excess: MinaTransactionLogicPartiesLogicLocalStateValueStableV1Excess,
    pub ledger: MinaBaseLedgerHash0StableV1,
    pub success: bool,
    pub party_index: UnsignedExtendedUInt32StableV1,
    pub failure_status_tbl: MinaBaseTransactionStatusFailureCollectionStableV1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaStateBlockchainStateValueStableV2Registers {
    pub ledger: MinaBaseLedgerHash0StableV1,
    pub pending_coinbase_stack: (),
    pub local_state: MinaTransactionLogicPartiesLogicLocalStateValueStableV1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockTimeMakeStrTimeStableV1(pub UnsignedExtendedUInt64StableV1);

#[derive(Debug, Serialize, Deserialize)]
pub struct Blake2MakeStableV1(pub ByteString);

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusBodyReferenceStableV1(pub Blake2MakeStableV1);

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaStateBlockchainStateValueStableV2 {
    pub staged_ledger_hash: MinaBaseStagedLedgerHashStableV1,
    pub genesis_ledger_hash: MinaBaseLedgerHash0StableV1,
    pub registers: MinaStateBlockchainStateValueStableV2Registers,
    pub timestamp: BlockTimeMakeStrTimeStableV1,
    pub body_reference: ConsensusBodyReferenceStableV1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusVrfOutputTruncatedStableV1(pub ByteString);

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusGlobalSlotStableV1 {
    pub slot_number: UnsignedExtendedUInt32StableV1,
    pub slots_per_epoch: UnsignedExtendedUInt32StableV1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseEpochLedgerValueStableV1 {
    pub hash: MinaBaseLedgerHash0StableV1,
    pub total_currency: CurrencyMakeStrAmountMakeStrStableV1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseEpochSeedStableV1(pub BigInt);

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 {
    pub ledger: MinaBaseEpochLedgerValueStableV1,
    pub seed: MinaBaseEpochSeedStableV1,
    pub start_checkpoint: DataHashLibStateHashStableV1,
    pub lock_checkpoint: DataHashLibStateHashStableV1,
    pub epoch_length: UnsignedExtendedUInt32StableV1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 {
    pub ledger: MinaBaseEpochLedgerValueStableV1,
    pub seed: MinaBaseEpochSeedStableV1,
    pub start_checkpoint: DataHashLibStateHashStableV1,
    pub lock_checkpoint: DataHashLibStateHashStableV1,
    pub epoch_length: UnsignedExtendedUInt32StableV1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonZeroCurvePointUncompressedStableV1 {
    pub x: BigInt,
    pub is_odd: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusProofOfStakeDataConsensusStateValueStableV1 {
    pub blockchain_length: UnsignedExtendedUInt32StableV1,
    pub epoch_count: UnsignedExtendedUInt32StableV1,
    pub min_window_density: UnsignedExtendedUInt32StableV1,
    pub sub_window_densities: Vec<UnsignedExtendedUInt32StableV1>,
    pub last_vrf_output: ConsensusVrfOutputTruncatedStableV1,
    pub total_currency: CurrencyMakeStrAmountMakeStrStableV1,
    pub curr_global_slot: ConsensusGlobalSlotStableV1,
    pub global_slot_since_genesis: UnsignedExtendedUInt32StableV1,
    pub staking_epoch_data: ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
    pub next_epoch_data: ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    pub has_ancestor_in_same_checkpoint_window: bool,
    pub block_stake_winner: NonZeroCurvePointUncompressedStableV1,
    pub block_creator: NonZeroCurvePointUncompressedStableV1,
    pub coinbase_receiver: NonZeroCurvePointUncompressedStableV1,
    pub supercharge_coinbase: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaBaseProtocolConstantsCheckedValueStableV1 {
    pub k: UnsignedExtendedUInt32StableV1,
    pub slots_per_epoch: UnsignedExtendedUInt32StableV1,
    pub slots_per_sub_window: UnsignedExtendedUInt32StableV1,
    pub delta: UnsignedExtendedUInt32StableV1,
    pub genesis_state_timestamp: BlockTimeMakeStrTimeStableV1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinaStateProtocolStateBodyValueStableV2 {
    pub genesis_state_hash: DataHashLibStateHashStableV1,
    pub blockchain_state: MinaStateBlockchainStateValueStableV2,
    pub consensus_state: ConsensusProofOfStakeDataConsensusStateValueStableV1,
    pub constants: MinaBaseProtocolConstantsCheckedValueStableV1,
}

#[rustfmt::skip]
type SixteenBigInt = (BigInt, (BigInt, (BigInt, (BigInt, (BigInt, (BigInt, (BigInt, (BigInt, (BigInt, (BigInt, (BigInt, (BigInt, (BigInt, (BigInt, (BigInt, (BigInt, ()))))))))))))))));

#[derive(Debug, Serialize, Deserialize)]
pub struct PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof {
    pub app_state: MinaStateProtocolStateValueStableV2,
    // pub app_state: mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2,
    pub dlog_plonk_index: MinaBaseVerificationKeyWireStableV1WrapIndex,
    pub challenge_polynomial_commitments: ((BigInt, BigInt), ((BigInt, BigInt), ())),
    // pub challenge_polynomial_commitments: Vec<(BigInt, BigInt)>,
    // pub challenge_polynomial_commitments: Vec<(BigInt, BigInt)>,
    // pub old_bulletproof_challenges: Vec<Vec<BigInt>>,
    pub old_bulletproof_challenges: (SixteenBigInt, (SixteenBigInt, ())),
}

impl From<PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof>
    for MessagesForNextStepProof
{
    fn from(value: PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof) -> Self {
        eprintln!("FROM START");

        Self {
            app_state: [Fp::one(); 1],
            // app_state: [value.app_state.0.into(); 1],
            dlog_plonk_index: {
                // TODO: Refactor with Account

                let idx = value.dlog_plonk_index;

                #[rustfmt::skip]
                let sigma = [
                    idx.sigma_comm.0.into(),
                    idx.sigma_comm.1.0.into(),
                    idx.sigma_comm.1.1.0.into(),
                    idx.sigma_comm.1.1.1.0.into(),
                    idx.sigma_comm.1.1.1.1.0.into(),
                    idx.sigma_comm.1.1.1.1.1.0.into(),
                    idx.sigma_comm.1.1.1.1.1.1.0.into(),
                ];

                #[rustfmt::skip]
                let coefficients = [
                    idx.coefficients_comm.0.into(),
                    idx.coefficients_comm.1.0.into(),
                    idx.coefficients_comm.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                    idx.coefficients_comm.1.1.1.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                ];

                PlonkVerificationKeyEvals {
                    sigma,
                    coefficients,
                    generic: idx.generic_comm.into(),
                    psm: idx.psm_comm.into(),
                    complete_add: idx.complete_add_comm.into(),
                    mul: idx.mul_comm.into(),
                    emul: idx.emul_comm.into(),
                    endomul_scalar: idx.endomul_scalar_comm.into(),
                }
            },
            challenge_polynomial_commitments: [
                value.challenge_polynomial_commitments.0.into(),
                value.challenge_polynomial_commitments.1 .0.into(),
            ],
            #[rustfmt::skip]
            old_bulletproof_challenges: [
                [
                    value.old_bulletproof_challenges.0.0.into(),
                    value.old_bulletproof_challenges.0.1.0.into(),
                    value.old_bulletproof_challenges.0.1.1.0.into(),
                    value.old_bulletproof_challenges.0.1.1.1.0.into(),
                    value.old_bulletproof_challenges.0.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.0.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.0.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.0.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.0.1.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.0.1.1.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.0.1.1.1.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.0.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.0.1.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.0.1.1.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.0.1.1.1.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.0.1.1.1.1.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                ],
                [
                    value.old_bulletproof_challenges.1.0.0.into(),
                    value.old_bulletproof_challenges.1.0.1.0.into(),
                    value.old_bulletproof_challenges.1.0.1.1.0.into(),
                    value.old_bulletproof_challenges.1.0.1.1.1.0.into(),
                    value.old_bulletproof_challenges.1.0.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.1.0.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.1.0.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.1.0.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.1.0.1.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.1.0.1.1.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.1.0.1.1.1.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.1.0.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.1.0.1.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.1.0.1.1.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.1.0.1.1.1.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                    value.old_bulletproof_challenges.1.0.1.1.1.1.1.1.1.1.1.1.1.1.1.1.1.0.into(),
                ]
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use mina_curves::pasta::Fq;
    use o1_utils::FieldHelpers;
    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use crate::FpExt;

    use super::*;

    #[test]
    fn test_messages_for_next_wrap_proof() {
        let s = "LnK1LUKilI70jBD0igi0XrL3FmM4B3V1qOFJuf51/z38YwYpxqGiN6PcHZX9VPv5zKBiSG6fV4UuvGTkBCzrPVNBexrQf2ERw75JFIrkbD2hUBHfMfYXExOD9Whg80gjIA3z72aHeYAz9z7THB3OneJNSVFLgV7bXGjXqHIAsmIrIFcf2r8zB7BanZ1yPoqto3z+AwYTNnJuYibjZZO5VZJtklQ7Ld3UhOtdGeyTHsrfgSjH/ZSpFcqgjTEK8Lwj2gZs8D6fqLZsTqcVM66/QLWKNw5vhu18rk6d1+WtWf74D/+2tsfcKCsKfBVIcB70+o53bnm5DG4NDtK1z7vrWuMHAGzGcOUmHvvZzEjFiM2i9LspJ9BZwK/QcMmO1pQuZkEGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAA/IhDl9aDAQAAIJjqNXyDMQ33oix7qxS+BZBxsZ+Gp+ZBlvi1YGPNmXuD/rQYAR0LBgMDBwYEBQMDBgIgfAK/QeAt6wZUtz1b97/UZ2g6i0rVFxRKRp1Hz4zslQD86E1CI6USKw7+5Sr+5Bv+5Sps8D6fqLZsTqcVM66/QLWKNw5vhu18rk6d1+WtWf74D/zoTRbFnvoPDm9MBnz1Z3FFQS1JnOlqEniomPx6r3YepGiFofFK6qsEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACfdBZs56T7AMQes4sg1y2N3FDCSOVTIrIpBVD5hEciCf75D20ktgl86dXrgUhOzlCpixXrNJhFIPHzUkwWTlAjML0y/OhNnXPhJSEO+XVFJ2ciCql/nMSbHpUuS3D6oHJnGDr+Ak2RITIexAFXDppQX3UxZeA6Mx3rHroFQthqarvZC33ENKRMPJo+Iy5ytS1CopSO9IwQ9IoItF6y9xZjOAd1dajhSbn+df89/rwIAVg/XQOpTH636Hb7nga9D2qrh6jVhJx8JAv6kZh6lIAiALliNXLg4Keq0ZJyxK/QNAx8SxrJeffYytk5lbcTF9s9AbliNXLg4Keq0ZJyxK/QNAx8SxrJeffYytk5lbcTF9s9AQH+IgH+5BsHAPzoC8dggwEAAMJ8dzWflMB4dWAZ9XGgWCOx6iDsMEtcHtYOkbl1ICcQ5Z5esAk6Wp03R5shLa2iarr4Xb8UWPFc8aDDVfJvpwDQiUNy6LAHPV9PoKCfBvR6NWh3T/JvOs5+Ng0BEcD/I9Tg4jrLVtb6nezJhWW/TdIseIiQtM/VnpU8cm/6/34JXq8NP8C+7XStUtzhS3VmtHry6nL4ODxn9FKLcbrlVz5nZIOKzBFq67SeOW4dgv9GWsuXP0mNCDHConLmABCOJ7tAONrsniMfCA3Qvl8rISDVKl4XcAuoEblbuecaoxENbJzXCKmROZBL6CXvMZpI8+A2x+K1pClIGOzZPxpXNDJdvcId7h4UFIhPZuE4odrx/HshllgA6d6KQLsWCTC6IbIaRgNr2TAYil3xcQMWiwlbLnYw3HQwXreBdCDdnbcVMCrLBPe3e+g70vh8X0Ax2OwF1PZeWM7p6hzuSmYIpj4D2v8NgOz2HnKhGHoDcDXt+uctvsrxIbdXFifYK9RZJ1p8he+tPY8mpPlbz90mvisLVqhczq+GORupLa0o8+ka30Cva8yzEJO1rdS70lx+IvPoeqUw1fQhtXOXPzcI5RIAKZFZSHgnINynjaf5sVolC62RyOPS2wg49vbnBmWcZSF7y+NreOzDn9i5r7EJHiRKlzg2qkVmtir2WcCBhb/hKMPEOlVLviMI/+oDgyC2N54bk7f4Zt+e6ogi5iT9P90jDdhK30cN5aFXspx78OijTIbxsxGB5zs+LFdmCYC8CxWshKWkTgHthjbsWpgLP3UZShCiAoUAwaaMglyZ6tIKBNDIPo+m6gxtW3baVzrPEpq85FM+ocVL7X3nj5AyIiEgJCWjlWwuqSK2kZGFhIZ/fh9MzviOzTIJ9v27ECAAhh7LKCuRGqGt7dNf3hFKQcEYDyxxkVNeAgEqy7MvX/TsL/KSi0tnZ8dDEpkvXXOvichZ3ZODx7mWohmT6M3CW5MQ9SelS/Wa0bV7ma00k4mjXZRiYsfb3YsesVZ+qjWSgBibPpOZS91D8erwrzDi4X7xSujNVOXRgk2POdZu6QdJMHD2uSwMMVJIWoX0ryeNICxlBrabXYq1rSMjuQdRtFwm+GLoVjPlH9VMj/HCtQ2+cSx/O01vucER9S8H2DSaAB8TzvYwsnxb3xRP2G/B2pTu/QOMAsZs3tPLWqhxJyDbPZyARY3bOEz5J2R2s5ZF0dnHSnDGABvsYvjg1oS15fgfIwgzIDv5y4WryIq9K6Pyutbzr2WI+PYaeCR7UudM1SaJ0nr2NNyYUK+RgFBLstOnLBMkfcJALY19RZx5ADE7MKTy3AyMaK7j6pp3D3dXRtsMAZOqzIeh+nlVnK1vnUQQid/uedjeMHc60SPrv/OVYXN/1c2iMCbsHC68YfCTHBrSjg4Na5EpFQXuFUd8ppkHNcGzZYjZS6cwQmqZdBOAO/2q3r8Aw7dYOECR1dnf+Y4UlgJcluQD469iolmlVQE+xEkzTv7SWe416ExQOqsFUSo4E6tiQLI1rp/+DqGjywmMiNsjzyVYwglJQV3+TTHT+rCMfe2yaRtA9y3E6WHvKzBLJs/3Oaonv50W3RSt5k/4qL6iK+b3FUtAcQ2VT7oqv5FClz4QloInHZ91utXZ/0HwhTsjO4v4BUTe2qz3Lj2WM+xFYtD7BcZnfahV0gWcaa4FGP5Ia0WSO8wxniCbPNGvr5F/EQ4FdJorxAAyQUIs3HNgTfpumM89m/GPppE2DNnWXtYlfoLwACzrP8VyoYvTUM2VNmQdWgtHxcLdFzw8XTQnmRkt0BYq60KxTJcjXSegCshLHToduJ9+sa2pOM2qIBSowAmttjl/KGmo+PBYtHg1LXvieprQZd34hj8CAIVhAUkW8GS8isjM7InWPXLReqS7SePtF3Tg+BjKfuoafnV8J7exyCf8Z4cymUXeyCcZsz0Wxvb9Jqgsybe8XRuifNjw3LIfB2hBcX57KeLlu73BIeOKX/GYDHI6wJ9CEFtuY6TZ+OYnX5M7AfyWZk7K6dsIarr4jL7YzEakvBkoKifsZ3+pdU2uQqJR8QLeO9sgGie6J7BYg69sIkttmyxJ125URAPXTaZqJJWZc1Pm+I1cCy40mdIjVmL3MDTlOtiEc+Ax6dAeU1MVHLnzHEvX27Y9ESajKnAcMD+06NQjYTDnHQk7ks4LzbsKZ3/rg17InWzxeweYXTrFUdHMMxbAYenLzYzLptFV3yuz3covqakkBh4W6hoiK9HN+iyoBE79vex92xzFTQjdsxfBzX/HE9pSQSzDRLxTq5T7a9Mmqaw/o01vifLxsIRtPpVRE4TygWa5KzbbzDLK9hg0fQkuqAUt+3LF1lpp3W6O65HTMCeAeCEdLUcds+4mXFEjJduBNvNvaCGiLZ8/sfjhbdQq2wTQbG9C78P2L40LiwwcX8TPa2NJMoZFH7iKXntoI7wh7r02Qwb+knWdgOteSSeT1xJq4m5Nj2lzw2fTrf5bB3sd5eohzaPJU9d7UVN6Lj6fjYGNiFhncVafnvZFtHz2JRVLF03u/h8Dlyfpz4UJAJVTEvz+XHca3Jc3KPxFb5XhpvIeKX8hTG8uKL5ZFp4PzfAi8La0aQfGJIjdDjhnV5mcwFp7ydDAGFDAJTepDC8rereMBmFOE9T5LVl7fceHvl7tSxdQkPBLnWNvoPdKLKIljzqiKNx6qTNYr1pusChbKGC80zCcCP5RB+2GH+Uo+h/4Mwa2kYonu3OI5hUnmzGrck8YjrcVAHyDfQpirzl6bVQmej6CH2sP0G4wy1CzECIYl8Q/hgwJqUzGxO57OUiYygUvBJQTCEcEW0RpD99RUwxQPf3U4q75LaPfI9UHcMW2opZvAUcvNEWJVRIA60jSUPCtaQgDOPB15or/qTkgq3Nurkg94e9dRQB7iGxlzmAgvzNzwRlF8Uc6o5YNHxernF/mRkX9EYMK/JhpN2j3ude+Tj4UMKENlDpzhgwrN26aB9nZTasOb9I8vMR4F35qlU9X70lNQjRlV7K27gKtJ9ErvCFUAM5iWg8YQZ1hkLIj0B9u4MqtSUYRUpSoJlfw0L+kBZK/ICqY3X/lfEICwDxZlaOQ2hoXMUgaAyMZVflSE40D2DtQr5tRwZVuNR/AewmUiFoXWNhbaMH1SQy9OFISm6iilko4xyjWBf0xQiC7JUiv8q2N9fiRUxi5JqQe0emG7K78dzDbnh4NxGRC4gPrj5ewyjg5H3klIYQYAGTwhe2Bq0ZETvZegRJm25xrNtHIfif7F7ryeQ8gZZIQBC5hlEBhAdeje2QEwDRwFsZOmu2jnNiklX15OtD+2wCmU2jJOZcGye2bA0J3qngAdaVJ2PZuZAV9ZH2B033XI5dWZ0hWPcAICEd8dlz+VPL6pKAJZHsM0DjKjiqZmXcriPL+G0VO/R2zAibBU6sb1nLECyu5PtxyOmBCOakYaQitiJggQS6Fthz0st94qapgEwWx34etwa0Bkkz+qW8OKewiRUXy9jLjTY1mHdoZ3h/0jqD95HAyZaME2jRZaWEGGqEk2f6UQ/+uHu27zftaDTkZnIyBCMhHPoulbX5jkDtg6LJi9K0UQ3lxK0qF9qd1ER9BZzyHItnulVe2M9tjGMO4OyZjNsiBAszMbHdpeN6hZuQZYjAAA8n2lbX1XasOvE5qQSzaGVKhz8HEQnlmS718q9q3UJwZ5G342nkc6AaMe4dEivzoLXvIHUXpAFf96Kel4GqzHtaZEojxNTvcMNra71Cd9iUXT8zqAw3eyMBOOTqeI/+EEOcT8LFFXQc7tehX/f4xDWh6loeZgmMQFs6VZGrhf9kww5t47FWxAiFk8iQ7ugEJuhQIByWhVLjkc0rkTywLO00iX2etybQ3HTwp1SfG+HGjWkveofDtU79AF7pvqjFtpV0kGXaAKz4VAAA=";

        let bytes = base64::decode(s).unwrap();

        println!("LEN={:?}", bytes.len());

        let result: MessagesForNextStepProof = serde_binprot::from_slice(&bytes).unwrap();
        println!("RESULT={:#?}", result);

        // for int in result.old_bulletproof_challenges.iter().flat_map(|s| s) {
        //     println!("old={}", int.to_hex());
        // }
    }
}
