#![allow(unused_variables, unreachable_code)]

use std::sync::Arc;

use ark_ff::fields::arithmetic::InvalidBigInt;
use mina_hasher::Fp;
use mina_p2p_messages::{
    binprot,
    list::List,
    pseq::PaddedSeq,
    string::CharString,
    v2::{
        self, BlockTimeTimeStableV1,
        ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
        ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
        CurrencyAmountStableV1, CurrencyBalanceStableV1, CurrencyFeeStableV1,
        DataHashLibStateHashStableV1, EpochSeed, LedgerProofProdStableV2,
        MinaBaseAccountIdDigestStableV1, MinaBaseAccountIdStableV2,
        MinaBaseAccountUpdateBodyEventsStableV1, MinaBaseAccountUpdateBodyFeePayerStableV1,
        MinaBaseAccountUpdateBodyStableV1, MinaBaseAccountUpdateFeePayerStableV1,
        MinaBaseAccountUpdateMayUseTokenStableV1, MinaBaseAccountUpdatePreconditionsStableV1,
        MinaBaseAccountUpdateTStableV1, MinaBaseAccountUpdateUpdateStableV1,
        MinaBaseAccountUpdateUpdateStableV1AppStateA,
        MinaBaseAccountUpdateUpdateTimingInfoStableV1, MinaBaseCallStackDigestStableV1,
        MinaBaseCoinbaseFeeTransferStableV1, MinaBaseCoinbaseStableV1, MinaBaseEpochSeedStableV1,
        MinaBaseFeeExcessStableV1, MinaBaseFeeTransferSingleStableV2, MinaBaseFeeTransferStableV2,
        MinaBaseLedgerHash0StableV1, MinaBasePaymentPayloadStableV2,
        MinaBasePendingCoinbaseCoinbaseStackStableV1, MinaBasePendingCoinbaseHashBuilderStableV1,
        MinaBasePendingCoinbaseHashVersionedStableV1,
        MinaBasePendingCoinbaseMerkleTreeVersionedStableV2,
        MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree, MinaBasePendingCoinbaseStableV2,
        MinaBasePendingCoinbaseStackHashStableV1, MinaBasePendingCoinbaseStackIdStableV1,
        MinaBasePendingCoinbaseStackVersionedStableV1, MinaBasePendingCoinbaseStateStackStableV1,
        MinaBasePendingCoinbaseUpdateActionStableV1, MinaBasePendingCoinbaseUpdateStableV1,
        MinaBaseReceiptChainHashStableV1, MinaBaseSignatureStableV1,
        MinaBaseSignedCommandMemoStableV1, MinaBaseSignedCommandPayloadBodyStableV2,
        MinaBaseSignedCommandPayloadCommonStableV2, MinaBaseSignedCommandPayloadStableV2,
        MinaBaseSignedCommandStableV2, MinaBaseSokMessageStableV1, MinaBaseStackFrameStableV1,
        MinaBaseStagedLedgerHashNonSnarkStableV1, MinaBaseStagedLedgerHashStableV1,
        MinaBaseStakeDelegationStableV2, MinaBaseStateBodyHashStableV1,
        MinaBaseTransactionStatusFailureCollectionStableV1,
        MinaBaseTransactionStatusFailureStableV2, MinaBaseTransactionStatusStableV2,
        MinaBaseUserCommandStableV2, MinaBaseZkappCommandTStableV1WireStableV1,
        MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA,
        MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA,
        MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA,
        MinaBaseZkappPreconditionAccountStableV2, MinaBaseZkappPreconditionAccountStableV2BalanceA,
        MinaBaseZkappPreconditionProtocolStateEpochDataStableV1,
        MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochLedger,
        MinaBaseZkappPreconditionProtocolStateStableV1,
        MinaBaseZkappPreconditionProtocolStateStableV1AmountA,
        MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlotA,
        MinaBaseZkappPreconditionProtocolStateStableV1Length,
        MinaBaseZkappPreconditionProtocolStateStableV1LengthA,
        MinaNumbersGlobalSlotSinceGenesisMStableV1, MinaNumbersGlobalSlotSinceHardForkMStableV1,
        MinaNumbersGlobalSlotSpanStableV1,
        MinaStateBlockchainStateValueStableV2LedgerProofStatement,
        MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
        MinaStateBlockchainStateValueStableV2SignedAmount, MinaStateSnarkedLedgerStateStableV2,
        MinaStateSnarkedLedgerStateWithSokStableV2,
        MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2,
        MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2Coinbase,
        MinaTransactionLogicTransactionAppliedCommandAppliedStableV2,
        MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2,
        MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2FeeTransfer,
        MinaTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV2,
        MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2,
        MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2UserCommand,
        MinaTransactionLogicTransactionAppliedSignedCommandAppliedStableV2,
        MinaTransactionLogicTransactionAppliedStableV2,
        MinaTransactionLogicTransactionAppliedVaryingStableV2,
        MinaTransactionLogicTransactionAppliedZkappCommandAppliedStableV1,
        MinaTransactionLogicTransactionAppliedZkappCommandAppliedStableV1Command,
        MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1,
        MinaTransactionTransactionStableV2, ParallelScanJobStatusStableV1,
        ParallelScanSequenceNumberStableV1, ParallelScanWeightStableV1, SgnStableV1, SignedAmount,
        StagedLedgerDiffDiffDiffStableV2, StagedLedgerDiffDiffFtStableV1,
        StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2,
        StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Coinbase,
        StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2,
        StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B,
        StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2Coinbase,
        StagedLedgerDiffDiffStableV2, StateHash, TokenFeeExcess,
        TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
        TransactionSnarkScanStateStableV2,
        TransactionSnarkScanStateStableV2PreviousIncompleteZkappUpdates1::Border_block_continued_in_the_next_tree,
        TransactionSnarkScanStateStableV2ScanState,
        TransactionSnarkScanStateStableV2ScanStateTreesABaseT1,
        TransactionSnarkScanStateStableV2ScanStateTreesABaseT1Full,
        TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1,
        TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1Full,
        TransactionSnarkScanStateStableV2TreesAMerge,
        TransactionSnarkScanStateTransactionWithWitnessStableV2, TransactionSnarkStableV2,
        TransactionSnarkWorkTStableV2, TransactionSnarkWorkTStableV2Proofs,
        UnsignedExtendedUInt32StableV1, UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
    },
};
use mina_signer::Signature;

use crate::{
    array_into_with,
    proofs::field::FieldWitness,
    scan_state::{
        currency::BlockTime,
        pending_coinbase::{Stack, StackHasher},
        scan_state::BorderBlockContinuedInTheNextTree,
        transaction_logic::{
            signed_command::{PaymentPayload, StakeDelegationPayload},
            zkapp_command::{self, AuthorizationKind, CallForest, MayUseToken},
            WithStatus,
        },
    },
    staged_ledger::hash::{AuxHash, NonStark, PendingCoinbaseAux, StagedLedgerHash},
    Account, AccountId, Address, HashesMatrix, TokenId, VerificationKey, VotingFor,
};

use super::{
    currency::{Amount, Balance, Fee, Index, Length, Nonce, Sgn, Signed, Slot, SlotSpan},
    fee_excess::FeeExcess,
    parallel_scan::{self, JobStatus, ParallelScan, SequenceNumber},
    pending_coinbase::{self, PendingCoinbase},
    scan_state::{
        transaction_snark::{
            LedgerProof, LedgerProofWithSokMessage, Registers, SokDigest, SokMessage, Statement,
            TransactionSnark, TransactionWithWitness,
        },
        AvailableJob, AvailableJobMessage, ScanState,
    },
    transaction_logic::{
        self,
        local_state::LocalState,
        protocol_state,
        signed_command::SignedCommand,
        transaction_applied::{self, TransactionApplied},
        zkapp_command::{
            verifiable, AccountUpdate, FeePayer, FeePayerBody, SetOrKeep, WithHash, WithStackHash,
        },
        zkapp_statement::ZkappStatement,
        CoinbaseFeeTransfer, FeeTransfer, Memo, SingleFeeTransfer, Transaction, TransactionFailure,
        TransactionStatus, UserCommand,
    },
};

impl From<CurrencyAmountStableV1> for Amount {
    fn from(value: CurrencyAmountStableV1) -> Self {
        Self(value.as_u64())
    }
}

impl From<CurrencyAmountStableV1> for Balance {
    fn from(value: CurrencyAmountStableV1) -> Self {
        Self(value.as_u64())
    }
}

