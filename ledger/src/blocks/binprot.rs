use serde::{de::Visitor, Deserialize, Serialize};

use crate::{
    BigInt, BlockchainState, BlockchainStateRegisters, ConsensusGlobalSlot, ConsensusState,
    DataStaking, EpochLedger, Excess, LocalState, MessagesForNextStepProof,
    MinaBaseVerificationKeyWireStableV1WrapIndex, NonZeroCurvePointUncompressedStableV1,
    PlonkVerificationKeyEvals, ProtocolConstants, ProtocolState, ProtocolStateBody, Sgn,
    StagedLedgerHash, StagedLedgerHashNonSnark, TransactionFailure,
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

impl From<MinaBaseTransactionStatusFailureStableV2> for TransactionFailure {
    fn from(value: MinaBaseTransactionStatusFailureStableV2) -> Self {
        use MinaBaseTransactionStatusFailureStableV2::*;

        match value {
            Predicate => TransactionFailure::Predicate,
            SourceNotPresent => TransactionFailure::SourceNotPresent,
            ReceiverNotPresent => TransactionFailure::ReceiverNotPresent,
            AmountInsufficientToCreateAccount => {
                TransactionFailure::AmountInsufficientToCreateAccount
            }
            CannotPayCreationFeeInToken => TransactionFailure::CannotPayCreationFeeInToken,
            SourceInsufficientBalance => TransactionFailure::SourceInsufficientBalance,
            SourceMinimumBalanceViolation => TransactionFailure::SourceMinimumBalanceViolation,
            ReceiverAlreadyExists => TransactionFailure::ReceiverAlreadyExists,
            TokenOwnerNotCaller => TransactionFailure::TokenOwnerNotCaller,
            Overflow => TransactionFailure::Overflow,
            GlobalExcessOverflow => TransactionFailure::GlobalExcessOverflow,
            LocalExcessOverflow => TransactionFailure::LocalExcessOverflow,
            SignedCommandOnZkappAccount => TransactionFailure::SignedCommandOnZkappAccount,
            ZkappAccountNotPresent => TransactionFailure::ZkappAccountNotPresent,
            UpdateNotPermittedBalance => TransactionFailure::UpdateNotPermittedBalance,
            UpdateNotPermittedTimingExistingAccount => {
                TransactionFailure::UpdateNotPermittedTimingExistingAccount
            }
            UpdateNotPermittedDelegate => TransactionFailure::UpdateNotPermittedDelegate,
            UpdateNotPermittedAppState => TransactionFailure::UpdateNotPermittedAppState,
            UpdateNotPermittedVerificationKey => {
                TransactionFailure::UpdateNotPermittedVerificationKey
            }
            UpdateNotPermittedSequenceState => TransactionFailure::UpdateNotPermittedSequenceState,
            UpdateNotPermittedZkappUri => TransactionFailure::UpdateNotPermittedZkappUri,
            UpdateNotPermittedTokenSymbol => TransactionFailure::UpdateNotPermittedTokenSymbol,
            UpdateNotPermittedPermissions => TransactionFailure::UpdateNotPermittedPermissions,
            UpdateNotPermittedNonce => TransactionFailure::UpdateNotPermittedNonce,
            UpdateNotPermittedVotingFor => TransactionFailure::UpdateNotPermittedVotingFor,
            PartiesReplayCheckFailed => TransactionFailure::PartiesReplayCheckFailed,
            FeePayerNonceMustIncrease => TransactionFailure::FeePayerNonceMustIncrease,
            FeePayerMustBeSigned => TransactionFailure::FeePayerMustBeSigned,
            AccountBalancePreconditionUnsatisfied => {
                TransactionFailure::AccountBalancePreconditionUnsatisfied
            }
            AccountNoncePreconditionUnsatisfied => {
                TransactionFailure::AccountNoncePreconditionUnsatisfied
            }
            AccountReceiptChainHashPreconditionUnsatisfied => {
                TransactionFailure::AccountReceiptChainHashPreconditionUnsatisfied
            }
            AccountDelegatePreconditionUnsatisfied => {
                TransactionFailure::AccountDelegatePreconditionUnsatisfied
            }
            AccountSequenceStatePreconditionUnsatisfied => {
                TransactionFailure::AccountSequenceStatePreconditionUnsatisfied
            }
            AccountAppStatePreconditionUnsatisfied(x) => {
                TransactionFailure::AccountAppStatePreconditionUnsatisfied(x)
            }
            AccountProvedStatePreconditionUnsatisfied => {
                TransactionFailure::AccountProvedStatePreconditionUnsatisfied
            }
            AccountIsNewPreconditionUnsatisfied => {
                TransactionFailure::AccountIsNewPreconditionUnsatisfied
            }
            ProtocolStatePreconditionUnsatisfied => {
                TransactionFailure::ProtocolStatePreconditionUnsatisfied
            }
            IncorrectNonce => TransactionFailure::IncorrectNonce,
            InvalidFeeExcess => TransactionFailure::InvalidFeeExcess,
        }
    }
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

// #[derive(Debug, Serialize, Deserialize)]
// pub struct NonZeroCurvePointUncompressedStableV1 {
//     pub x: BigInt,
//     pub is_odd: bool,
// }

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

impl From<MinaStateProtocolStateValueStableV2> for ProtocolState {
    fn from(value: MinaStateProtocolStateValueStableV2) -> Self {
        Self {
            previous_state_hash: value.previous_state_hash.0.into(),
            body: ProtocolStateBody {
                genesis_state_hash: value.body.genesis_state_hash.0.into(),
                blockchain_state: BlockchainState {
                    staged_ledger_hash: StagedLedgerHash {
                        non_snark: StagedLedgerHashNonSnark {
                            ledger_hash: value
                                .body
                                .blockchain_state
                                .staged_ledger_hash
                                .non_snark
                                .ledger_hash
                                .0
                                .into(),
                            aux_hash: value
                                .body
                                .blockchain_state
                                .staged_ledger_hash
                                .non_snark
                                .aux_hash
                                .0
                                 .0
                                .try_into()
                                .unwrap(),
                            pending_coinbase_aux: value
                                .body
                                .blockchain_state
                                .staged_ledger_hash
                                .non_snark
                                .pending_coinbase_aux
                                .0
                                 .0
                                .try_into()
                                .unwrap(),
                        },
                        pending_coinbase_hash: value
                            .body
                            .blockchain_state
                            .staged_ledger_hash
                            .pending_coinbase_hash
                            .0
                             .0
                            .into(),
                    },
                    genesis_ledger_hash: value.body.blockchain_state.genesis_ledger_hash.0.into(),
                    registers: BlockchainStateRegisters {
                        ledger: value.body.blockchain_state.registers.ledger.0.into(),
                        pending_coinbase_stack: (),
                        local_state: LocalState {
                            stack_frame: value
                                .body
                                .blockchain_state
                                .registers
                                .local_state
                                .stack_frame
                                .0
                                .into(),
                            call_stack: value
                                .body
                                .blockchain_state
                                .registers
                                .local_state
                                .call_stack
                                .0
                                .into(),
                            transaction_commitment: value
                                .body
                                .blockchain_state
                                .registers
                                .local_state
                                .transaction_commitment
                                .into(),
                            full_transaction_commitment: value
                                .body
                                .blockchain_state
                                .registers
                                .local_state
                                .full_transaction_commitment
                                .into(),
                            token_id: value
                                .body
                                .blockchain_state
                                .registers
                                .local_state
                                .token_id
                                .0
                                .into(),
                            excess: Excess {
                                magnitude: value
                                    .body
                                    .blockchain_state
                                    .registers
                                    .local_state
                                    .excess
                                    .magnitude
                                    .0,
                                sgn: match value
                                    .body
                                    .blockchain_state
                                    .registers
                                    .local_state
                                    .excess
                                    .sgn
                                {
                                    SgnStableV1::Pos => Sgn::Pos,
                                    SgnStableV1::Neg => Sgn::Neg,
                                },
                            },
                            ledger: value
                                .body
                                .blockchain_state
                                .registers
                                .local_state
                                .ledger
                                .0
                                .into(),
                            success: value.body.blockchain_state.registers.local_state.success,
                            party_index: value
                                .body
                                .blockchain_state
                                .registers
                                .local_state
                                .party_index
                                .0,
                            failure_status_tbl: value
                                .body
                                .blockchain_state
                                .registers
                                .local_state
                                .failure_status_tbl
                                .0
                                .into_iter()
                                .map(|f| f.into_iter().map(|f| f.into()).collect())
                                .collect(),
                        },
                    },
                    timestamp: value.body.blockchain_state.timestamp.0 .0 as u64,
                    body_reference: value
                        .body
                        .blockchain_state
                        .body_reference
                        .0
                         .0
                         .0
                        .try_into()
                        .unwrap(),
                },
                consensus_state: ConsensusState {
                    blockchain_length: value.body.consensus_state.blockchain_length.0 as u32,
                    epoch_count: value.body.consensus_state.epoch_count.0 as u32,
                    min_window_density: value.body.consensus_state.min_window_density.0 as u32,
                    sub_window_densities: value
                        .body
                        .consensus_state
                        .sub_window_densities
                        .iter()
                        .map(|i| i.0 as u32)
                        .collect(),
                    last_vrf_output: value
                        .body
                        .consensus_state
                        .last_vrf_output
                        .0
                         .0
                        .try_into()
                        .unwrap(),
                    total_currency: value.body.consensus_state.total_currency.0,
                    curr_global_slot: ConsensusGlobalSlot {
                        slot_number: value.body.consensus_state.curr_global_slot.slot_number.0
                            as u32,
                        slots_per_epoch: value
                            .body
                            .consensus_state
                            .curr_global_slot
                            .slots_per_epoch
                            .0 as u32,
                    },
                    global_slot_since_genesis: value
                        .body
                        .consensus_state
                        .global_slot_since_genesis
                        .0 as u32,
                    staking_epoch_data: DataStaking {
                        ledger: EpochLedger {
                            hash: value
                                .body
                                .consensus_state
                                .staking_epoch_data
                                .ledger
                                .hash
                                .0
                                .into(),
                            total_currency: value
                                .body
                                .consensus_state
                                .staking_epoch_data
                                .ledger
                                .total_currency
                                .0,
                        },
                        seed: value.body.consensus_state.staking_epoch_data.seed.0.into(),
                        start_checkpoint: value
                            .body
                            .consensus_state
                            .staking_epoch_data
                            .start_checkpoint
                            .0
                            .into(),
                        lock_checkpoint: value
                            .body
                            .consensus_state
                            .staking_epoch_data
                            .lock_checkpoint
                            .0
                            .into(),
                        epoch_length: value.body.consensus_state.staking_epoch_data.epoch_length.0
                            as u32,
                    },
                    next_epoch_data: DataStaking {
                        ledger: EpochLedger {
                            hash: value
                                .body
                                .consensus_state
                                .next_epoch_data
                                .ledger
                                .hash
                                .0
                                .into(),
                            total_currency: value
                                .body
                                .consensus_state
                                .next_epoch_data
                                .ledger
                                .total_currency
                                .0,
                        },
                        seed: value.body.consensus_state.next_epoch_data.seed.0.into(),
                        start_checkpoint: value
                            .body
                            .consensus_state
                            .next_epoch_data
                            .start_checkpoint
                            .0
                            .into(),
                        lock_checkpoint: value
                            .body
                            .consensus_state
                            .next_epoch_data
                            .lock_checkpoint
                            .0
                            .into(),
                        epoch_length: value.body.consensus_state.next_epoch_data.epoch_length.0
                            as u32,
                    },
                    has_ancestor_in_same_checkpoint_window: value
                        .body
                        .consensus_state
                        .has_ancestor_in_same_checkpoint_window,
                    block_stake_winner: value.body.consensus_state.block_stake_winner.into(),
                    block_creator: value.body.consensus_state.block_creator.into(),
                    coinbase_receiver: value.body.consensus_state.coinbase_receiver.into(),
                    supercharge_coinbase: value.body.consensus_state.supercharge_coinbase,
                },
                constants: ProtocolConstants {
                    k: value.body.constants.k.0 as u32,
                    slots_per_epoch: value.body.constants.slots_per_epoch.0 as u32,
                    slots_per_sub_window: value.body.constants.slots_per_sub_window.0 as u32,
                    delta: value.body.constants.delta.0 as u32,
                    genesis_state_timestamp: value.body.constants.genesis_state_timestamp.0 .0
                        as u64,
                },
            },
        }
    }
}

impl From<PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof>
    for MessagesForNextStepProof
{
    fn from(value: PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof) -> Self {
        Self {
            protocol_state: ProtocolState::from(value.app_state),
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
    use o1_utils::FieldHelpers;
    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use crate::FpExt;

    use super::*;

    #[test]
    fn test_messages_for_next_step_proof() {
        let s = "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";

        let bytes = base64::decode(s).unwrap();
        let msg: MessagesForNextStepProof = serde_binprot::from_slice(&bytes).unwrap();

        msg.hash();

        let values = [
            "955312fcfe5c771adc973728fc456f95e1a6f21e297f214c6f2e28be59169e0f",
            "cdf022f0b6b46907c62488dd0e386757999cc05a7bc9d0c01850c02537a90c2f",
            "2b7ab78c06614e13d4f92d597b7dc787be5eed4b175090f04b9d636fa0f74a2c",
            "a2258f3aa228dc7aa93358af5a6eb0285b2860bcd3309c08fe5107ed861fe528",
            "fa1ff83306b6918a27bb7388e615279b31ab724f188eb715007c837d0a62af39",
            "7a6d54267a3e821f6b0fd06e30cb50b310221897c43f860c09a94cc6c4ee7b39",
            "4898ca052f0494130847045b44690fdf51530c503dfdd4e2aef92da3df23d507",
            "70c5b6a2966f01472f344589551200eb48d250f0ad69080338f075e68affa939",
            "20ab736eae483de1ef5d45007b886c65ce6020bf3373c11945f1473aa3960d1f",
            "17ab9c5fe64645fd11830afc98693768f7b9d7be4e3e1430a10d943a73860c2b",
            "376e9a07d9d94dab0e6fd23cbcc478177e6a954f57ef494d42346557b2b6ee02",
            "ad27d12bbc215400ce625a0f18419d6190b223d01f6ee0caad4946115294a826",
            "57f0d0bfa40592bf202a98dd7fe57c4202c03c5995a390da1a1731481a032319",
            "55f952138d03d83b50af9b51c1956e351fc07b0994885a1758d85b68c1f5490c",
            "bd3852129ba8a2964a38c728d605fd314220bb2548aff2ad8df5f8915318b926",
            "a41ed1e986ecaefc7730db9e1e0dc46442e203eb8f97b0ca38391f7925218418",
            "64f085ed81ab46444ef65e811266db9c6b36d1c87e27fb17baf2790f20659210",
            "042e6194406101d7a37b6404c0347016c64e9aeda39cd8a4957d793ad0fedb00",
            "a65368c9399706c9ed9b034277aa780075a549d8f66e64057d647d81d37dd723",
            "97566748563dc00808477c765cfe54f2faa4a009647b0cd038ca8e2a9999772b",
            "88f2fe1b454efd1db30226c153ab1bd672c40b2bb93edc723a604239a9186908",
            "ad889820412e85b61cf4b2df78a9aa601305b1df87adc1ad01924cfea96f0e29",
            "ec224545f2f632e34d8d661dda19de1ff48ea0fde4703265a304da3459696106",
            "1aa124d9fe9443ffae1eedbbcdfb5a0d39199c8c8108c8473e8ba56d7e63903b",
            "60e8b262f4ad144379712b4a85f6a775111f41673c8722d9ee9557b633db6318",
            "c3b83b266336c88102cccc6c776978dea166e41962300003c9f695b5f55dab0e",
            "bc4e6a412cda1952a1cfc1c44279664bbd7cabdab7509c19e46df8da791ce806",
            "8c7b87448afce82d7bc81d45e90057fde8a7a5e06ab31ed6991288f1353bdc30",
            "dadaef509df625174fccea030ddec8c04e393a9e23ff8410e713f0b1455d073b",
            "b5e857fdfe310d687a96879982631016ce95646ae17fd930c39b78ec55b10221",
            "64f2243bba0109ba14080725a154b8e4734ae44f2c0b3b4d225f67adc9b4371d",
            "3c29d527c6f871a35a4bdea1f0ed53bf4017ba6faa316da55d241976802b3e15",
        ];

        let our_values = msg
            .old_bulletproof_challenges
            .iter()
            .flatten()
            .map(|f| f.to_hex())
            .collect::<Vec<_>>();

        assert_eq!(&values[..], our_values);

        let protocol_body_hash = msg.protocol_state.body.hash();
        assert_eq!(
            protocol_body_hash.to_decimal(),
            "27716033973929967331361592821244189794615869085488714143712240393728003537186"
        );

        let protocol_hash = msg.protocol_state.hash();
        assert_eq!(
            protocol_hash.to_decimal(),
            "18374648090405860613992788233169821778993111003846333489723278775260971534309"
        );

        let result = msg.hash();
        const OCAML_RESULT: [u64; 4] = [
            7912308706379928291,
            8689988569980666660,
            5997160798854948936,
            3770142804027174900,
        ];

        // Same result as OCaml
        assert_eq!(result, OCAML_RESULT);
    }
}