impl From<Amount> for CurrencyAmountStableV1 {
    fn from(value: Amount) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&Balance> for CurrencyBalanceStableV1 {
    fn from(value: &Balance) -> Self {
        Self((*value).into())
    }
}

impl From<Balance> for CurrencyAmountStableV1 {
    fn from(value: Balance) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&SignedAmount> for Signed<Amount> {
    fn from(value: &SignedAmount) -> Self {
        Self {
            magnitude: Amount(value.magnitude.clone().as_u64()),
            sgn: value.sgn.clone().into(),
        }
    }
}

impl From<&Amount> for CurrencyAmountStableV1 {
    fn from(value: &Amount) -> Self {
        CurrencyAmountStableV1(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&Amount> for CurrencyFeeStableV1 {
    fn from(value: &Amount) -> Self {
        CurrencyFeeStableV1(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&Signed<Amount>> for SignedAmount {
    fn from(value: &Signed<Amount>) -> Self {
        Self {
            magnitude: (&value.magnitude).into(),
            sgn: (&value.sgn).into(),
        }
    }
}

impl From<&CurrencyFeeStableV1> for Fee {
    fn from(value: &CurrencyFeeStableV1) -> Self {
        Self(value.as_u64())
    }
}

impl From<&CurrencyAmountStableV1> for Fee {
    fn from(value: &CurrencyAmountStableV1) -> Self {
        Self(value.as_u64())
    }
}

impl From<&Nonce> for mina_p2p_messages::v2::UnsignedExtendedUInt32StableV1 {
    fn from(value: &Nonce) -> Self {
        Self(value.as_u32().into())
    }
}

impl From<&mina_p2p_messages::v2::UnsignedExtendedUInt32StableV1> for Nonce {
    fn from(value: &mina_p2p_messages::v2::UnsignedExtendedUInt32StableV1) -> Self {
        Self::from_u32(value.as_u32())
    }
}

impl From<&mina_p2p_messages::v2::UnsignedExtendedUInt32StableV1> for Slot {
    fn from(value: &mina_p2p_messages::v2::UnsignedExtendedUInt32StableV1) -> Self {
        Self::from_u32(value.as_u32())
    }
}

impl From<&Slot> for mina_p2p_messages::v2::UnsignedExtendedUInt32StableV1 {
    fn from(value: &Slot) -> Self {
        Self(value.as_u32().into())
    }
}

impl From<&mina_p2p_messages::v2::UnsignedExtendedUInt32StableV1> for Length {
    fn from(value: &mina_p2p_messages::v2::UnsignedExtendedUInt32StableV1) -> Self {
        Self::from_u32(value.0.as_u32())
    }
}

impl From<&Length> for mina_p2p_messages::v2::UnsignedExtendedUInt32StableV1 {
    fn from(value: &Length) -> Self {
        Self(value.as_u32().into())
    }
}

impl From<SgnStableV1> for Sgn {
    fn from(value: SgnStableV1) -> Self {
        match value {
            SgnStableV1::Pos => Self::Pos,
            SgnStableV1::Neg => Self::Neg,
        }
    }
}

impl From<&SignedAmount> for Signed<Fee> {
    fn from(value: &SignedAmount) -> Self {
        Self {
            magnitude: (&value.magnitude).into(),
            sgn: value.sgn.clone().into(),
        }
    }
}

impl From<&Sgn> for SgnStableV1 {
    fn from(value: &Sgn) -> Self {
        match value {
            Sgn::Pos => Self::Pos,
            Sgn::Neg => Self::Neg,
        }
    }
}

impl From<&Fee> for CurrencyFeeStableV1 {
    fn from(value: &Fee) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&Fee> for CurrencyAmountStableV1 {
    fn from(value: &Fee) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&Signed<Fee>> for SignedAmount {
    fn from(value: &Signed<Fee>) -> Self {
        Self {
            magnitude: (&value.magnitude).into(),
            sgn: (&value.sgn).into(),
        }
    }
}

impl TryFrom<&MinaBaseFeeExcessStableV1> for FeeExcess {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaBaseFeeExcessStableV1) -> Result<Self, Self::Error> {
        Ok(Self {
            fee_token_l: (&*value.0.token).try_into()?,
            fee_excess_l: (&value.0.amount).into(),
            fee_token_r: (&*value.1.token).try_into()?,
            fee_excess_r: (&value.1.amount).into(),
        })
    }
}

impl From<&FeeExcess> for MinaBaseFeeExcessStableV1 {
    fn from(value: &FeeExcess) -> Self {
        Self(
            TokenFeeExcess {
                token: (&value.fee_token_l).into(),
                amount: (&value.fee_excess_l).into(),
            },
            TokenFeeExcess {
                token: (&value.fee_token_r).into(),
                amount: (&value.fee_excess_r).into(),
            },
        )
    }
}

impl TryFrom<&MinaBasePendingCoinbaseStackVersionedStableV1> for pending_coinbase::Stack {
    type Error = InvalidBigInt;

    fn try_from(
        value: &MinaBasePendingCoinbaseStackVersionedStableV1,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            data: pending_coinbase::CoinbaseStack(value.data.0.to_field()?),
            state: pending_coinbase::StateStack {
                init: value.state.init.0.to_field()?,
                curr: value.state.curr.0.to_field()?,
            },
        })
    }
}

impl From<&pending_coinbase::Stack> for MinaBasePendingCoinbaseStackVersionedStableV1 {
    fn from(value: &pending_coinbase::Stack) -> Self {
        Self {
            data: MinaBasePendingCoinbaseCoinbaseStackStableV1::from(&value.data).into(),
            state: (&value.state).into(),
        }
    }
}

impl From<&pending_coinbase::CoinbaseStack> for MinaBasePendingCoinbaseCoinbaseStackStableV1 {
    fn from(value: &pending_coinbase::CoinbaseStack) -> Self {
        Self(value.0.into())
    }
}

impl From<&pending_coinbase::StateStack> for MinaBasePendingCoinbaseStateStackStableV1 {
    fn from(value: &pending_coinbase::StateStack) -> Self {
        Self {
            init: MinaBasePendingCoinbaseStackHashStableV1(value.init.into()).into(),
            curr: MinaBasePendingCoinbaseStackHashStableV1(value.curr.into()).into(),
        }
    }
}

impl From<&MinaBaseTransactionStatusFailureStableV2> for TransactionFailure {
    fn from(value: &MinaBaseTransactionStatusFailureStableV2) -> Self {
        use MinaBaseTransactionStatusFailureStableV2 as P2P;

        match value {
            P2P::Predicate => Self::Predicate,
            P2P::SourceNotPresent => Self::SourceNotPresent,
            P2P::ReceiverNotPresent => Self::ReceiverNotPresent,
            P2P::AmountInsufficientToCreateAccount => Self::AmountInsufficientToCreateAccount,
            P2P::CannotPayCreationFeeInToken => Self::CannotPayCreationFeeInToken,
            P2P::SourceInsufficientBalance => Self::SourceInsufficientBalance,
            P2P::SourceMinimumBalanceViolation => Self::SourceMinimumBalanceViolation,
            P2P::ReceiverAlreadyExists => Self::ReceiverAlreadyExists,
            P2P::TokenOwnerNotCaller => Self::TokenOwnerNotCaller,
            P2P::Overflow => Self::Overflow,
            P2P::GlobalExcessOverflow => Self::GlobalExcessOverflow,
            P2P::LocalExcessOverflow => Self::LocalExcessOverflow,
            P2P::LocalSupplyIncreaseOverflow => Self::LocalSupplyIncreaseOverflow,
            P2P::GlobalSupplyIncreaseOverflow => Self::GlobalSupplyIncreaseOverflow,
            P2P::SignedCommandOnZkappAccount => Self::SignedCommandOnZkappAccount,
            P2P::ZkappAccountNotPresent => Self::ZkappAccountNotPresent,
            P2P::UpdateNotPermittedBalance => Self::UpdateNotPermittedBalance,
            // P2P::UpdateNotPermittedTimingExistingAccount => {
            //     Self::UpdateNotPermittedTimingExistingAccount
            // }
            P2P::UpdateNotPermittedDelegate => Self::UpdateNotPermittedDelegate,
            P2P::UpdateNotPermittedAppState => Self::UpdateNotPermittedAppState,
            P2P::UpdateNotPermittedVerificationKey => Self::UpdateNotPermittedVerificationKey,
            P2P::UpdateNotPermittedActionState => Self::UpdateNotPermittedActionState,
            P2P::UpdateNotPermittedZkappUri => Self::UpdateNotPermittedZkappUri,
            P2P::UpdateNotPermittedTokenSymbol => Self::UpdateNotPermittedTokenSymbol,
            P2P::UpdateNotPermittedPermissions => Self::UpdateNotPermittedPermissions,
            P2P::UpdateNotPermittedNonce => Self::UpdateNotPermittedNonce,
            P2P::UpdateNotPermittedVotingFor => Self::UpdateNotPermittedVotingFor,
            P2P::ZkappCommandReplayCheckFailed => Self::ZkappCommandReplayCheckFailed,
            P2P::FeePayerNonceMustIncrease => Self::FeePayerNonceMustIncrease,
            P2P::FeePayerMustBeSigned => Self::FeePayerMustBeSigned,
            P2P::AccountBalancePreconditionUnsatisfied => {
                Self::AccountBalancePreconditionUnsatisfied
            }
            P2P::AccountNoncePreconditionUnsatisfied => Self::AccountNoncePreconditionUnsatisfied,
            P2P::AccountReceiptChainHashPreconditionUnsatisfied => {
                Self::AccountReceiptChainHashPreconditionUnsatisfied
            }
            P2P::AccountDelegatePreconditionUnsatisfied => {
                Self::AccountDelegatePreconditionUnsatisfied
            }
            P2P::AccountActionStatePreconditionUnsatisfied => {
                Self::AccountActionStatePreconditionUnsatisfied
            }
            P2P::AccountAppStatePreconditionUnsatisfied(v) => {
                Self::AccountAppStatePreconditionUnsatisfied(**v)
            }
            P2P::AccountProvedStatePreconditionUnsatisfied => {
                Self::AccountProvedStatePreconditionUnsatisfied
            }
            P2P::AccountIsNewPreconditionUnsatisfied => Self::AccountIsNewPreconditionUnsatisfied,
            P2P::ProtocolStatePreconditionUnsatisfied => Self::ProtocolStatePreconditionUnsatisfied,
            P2P::IncorrectNonce => Self::IncorrectNonce,
            P2P::InvalidFeeExcess => Self::InvalidFeeExcess,
            P2P::Cancelled => Self::Cancelled,
            P2P::UpdateNotPermittedAccess => Self::UpdateNotPermittedAccess,
            P2P::UpdateNotPermittedTiming => Self::UpdateNotPermittedTiming,
            P2P::UnexpectedVerificationKeyHash => Self::UnexpectedVerificationKeyHash,
            P2P::ValidWhilePreconditionUnsatisfied => Self::ValidWhilePreconditionUnsatisfied,
        }
    }
}

impl From<&TransactionFailure> for MinaBaseTransactionStatusFailureStableV2 {
    fn from(value: &TransactionFailure) -> Self {
        use TransactionFailure as P2P;

        match value {
            P2P::Predicate => Self::Predicate,
            P2P::SourceNotPresent => Self::SourceNotPresent,
            P2P::ReceiverNotPresent => Self::ReceiverNotPresent,
            P2P::AmountInsufficientToCreateAccount => Self::AmountInsufficientToCreateAccount,
            P2P::CannotPayCreationFeeInToken => Self::CannotPayCreationFeeInToken,
            P2P::SourceInsufficientBalance => Self::SourceInsufficientBalance,
            P2P::SourceMinimumBalanceViolation => Self::SourceMinimumBalanceViolation,
            P2P::ReceiverAlreadyExists => Self::ReceiverAlreadyExists,
            P2P::TokenOwnerNotCaller => Self::TokenOwnerNotCaller,
            P2P::Overflow => Self::Overflow,
            P2P::GlobalExcessOverflow => Self::GlobalExcessOverflow,
            P2P::LocalExcessOverflow => Self::LocalExcessOverflow,
            P2P::LocalSupplyIncreaseOverflow => Self::LocalSupplyIncreaseOverflow,
            P2P::GlobalSupplyIncreaseOverflow => Self::GlobalSupplyIncreaseOverflow,
            P2P::SignedCommandOnZkappAccount => Self::SignedCommandOnZkappAccount,
            P2P::ZkappAccountNotPresent => Self::ZkappAccountNotPresent,
            P2P::UpdateNotPermittedBalance => Self::UpdateNotPermittedBalance,
            // P2P::UpdateNotPermittedTimingExistingAccount => {
            //     Self::UpdateNotPermittedTimingExistingAccount
            // }
            P2P::UpdateNotPermittedDelegate => Self::UpdateNotPermittedDelegate,
            P2P::UpdateNotPermittedAppState => Self::UpdateNotPermittedAppState,
            P2P::UpdateNotPermittedVerificationKey => Self::UpdateNotPermittedVerificationKey,
            P2P::UpdateNotPermittedActionState => Self::UpdateNotPermittedActionState,
            P2P::UpdateNotPermittedZkappUri => Self::UpdateNotPermittedZkappUri,
            P2P::UpdateNotPermittedTokenSymbol => Self::UpdateNotPermittedTokenSymbol,
            P2P::UpdateNotPermittedPermissions => Self::UpdateNotPermittedPermissions,
            P2P::UpdateNotPermittedNonce => Self::UpdateNotPermittedNonce,
            P2P::UpdateNotPermittedVotingFor => Self::UpdateNotPermittedVotingFor,
            P2P::ZkappCommandReplayCheckFailed => Self::ZkappCommandReplayCheckFailed,
            P2P::FeePayerNonceMustIncrease => Self::FeePayerNonceMustIncrease,
            P2P::FeePayerMustBeSigned => Self::FeePayerMustBeSigned,
            P2P::AccountBalancePreconditionUnsatisfied => {
                Self::AccountBalancePreconditionUnsatisfied
            }
            P2P::AccountNoncePreconditionUnsatisfied => Self::AccountNoncePreconditionUnsatisfied,
            P2P::AccountReceiptChainHashPreconditionUnsatisfied => {
                Self::AccountReceiptChainHashPreconditionUnsatisfied
            }
            P2P::AccountDelegatePreconditionUnsatisfied => {
                Self::AccountDelegatePreconditionUnsatisfied
            }
            P2P::AccountActionStatePreconditionUnsatisfied => {
                Self::AccountActionStatePreconditionUnsatisfied
            }
            P2P::AccountAppStatePreconditionUnsatisfied(v) => {
                Self::AccountAppStatePreconditionUnsatisfied((*v).into())
            }
            P2P::AccountProvedStatePreconditionUnsatisfied => {
                Self::AccountProvedStatePreconditionUnsatisfied
            }
            P2P::AccountIsNewPreconditionUnsatisfied => Self::AccountIsNewPreconditionUnsatisfied,
            P2P::ProtocolStatePreconditionUnsatisfied => Self::ProtocolStatePreconditionUnsatisfied,
            P2P::IncorrectNonce => Self::IncorrectNonce,
            P2P::InvalidFeeExcess => Self::InvalidFeeExcess,
            P2P::Cancelled => Self::Cancelled,
            P2P::UpdateNotPermittedAccess => Self::UpdateNotPermittedAccess,
            P2P::UpdateNotPermittedTiming => Self::UpdateNotPermittedTiming,
            P2P::UnexpectedVerificationKeyHash => Self::UnexpectedVerificationKeyHash,
            P2P::ValidWhilePreconditionUnsatisfied => Self::ValidWhilePreconditionUnsatisfied,
        }
    }
}

impl TryFrom<&MinaStateBlockchainStateValueStableV2LedgerProofStatementSource> for Registers {
    type Error = InvalidBigInt;

    fn try_from(
        value: &MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            first_pass_ledger: value.first_pass_ledger.to_field()?,
            second_pass_ledger: value.second_pass_ledger.to_field()?,
            pending_coinbase_stack: (&value.pending_coinbase_stack).try_into()?,
            local_state: LocalState {
                stack_frame: value.local_state.stack_frame.0.to_field()?,
                call_stack: value.local_state.call_stack.0.to_field()?,
                transaction_commitment: value.local_state.transaction_commitment.to_field()?,
                full_transaction_commitment: value
                    .local_state
                    .full_transaction_commitment
                    .to_field()?,
                excess: (&value.local_state.excess).into(),
                supply_increase: (&value.local_state.supply_increase).into(),
                ledger: value.local_state.ledger.0.to_field()?,
                success: value.local_state.success,
                account_update_index: Index(value.local_state.account_update_index.0.as_u32()),
                failure_status_tbl: value
                    .local_state
                    .failure_status_tbl
                    .0
                    .iter()
                    .map(|s| s.iter().map(|s| s.into()).collect())
                    .collect(),
                will_succeed: value.local_state.will_succeed,
            },
        })
    }
}

impl From<&MinaStateBlockchainStateValueStableV2SignedAmount> for Signed<Amount> {
    fn from(value: &MinaStateBlockchainStateValueStableV2SignedAmount) -> Self {
        let MinaStateBlockchainStateValueStableV2SignedAmount { magnitude, sgn } = value;

        Self {
            magnitude: (magnitude.clone()).into(),
            sgn: (sgn.clone()).into(),
        }
    }
}

impl From<&Signed<Amount>> for MinaStateBlockchainStateValueStableV2SignedAmount {
    fn from(value: &Signed<Amount>) -> Self {
        let Signed::<Amount> { magnitude, sgn } = value;

        Self {
            magnitude: (*magnitude).into(),
            sgn: sgn.into(),
        }
    }
}

impl TryFrom<&MinaStateBlockchainStateValueStableV2LedgerProofStatement> for Statement<()> {
    type Error = InvalidBigInt;

    fn try_from(
        value: &MinaStateBlockchainStateValueStableV2LedgerProofStatement,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            source: (&value.source).try_into()?,
            target: (&value.target).try_into()?,
            connecting_ledger_left: value.connecting_ledger_left.to_field()?,
            connecting_ledger_right: value.connecting_ledger_right.to_field()?,
            supply_increase: (&value.supply_increase).into(),
            fee_excess: (&value.fee_excess).try_into()?,
            sok_digest: (),
        })
    }
}

impl TryFrom<&MinaStateSnarkedLedgerStateWithSokStableV2> for Statement<SokDigest> {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaStateSnarkedLedgerStateWithSokStableV2) -> Result<Self, Self::Error> {
        Ok(Self {
            source: (&value.source).try_into()?,
            target: (&value.target).try_into()?,
            connecting_ledger_left: value.connecting_ledger_left.to_field()?,
            connecting_ledger_right: value.connecting_ledger_right.to_field()?,
            supply_increase: (&value.supply_increase).into(),
            fee_excess: (&value.fee_excess).try_into()?,
            sok_digest: SokDigest(value.sok_digest.to_vec()),
        })
    }
}

impl TryFrom<&MinaStateSnarkedLedgerStateWithSokStableV2> for Statement<()> {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaStateSnarkedLedgerStateWithSokStableV2) -> Result<Self, Self::Error> {
        Ok(Self {
            source: (&value.source).try_into()?,
            target: (&value.target).try_into()?,
            connecting_ledger_left: value.connecting_ledger_left.to_field()?,
            connecting_ledger_right: value.connecting_ledger_right.to_field()?,
            supply_increase: (&value.supply_increase).into(),
            fee_excess: (&value.fee_excess).try_into()?,
            sok_digest: (),
        })
    }
}

impl From<&Statement<SokDigest>> for MinaStateSnarkedLedgerStateWithSokStableV2 {
    fn from(value: &Statement<SokDigest>) -> Self {
        Self {
            source: (&value.source).into(),
            target: (&value.target).into(),
            connecting_ledger_left: MinaBaseLedgerHash0StableV1(
                (&value.connecting_ledger_left).into(),
            )
            .into(),
            connecting_ledger_right: MinaBaseLedgerHash0StableV1(
                (&value.connecting_ledger_right).into(),
            )
            .into(),
            supply_increase: (&value.supply_increase).into(),
            fee_excess: (&value.fee_excess).into(),
            sok_digest: (&value.sok_digest).into(),
        }
    }
}

impl From<&MinaBaseTransactionStatusStableV2> for TransactionStatus {
    fn from(value: &MinaBaseTransactionStatusStableV2) -> Self {
        match value {
            MinaBaseTransactionStatusStableV2::Applied => Self::Applied,
            MinaBaseTransactionStatusStableV2::Failed(faileds) => Self::Failed(
                faileds
                    .0
                    .iter()
                    .map(|s| s.iter().map(Into::into).collect())
                    .collect(),
            ),
        }
    }
}

impl From<&TransactionStatus> for MinaBaseTransactionStatusStableV2 {
    fn from(value: &TransactionStatus) -> Self {
        match value {
            TransactionStatus::Applied => Self::Applied,
            TransactionStatus::Failed(faileds) => {
                Self::Failed(MinaBaseTransactionStatusFailureCollectionStableV1(
                    faileds
                        .iter()
                        .map(|s| s.iter().map(Into::into).collect())
                        .collect(),
                ))
            }
        }
    }
}

impl TryFrom<&MinaBaseAccountUpdateFeePayerStableV1> for FeePayer {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaBaseAccountUpdateFeePayerStableV1) -> Result<Self, Self::Error> {
        Ok(Self {
            body: FeePayerBody {
                public_key: value.body.public_key.clone().into_inner().try_into()?,
                fee: Fee::from_u64(value.body.fee.as_u64()),
                valid_until: value
                    .body
                    .valid_until
                    .as_ref()
                    .map(|until| Slot::from_u32(until.as_u32())),
                nonce: Nonce::from_u32(value.body.nonce.as_u32()),
            },
            authorization: Signature {
                rx: value.authorization.0.to_field()?,
                s: value.authorization.1.to_field()?,
            },
        })
    }
}

impl From<&FeePayer> for MinaBaseAccountUpdateFeePayerStableV1 {
    fn from(value: &FeePayer) -> Self {
        Self {
            body: MinaBaseAccountUpdateBodyFeePayerStableV1 {
                public_key: (&value.body.public_key).into(),
                fee: (&value.body.fee).into(),
                valid_until: value.body.valid_until.as_ref().map(|until| until.into()),
                nonce: (&value.body.nonce).into(),
            },
            authorization: MinaBaseSignatureStableV1::from(&value.authorization).into(),
        }
    }
}

impl From<&MinaBaseAccountUpdateUpdateTimingInfoStableV1> for zkapp_command::Timing {
    fn from(t: &MinaBaseAccountUpdateUpdateTimingInfoStableV1) -> Self {
        Self {
            initial_minimum_balance: Balance::from_u64(t.initial_minimum_balance.as_u64()),
            cliff_time: Slot::from_u32(t.cliff_time.as_u32()),
            cliff_amount: Amount::from_u64(t.cliff_amount.as_u64()),
            vesting_period: SlotSpan::from_u32(t.vesting_period.as_u32()),
            vesting_increment: Amount::from_u64(t.vesting_increment.as_u64()),
        }
    }
}

impl From<&zkapp_command::Timing> for MinaBaseAccountUpdateUpdateTimingInfoStableV1 {
    fn from(t: &zkapp_command::Timing) -> Self {
        Self {
            initial_minimum_balance: CurrencyBalanceStableV1(t.initial_minimum_balance.into()),
            cliff_time: (&t.cliff_time).into(),
            cliff_amount: t.cliff_amount.into(),
            vesting_period: (&t.vesting_period).into(),
            vesting_increment: t.vesting_increment.into(),
        }
    }
}

impl From<&MinaBaseZkappPreconditionProtocolStateStableV1Length>
    for zkapp_command::Numeric<Length>
{
    fn from(value: &MinaBaseZkappPreconditionProtocolStateStableV1Length) -> Self {
        use zkapp_command::{ClosedInterval, Numeric};
        use MinaBaseZkappPreconditionProtocolStateStableV1Length as MLength;

        match value {
            MLength::Check(length) => Numeric::Check(ClosedInterval {
                lower: Length::from_u32(length.lower.0.as_u32()),
                upper: Length::from_u32(length.upper.0.as_u32()),
            }),
            MLength::Ignore => Numeric::Ignore,
        }
    }
}

impl From<&zkapp_command::Numeric<Length>>
    for MinaBaseZkappPreconditionProtocolStateStableV1Length
{
    fn from(value: &zkapp_command::Numeric<Length>) -> Self {
        use zkapp_command::Numeric;
        use MinaBaseZkappPreconditionProtocolStateStableV1Length as MLength;

        match value {
            Numeric::Check(length) => {
                MLength::Check(MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
                    lower: (&length.lower).into(),
                    upper: (&length.upper).into(),
                })
            }
            Numeric::Ignore => MLength::Ignore,
        }
    }
}

impl<F: FieldWitness> TryFrom<&ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1>
    for protocol_state::EpochData<F>
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            ledger: protocol_state::EpochLedger {
                hash: value.ledger.hash.0.to_field()?,
                total_currency: value.ledger.total_currency.clone().into(),
            },
            seed: value.seed.0.to_field()?,
            start_checkpoint: value.start_checkpoint.0.to_field()?,
            lock_checkpoint: value.lock_checkpoint.0.to_field()?,
            epoch_length: (&value.epoch_length).into(),
        })
    }
}

impl<F: FieldWitness> TryFrom<&ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1>
    for protocol_state::EpochData<F>
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            ledger: protocol_state::EpochLedger {
                hash: value.ledger.hash.0.to_field()?,
                total_currency: value.ledger.total_currency.clone().into(),
            },
            seed: value.seed.0.to_field()?,
            start_checkpoint: value.start_checkpoint.0.to_field()?,
            lock_checkpoint: value.lock_checkpoint.0.to_field()?,
            epoch_length: (&value.epoch_length).into(),
        })
    }
}

impl TryFrom<&MinaBaseZkappPreconditionProtocolStateEpochDataStableV1>
    for zkapp_command::EpochData
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &MinaBaseZkappPreconditionProtocolStateEpochDataStableV1,
    ) -> Result<Self, Self::Error> {
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochSeed as Seed;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint as Start;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1Amount as MAmount;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash as Hash;
        use zkapp_command::{ClosedInterval, OrIgnore};

        Ok(Self {
            ledger: zkapp_command::EpochLedger {
                hash: match &value.ledger.hash {
                    Hash::Check(hash) => OrIgnore::Check(hash.to_field()?),
                    Hash::Ignore => OrIgnore::Ignore,
                },
                total_currency: match &value.ledger.total_currency {
                    MAmount::Check(amount) => OrIgnore::Check(ClosedInterval {
                        lower: Amount::from_u64(amount.lower.0 .0.as_u64()),
                        upper: Amount::from_u64(amount.upper.0 .0.as_u64()),
                    }),
                    MAmount::Ignore => OrIgnore::Ignore,
                },
            },
            seed: match &value.seed {
                Seed::Check(seed) => OrIgnore::Check(seed.to_field()?),
                Seed::Ignore => OrIgnore::Ignore,
            },
            start_checkpoint: match &value.start_checkpoint {
                Start::Check(start) => OrIgnore::Check(start.to_field()?),
                Start::Ignore => OrIgnore::Ignore,
            },
            lock_checkpoint: match &value.lock_checkpoint {
                Start::Check(start) => OrIgnore::Check(start.to_field()?),
                Start::Ignore => OrIgnore::Ignore,
            },
            epoch_length: (&value.epoch_length).into(),
        })
    }
}

fn fp_to_epochseed(value: &Fp) -> EpochSeed {
    let hash: MinaBaseEpochSeedStableV1 = MinaBaseEpochSeedStableV1(value.into());
    hash.into()
}

fn fp_to_statehash(value: &Fp) -> StateHash {
    let hash: DataHashLibStateHashStableV1 = DataHashLibStateHashStableV1(value.into());
    hash.into()
}

impl From<&zkapp_command::EpochData> for MinaBaseZkappPreconditionProtocolStateEpochDataStableV1 {
    fn from(value: &zkapp_command::EpochData) -> Self {
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochSeed as Seed;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint as Start;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1Amount as MAmount;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash as Hash;
        use zkapp_command::OrIgnore;

        Self {
            ledger: MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochLedger {
                hash: match &value.ledger.hash {
                    OrIgnore::Check(hash) => Hash::Check({
                        let hash = MinaBaseLedgerHash0StableV1(hash.into());
                        hash.into()
                    }),
                    OrIgnore::Ignore => Hash::Ignore,
                },
                total_currency: match &value.ledger.total_currency {
                    OrIgnore::Check(amount) => {
                        MAmount::Check(MinaBaseZkappPreconditionProtocolStateStableV1AmountA {
                            lower: amount.lower.into(),
                            upper: amount.upper.into(),
                        })
                    }
                    OrIgnore::Ignore => MAmount::Ignore,
                },
            },
            seed: match &value.seed {
                OrIgnore::Check(seed) => Seed::Check(fp_to_epochseed(seed)),
                OrIgnore::Ignore => Seed::Ignore,
            },
            start_checkpoint: match &value.start_checkpoint {
                OrIgnore::Check(start) => Start::Check(fp_to_statehash(start)),
                OrIgnore::Ignore => Start::Ignore,
            },
            lock_checkpoint: match &value.lock_checkpoint {
                OrIgnore::Check(start) => Start::Check(fp_to_statehash(start)),
                OrIgnore::Ignore => Start::Ignore,
            },
            epoch_length: (&value.epoch_length).into(),
        }
    }
}

impl TryFrom<&MinaBaseAccountUpdatePreconditionsStableV1> for zkapp_command::Preconditions {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaBaseAccountUpdatePreconditionsStableV1) -> Result<Self, Self::Error> {
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1Amount as MAmount;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot as MSlot;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash as Ledger;
        use zkapp_command::AccountPreconditions;
        use zkapp_command::{ClosedInterval, Numeric, OrIgnore};

        Ok(Self {
            network: zkapp_command::ZkAppPreconditions {
                snarked_ledger_hash: match &value.network.snarked_ledger_hash {
                    Ledger::Check(hash) => OrIgnore::Check(hash.to_field()?),
                    Ledger::Ignore => OrIgnore::Ignore,
                },
                blockchain_length: (&value.network.blockchain_length).into(),
                min_window_density: (&value.network.min_window_density).into(),
                total_currency: match &value.network.total_currency {
                    MAmount::Check(amount) => OrIgnore::Check(ClosedInterval {
                        lower: Amount::from_u64(amount.lower.0 .0.as_u64()),
                        upper: Amount::from_u64(amount.upper.0 .0.as_u64()),
                    }),
                    MAmount::Ignore => OrIgnore::Ignore,
                },
                global_slot_since_genesis: match &value.network.global_slot_since_genesis {
                    MSlot::Check(length) => Numeric::Check(ClosedInterval {
                        lower: (&length.lower).into(),
                        upper: (&length.upper).into(),
                    }),
                    MSlot::Ignore => OrIgnore::Ignore,
                },
                staking_epoch_data: (&value.network.staking_epoch_data).try_into()?,
                next_epoch_data: (&value.network.next_epoch_data).try_into()?,
            },
            account: {
                let account = &value.account.0;
                use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2Balance as MBalance;
                use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2Delegate as Delegate;
                use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2ProvedState as Proved;
                use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash as Receipt;
                use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2StateA as State;
                use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1Length as MNonce;

                AccountPreconditions(zkapp_command::Account {
                    balance: match &account.balance {
                        MBalance::Check(balance) => OrIgnore::Check(ClosedInterval {
                            lower: Balance::from_u64(balance.lower.0.as_u64()),
                            upper: Balance::from_u64(balance.upper.0.as_u64()),
                        }),
                        MBalance::Ignore => OrIgnore::Ignore,
                    },
                    nonce: match &account.nonce {
                        MNonce::Check(balance) => OrIgnore::Check(ClosedInterval {
                            lower: Nonce::from_u32(balance.lower.0.as_u32()),
                            upper: Nonce::from_u32(balance.upper.0.as_u32()),
                        }),
                        MNonce::Ignore => OrIgnore::Ignore,
                    },
                    receipt_chain_hash: match &account.receipt_chain_hash {
                        Receipt::Check(hash) => OrIgnore::Check(hash.to_field()?),
                        Receipt::Ignore => OrIgnore::Ignore,
                    },
                    delegate: match &account.delegate {
                        Delegate::Check(delegate) => {
                            OrIgnore::Check(delegate.clone().into_inner().try_into()?)
                        }
                        Delegate::Ignore => OrIgnore::Ignore,
                    },
                    state: crate::try_array_into_with(&account.state, |s| match s {
                        State::Check(s) => Ok(OrIgnore::Check(s.to_field()?)),
                        State::Ignore => Ok(OrIgnore::Ignore),
                    })?,
                    action_state: match &account.action_state {
                        State::Check(s) => OrIgnore::Check(s.to_field()?),
                        State::Ignore => OrIgnore::Ignore,
                    },
                    proved_state: match account.proved_state {
                        Proved::Check(state) => OrIgnore::Check(state),
                        Proved::Ignore => OrIgnore::Ignore,
                    },
                    is_new: match account.is_new {
                        Proved::Check(state) => OrIgnore::Check(state),
                        Proved::Ignore => OrIgnore::Ignore,
                    },
                })
            },
            valid_while: match &value.valid_while {
                MSlot::Check(valid_while) => OrIgnore::Check(ClosedInterval {
                    lower: (&valid_while.lower).into(),
                    upper: (&valid_while.upper).into(),
                }),
                MSlot::Ignore => OrIgnore::Ignore,
            },
        })
    }
}

impl From<&BlockTime> for BlockTimeTimeStableV1 {
    fn from(value: &BlockTime) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            value.as_u64().into(),
        ))
    }
}

impl From<&zkapp_command::Preconditions> for MinaBaseAccountUpdatePreconditionsStableV1 {
    fn from(value: &zkapp_command::Preconditions) -> Self {
        use mina_p2p_messages::v2::MinaBaseAccountUpdateAccountPreconditionStableV1 as MAccount;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1Amount as MAmount;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlot as MSlot;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash as Ledger;
        use zkapp_command::{Numeric, OrIgnore};

        Self {
            network: MinaBaseZkappPreconditionProtocolStateStableV1 {
                snarked_ledger_hash: match &value.network.snarked_ledger_hash {
                    OrIgnore::Check(hash) => Ledger::Check({
                        let hash = MinaBaseLedgerHash0StableV1(hash.into());
                        hash.into()
                    }),
                    OrIgnore::Ignore => Ledger::Ignore,
                },
                blockchain_length: (&value.network.blockchain_length).into(),
                min_window_density: (&value.network.min_window_density).into(),
                total_currency: match &value.network.total_currency {
                    OrIgnore::Check(amount) => {
                        MAmount::Check(MinaBaseZkappPreconditionProtocolStateStableV1AmountA {
                            lower: amount.lower.into(),
                            upper: amount.upper.into(),
                        })
                    }
                    OrIgnore::Ignore => MAmount::Ignore,
                },
                global_slot_since_genesis: match &value.network.global_slot_since_genesis {
                    Numeric::Check(length) => {
                        MSlot::Check(MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlotA {
                            lower: (&length.lower).into(),
                            upper: (&length.upper).into(),
                        })
                    }
                    Numeric::Ignore => MSlot::Ignore,
                },
                staking_epoch_data: (&value.network.staking_epoch_data).into(),
                next_epoch_data: (&value.network.next_epoch_data).into(),
            },
            account: {
                use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2Balance as MBalance;
                use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2Delegate as Delegate;
                use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2ProvedState as Proved;
                use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash as Receipt;
                use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2StateA as State;
                use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1Length as MNonce;

                let account = &value.account.0;
                MAccount(MinaBaseZkappPreconditionAccountStableV2 {
                    balance: match &account.balance {
                        OrIgnore::Check(balance) => {
                            MBalance::Check(MinaBaseZkappPreconditionAccountStableV2BalanceA {
                                lower: (&balance.lower).into(),
                                upper: (&balance.upper).into(),
                            })
                        }
                        OrIgnore::Ignore => MBalance::Ignore,
                    },
                    nonce: match &account.nonce {
                        OrIgnore::Check(nonce) => {
                            MNonce::Check(MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
                                lower: (&nonce.lower).into(),
                                upper: (&nonce.upper).into(),
                            })
                        }
                        OrIgnore::Ignore => MNonce::Ignore,
                    },
                    receipt_chain_hash: match &account.receipt_chain_hash {
                        OrIgnore::Check(hash) => {
                            Receipt::Check(MinaBaseReceiptChainHashStableV1(hash.into()))
                        }
                        OrIgnore::Ignore => Receipt::Ignore,
                    },
                    delegate: match &account.delegate {
                        OrIgnore::Check(delegate) => Delegate::Check(delegate.into()),
                        OrIgnore::Ignore => Delegate::Ignore,
                    },
                    state: PaddedSeq(array_into_with(&account.state, |s| match s {
                        OrIgnore::Check(s) => State::Check(s.into()),
                        OrIgnore::Ignore => State::Ignore,
                    })),
                    action_state: match &account.action_state {
                        OrIgnore::Check(s) => State::Check(s.into()),
                        OrIgnore::Ignore => State::Ignore,
                    },
                    proved_state: match account.proved_state {
                        OrIgnore::Check(state) => Proved::Check(state),
                        OrIgnore::Ignore => Proved::Ignore,
                    },
                    is_new: match account.is_new {
                        OrIgnore::Check(state) => Proved::Check(state),
                        OrIgnore::Ignore => Proved::Ignore,
                    },
                })
            },
            valid_while: match &value.valid_while {
                OrIgnore::Check(valid_while) => {
                    MSlot::Check(MinaBaseZkappPreconditionProtocolStateStableV1GlobalSlotA {
                        lower: (&valid_while.lower).into(),
                        upper: (&valid_while.upper).into(),
                    })
                }
                OrIgnore::Ignore => MSlot::Ignore,
            },
        }
    }
}

/// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/verification_key_wire.ml#L37
fn of_vk(data: VerificationKey) -> WithHash<VerificationKey> {
    let hash = data.hash();
    WithHash { data, hash }
}

impl TryFrom<&MinaBaseAccountUpdateTStableV1> for AccountUpdate {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaBaseAccountUpdateTStableV1) -> Result<Self, Self::Error> {
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1Delegate as Delegate;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1Permissions as Perm;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1Timing as Timing;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1TokenSymbol as TokenSymbol;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1VerificationKey as VK;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1VotingFor as Voting;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1ZkappUri as ZkAppUri;
        use MinaBaseAccountUpdateUpdateStableV1AppStateA as AppState;

        Ok(Self {
            body: zkapp_command::Body {
                public_key: value.body.public_key.clone().into_inner().try_into()?,
                token_id: value.body.token_id.clone().into_inner().try_into()?,
                update: zkapp_command::Update {
                    app_state: crate::try_array_into_with(&value.body.update.app_state, |s| match s {
                            AppState::Set(bigint) => Ok(SetOrKeep::Set(bigint.to_field()?)),
                            AppState::Keep => Ok(SetOrKeep::Keep),
                    })?,
                    delegate: match &value.body.update.delegate {
                        Delegate::Set(v) => SetOrKeep::Set(v.try_into()?),
                        Delegate::Keep => SetOrKeep::Keep,
                    },
                    verification_key: match &value.body.update.verification_key {
                        VK::Set(vk) => SetOrKeep::Set(of_vk((&**vk).try_into()?)),
                        VK::Keep => SetOrKeep::Keep,
                    },
                    permissions: match &value.body.update.permissions {
                        Perm::Set(perms) => SetOrKeep::Set((&**perms).into()),
                        Perm::Keep => SetOrKeep::Keep,
                    },
                    zkapp_uri: match &value.body.update.zkapp_uri {
                        ZkAppUri::Set(s) => SetOrKeep::Set(s.try_into().unwrap()),
                        ZkAppUri::Keep => SetOrKeep::Keep,
                    },
                    token_symbol: match &value.body.update.token_symbol {
                        TokenSymbol::Set(s) => SetOrKeep::Set(s.try_into().unwrap()),
                        TokenSymbol::Keep => SetOrKeep::Keep,
                    },
                    timing: match &value.body.update.timing {
                        Timing::Set(timing) => SetOrKeep::Set((&**timing).into()),
                        Timing::Keep => SetOrKeep::Keep,
                    },
                    voting_for: match &value.body.update.voting_for {
                        Voting::Set(bigint) => SetOrKeep::Set(VotingFor(bigint.to_field()?)),
                        Voting::Keep => SetOrKeep::Keep,
                    },
                },
                balance_change: Signed::<Amount> {
                    magnitude: Amount(value.body.balance_change.magnitude.as_u64()),
                    sgn: value.body.balance_change.sgn.clone().into(),
                },
                increment_nonce: value.body.increment_nonce,
                events: zkapp_command::Events(
                    value
                        .body
                        .events
                        .0
                        .iter()
                        .map(|e| Ok(zkapp_command::Event(e.iter().map(|e| e.to_field()).collect::<Result<Vec<Fp>, _>>()?)))
                        .collect::<Result<_, _>>()?,
                ),
                actions: zkapp_command::Actions(
                    value
                        .body
                        .actions
                        .0
                        .iter()
                        .map(|e| Ok(zkapp_command::Event(e.iter().map(|e| e.to_field()).collect::<Result<Vec<Fp>, _>>()?)))
                        .collect::<Result<_, _>>()?,
                ),
                call_data: value.body.call_data.to_field()?,
                preconditions: (&value.body.preconditions).try_into()?,
                use_full_commitment: value.body.use_full_commitment,
                authorization_kind: match &value.body.authorization_kind {
                    mina_p2p_messages::v2::MinaBaseAccountUpdateAuthorizationKindStableV1::NoneGiven => AuthorizationKind::NoneGiven,
                    mina_p2p_messages::v2::MinaBaseAccountUpdateAuthorizationKindStableV1::Signature => AuthorizationKind::Signature,
                    mina_p2p_messages::v2::MinaBaseAccountUpdateAuthorizationKindStableV1::Proof(hash) => AuthorizationKind::Proof(hash.to_field()?),
                },
                implicit_account_creation_fee: value.body.implicit_account_creation_fee,
                may_use_token: match value.body.may_use_token {
                    MinaBaseAccountUpdateMayUseTokenStableV1::No => MayUseToken::No,
                    MinaBaseAccountUpdateMayUseTokenStableV1::ParentsOwnToken => MayUseToken::ParentsOwnToken,
                    MinaBaseAccountUpdateMayUseTokenStableV1::InheritFromParent => MayUseToken::InheritFromParent,
                },
            },
            authorization: match &value.authorization {
                mina_p2p_messages::v2::MinaBaseControlStableV2::Proof(proof) => zkapp_command::Control::Proof((**proof).clone().into()),
                mina_p2p_messages::v2::MinaBaseControlStableV2::Signature(signature) => zkapp_command::Control::Signature(Signature{
                    rx: signature.0.to_field()?,
                    s: signature.1.to_field()?
            }),
                mina_p2p_messages::v2::MinaBaseControlStableV2::NoneGiven => zkapp_command::Control::NoneGiven,
            },
        })
    }
}

/// Notes: childs
impl TryFrom<&List<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA>>
    for CallForest<AccountUpdate>
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &List<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA>,
    ) -> Result<Self, Self::Error> {
        use ark_ff::Zero;

        Ok(Self(
            value
                .iter()
                .map(|update| {
                    Ok(WithStackHash {
                        elt: zkapp_command::Tree {
                            account_update: (&update.elt.account_update).try_into()?,
                            account_update_digest: Fp::zero(), // replaced later
                            calls: (&update.elt.calls).try_into()?,
                        },
                        stack_hash: Fp::zero(), // replaced later
                    })
                })
                .collect::<Result<_, _>>()?,
        ))
    }
}
/// Notes: root
impl TryFrom<&List<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA>>
    for CallForest<AccountUpdate>
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &List<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA>,
    ) -> Result<Self, Self::Error> {
        use ark_ff::Zero;

        let values = value
            .iter()
            .map(|update| {
                Ok(WithStackHash {
                    elt: zkapp_command::Tree {
                        account_update: (&update.elt.account_update).try_into()?,
                        account_update_digest: Fp::zero(), // replaced later in `of_wire`
                        calls: (&update.elt.calls).try_into()?,
                    },
                    stack_hash: Fp::zero(), // replaced later in `of_wire`
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_command.ml#L1113-L1115

        let mut call_forest = CallForest(values);
        call_forest.of_wire(&[]);
        // call_forest.of_wire(value);

        Ok(call_forest)
    }
}

impl TryFrom<&v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesDataA>
    for WithHash<VerificationKey>
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesDataA,
    ) -> Result<Self, Self::Error> {
        let v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesDataA { data, hash } = value;
        Ok(Self {
            data: data.try_into()?,
            hash: hash.try_into()?,
        })
    }
}

impl From<&WithHash<VerificationKey>>
    for v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesDataA
{
    fn from(value: &WithHash<VerificationKey>) -> Self {
        let WithHash::<VerificationKey> { data, hash } = value;
        Self {
            data: data.into(),
            hash: hash.into(),
        }
    }
}

/// Notes: childs for verifiable
impl TryFrom<&List<v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesAACallsA>>
    for CallForest<(AccountUpdate, Option<WithHash<VerificationKey>>)>
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &List<v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesAACallsA>,
    ) -> Result<Self, Self::Error> {
        Ok(Self(
            value
                .iter()
                .map(|update| {
                    let v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesAACallsA {
                        elt,
                        stack_hash,
                    } = update;
                    let v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesAA {
                        account_update: (account, vk_opt),
                        account_update_digest,
                        calls,
                    } = &**elt;
                    let vk_opt = match vk_opt.as_ref() {
                        Some(vk) => Some(vk.try_into()?),
                        None => None,
                    };
                    Ok(WithStackHash {
                        elt: zkapp_command::Tree {
                            account_update: (account.try_into()?, vk_opt),
                            account_update_digest: account_update_digest.to_field()?,
                            calls: calls.try_into()?,
                        },
                        stack_hash: stack_hash.to_field()?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}
/// Notes: root for verifiable
impl TryFrom<&List<v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesA>>
    for CallForest<(AccountUpdate, Option<WithHash<VerificationKey>>)>
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &List<v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesA>,
    ) -> Result<Self, Self::Error> {
        let values = value
            .iter()
            .map(|update| {
                let v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesA { elt, stack_hash } =
                    update;
                let v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesAA {
                    account_update: (account, vk_opt),
                    account_update_digest,
                    calls,
                } = elt;
                let vk_opt = match vk_opt.as_ref() {
                    Some(vk) => Some(vk.try_into()?),
                    None => None,
                };
                Ok(WithStackHash {
                    elt: zkapp_command::Tree {
                        account_update: (account.try_into()?, vk_opt),
                        account_update_digest: account_update_digest.to_field()?,
                        calls: calls.try_into()?,
                    },
                    stack_hash: stack_hash.to_field()?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // There is no need to call `of_wire`, because hashes are in our serialized types (verifiables types only)
        // call_forest.of_wire(&[]);

        Ok(CallForest(values))
    }
}

/// Childs for verifiable
impl From<&CallForest<(AccountUpdate, Option<WithHash<VerificationKey>>)>>
    for List<v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesAACallsA>
{
    fn from(value: &CallForest<(AccountUpdate, Option<WithHash<VerificationKey>>)>) -> Self {
        value
            .0
            .iter()
            .map(|update| {
                let (acc, opt) = &update.elt.account_update;
                v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesAACallsA {
                    elt: Box::new(v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesAA {
                        account_update: (acc.into(), opt.as_ref().map(Into::into)),
                        account_update_digest:
                            v2::MinaBaseZkappCommandCallForestMakeDigestStrAccountUpdateStableV1(
                                update.elt.account_update_digest.into(),
                            ),
                        calls: (&update.elt.calls).into(),
                    }),
                    stack_hash: v2::MinaBaseZkappCommandCallForestMakeDigestStrForestStableV1(
                        update.stack_hash.into(),
                    ),
                }
            })
            .collect()
    }
}
/// Root
impl From<&CallForest<(AccountUpdate, Option<WithHash<VerificationKey>>)>>
    for List<v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesA>
{
    fn from(value: &CallForest<(AccountUpdate, Option<WithHash<VerificationKey>>)>) -> Self {
        value
            .0
            .iter()
            .map(|update| {
                let (acc, opt) = &update.elt.account_update;
                v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesA {
                    elt: v2::MinaBaseZkappCommandVerifiableStableV1AccountUpdatesAA {
                        account_update: (acc.into(), opt.as_ref().map(Into::into)),
                        account_update_digest:
                            v2::MinaBaseZkappCommandCallForestMakeDigestStrAccountUpdateStableV1(
                                update.elt.account_update_digest.into(),
                            ),
                        calls: (&update.elt.calls).into(),
                    },
                    stack_hash: v2::MinaBaseZkappCommandCallForestMakeDigestStrForestStableV1(
                        update.stack_hash.into(),
                    ),
                }
            })
            .collect()

        // There is no need to call `to_wire`, because hashes are in our serialized types (verifiables types only)
        // value.to_wire(&mut wired);
        // wired.into_iter().collect()
    }
}

/// We need this trait because `mina-p2p-messages` contains different types for the same data
pub trait AsAccountUpdateWithHash {
    fn elt(&self) -> &MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA;
    fn elt_mut(&mut self) -> &mut MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA;
}

impl AsAccountUpdateWithHash for MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA {
    fn elt(&self) -> &MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA {
        &self.elt
    }
    fn elt_mut(&mut self) -> &mut MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA {
        &mut self.elt
    }
}

impl AsAccountUpdateWithHash for MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA {
    fn elt(&self) -> &MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA {
        &self.elt
    }
    fn elt_mut(&mut self) -> &mut MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA {
        &mut self.elt
    }
}

impl From<&AccountUpdate> for MinaBaseAccountUpdateTStableV1 {
    fn from(value: &AccountUpdate) -> Self {
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1Delegate as Delegate;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1Permissions as Perm;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1Timing as Timing;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1TokenSymbol as TokenSymbol;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1VerificationKey as VK;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1VotingFor as Voting;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1ZkappUri as ZkAppUri;
        use MinaBaseAccountUpdateUpdateStableV1AppStateA as AppState;

        Self {
            body: MinaBaseAccountUpdateBodyStableV1 {
                public_key: (&value.body.public_key).into(),
                token_id: (&value.body.token_id).into(),
                update: MinaBaseAccountUpdateUpdateStableV1 {
                    app_state: PaddedSeq(value.body.update.app_state.each_ref().map(|s| match s {
                        SetOrKeep::Set(bigint) => AppState::Set(bigint.into()),
                        SetOrKeep::Keep => AppState::Keep,
                    })),
                    delegate: match &value.body.update.delegate {
                        SetOrKeep::Set(v) => Delegate::Set(v.into()),
                        SetOrKeep::Keep => Delegate::Keep,
                    },
                    verification_key: match &value.body.update.verification_key {
                        SetOrKeep::Set(vk) => VK::Set(Box::new((&vk.data).into())),
                        SetOrKeep::Keep => VK::Keep,
                    },
                    permissions: match &value.body.update.permissions {
                        SetOrKeep::Set(perms) => Perm::Set(Box::new(perms.into())),
                        SetOrKeep::Keep => Perm::Keep,
                    },
                    zkapp_uri: match &value.body.update.zkapp_uri {
                        SetOrKeep::Set(s) => ZkAppUri::Set(s.into()),
                        SetOrKeep::Keep => ZkAppUri::Keep,
                    },
                    token_symbol: match &value.body.update.token_symbol {
                        SetOrKeep::Set(s) => TokenSymbol::Set(s.into()),
                        SetOrKeep::Keep => TokenSymbol::Keep,
                    },
                    timing: match &value.body.update.timing {
                        SetOrKeep::Set(timing) => Timing::Set(Box::new(timing.into())),
                        SetOrKeep::Keep => Timing::Keep,
                    },
                    voting_for: match &value.body.update.voting_for {
                        SetOrKeep::Set(bigint) => Voting::Set(DataHashLibStateHashStableV1(bigint.0.into()).into()),
                        SetOrKeep::Keep => Voting::Keep,
                    },
                },
                balance_change: (&value.body.balance_change).into(),
                increment_nonce: value.body.increment_nonce,
                events: MinaBaseAccountUpdateBodyEventsStableV1(
                    value
                        .body
                        .events
                        .0
                        .iter()
                        .map(|e| e.0.iter().map(|e| e.into()).collect())
                        .collect(),
                ),
                actions: MinaBaseAccountUpdateBodyEventsStableV1(
                    value
                        .body
                        .actions
                        .0
                        .iter()
                        .map(|e| e.0.iter().map(|e| e.into()).collect())
                        .collect(),
                ),
                call_data: value.body.call_data.into(),
                preconditions: (&value.body.preconditions).into(),
                use_full_commitment: value.body.use_full_commitment,
                authorization_kind: match value.body.authorization_kind {
                    AuthorizationKind::NoneGiven => mina_p2p_messages::v2::MinaBaseAccountUpdateAuthorizationKindStableV1::NoneGiven ,
                    AuthorizationKind::Signature => mina_p2p_messages::v2::MinaBaseAccountUpdateAuthorizationKindStableV1::Signature ,
                    AuthorizationKind::Proof(hash) => mina_p2p_messages::v2::MinaBaseAccountUpdateAuthorizationKindStableV1::Proof(hash.into()) ,
                },
                implicit_account_creation_fee: value.body.implicit_account_creation_fee,
                may_use_token: match value.body.may_use_token {
                    zkapp_command::MayUseToken::No => MinaBaseAccountUpdateMayUseTokenStableV1::No,
                    zkapp_command::MayUseToken::ParentsOwnToken => MinaBaseAccountUpdateMayUseTokenStableV1::ParentsOwnToken,
                    zkapp_command::MayUseToken::InheritFromParent => MinaBaseAccountUpdateMayUseTokenStableV1::InheritFromParent,
                },
            },
            authorization: match &value.authorization {
                zkapp_command::Control::Proof(proof) => mina_p2p_messages::v2::MinaBaseControlStableV2::Proof(Box::new((**proof).clone())),
                zkapp_command::Control::Signature(sig) => mina_p2p_messages::v2::MinaBaseControlStableV2::Signature({
                    let sig: MinaBaseSignatureStableV1 = sig.into();
                    sig.into()
                }),
                zkapp_command::Control::NoneGiven => mina_p2p_messages::v2::MinaBaseControlStableV2::NoneGiven,
            },
        }
    }
}

/// Childs
impl From<&CallForest<AccountUpdate>>
    for List<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA>
{
    fn from(value: &CallForest<AccountUpdate>) -> Self {
        value
            .0
            .iter()
            .map(
                |update| MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA {
                    elt: Box::new(MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA {
                        account_update: (&update.elt.account_update).into(),
                        account_update_digest: (),
                        calls: (&update.elt.calls).into(),
                    }),
                    stack_hash: (),
                },
            )
            .collect()
    }
}

/// Root
impl From<&CallForest<AccountUpdate>>
    for List<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA>
{
    fn from(value: &CallForest<AccountUpdate>) -> Self {
        let mut wired: Vec<_> = value
            .0
            .iter()
            .map(
                |update| MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA {
                    elt: MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA {
                        account_update: (&update.elt.account_update).into(),
                        account_update_digest: (),
                        calls: (&update.elt.calls).into(),
                    },
                    stack_hash: (),
                },
            )
            .collect();

        value.to_wire(&mut wired);
        wired.into_iter().collect()
    }
}

impl TryFrom<&MinaBaseFeeTransferSingleStableV2> for SingleFeeTransfer {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaBaseFeeTransferSingleStableV2) -> Result<Self, Self::Error> {
        Ok(Self {
            receiver_pk: (&value.receiver_pk).try_into()?,
            fee: Fee::from_u64(value.fee.as_u64()),
            fee_token: (&*value.fee_token).try_into()?,
        })
    }
}

impl From<&SingleFeeTransfer> for MinaBaseFeeTransferSingleStableV2 {
    fn from(value: &SingleFeeTransfer) -> Self {
        Self {
            receiver_pk: (&value.receiver_pk).into(),
            fee: (&value.fee).into(),
            fee_token: (&value.fee_token).into(),
        }
    }
}

impl TryFrom<&MinaBaseFeeTransferStableV2> for FeeTransfer {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaBaseFeeTransferStableV2) -> Result<Self, Self::Error> {
        use super::scan_state::transaction_snark::OneOrTwo::{One, Two};

        match value {
            MinaBaseFeeTransferStableV2::One(ft) => Ok(FeeTransfer(One(ft.try_into()?))),
            MinaBaseFeeTransferStableV2::Two((a, b)) => {
                Ok(FeeTransfer(Two((a.try_into()?, b.try_into()?))))
            }
        }
    }
}

impl From<&FeeTransfer> for MinaBaseFeeTransferStableV2 {
    fn from(value: &FeeTransfer) -> Self {
        use super::scan_state::transaction_snark::OneOrTwo::{One, Two};

        match &value.0 {
            One(ft) => MinaBaseFeeTransferStableV2::One(ft.into()),
            Two((a, b)) => MinaBaseFeeTransferStableV2::Two((a.into(), b.into())),
        }
    }
}

impl From<&MinaBaseSignedCommandMemoStableV1> for Memo {
    fn from(value: &MinaBaseSignedCommandMemoStableV1) -> Self {
        Self(value.0.as_ref().try_into().unwrap())
    }
}

impl From<&Memo> for MinaBaseSignedCommandMemoStableV1 {
    fn from(value: &Memo) -> Self {
        Self(CharString::from(value.as_slice().to_vec()))
    }
}

impl TryFrom<MinaBaseSignedCommandStableV2> for SignedCommand {
    type Error = InvalidBigInt;

    fn try_from(value: MinaBaseSignedCommandStableV2) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl TryFrom<&MinaBaseSignedCommandStableV2> for SignedCommand {
    type Error = InvalidBigInt;

    fn try_from(cmd: &MinaBaseSignedCommandStableV2) -> Result<Self, Self::Error> {
        Ok(Self {
            payload: transaction_logic::signed_command::SignedCommandPayload {
                common: transaction_logic::signed_command::Common {
                    fee: (&cmd.payload.common.fee).into(),
                    fee_payer_pk: (&cmd.payload.common.fee_payer_pk).try_into()?,
                    nonce: (&cmd.payload.common.nonce).into(),
                    valid_until: (&cmd.payload.common.valid_until).into(),
                    memo: (&cmd.payload.common.memo).into(),
                },
                body: match &cmd.payload.body {
                    MinaBaseSignedCommandPayloadBodyStableV2::Payment(payload) => {
                        transaction_logic::signed_command::Body::Payment(PaymentPayload {
                            receiver_pk: (&payload.receiver_pk).try_into()?,
                            amount: payload.amount.clone().into(),
                        })
                    }
                    MinaBaseSignedCommandPayloadBodyStableV2::StakeDelegation(
                        MinaBaseStakeDelegationStableV2::SetDelegate { new_delegate },
                    ) => transaction_logic::signed_command::Body::StakeDelegation(
                        StakeDelegationPayload::SetDelegate {
                            new_delegate: new_delegate.try_into()?,
                        },
                    ),
                },
            },
            signer: (&cmd.signer).try_into()?,
            signature: (&*cmd.signature).try_into()?,
        })
    }
}

impl From<SignedCommand> for MinaBaseSignedCommandStableV2 {
    fn from(value: SignedCommand) -> Self {
        (&value).into()
    }
}

impl From<&SignedCommand> for MinaBaseSignedCommandStableV2 {
    fn from(cmd: &SignedCommand) -> Self {
        Self {
            payload: MinaBaseSignedCommandPayloadStableV2 {
                common: MinaBaseSignedCommandPayloadCommonStableV2 {
                    fee: (&cmd.payload.common.fee).into(),
                    fee_payer_pk: (&cmd.payload.common.fee_payer_pk).into(),
                    nonce: (&cmd.payload.common.nonce).into(),
                    valid_until: (&cmd.payload.common.valid_until).into(),
                    memo: MinaBaseSignedCommandMemoStableV1(
                        cmd.payload.common.memo.as_slice().into(),
                    ),
                },
                body: match &cmd.payload.body {
                    crate::scan_state::transaction_logic::signed_command::Body::Payment(
                        payload,
                    ) => MinaBaseSignedCommandPayloadBodyStableV2::Payment(
                        MinaBasePaymentPayloadStableV2 {
                            receiver_pk: (&payload.receiver_pk).into(),
                            amount: payload.amount.into(),
                        },
                    ),
                    crate::scan_state::transaction_logic::signed_command::Body::StakeDelegation(
                        StakeDelegationPayload::SetDelegate { new_delegate },
                    ) => MinaBaseSignedCommandPayloadBodyStableV2::StakeDelegation(
                        MinaBaseStakeDelegationStableV2::SetDelegate {
                            new_delegate: new_delegate.into(),
                        },
                    ),
                },
            },
            signer: (&cmd.signer).into(),
            signature: MinaBaseSignatureStableV1::from(&cmd.signature).into(),
        }
    }
}

impl TryFrom<&MinaBaseZkappCommandTStableV1WireStableV1> for zkapp_command::ZkAppCommand {
    type Error = InvalidBigInt;

    fn try_from(cmd: &MinaBaseZkappCommandTStableV1WireStableV1) -> Result<Self, Self::Error> {
        Ok(Self {
            fee_payer: (&cmd.fee_payer).try_into()?,
            account_updates: (&cmd.account_updates).try_into()?,
            memo: (&cmd.memo).into(),
        })
    }
}

impl TryFrom<v2::MinaBaseZkappCommandVerifiableStableV1> for verifiable::ZkAppCommand {
    type Error = InvalidBigInt;

    fn try_from(value: v2::MinaBaseZkappCommandVerifiableStableV1) -> Result<Self, Self::Error> {
        let v2::MinaBaseZkappCommandVerifiableStableV1 {
            fee_payer,
            account_updates,
            memo,
        } = &value;

        Ok(verifiable::ZkAppCommand {
            fee_payer: fee_payer.try_into()?,
            account_updates: account_updates.try_into()?,
            memo: memo.into(),
        })
    }
}

impl From<verifiable::ZkAppCommand> for v2::MinaBaseZkappCommandVerifiableStableV1 {
    fn from(value: verifiable::ZkAppCommand) -> Self {
        let verifiable::ZkAppCommand {
            fee_payer,
            account_updates,
            memo,
        } = &value;

        v2::MinaBaseZkappCommandVerifiableStableV1 {
            fee_payer: fee_payer.into(),
            account_updates: account_updates.into(),
            memo: memo.into(),
        }
    }
}

impl From<&zkapp_command::ZkAppCommand> for MinaBaseZkappCommandTStableV1WireStableV1 {
    fn from(cmd: &zkapp_command::ZkAppCommand) -> Self {
        Self {
            fee_payer: (&cmd.fee_payer).into(),
            account_updates: (&cmd.account_updates).into(),
            memo: (&cmd.memo).into(),
        }
    }
}

impl TryFrom<&TransactionSnarkScanStateTransactionWithWitnessStableV2> for TransactionWithWitness {
    type Error = InvalidBigInt;

    fn try_from(
        value: &TransactionSnarkScanStateTransactionWithWitnessStableV2,
    ) -> Result<Self, Self::Error> {
        use mina_p2p_messages::v2::MinaTransactionLogicTransactionAppliedVaryingStableV2::*;
        use mina_p2p_messages::v2::MinaTransactionLogicTransactionAppliedCommandAppliedStableV2::*;
        use mina_p2p_messages::v2::MinaTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV2::*;
        // use mina_p2p_messages::v2::TransactionSnarkPendingCoinbaseStackStateInitStackStableV1::{Base, Merge};
        use mina_p2p_messages::v2::MinaStateSnarkedLedgerStatePendingCoinbaseStackStateInitStackStableV1::{Base, Merge};
        use crate::scan_state::scan_state::transaction_snark::InitStack;
        use transaction_applied::signed_command_applied;

        Ok(Self {
            transaction_with_info: TransactionApplied {
                previous_hash: value.transaction_with_info.previous_hash.to_field()?,
                varying: match &value.transaction_with_info.varying {
                    Command(cmd) => match cmd {
                        SignedCommand(cmd) => transaction_applied::Varying::Command(
                            transaction_applied::CommandApplied::SignedCommand(Box::new(
                                transaction_applied::SignedCommandApplied {
                                    common: transaction_applied::signed_command_applied::Common {
                                        user_command: WithStatus {
                                            data: (&cmd.common.user_command.data).try_into()?,
                                            status: (&cmd.common.user_command.status).into(),
                                        },
                                    },
                                    body: match &cmd.body {
                                        Payment { new_accounts } => {
                                            signed_command_applied::Body::Payments {
                                                new_accounts: new_accounts
                                                    .iter()
                                                    .cloned()
                                                    .map(TryInto::try_into)
                                                    .collect::<Result<_, _>>()?,
                                            }
                                        }
                                        StakeDelegation { previous_delegate } => {
                                            signed_command_applied::Body::StakeDelegation {
                                                previous_delegate: match previous_delegate.as_ref() {
                                                    Some(prev) => Some(prev.try_into()?),
                                                    None => None,
                                                }
                                            }
                                        }
                                        Failed => signed_command_applied::Body::Failed,
                                    },
                                },
                            )),
                        ),
                        ZkappCommand(cmd) => transaction_applied::Varying::Command(
                            transaction_applied::CommandApplied::ZkappCommand(Box::new(
                                transaction_applied::ZkappCommandApplied {
                                    accounts: cmd
                                        .accounts
                                        .iter()
                                        .map(|(id, account_opt)| {
                                            let id: AccountId = id.try_into()?;
                                            let account: Option<Account> = match account_opt.as_ref() {
                                                Some(account) => Some(account.try_into()?),
                                                None => None,
                                            };
                                            let account = account.map(Box::new);

                                            Ok((id, account))
                                        })
                                        .collect::<Result<_, _>>()?,
                                    command: WithStatus {
                                        data: (&cmd.command.data).try_into()?,
                                        status: (&cmd.command.status).into(),
                                    },
                                    new_accounts: cmd.new_accounts.iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
                                },
                            )),
                        ),
                    },
                    FeeTransfer(ft) => transaction_applied::Varying::FeeTransfer(
                        transaction_applied::FeeTransferApplied {
                            fee_transfer: WithStatus {
                                data: (&ft.fee_transfer.data).try_into()?,
                                status: (&ft.fee_transfer.status).into(),
                            },
                            new_accounts: ft.new_accounts.iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
                            burned_tokens: ft.burned_tokens.clone().into(),
                        },
                    ),
                    Coinbase(cb) => transaction_applied::Varying::Coinbase(transaction_applied::CoinbaseApplied {
                        coinbase: WithStatus {
                            data: crate::scan_state::transaction_logic::Coinbase {
                                receiver: (&cb.coinbase.data.receiver).try_into()?,
                                amount: cb.coinbase.data.amount.clone().into(),
                                fee_transfer: match cb.coinbase.data.fee_transfer.as_ref() {
                                    Some(ft) => Some(crate::scan_state::transaction_logic::CoinbaseFeeTransfer {
                                        receiver_pk: (&ft.receiver_pk).try_into()?,
                                        fee: Fee::from_u64(ft.fee.as_u64()),
                                    }),
                                    None => None,
                                }
                            },
                            status: (&cb.coinbase.status).into(),
                        },
                        new_accounts: cb.new_accounts.iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
                        burned_tokens: cb.burned_tokens.clone().into(),
                    }),
                },
            },
            state_hash: {
                let (state, body) = &value.state_hash;
                (state.to_field()?, body.to_field()?)
            },
            statement: (&*value.statement).try_into()?,
            init_stack: match &value.init_stack {
                Base(base) => InitStack::Base(pending_coinbase::Stack {
                    data: pending_coinbase::CoinbaseStack(base.data.to_field()?),
                    state: pending_coinbase::StateStack {
                        init: base.state.init.to_field()?,
                        curr: base.state.curr.to_field()?,
                    },
                }),
                Merge => InitStack::Merge,
            },
            first_pass_ledger_witness: (&value.first_pass_ledger_witness).try_into()?,
            second_pass_ledger_witness: (&value.second_pass_ledger_witness).try_into()?,
            block_global_slot: Slot::from_u32(value.block_global_slot.as_u32()),
        })
    }
}

impl From<&TokenId> for mina_p2p_messages::v2::TokenIdKeyHash {
    fn from(value: &TokenId) -> Self {
        let id: MinaBaseAccountIdDigestStableV1 = value.clone().into();
        id.into()
    }
}

impl From<&Registers> for MinaStateBlockchainStateValueStableV2LedgerProofStatementSource {
    fn from(value: &Registers) -> Self {
        Self {
            first_pass_ledger: MinaBaseLedgerHash0StableV1(value.first_pass_ledger.into()).into(),
            second_pass_ledger: MinaBaseLedgerHash0StableV1(value.second_pass_ledger.into()).into(),
            pending_coinbase_stack: (&value.pending_coinbase_stack).into(),
            local_state: (&value.local_state).into(),
        }
    }
}

impl From<&LocalState> for MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 {
    fn from(value: &LocalState) -> Self {
        Self {
            stack_frame: MinaBaseStackFrameStableV1(value.stack_frame.into()),
            call_stack: MinaBaseCallStackDigestStableV1(value.call_stack.into()),
            transaction_commitment: value.transaction_commitment.into(),
            full_transaction_commitment: value.full_transaction_commitment.into(),
            excess: SignedAmount {
                magnitude: (&value.excess.magnitude).into(),
                sgn: (&value.excess.sgn).into(),
            },
            supply_increase: SignedAmount {
                magnitude: (&value.supply_increase.magnitude).into(),
                sgn: (&value.supply_increase.sgn).into(),
            },
            ledger: {
                let hash = MinaBaseLedgerHash0StableV1(value.ledger.into());
                hash.into()
            },
            success: value.success,
            account_update_index: UnsignedExtendedUInt32StableV1(
                value.account_update_index.as_u32().into(),
            ),
            failure_status_tbl: MinaBaseTransactionStatusFailureCollectionStableV1(
                value
                    .failure_status_tbl
                    .iter()
                    .map(|s| s.iter().map(|s| s.into()).collect())
                    .collect(),
            ),
            will_succeed: value.will_succeed,
        }
    }
}

impl From<&Statement<()>> for MinaStateBlockchainStateValueStableV2LedgerProofStatement {
    fn from(value: &Statement<()>) -> Self {
        Self {
            source: (&value.source).into(),
            target: (&value.target).into(),
            supply_increase: (&value.supply_increase).into(),
            fee_excess: (&value.fee_excess).into(),
            sok_digest: (),
            connecting_ledger_left: MinaBaseLedgerHash0StableV1(
                (&value.connecting_ledger_left).into(),
            )
            .into(),
            connecting_ledger_right: MinaBaseLedgerHash0StableV1(
                (&value.connecting_ledger_right).into(),
            )
            .into(),
        }
    }
}

impl From<&Statement<()>> for MinaStateSnarkedLedgerStateStableV2 {
    fn from(value: &Statement<()>) -> Self {
        Self(value.into())
    }
}

impl From<&transaction_logic::Coinbase> for MinaBaseCoinbaseStableV1 {
    fn from(value: &transaction_logic::Coinbase) -> Self {
        Self {
            receiver: (&value.receiver).into(),
            amount: value.amount.into(),
            fee_transfer: value.fee_transfer.as_ref().map(|ft| {
                MinaBaseCoinbaseFeeTransferStableV1 {
                    receiver_pk: (&ft.receiver_pk).into(),
                    fee: (&ft.fee).into(),
                }
            }),
        }
    }
}

impl TryFrom<&MinaBaseCoinbaseStableV1> for transaction_logic::Coinbase {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaBaseCoinbaseStableV1) -> Result<Self, Self::Error> {
        Ok(Self {
            receiver: (&value.receiver).try_into()?,
            amount: value.amount.clone().into(),
            fee_transfer: match value.fee_transfer.as_ref() {
                Some(ft) => Some(CoinbaseFeeTransfer {
                    receiver_pk: (&ft.receiver_pk).try_into()?,
                    fee: (&ft.fee).into(),
                }),
                None => None,
            },
        })
    }
}

pub fn to_ledger_hash(value: &Fp) -> mina_p2p_messages::v2::LedgerHash {
    let hash = MinaBaseLedgerHash0StableV1(value.into());
    hash.into()
}

pub fn to_pending_coinbase_hash(value: &Fp) -> mina_p2p_messages::v2::PendingCoinbaseHash {
    let hash = MinaBasePendingCoinbaseHashVersionedStableV1(
        MinaBasePendingCoinbaseHashBuilderStableV1(value.into()),
    );
    hash.into()
}

impl From<&AvailableJob> for AvailableJobMessage {
    fn from(value: &AvailableJob) -> Self {
        match value {
            AvailableJob::Base(v) => AvailableJobMessage::Base(v.as_ref().into()),
            AvailableJob::Merge { left, right } => AvailableJobMessage::Merge {
                left: left.as_ref().into(),
                right: right.as_ref().into(),
            },
        }
    }
}

impl From<&TransactionWithWitness> for TransactionSnarkScanStateTransactionWithWitnessStableV2 {
    fn from(value: &TransactionWithWitness) -> Self {
        use super::scan_state::transaction_snark::InitStack;
        use mina_p2p_messages::v2::MinaStateSnarkedLedgerStatePendingCoinbaseStackStateInitStackStableV1::{Base, Merge};

        Self {
            transaction_with_info: MinaTransactionLogicTransactionAppliedStableV2 {
                previous_hash: {
                    let hash = MinaBaseLedgerHash0StableV1(
                        value.transaction_with_info.previous_hash.into(),
                    );
                    hash.into()
                },
                varying: match &value.transaction_with_info.varying {
                    transaction_applied::Varying::Command(
                        transaction_applied::CommandApplied::SignedCommand(cmd),
                    ) => {
                        MinaTransactionLogicTransactionAppliedVaryingStableV2::Command(
                            MinaTransactionLogicTransactionAppliedCommandAppliedStableV2::SignedCommand(
                                MinaTransactionLogicTransactionAppliedSignedCommandAppliedStableV2 {
                                    common: MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2 {
                                        user_command: MinaTransactionLogicTransactionAppliedSignedCommandAppliedCommonStableV2UserCommand {
                                            data: (&cmd.common.user_command.data).into(),
                                            status: (&cmd.common.user_command.status).into(),
                                        },
                                    },
                                    body: match &cmd.body {
                                        transaction_applied::signed_command_applied::Body::Payments { new_accounts } =>
                                            MinaTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV2::Payment {
                                            new_accounts: new_accounts.iter().cloned().map(Into::into).collect(),
                                        },
                                        transaction_applied::signed_command_applied::Body::StakeDelegation { previous_delegate } =>
                                            MinaTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV2::StakeDelegation {
                                            previous_delegate: previous_delegate.as_ref().map(Into::into)
                                        },
                                        transaction_applied::signed_command_applied::Body::Failed =>
                                            MinaTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV2::Failed,
                                    },
                                }))
                    }
                    transaction_applied::Varying::Command(
                        transaction_applied::CommandApplied::ZkappCommand(cmd),
                    ) =>
                        MinaTransactionLogicTransactionAppliedVaryingStableV2::Command(
                            MinaTransactionLogicTransactionAppliedCommandAppliedStableV2::ZkappCommand(
                                MinaTransactionLogicTransactionAppliedZkappCommandAppliedStableV1 {
                                accounts: cmd.accounts.iter().map(|(id, account_opt)| {
                                    let id: MinaBaseAccountIdStableV2 = id.clone().into();
                                    let account_opt = account_opt.as_ref().map(|acc| (&**acc).into());
                                    (id, account_opt)
                                }).collect(),
                                command: MinaTransactionLogicTransactionAppliedZkappCommandAppliedStableV1Command {
                                    data: (&cmd.command.data).into(),
                                    status: (&cmd.command.status).into(),
                                },
                                new_accounts: cmd.new_accounts.iter().cloned().map(Into::into).collect(),
                            })
                        ),
                    transaction_applied::Varying::FeeTransfer(ft) =>
                        MinaTransactionLogicTransactionAppliedVaryingStableV2::FeeTransfer(
                            MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2 {
                                fee_transfer: MinaTransactionLogicTransactionAppliedFeeTransferAppliedStableV2FeeTransfer {
                                    data: (&ft.fee_transfer.data).into(),
                                    status: (&ft.fee_transfer.status).into(),
                                },
                                new_accounts: ft.new_accounts.iter().cloned().map(Into::into).collect(),
                                burned_tokens: ft.burned_tokens.into(),
                            }),
                    transaction_applied::Varying::Coinbase(cb) =>
                        MinaTransactionLogicTransactionAppliedVaryingStableV2::Coinbase(
                            MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2 {
                                coinbase: MinaTransactionLogicTransactionAppliedCoinbaseAppliedStableV2Coinbase {
                                    data: (&cb.coinbase.data).into(),
                                    status: (&cb.coinbase.status).into(),
                                },
                                new_accounts: cb.new_accounts.iter().cloned().map(Into::into).collect(),
                                burned_tokens: cb.burned_tokens.into(),
                            }
                        ),
                },
            },
            state_hash: {
                let (state, body) = &value.state_hash;
                let state = DataHashLibStateHashStableV1(state.into());

                (state.into(), MinaBaseStateBodyHashStableV1(body.into()).into())
            },
            statement: (&value.statement).into(),
            init_stack: match &value.init_stack {
                InitStack::Base(base) => Base(MinaBasePendingCoinbaseStackVersionedStableV1 {
                    data: MinaBasePendingCoinbaseCoinbaseStackStableV1(base.data.0.into()).into(),
                    state: MinaBasePendingCoinbaseStateStackStableV1 {
                        init: MinaBasePendingCoinbaseStackHashStableV1(base.state.init.into()).into(),
                        curr: MinaBasePendingCoinbaseStackHashStableV1(base.state.curr.into()).into(),
                    },
                }),
                InitStack::Merge => Merge,
            },
            first_pass_ledger_witness: (&value.first_pass_ledger_witness).into(),
            second_pass_ledger_witness: (&value.second_pass_ledger_witness).into(),
            block_global_slot: (&value.block_global_slot).into(),
        }
    }
}

impl binprot::BinProtWrite for TransactionWithWitness {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let p2p: TransactionSnarkScanStateTransactionWithWitnessStableV2 = self.into();
        p2p.binprot_write(w)
    }
}

impl TryFrom<&TransactionSnarkStableV2> for TransactionSnark<SokDigest> {
    type Error = InvalidBigInt;

    fn try_from(value: &TransactionSnarkStableV2) -> Result<Self, Self::Error> {
        Ok(Self {
            statement: (&value.statement).try_into()?,
            proof: Arc::new(value.proof.clone()),
        })
    }
}

impl From<&TransactionSnark<SokDigest>> for TransactionSnarkStableV2 {
    fn from(value: &TransactionSnark<SokDigest>) -> Self {
        Self {
            statement: (&value.statement).into(),
            proof: (*value.proof).clone(),
        }
    }
}

impl TryFrom<&LedgerProofProdStableV2> for LedgerProof {
    type Error = InvalidBigInt;

    fn try_from(value: &LedgerProofProdStableV2) -> Result<Self, Self::Error> {
        Ok(Self((&value.0).try_into()?))
    }
}

impl From<&LedgerProof> for LedgerProofProdStableV2 {
    fn from(value: &LedgerProof) -> Self {
        Self((&value.0).into())
    }
}

// impl binprot::BinProtWrite for LedgerProof {
//     fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
//         let p2p: LedgerProofProdStableV2 = self.into();
//         p2p.binprot_write(w)
//     }
// }

impl TryFrom<&MinaBaseSokMessageStableV1> for SokMessage {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaBaseSokMessageStableV1) -> Result<Self, Self::Error> {
        Ok(Self {
            fee: (&value.fee).into(),
            prover: (&value.prover).try_into()?,
        })
    }
}

impl From<&SokMessage> for MinaBaseSokMessageStableV1 {
    fn from(value: &SokMessage) -> Self {
        Self {
            fee: (&value.fee).into(),
            prover: (&value.prover).into(),
        }
    }
}

impl From<&LedgerProofWithSokMessage>
    for TransactionSnarkScanStateLedgerProofWithSokMessageStableV2
{
    fn from(value: &LedgerProofWithSokMessage) -> Self {
        Self((&value.proof).into(), (&value.sok_message).into())
    }
}

impl TryFrom<&TransactionSnarkScanStateLedgerProofWithSokMessageStableV2>
    for LedgerProofWithSokMessage
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
    ) -> Result<Self, Self::Error> {
        let TransactionSnarkScanStateLedgerProofWithSokMessageStableV2(proof, msg) = value;

        Ok(Self {
            proof: proof.try_into()?,
            sok_message: msg.try_into()?,
        })
    }
}

impl binprot::BinProtWrite for LedgerProofWithSokMessage {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let p2p: TransactionSnarkScanStateLedgerProofWithSokMessageStableV2 = self.into();
        p2p.binprot_write(w)
    }
}

impl TryFrom<MinaBaseUserCommandStableV2> for transaction_logic::valid::UserCommand {
    type Error = InvalidBigInt;

    fn try_from(value: MinaBaseUserCommandStableV2) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl TryFrom<&MinaBaseUserCommandStableV2> for transaction_logic::valid::UserCommand {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaBaseUserCommandStableV2) -> Result<Self, Self::Error> {
        Ok(match value {
            MinaBaseUserCommandStableV2::ZkappCommand(cmd) => {
                Self::ZkAppCommand(Box::new(zkapp_command::valid::ZkAppCommand {
                    zkapp_command: cmd.try_into()?,
                }))
            }
            MinaBaseUserCommandStableV2::SignedCommand(cmd) => {
                Self::SignedCommand(Box::new(cmd.try_into()?))
            }
        })
    }
}

impl From<transaction_logic::valid::UserCommand> for MinaBaseUserCommandStableV2 {
    fn from(value: transaction_logic::valid::UserCommand) -> Self {
        match value {
            transaction_logic::valid::UserCommand::SignedCommand(cmd) => {
                MinaBaseUserCommandStableV2::SignedCommand((&*cmd).into())
            }
            transaction_logic::valid::UserCommand::ZkAppCommand(cmd) => {
                let zkapp_command::valid::ZkAppCommand { zkapp_command } = &*cmd;
                MinaBaseUserCommandStableV2::ZkappCommand(zkapp_command.into())
            }
        }
    }
}

impl From<&ParallelScanWeightStableV1> for super::parallel_scan::Weight {
    fn from(value: &ParallelScanWeightStableV1) -> Self {
        let ParallelScanWeightStableV1 { base, merge } = value;

        Self {
            base: base.as_u64(),
            merge: merge.as_u64(),
        }
    }
}

// Tuples are foreign types, we cannot use `From`
fn from_two_weights(
    value: &(ParallelScanWeightStableV1, ParallelScanWeightStableV1),
) -> (super::parallel_scan::Weight, super::parallel_scan::Weight) {
    let (first, second) = value;
    (first.into(), second.into())
}

impl From<&ParallelScanSequenceNumberStableV1> for SequenceNumber {
    fn from(value: &ParallelScanSequenceNumberStableV1) -> Self {
        SequenceNumber::new(value.as_u64())
    }
}

impl From<&ParallelScanJobStatusStableV1> for JobStatus {
    fn from(value: &ParallelScanJobStatusStableV1) -> Self {
        match value {
            ParallelScanJobStatusStableV1::Todo => Self::Todo,
            ParallelScanJobStatusStableV1::Done => Self::Done,
        }
    }
}

impl TryFrom<&TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1>
    for super::parallel_scan::merge::Job<Arc<LedgerProofWithSokMessage>>
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1::Empty => Self::Empty,
            TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1::Part(proof) => {
                Self::Part(Arc::new((&**proof).try_into()?))
            }
            TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1::Full(record) => {
                let TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1Full {
                    left,
                    right,
                    seq_no,
                    status,
                } = &**record;

                Self::Full(super::parallel_scan::merge::Record {
                    left: Arc::new(left.try_into()?),
                    right: Arc::new(right.try_into()?),
                    seq_no: seq_no.into(),
                    state: status.into(),
                })
            }
        })
    }
}

impl TryFrom<&TransactionSnarkScanStateStableV2ScanStateTreesABaseT1>
    for super::parallel_scan::base::Job<Arc<TransactionWithWitness>>
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &TransactionSnarkScanStateStableV2ScanStateTreesABaseT1,
    ) -> Result<Self, Self::Error> {
        Ok(match value {
            TransactionSnarkScanStateStableV2ScanStateTreesABaseT1::Empty => Self::Empty,
            TransactionSnarkScanStateStableV2ScanStateTreesABaseT1::Full(record) => {
                let TransactionSnarkScanStateStableV2ScanStateTreesABaseT1Full {
                    job,
                    seq_no,
                    status,
                } = &**record;

                Self::Full(super::parallel_scan::base::Record {
                    job: Arc::new(job.try_into()?),
                    seq_no: seq_no.into(),
                    state: status.into(),
                })
            }
        })
    }
}

impl TryFrom<&TransactionSnarkScanStateStableV2> for ScanState {
    type Error = InvalidBigInt;

    fn try_from(value: &TransactionSnarkScanStateStableV2) -> Result<Self, Self::Error> {
        let TransactionSnarkScanStateStableV2 {
            scan_state,
            previous_incomplete_zkapp_updates,
        } = value;

        Ok(Self {
            scan_state: {
                let TransactionSnarkScanStateStableV2ScanState {
                    trees,
                    acc,
                    curr_job_seq_no,
                    max_base_jobs,
                    delay,
                } = scan_state;

                ParallelScan::<Arc<TransactionWithWitness>, Arc<LedgerProofWithSokMessage>> {
                    trees: {
                        use mina_p2p_messages::v2::TransactionSnarkScanStateStableV2ScanStateTreesA::{Leaf, Node};
                        use super::parallel_scan::Weight;

                        let (first, rest) = trees;

                        std::iter::once(first)
                            .chain(rest)
                            .map(|tree| {
                                let mut rust_tree = super::parallel_scan::Tree {
                                    values: Vec::with_capacity(255),
                                };

                                let mut current = tree;

                                // We iterator on the first "depths", those are nodes only
                                while let Node {
                                    depth,
                                    value,
                                    sub_tree,
                                } = current
                                {
                                    for (weights, job) in value.iter() {
                                        let weight: (Weight, Weight) = from_two_weights(weights);
                                        let job: super::parallel_scan::merge::Job<
                                            Arc<LedgerProofWithSokMessage>,
                                        > = job.try_into()?;

                                        let merge =
                                            super::parallel_scan::merge::Merge { weight, job };

                                        rust_tree
                                            .values
                                            .push(super::parallel_scan::Value::Node(merge));
                                    }

                                    current = sub_tree;
                                }

                                // Last depth is all leaves
                                let Leaf(leaves) = current else {
                                    panic!("Invalid tree")
                                };

                                for (weight, job) in leaves {
                                    let weight: Weight = weight.into();
                                    let job: super::parallel_scan::base::Job<
                                        Arc<TransactionWithWitness>,
                                    > = job.try_into()?;

                                    let base = super::parallel_scan::base::Base { weight, job };

                                    rust_tree
                                        .values
                                        .push(super::parallel_scan::Value::Leaf(base))
                                }

                                Ok(rust_tree)
                            })
                            .collect::<Result<_, _>>()?
                    },
                    acc: match acc.as_ref() {
                        Some((proof, txns)) => Some((
                            Arc::new(proof.try_into()?),
                            txns.iter()
                                .map(|t| Ok(Arc::new(t.try_into()?)))
                                .collect::<Result<_, _>>()?,
                        )),
                        None => None,
                    },
                    curr_job_seq_no: { SequenceNumber::new(curr_job_seq_no.as_u64()) },
                    max_base_jobs: max_base_jobs.as_u64(),
                    delay: delay.as_u64(),
                }
            },
            previous_incomplete_zkapp_updates: {
                let (txns, continue_next) = previous_incomplete_zkapp_updates;

                let continue_next = match continue_next {
                    Border_block_continued_in_the_next_tree(continue_next) => {
                        BorderBlockContinuedInTheNextTree(*continue_next)
                    }
                };

                (
                    txns.iter()
                        .map(
                            |t: &TransactionSnarkScanStateTransactionWithWitnessStableV2| {
                                Ok(Arc::new(t.try_into()?))
                            },
                        )
                        .collect::<Result<_, _>>()?,
                    continue_next,
                )
            },
        })
    }
}

impl From<&parallel_scan::Weight> for ParallelScanWeightStableV1 {
    fn from(value: &parallel_scan::Weight) -> Self {
        let parallel_scan::Weight { base, merge } = value;

        Self {
            base: (*base).into(),
            merge: (*merge).into(),
        }
    }
}

impl From<&SequenceNumber> for ParallelScanSequenceNumberStableV1 {
    fn from(value: &SequenceNumber) -> Self {
        ParallelScanSequenceNumberStableV1(value.as_u64().into())
    }
}

impl From<&JobStatus> for ParallelScanJobStatusStableV1 {
    fn from(value: &JobStatus) -> Self {
        match value {
            JobStatus::Todo => Self::Todo,
            JobStatus::Done => Self::Done,
        }
    }
}

impl From<&super::parallel_scan::base::Job<Arc<TransactionWithWitness>>>
    for TransactionSnarkScanStateStableV2ScanStateTreesABaseT1
{
    fn from(value: &super::parallel_scan::base::Job<Arc<TransactionWithWitness>>) -> Self {
        match value {
            parallel_scan::base::Job::Empty => Self::Empty,
            parallel_scan::base::Job::Full(record) => {
                let parallel_scan::base::Record::<Arc<TransactionWithWitness>> {
                    job,
                    seq_no,
                    state,
                } = record;

                Self::Full(Box::new(
                    TransactionSnarkScanStateStableV2ScanStateTreesABaseT1Full {
                        job: job.as_ref().into(),
                        seq_no: seq_no.into(),
                        status: state.into(),
                    },
                ))
            }
        }
    }
}

impl From<&parallel_scan::merge::Job<Arc<LedgerProofWithSokMessage>>>
    for TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1
{
    fn from(value: &parallel_scan::merge::Job<Arc<LedgerProofWithSokMessage>>) -> Self {
        match value {
            parallel_scan::merge::Job::Empty => Self::Empty,
            parallel_scan::merge::Job::Part(part) => Self::Part(Box::new(part.as_ref().into())),
            parallel_scan::merge::Job::Full(record) => {
                let parallel_scan::merge::Record::<Arc<LedgerProofWithSokMessage>> {
                    left,
                    right,
                    seq_no,
                    state,
                } = record;

                Self::Full(Box::new(
                    TransactionSnarkScanStateStableV2ScanStateTreesAMergeT1Full {
                        left: left.as_ref().into(),
                        right: right.as_ref().into(),
                        seq_no: seq_no.into(),
                        status: state.into(),
                    },
                ))
            }
        }
    }
}

impl From<&&parallel_scan::base::Base<Arc<TransactionWithWitness>>>
    for (
        ParallelScanWeightStableV1,
        TransactionSnarkScanStateStableV2ScanStateTreesABaseT1,
    )
{
    fn from(value: &&parallel_scan::base::Base<Arc<TransactionWithWitness>>) -> Self {
        let parallel_scan::base::Base::<Arc<TransactionWithWitness>> { weight, job } = value;

        (weight.into(), job.into())
    }
}

impl From<&&parallel_scan::merge::Merge<Arc<LedgerProofWithSokMessage>>>
    for TransactionSnarkScanStateStableV2TreesAMerge
{
    fn from(value: &&parallel_scan::merge::Merge<Arc<LedgerProofWithSokMessage>>) -> Self {
        let parallel_scan::merge::Merge::<Arc<LedgerProofWithSokMessage>> { weight, job } = value;

        let (w1, w2) = weight;
        ((w1.into(), w2.into()), job.into())
    }
}

impl From<&ScanState> for TransactionSnarkScanStateStableV2 {
    fn from(value: &ScanState) -> Self {
        let ScanState {
            scan_state,
            previous_incomplete_zkapp_updates,
        } = value;

        Self {
            scan_state: {
                let ParallelScan {
                    trees,
                    acc,
                    curr_job_seq_no,
                    max_base_jobs,
                    delay,
                } = scan_state;

                TransactionSnarkScanStateStableV2ScanState {
                    trees: {
                        use mina_p2p_messages::v2::TransactionSnarkScanStateStableV2ScanStateTreesA::{Leaf, Node};

                        let mut trees: Vec<_> = trees
                            .iter()
                            .map(|tree| {
                                let values_by_depth = tree.values_by_depth();

                                let mut previous = None;

                                // rev to start from leaves
                                for (depth, values) in values_by_depth.iter().rev() {
                                    match values {
                                        crate::scan_state::parallel_scan::Value::Leaf(leaves) => {
                                            previous =
                                                Some(Leaf(leaves.iter().map(Into::into).collect()));
                                        }
                                        crate::scan_state::parallel_scan::Value::Node(nodes) => {
                                            let depth: u32 = (*depth).try_into().unwrap();

                                            previous = Some(Node {
                                                depth: depth.into(),
                                                value: nodes.iter().map(Into::into).collect(),
                                                sub_tree: Box::new(previous.take().unwrap()),
                                            });
                                        }
                                    };
                                }

                                previous.take().unwrap()
                            })
                            .collect();

                        let first = trees.remove(0);
                        let rest = trees;

                        (first, rest.into_iter().collect())
                    },
                    acc: acc.as_ref().map(|(proof, txns)| {
                        (
                            proof.as_ref().into(),
                            txns.iter().map(|t| t.as_ref().into()).collect(),
                        )
                    }),
                    curr_job_seq_no: curr_job_seq_no.as_u64().into(),
                    max_base_jobs: max_base_jobs.into(),
                    delay: delay.into(),
                }
            },
            previous_incomplete_zkapp_updates: {
                let (txns, continue_next) = previous_incomplete_zkapp_updates;

                let continue_next = match continue_next {
                    BorderBlockContinuedInTheNextTree(continue_next) => {
                        Border_block_continued_in_the_next_tree(*continue_next)
                    }
                };

                (
                    txns.iter().map(|t| t.as_ref().into()).collect(),
                    continue_next,
                )
            },
        }
    }
}

impl From<&MinaBasePendingCoinbaseStackIdStableV1> for pending_coinbase::StackId {
    fn from(value: &MinaBasePendingCoinbaseStackIdStableV1) -> Self {
        Self::new(value.as_u64())
    }
}

impl From<&pending_coinbase::StackId> for MinaBasePendingCoinbaseStackIdStableV1 {
    fn from(value: &pending_coinbase::StackId) -> Self {
        Self(value.as_u64().into())
    }
}

impl TryFrom<&MinaBasePendingCoinbaseStableV2> for PendingCoinbase {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaBasePendingCoinbaseStableV2) -> Result<Self, Self::Error> {
        let MinaBasePendingCoinbaseStableV2 {
            tree,
            pos_list,
            new_pos,
        } = value;

        Ok(Self {
            tree: {
                // NOTE: Same implementation than with `SparseLedger`

                use MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree::{Account, Hash, Node};

                fn build_matrix(
                    matrix: &mut HashesMatrix,
                    addr: Address,
                    node: &MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree,
                    values: &mut Vec<Stack>,
                ) -> Result<(), InvalidBigInt> {
                    match node {
                        Account(stack) => {
                            let stack: Stack = stack.try_into()?;
                            matrix.set(&addr, <StackHasher as pending_coinbase::merkle_tree::TreeHasher::<Stack>>::hash_value(&stack));
                            values.push(stack);
                        }
                        Hash(hash) => {
                            matrix.set(&addr, hash.to_field()?);
                        }
                        Node(hash, left, right) => {
                            matrix.set(&addr, hash.to_field()?);
                            build_matrix(matrix, addr.child_left(), left, values)?;
                            build_matrix(matrix, addr.child_right(), right, values)?;
                        }
                    }
                    Ok(())
                }

                let MinaBasePendingCoinbaseMerkleTreeVersionedStableV2 {
                    indexes,
                    depth,
                    tree,
                } = tree;

                let depth = depth.as_u64() as usize;
                let mut our_index = std::collections::HashMap::with_capacity(indexes.len());
                // let mut index_list = std::collections::VecDeque::with_capacity(indexes.len());
                let mut hashes_matrix = HashesMatrix::new(depth);
                let mut values = Vec::new();

                for (stack_id, stack_index) in indexes.iter() {
                    let stack_id: pending_coinbase::StackId = stack_id.into();
                    let stack_index = crate::AccountIndex::from(stack_index.as_u64() as usize);

                    let addr = Address::from_index(stack_index, depth);

                    our_index.insert(stack_id, addr);
                    // index_list.push_back(stack_id);
                }

                build_matrix(&mut hashes_matrix, Address::root(), tree, &mut values)?;

                pending_coinbase::merkle_tree::MiniMerkleTree {
                    values,
                    indexes: our_index,
                    hashes_matrix,
                    depth,
                    _hasher: std::marker::PhantomData,
                }
            },
            pos_list: pos_list.iter().rev().map(Into::into).collect(),
            new_pos: new_pos.into(),
        })
    }
}

impl From<&super::pending_coinbase::update::Update> for MinaBasePendingCoinbaseUpdateStableV1 {
    fn from(value: &super::pending_coinbase::update::Update) -> Self {
        Self {
            action: (&value.action).into(),
            coinbase_amount: (&value.coinbase_amount).into(),
        }
    }
}

impl From<&MinaBasePendingCoinbaseUpdateStableV1> for super::pending_coinbase::update::Update {
    fn from(value: &MinaBasePendingCoinbaseUpdateStableV1) -> Self {
        Self {
            action: (&value.action).into(),
            coinbase_amount: value.coinbase_amount.clone().into(),
        }
    }
}

impl From<&super::pending_coinbase::update::Action>
    for MinaBasePendingCoinbaseUpdateActionStableV1
{
    fn from(value: &super::pending_coinbase::update::Action) -> Self {
        use super::pending_coinbase::update::Action;
        match value {
            Action::None => Self::UpdateNone,
            Action::One => Self::UpdateOne,
            Action::TwoCoinbaseInFirst => Self::UpdateTwoCoinbaseInFirst,
            Action::TwoCoinbaseInSecond => Self::UpdateTwoCoinbaseInSecond,
        }
    }
}

impl From<&MinaBasePendingCoinbaseUpdateActionStableV1>
    for super::pending_coinbase::update::Action
{
    fn from(value: &MinaBasePendingCoinbaseUpdateActionStableV1) -> Self {
        use MinaBasePendingCoinbaseUpdateActionStableV1::*;
        match value {
            UpdateNone => Self::None,
            UpdateOne => Self::One,
            UpdateTwoCoinbaseInFirst => Self::TwoCoinbaseInFirst,
            UpdateTwoCoinbaseInSecond => Self::TwoCoinbaseInSecond,
        }
    }
}

impl From<&PendingCoinbase> for MinaBasePendingCoinbaseStableV2 {
    fn from(value: &PendingCoinbase) -> Self {
        let PendingCoinbase {
            tree,
            pos_list,
            new_pos,
        } = value;

        // NOTE: Same implementation than with `SparseLedger`

        MinaBasePendingCoinbaseStableV2 {
            tree: {
                let value = tree;
                assert!(value.hashes_matrix.get(&Address::root()).is_some());

                let indexes: Vec<_> = value
                    .indexes
                    .iter()
                    .map(|(id, addr)| {
                        let addr = value.indexes.get(id).unwrap();

                        let index = addr.to_index();
                        let index: mina_p2p_messages::number::UInt64 = index.as_u64().into();

                        let id: MinaBasePendingCoinbaseStackIdStableV1 = id.into();

                        (id, index)
                    })
                    .collect();

                fn build_tree(
                    addr: Address,
                    matrix: &HashesMatrix,
                    ledger_depth: usize,
                    values: &Vec<Stack>,
                ) -> MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree {
                    if addr.length() == ledger_depth {
                        let account_index = addr.to_index().as_u64() as usize;

                        return match values.get(account_index).cloned() {
                            Some(account) => {
                                let account: MinaBasePendingCoinbaseStackVersionedStableV1 =
                                    (&account).into();
                                MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree::Account(
                                    account,
                                )
                            }
                            None => {
                                let hash = matrix.get(&addr).unwrap();
                                MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree::Hash(
                                    to_pending_coinbase_hash(hash),
                                )
                            }
                        };
                    }

                    let child_left = addr.child_left();
                    let child_right = addr.child_right();

                    let is_left = matrix.get(&child_left).is_some();
                    let is_right = matrix.get(&child_right).is_some();

                    if is_left && is_right {
                        let hash = matrix.get(&addr).unwrap();
                        let left_node = build_tree(child_left, matrix, ledger_depth, values);
                        let right_node = build_tree(child_right, matrix, ledger_depth, values);

                        MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree::Node(
                            to_pending_coinbase_hash(hash),
                            Box::new(left_node),
                            Box::new(right_node),
                        )
                    } else {
                        assert!(!is_left && !is_right);
                        let hash = matrix.get(&addr).unwrap();
                        MinaBasePendingCoinbaseMerkleTreeVersionedStableV2Tree::Hash(
                            to_pending_coinbase_hash(hash),
                        )
                    }
                }

                let tree = build_tree(
                    Address::root(),
                    &value.hashes_matrix,
                    value.depth,
                    &value.values,
                );

                let depth: u64 = value.depth.try_into().unwrap();

                MinaBasePendingCoinbaseMerkleTreeVersionedStableV2 {
                    indexes: indexes.into_iter().collect(),
                    depth: depth.into(),
                    tree,
                }
            },
            pos_list: pos_list.iter().rev().map(Into::into).collect(),
            new_pos: new_pos.into(),
        }
    }
}

impl From<&NonStark> for MinaBaseStagedLedgerHashNonSnarkStableV1 {
    fn from(value: &NonStark) -> Self {
        let NonStark {
            ledger_hash,
            aux_hash,
            pending_coinbase_aux,
        } = value;

        Self {
            ledger_hash: MinaBaseLedgerHash0StableV1(ledger_hash.into()).into(),
            aux_hash: mina_p2p_messages::string::ByteString::from(aux_hash.0.as_slice()).into(),
            pending_coinbase_aux: mina_p2p_messages::string::ByteString::from(
                pending_coinbase_aux.0.as_slice(),
            )
            .into(),
        }
    }
}

impl<F: FieldWitness> From<&StagedLedgerHash<F>> for MinaBaseStagedLedgerHashStableV1 {
    fn from(value: &StagedLedgerHash<F>) -> Self {
        let StagedLedgerHash {
            non_snark,
            pending_coinbase_hash,
        } = value;

        Self {
            non_snark: non_snark.into(),
            pending_coinbase_hash: MinaBasePendingCoinbaseHashVersionedStableV1(
                MinaBasePendingCoinbaseHashBuilderStableV1((*pending_coinbase_hash).into()),
            )
            .into(),
        }
    }
}

impl TryFrom<&MinaBaseStagedLedgerHashNonSnarkStableV1> for NonStark {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaBaseStagedLedgerHashNonSnarkStableV1) -> Result<Self, Self::Error> {
        let MinaBaseStagedLedgerHashNonSnarkStableV1 {
            ledger_hash,
            aux_hash,
            pending_coinbase_aux,
        } = value;

        Ok(Self {
            ledger_hash: ledger_hash.inner().to_field()?,
            aux_hash: AuxHash(aux_hash.as_slice().try_into().map_err(|_| InvalidBigInt)?), // TODO: Don't use `InvalidBigInt` here
            pending_coinbase_aux: PendingCoinbaseAux(
                pending_coinbase_aux
                    .as_slice()
                    .try_into()
                    .map_err(|_| InvalidBigInt)?, // TODO: Don't use `InvalidBigInt` here
            ),
        })
    }
}

impl<F: FieldWitness> TryFrom<&MinaBaseStagedLedgerHashStableV1> for StagedLedgerHash<F> {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaBaseStagedLedgerHashStableV1) -> Result<Self, Self::Error> {
        let MinaBaseStagedLedgerHashStableV1 {
            non_snark,
            pending_coinbase_hash,
        } = value;

        Ok(Self {
            non_snark: non_snark.try_into()?,
            pending_coinbase_hash: pending_coinbase_hash.inner().to_field()?,
        })
    }
}

impl TryFrom<&MinaTransactionTransactionStableV2> for Transaction {
    type Error = InvalidBigInt;

    fn try_from(value: &MinaTransactionTransactionStableV2) -> Result<Self, Self::Error> {
        Ok(match value {
            MinaTransactionTransactionStableV2::Command(cmd) => Self::Command(match &**cmd {
                MinaBaseUserCommandStableV2::SignedCommand(cmd) => {
                    UserCommand::SignedCommand(Box::new(cmd.try_into()?))
                }
                MinaBaseUserCommandStableV2::ZkappCommand(cmd) => {
                    UserCommand::ZkAppCommand(Box::new(cmd.try_into()?))
                }
            }),
            MinaTransactionTransactionStableV2::FeeTransfer(ft) => {
                Self::FeeTransfer(ft.try_into()?)
            }
            MinaTransactionTransactionStableV2::Coinbase(cb) => Self::Coinbase(cb.try_into()?),
        })
    }
}

impl TryFrom<&TransactionSnarkWorkTStableV2> for super::scan_state::transaction_snark::work::Work {
    type Error = InvalidBigInt;

    fn try_from(value: &TransactionSnarkWorkTStableV2) -> Result<Self, Self::Error> {
        let TransactionSnarkWorkTStableV2 {
            fee,
            proofs,
            prover,
        } = value;

        Ok(Self {
            fee: fee.into(),
            proofs: proofs.try_into()?,
            prover: prover.try_into()?,
        })
    }
}

impl From<&super::scan_state::transaction_snark::work::Work> for TransactionSnarkWorkTStableV2 {
    fn from(value: &super::scan_state::transaction_snark::work::Work) -> Self {
        let super::scan_state::transaction_snark::work::Work {
            fee,
            proofs,
            prover,
        } = value;

        Self {
            fee: fee.into(),
            proofs: proofs.into(),
            prover: prover.into(),
        }
    }
}

impl TryFrom<&TransactionSnarkWorkTStableV2Proofs>
    for super::scan_state::transaction_snark::OneOrTwo<LedgerProof>
{
    type Error = InvalidBigInt;

    fn try_from(value: &TransactionSnarkWorkTStableV2Proofs) -> Result<Self, Self::Error> {
        use super::scan_state::transaction_snark::OneOrTwo::{One, Two};
        use TransactionSnarkWorkTStableV2Proofs as B;

        Ok(match value {
            B::One(proof) => One(proof.try_into()?),
            B::Two((p1, p2)) => Two((p1.try_into()?, p2.try_into()?)),
        })
    }
}

impl From<&super::scan_state::transaction_snark::OneOrTwo<LedgerProof>>
    for TransactionSnarkWorkTStableV2Proofs
{
    fn from(value: &super::scan_state::transaction_snark::OneOrTwo<LedgerProof>) -> Self {
        use super::scan_state::transaction_snark::OneOrTwo as B;
        use TransactionSnarkWorkTStableV2Proofs::{One, Two};

        match value {
            B::One(proof) => One(proof.into()),
            B::Two((p1, p2)) => Two((p1.into(), p2.into())),
        }
    }
}

impl TryFrom<&StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B>
    for WithStatus<UserCommand>
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B,
    ) -> Result<Self, Self::Error> {
        let StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B { data, status } = value;

        Ok(Self {
            data: data.try_into()?,
            status: status.into(),
        })
    }
}

impl From<&WithStatus<UserCommand>> for StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B {
    fn from(value: &WithStatus<UserCommand>) -> Self {
        let WithStatus { data, status } = value;

        Self {
            data: data.into(),
            status: status.into(),
        }
    }
}

impl From<&WithStatus<transaction_logic::valid::UserCommand>>
    for StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B
{
    fn from(value: &WithStatus<transaction_logic::valid::UserCommand>) -> Self {
        let WithStatus { data, status } = value;

        Self {
            data: (&data.forget_check()).into(),
            status: status.into(),
        }
    }
}

impl TryFrom<&StagedLedgerDiffDiffFtStableV1> for transaction_logic::CoinbaseFeeTransfer {
    type Error = InvalidBigInt;

    fn try_from(value: &StagedLedgerDiffDiffFtStableV1) -> Result<Self, Self::Error> {
        let StagedLedgerDiffDiffFtStableV1(value) = value;
        let MinaBaseCoinbaseFeeTransferStableV1 { receiver_pk, fee } = value;

        Ok(Self {
            receiver_pk: receiver_pk.try_into()?,
            fee: fee.into(),
        })
    }
}

impl From<&transaction_logic::CoinbaseFeeTransfer> for StagedLedgerDiffDiffFtStableV1 {
    fn from(value: &transaction_logic::CoinbaseFeeTransfer) -> Self {
        let transaction_logic::CoinbaseFeeTransfer { receiver_pk, fee } = value;

        Self(MinaBaseCoinbaseFeeTransferStableV1 {
            receiver_pk: receiver_pk.into(),
            fee: fee.into(),
        })
    }
}

impl TryFrom<&StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2Coinbase>
    for crate::staged_ledger::diff::AtMostTwo<transaction_logic::CoinbaseFeeTransfer>
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2Coinbase,
    ) -> Result<Self, Self::Error> {
        use crate::staged_ledger::diff::AtMostTwo::*;
        use StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2Coinbase as B;

        let conv = |v: &Option<StagedLedgerDiffDiffFtStableV1>| -> Result<Option<transaction_logic::CoinbaseFeeTransfer>, _> {
            match v.as_ref() {
                Some(v) => Ok(Some(v.try_into()?)),
                None => Ok(None),
            }
        };

        Ok(match value {
            B::Zero => Zero,
            B::One(one) => One(conv(one)?),
            B::Two(twos) => Two(match twos {
                Some((one, two)) => Some((one.try_into()?, conv(two)?)),
                None => None,
            }),
        })
    }
}

impl From<&crate::staged_ledger::diff::AtMostTwo<transaction_logic::CoinbaseFeeTransfer>>
    for StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2Coinbase
{
    fn from(
        value: &crate::staged_ledger::diff::AtMostTwo<transaction_logic::CoinbaseFeeTransfer>,
    ) -> Self {
        use crate::staged_ledger::diff::AtMostTwo as B;
        use StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2Coinbase::*;

        match value {
            B::Zero => Zero,
            B::One(one) => One(one.as_ref().map(Into::into)),
            B::Two(twos) => Two(twos
                .as_ref()
                .map(|(one, two)| (one.into(), two.as_ref().map(Into::into)))),
        }
    }
}

impl TryFrom<&StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2>
    for crate::staged_ledger::diff::PreDiffWithAtMostTwoCoinbase
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2,
    ) -> Result<Self, Self::Error> {
        let StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2 {
            completed_works,
            commands,
            coinbase,
            internal_command_statuses,
        } = value;

        Ok(Self {
            completed_works: completed_works
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            commands: commands
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            coinbase: coinbase.try_into()?,
            internal_command_statuses: internal_command_statuses.iter().map(Into::into).collect(),
        })
    }
}

impl From<&crate::staged_ledger::diff::PreDiffWithAtMostTwoCoinbase>
    for StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2
{
    fn from(value: &crate::staged_ledger::diff::PreDiffWithAtMostTwoCoinbase) -> Self {
        let crate::staged_ledger::diff::PreDiffWithAtMostTwoCoinbase {
            completed_works,
            commands,
            coinbase,
            internal_command_statuses,
        } = value;

        Self {
            completed_works: completed_works.iter().map(Into::into).collect(),
            commands: commands.iter().map(Into::into).collect(),
            coinbase: coinbase.into(),
            internal_command_statuses: internal_command_statuses.iter().map(Into::into).collect(),
        }
    }
}

impl
    From<
        &crate::staged_ledger::diff::PreDiffTwo<
            crate::scan_state::scan_state::transaction_snark::work::Work,
            WithStatus<transaction_logic::valid::UserCommand>,
        >,
    > for StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2
{
    fn from(
        value: &crate::staged_ledger::diff::PreDiffTwo<
            crate::scan_state::scan_state::transaction_snark::work::Work,
            WithStatus<transaction_logic::valid::UserCommand>,
        >,
    ) -> Self {
        let crate::staged_ledger::diff::PreDiffTwo {
            completed_works,
            commands,
            coinbase,
            internal_command_statuses,
        } = value;

        Self {
            completed_works: completed_works.iter().map(Into::into).collect(),
            commands: commands.iter().map(Into::into).collect(),
            coinbase: coinbase.into(),
            internal_command_statuses: internal_command_statuses.iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<&StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Coinbase>
    for crate::staged_ledger::diff::AtMostOne<transaction_logic::CoinbaseFeeTransfer>
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Coinbase,
    ) -> Result<Self, Self::Error> {
        use crate::staged_ledger::diff::AtMostOne::*;
        use StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Coinbase as B;

        Ok(match value {
            B::Zero => Zero,
            B::One(one) => One(match one.as_ref() {
                Some(one) => Some(one.try_into()?),
                None => None,
            }),
        })
    }
}

impl From<&crate::staged_ledger::diff::AtMostOne<transaction_logic::CoinbaseFeeTransfer>>
    for StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Coinbase
{
    fn from(
        value: &crate::staged_ledger::diff::AtMostOne<transaction_logic::CoinbaseFeeTransfer>,
    ) -> Self {
        use crate::staged_ledger::diff::AtMostOne as B;
        use StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Coinbase::*;

        match value {
            B::Zero => Zero,
            B::One(one) => One(one.as_ref().map(Into::into)),
        }
    }
}

impl TryFrom<&StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2>
    for crate::staged_ledger::diff::PreDiffWithAtMostOneCoinbase
{
    type Error = InvalidBigInt;

    fn try_from(
        value: &StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2,
    ) -> Result<Self, Self::Error> {
        let StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2 {
            completed_works,
            commands,
            coinbase,
            internal_command_statuses,
        } = value;

        Ok(Self {
            completed_works: completed_works
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            commands: commands
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            coinbase: coinbase.try_into()?,
            internal_command_statuses: internal_command_statuses.iter().map(Into::into).collect(),
        })
    }
}

impl From<&crate::staged_ledger::diff::PreDiffWithAtMostOneCoinbase>
    for StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2
{
    fn from(value: &crate::staged_ledger::diff::PreDiffWithAtMostOneCoinbase) -> Self {
        let crate::staged_ledger::diff::PreDiffOne {
            completed_works,
            commands,
            coinbase,
            internal_command_statuses,
        } = value;

        Self {
            completed_works: completed_works.iter().map(Into::into).collect(),
            commands: commands.iter().map(Into::into).collect(),
            coinbase: coinbase.into(),
            internal_command_statuses: internal_command_statuses.iter().map(Into::into).collect(),
        }
    }
}

impl
    From<
        &crate::staged_ledger::diff::PreDiffOne<
            crate::scan_state::scan_state::transaction_snark::work::Work,
            WithStatus<transaction_logic::valid::UserCommand>,
        >,
    > for StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2
{
    fn from(
        value: &crate::staged_ledger::diff::PreDiffOne<
            crate::scan_state::scan_state::transaction_snark::work::Work,
            WithStatus<transaction_logic::valid::UserCommand>,
        >,
    ) -> Self {
        let crate::staged_ledger::diff::PreDiffOne {
            completed_works,
            commands,
            coinbase,
            internal_command_statuses,
        } = value;

        Self {
            completed_works: completed_works.iter().map(Into::into).collect(),
            commands: commands.iter().map(Into::into).collect(),
            coinbase: coinbase.into(),
            internal_command_statuses: internal_command_statuses.iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<&StagedLedgerDiffDiffStableV2> for crate::staged_ledger::diff::Diff {
    type Error = InvalidBigInt;

    fn try_from(value: &StagedLedgerDiffDiffStableV2) -> Result<Self, Self::Error> {
        let StagedLedgerDiffDiffStableV2 { diff } = value;
        let StagedLedgerDiffDiffDiffStableV2(first, second) = diff;

        Ok(Self {
            diff: (
                first.try_into()?,
                match second.as_ref() {
                    Some(second) => Some(second.try_into()?),
                    None => None,
                },
            ),
        })
    }
}

impl From<&crate::staged_ledger::diff::with_valid_signatures_and_proofs::Diff>
    for StagedLedgerDiffDiffStableV2
{
    fn from(value: &crate::staged_ledger::diff::with_valid_signatures_and_proofs::Diff) -> Self {
        let (first, second) = &value.diff;

        StagedLedgerDiffDiffStableV2 {
            diff: StagedLedgerDiffDiffDiffStableV2(first.into(), second.as_ref().map(Into::into)),
        }
    }
}

impl From<&MinaNumbersGlobalSlotSinceGenesisMStableV1> for Slot {
    fn from(value: &MinaNumbersGlobalSlotSinceGenesisMStableV1) -> Self {
        let MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(slot) = value;
        Self(slot.as_u32())
    }
}

impl From<&MinaNumbersGlobalSlotSinceHardForkMStableV1> for Slot {
    fn from(value: &MinaNumbersGlobalSlotSinceHardForkMStableV1) -> Self {
        let MinaNumbersGlobalSlotSinceHardForkMStableV1::SinceHardFork(slot) = value;
        Self(slot.as_u32())
    }
}

impl From<&MinaNumbersGlobalSlotSpanStableV1> for SlotSpan {
    fn from(value: &MinaNumbersGlobalSlotSpanStableV1) -> Self {
        let MinaNumbersGlobalSlotSpanStableV1::GlobalSlotSpan(slot) = value;
        Self(slot.as_u32())
    }
}

impl From<&Slot> for MinaNumbersGlobalSlotSinceGenesisMStableV1 {
    fn from(value: &Slot) -> Self {
        Self::SinceGenesis(value.as_u32().into())
    }
}

impl From<&Slot> for MinaNumbersGlobalSlotSinceHardForkMStableV1 {
    fn from(value: &Slot) -> Self {
        Self::SinceHardFork(value.as_u32().into())
    }
}

impl From<&SlotSpan> for MinaNumbersGlobalSlotSpanStableV1 {
    fn from(value: &SlotSpan) -> Self {
        Self::GlobalSlotSpan(value.as_u32().into())
    }
}

impl From<&ZkappStatement> for v2::MinaBaseZkappStatementStableV2 {
    fn from(value: &ZkappStatement) -> Self {
        use transaction_logic::zkapp_statement::TransactionCommitment;

        let ZkappStatement {
            account_update: TransactionCommitment(account_update),
            calls: TransactionCommitment(calls),
        } = value;

        v2::MinaBaseZkappStatementStableV2 {
            account_update: account_update.into(),
            calls: calls.into(),
        }
    }
}

impl TryFrom<&v2::MinaBaseZkappStatementStableV2> for ZkappStatement {
    type Error = InvalidBigInt;

    fn try_from(value: &v2::MinaBaseZkappStatementStableV2) -> Result<Self, Self::Error> {
        use transaction_logic::zkapp_statement::TransactionCommitment;

        let v2::MinaBaseZkappStatementStableV2 {
            account_update,
            calls,
        } = value;

        Ok(ZkappStatement {
            account_update: TransactionCommitment(account_update.to_field()?),
            calls: TransactionCommitment(calls.to_field()?),
        })
    }
}
