#![allow(unused_variables, unreachable_code)]

use std::rc::Rc;

use mina_hasher::Fp;
use mina_p2p_messages::{
    pseq::PaddedSeq,
    string::CharString,
    v2::{
        BlockTimeTimeStableV1, CurrencyAmountStableV1, CurrencyBalanceStableV1,
        CurrencyFeeStableV1, DataHashLibStateHashStableV1, EpochSeed, LedgerProofProdStableV2,
        MinaBaseAccountIdDigestStableV1, MinaBaseAccountIdStableV2,
        MinaBaseAccountUpdateBodyEventsStableV1, MinaBaseAccountUpdateBodyFeePayerStableV1,
        MinaBaseAccountUpdateBodyStableV1, MinaBaseAccountUpdateFeePayerStableV1,
        MinaBaseAccountUpdateMayUseTokenStableV1, MinaBaseAccountUpdatePreconditionsStableV1,
        MinaBaseAccountUpdateTStableV1, MinaBaseAccountUpdateUpdateStableV1,
        MinaBaseAccountUpdateUpdateStableV1AppStateA,
        MinaBaseAccountUpdateUpdateTimingInfoStableV1, MinaBaseCallStackDigestStableV1,
        MinaBaseCoinbaseFeeTransferStableV1, MinaBaseCoinbaseStableV1, MinaBaseEpochSeedStableV1,
        MinaBaseFeeExcessStableV1, MinaBaseFeeExcessStableV1Fee, MinaBaseFeeTransferSingleStableV2,
        MinaBaseFeeTransferStableV2, MinaBaseLedgerHash0StableV1, MinaBasePaymentPayloadStableV2,
        MinaBasePendingCoinbaseCoinbaseStackStableV1, MinaBasePendingCoinbaseStackHashStableV1,
        MinaBasePendingCoinbaseStackVersionedStableV1, MinaBasePendingCoinbaseStateStackStableV1,
        MinaBaseReceiptChainHashStableV1, MinaBaseSignedCommandMemoStableV1,
        MinaBaseSignedCommandPayloadBodyStableV2, MinaBaseSignedCommandPayloadCommonStableV2,
        MinaBaseSignedCommandPayloadStableV2, MinaBaseSignedCommandStableV2,
        MinaBaseSokMessageStableV1, MinaBaseStackFrameStableV1, MinaBaseStakeDelegationStableV1,
        MinaBaseStateBodyHashStableV1, MinaBaseTransactionStatusFailureCollectionStableV1,
        MinaBaseTransactionStatusFailureStableV2, MinaBaseTransactionStatusStableV2,
        MinaBaseUserCommandStableV2, MinaBaseZkappAccountZkappUriStableV1,
        MinaBaseZkappCommandTStableV1WireStableV1,
        MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA,
        MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAA,
        MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA,
        MinaBaseZkappPreconditionAccountStableV2, MinaBaseZkappPreconditionAccountStableV2BalanceA,
        MinaBaseZkappPreconditionProtocolStateEpochDataStableV1,
        MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochLedger,
        MinaBaseZkappPreconditionProtocolStateStableV1,
        MinaBaseZkappPreconditionProtocolStateStableV1AmountA,
        MinaBaseZkappPreconditionProtocolStateStableV1Length,
        MinaBaseZkappPreconditionProtocolStateStableV1LengthA,
        MinaStateBlockchainStateValueStableV2LedgerProofStatement,
        MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
        MinaStateSnarkedLedgerStateStableV2, MinaStateSnarkedLedgerStateWithSokStableV2,
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
        MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount, SgnStableV1,
        StateHash, TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
        TransactionSnarkScanStateTransactionWithWitnessStableV2, TransactionSnarkStableV2,
        UnsignedExtendedUInt32StableV1, UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
    },
};
use mina_signer::Signature;

use crate::{
    array_into_with,
    scan_state::{
        currency::BlockTime,
        transaction_logic::{
            signed_command::{PaymentPayload, StakeDelegationPayload},
            zkapp_command::{self, AuthorizationKind, CallForest, MayUseToken},
            WithStatus,
        },
    },
    Account, AccountId, TokenId, VerificationKey, VotingFor,
};

use super::{
    currency::{Amount, Balance, Fee, Index, Length, Nonce, Sgn, Signed, Slot},
    fee_excess::FeeExcess,
    pending_coinbase,
    scan_state::transaction_snark::{
        LedgerProof, LedgerProofWithSokMessage, Registers, SokDigest, SokMessage, Statement,
        TransactionSnark, TransactionWithWitness,
    },
    transaction_logic::{
        self,
        local_state::LocalState,
        signed_command::SignedCommand,
        transaction_applied::{self, TransactionApplied},
        zkapp_command::{
            AccountUpdate, FeePayer, FeePayerBody, SetOrKeep, WithHash, WithStackHash,
        },
        FeeTransfer, Memo, SingleFeeTransfer, TransactionFailure, TransactionStatus,
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
            (value.0 as i64).into(),
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
            (value.0 as i64).into(),
        ))
    }
}

impl From<&MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount>
    for Signed<Amount>
{
    fn from(
        value: &MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount,
    ) -> Self {
        Self {
            magnitude: value.magnitude.clone().into(),
            sgn: value.sgn.clone().into(),
        }
    }
}

impl From<&Signed<Amount>>
    for MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount
{
    fn from(value: &Signed<Amount>) -> Self {
        Self {
            magnitude: value.magnitude.into(),
            sgn: (&value.sgn).into(),
        }
    }
}

impl From<&CurrencyFeeStableV1> for Fee {
    fn from(value: &CurrencyFeeStableV1) -> Self {
        Self(value.as_u64())
    }
}

impl From<&Nonce> for mina_p2p_messages::v2::UnsignedExtendedUInt32StableV1 {
    fn from(value: &Nonce) -> Self {
        Self((value.as_u32() as i32).into())
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
        Self((value.as_u32() as i32).into())
    }
}

impl From<&Length> for mina_p2p_messages::v2::UnsignedExtendedUInt32StableV1 {
    fn from(value: &Length) -> Self {
        Self((value.as_u32() as i32).into())
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

impl From<&MinaBaseFeeExcessStableV1Fee> for Signed<Fee> {
    fn from(value: &MinaBaseFeeExcessStableV1Fee) -> Self {
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
            (value.0 as i64).into(),
        ))
    }
}

impl From<&Signed<Fee>> for MinaBaseFeeExcessStableV1Fee {
    fn from(value: &Signed<Fee>) -> Self {
        Self {
            magnitude: (&value.magnitude).into(),
            sgn: (&value.sgn).into(),
        }
    }
}

impl From<&MinaBaseFeeExcessStableV1> for FeeExcess {
    fn from(value: &MinaBaseFeeExcessStableV1) -> Self {
        Self {
            fee_token_l: (&*value.fee_token_l).into(),
            fee_excess_l: (&value.fee_excess_l).into(),
            fee_token_r: (&*value.fee_token_r).into(),
            fee_excess_r: (&value.fee_excess_r).into(),
        }
    }
}

impl From<&FeeExcess> for MinaBaseFeeExcessStableV1 {
    fn from(value: &FeeExcess) -> Self {
        Self {
            fee_token_l: (&value.fee_token_l).into(),
            fee_excess_l: (&value.fee_excess_l).into(),
            fee_token_r: (&value.fee_token_r).into(),
            fee_excess_r: (&value.fee_excess_r).into(),
        }
    }
}

impl From<&MinaBasePendingCoinbaseStackVersionedStableV1> for pending_coinbase::Stack {
    fn from(value: &MinaBasePendingCoinbaseStackVersionedStableV1) -> Self {
        Self {
            data: pending_coinbase::CoinbaseStack(value.data.0.to_field()),
            state: pending_coinbase::StateStack {
                init: value.state.init.0.to_field(),
                curr: value.state.curr.0.to_field(),
            },
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
            P2P::UpdateNotPermittedSequenceState => Self::UpdateNotPermittedActionState,
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
            P2P::AccountSequenceStatePreconditionUnsatisfied => {
                Self::AccountActionStatePreconditionUnsatisfied
            }
            P2P::AccountAppStatePreconditionUnsatisfied(v) => {
                Self::AccountAppStatePreconditionUnsatisfied(v.as_u32() as i64)
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
            P2P::UpdateNotPermittedActionState => Self::UpdateNotPermittedSequenceState,
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
                Self::AccountSequenceStatePreconditionUnsatisfied
            }
            P2P::AccountAppStatePreconditionUnsatisfied(v) => {
                Self::AccountAppStatePreconditionUnsatisfied((*v as i32).into())
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

impl From<&MinaStateBlockchainStateValueStableV2LedgerProofStatementSource> for Registers {
    fn from(value: &MinaStateBlockchainStateValueStableV2LedgerProofStatementSource) -> Self {
        Self {
            first_pass_ledger: value.first_pass_ledger.to_field(),
            second_pass_ledger: value.second_pass_ledger.to_field(),
            pending_coinbase_stack: (&value.pending_coinbase_stack).into(),
            local_state: LocalState {
                stack_frame: value.local_state.stack_frame.0.to_field(),
                call_stack: value.local_state.call_stack.0.to_field(),
                transaction_commitment: value.local_state.transaction_commitment.to_field(),
                full_transaction_commitment: value
                    .local_state
                    .full_transaction_commitment
                    .to_field(),
                token_id: {
                    let id: MinaBaseAccountIdDigestStableV1 =
                        value.local_state.token_id.clone().into_inner();
                    id.into()
                },
                excess: (&value.local_state.excess).into(),
                supply_increase: (&value.local_state.supply_increase).into(),
                ledger: value.local_state.ledger.0.to_field(),
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
        }
    }
}

impl From<&MinaStateBlockchainStateValueStableV2LedgerProofStatement> for Statement<()> {
    fn from(value: &MinaStateBlockchainStateValueStableV2LedgerProofStatement) -> Self {
        Self {
            source: (&value.source).into(),
            target: (&value.target).into(),
            connecting_ledger_left: value.connecting_ledger_left.to_field(),
            connecting_ledger_right: value.connecting_ledger_right.to_field(),
            supply_increase: (&value.supply_increase).into(),
            fee_excess: (&value.fee_excess).into(),
            sok_digest: (),
        }
    }
}

impl From<&MinaStateSnarkedLedgerStateWithSokStableV2> for Statement<SokDigest> {
    fn from(value: &MinaStateSnarkedLedgerStateWithSokStableV2) -> Self {
        Self {
            source: (&value.source).into(),
            target: (&value.target).into(),
            connecting_ledger_left: value.connecting_ledger_left.to_field(),
            connecting_ledger_right: value.connecting_ledger_right.to_field(),
            supply_increase: (&value.supply_increase).into(),
            fee_excess: (&value.fee_excess).into(),
            sok_digest: SokDigest(value.sok_digest.to_vec()),
        }
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
            sok_digest: MinaBaseZkappAccountZkappUriStableV1(value.sok_digest.as_slice().into()),
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

impl From<&MinaBaseAccountUpdateFeePayerStableV1> for FeePayer {
    fn from(value: &MinaBaseAccountUpdateFeePayerStableV1) -> Self {
        Self {
            body: FeePayerBody {
                public_key: value.body.public_key.clone().into_inner().into(),
                fee: Fee::from_u64(value.body.fee.as_u64()),
                valid_until: value
                    .body
                    .valid_until
                    .as_ref()
                    .map(|until| Slot::from_u32(until.as_u32())),
                nonce: Nonce::from_u32(value.body.nonce.as_u32()),
            },
            authorization: Signature {
                rx: value.authorization.0.to_field(),
                s: value.authorization.1.to_field(),
            },
        }
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
            authorization: (&value.authorization).into(),
        }
    }
}

impl From<&MinaBaseAccountUpdateUpdateTimingInfoStableV1> for zkapp_command::Timing {
    fn from(t: &MinaBaseAccountUpdateUpdateTimingInfoStableV1) -> Self {
        Self {
            initial_minimum_balance: Balance::from_u64(t.initial_minimum_balance.as_u64()),
            cliff_time: Slot::from_u32(t.cliff_time.as_u32()),
            cliff_amount: Amount::from_u64(t.cliff_amount.as_u64()),
            vesting_period: Slot::from_u32(t.vesting_period.as_u32()),
            vesting_increment: Amount::from_u64(t.vesting_increment.as_u64()),
        }
    }
}

impl From<&zkapp_command::Timing> for MinaBaseAccountUpdateUpdateTimingInfoStableV1 {
    fn from(t: &zkapp_command::Timing) -> Self {
        Self {
            initial_minimum_balance: CurrencyBalanceStableV1(t.initial_minimum_balance.into()),
            cliff_time: UnsignedExtendedUInt32StableV1((t.cliff_time.as_u32() as i32).into()),
            cliff_amount: t.cliff_amount.into(),
            vesting_period: UnsignedExtendedUInt32StableV1(
                (t.vesting_period.as_u32() as i32).into(),
            ),
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

impl From<&MinaBaseZkappPreconditionProtocolStateEpochDataStableV1> for zkapp_command::EpochData {
    fn from(value: &MinaBaseZkappPreconditionProtocolStateEpochDataStableV1) -> Self {
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateEpochDataStableV1EpochSeed as Seed;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateEpochDataStableV1StartCheckpoint as Start;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1Amount as MAmount;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash as Hash;
        use zkapp_command::{ClosedInterval, OrIgnore};

        Self {
            ledger: zkapp_command::EpochLedger {
                hash: match &value.ledger.hash {
                    Hash::Check(hash) => OrIgnore::Check(hash.to_field()),
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
                Seed::Check(seed) => OrIgnore::Check(seed.to_field()),
                Seed::Ignore => OrIgnore::Ignore,
            },
            start_checkpoint: match &value.start_checkpoint {
                Start::Check(start) => OrIgnore::Check(start.to_field()),
                Start::Ignore => OrIgnore::Ignore,
            },
            lock_checkpoint: match &value.lock_checkpoint {
                Start::Check(start) => OrIgnore::Check(start.to_field()),
                Start::Ignore => OrIgnore::Ignore,
            },
            epoch_length: (&value.epoch_length).into(),
        }
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

impl From<&MinaBaseAccountUpdatePreconditionsStableV1> for zkapp_command::Preconditions {
    fn from(value: &MinaBaseAccountUpdatePreconditionsStableV1) -> Self {
        use mina_p2p_messages::v2::MinaBaseAccountUpdateAccountPreconditionStableV1 as MAccount;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1Amount as MAmount;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash as Ledger;
        use zkapp_command::AccountPreconditions;
        use zkapp_command::{ClosedInterval, Numeric, OrIgnore};
        use MinaBaseZkappPreconditionProtocolStateStableV1Length as MLength;

        Self {
            network: zkapp_command::ZkAppPreconditions {
                snarked_ledger_hash: match &value.network.snarked_ledger_hash {
                    Ledger::Check(hash) => OrIgnore::Check(hash.to_field()),
                    Ledger::Ignore => OrIgnore::Ignore,
                },
                blockchain_length: (&value.network.blockchain_length).into(),
                min_window_density: (&value.network.min_window_density).into(),
                last_vrf_output: value.network.last_vrf_output,
                total_currency: match &value.network.total_currency {
                    MAmount::Check(amount) => OrIgnore::Check(ClosedInterval {
                        lower: Amount::from_u64(amount.lower.0 .0.as_u64()),
                        upper: Amount::from_u64(amount.upper.0 .0.as_u64()),
                    }),
                    MAmount::Ignore => OrIgnore::Ignore,
                },
                global_slot_since_genesis: match &value.network.global_slot_since_genesis {
                    MLength::Check(length) => Numeric::Check(ClosedInterval {
                        lower: Slot::from_u32(length.lower.0.as_u32()),
                        upper: Slot::from_u32(length.upper.0.as_u32()),
                    }),
                    MLength::Ignore => OrIgnore::Ignore,
                },
                staking_epoch_data: (&value.network.staking_epoch_data).into(),
                next_epoch_data: (&value.network.staking_epoch_data).into(),
            },
            account: match &value.account {
                MAccount::Full(account) => {
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2Balance as MBalance;
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2Delegate as Delegate;
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2ProvedState as Proved;
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash as Receipt;
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2StateA as State;
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1Length as MNonce;

                    let account = &**account;
                    AccountPreconditions::Full(Box::new(zkapp_command::Account {
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
                            Receipt::Check(hash) => OrIgnore::Check(hash.to_field()),
                            Receipt::Ignore => OrIgnore::Ignore,
                        },
                        delegate: match &account.delegate {
                            Delegate::Check(delegate) => {
                                OrIgnore::Check(delegate.clone().into_inner().into())
                            }
                            Delegate::Ignore => OrIgnore::Ignore,
                        },
                        state: array_into_with(&account.state, |s| match s {
                            State::Check(s) => OrIgnore::Check(s.to_field()),
                            State::Ignore => OrIgnore::Ignore,
                        }),
                        action_state: match &account.sequence_state {
                            State::Check(s) => OrIgnore::Check(s.to_field()),
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
                    }))
                }
                MAccount::Nonce(nonce) => {
                    AccountPreconditions::Nonce(Nonce::from_u32(nonce.as_u32()))
                }
                MAccount::Accept => AccountPreconditions::Accept,
            },
            valid_while: match &value.valid_while {
                MLength::Check(valid_while) => OrIgnore::Check(ClosedInterval {
                    lower: Slot::from_u32(valid_while.lower.0.as_u32()),
                    upper: Slot::from_u32(valid_while.upper.0.as_u32()),
                }),
                MLength::Ignore => OrIgnore::Ignore,
            },
        }
    }
}

impl From<&BlockTime> for BlockTimeTimeStableV1 {
    fn from(value: &BlockTime) -> Self {
        Self(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            (value.as_u64() as i64).into(),
        ))
    }
}

impl From<&zkapp_command::Preconditions> for MinaBaseAccountUpdatePreconditionsStableV1 {
    fn from(value: &zkapp_command::Preconditions) -> Self {
        use mina_p2p_messages::v2::MinaBaseAccountUpdateAccountPreconditionStableV1 as MAccount;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1Amount as MAmount;
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1SnarkedLedgerHash as Ledger;
        use zkapp_command::AccountPreconditions;
        use zkapp_command::{Numeric, OrIgnore};
        use MinaBaseZkappPreconditionProtocolStateStableV1Length as MLength;

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
                last_vrf_output: value.network.last_vrf_output,
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
                        MLength::Check(MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
                            lower: (&length.lower).into(),
                            upper: (&length.upper).into(),
                        })
                    }
                    Numeric::Ignore => MLength::Ignore,
                },
                staking_epoch_data: (&value.network.staking_epoch_data).into(),
                next_epoch_data: (&value.network.staking_epoch_data).into(),
            },
            account: match &value.account {
                AccountPreconditions::Full(account) => {
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2Balance as MBalance;
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2Delegate as Delegate;
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2ProvedState as Proved;
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash as Receipt;
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2StateA as State;
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1Length as MNonce;

                    let account = &**account;
                    MAccount::Full(Box::new(MinaBaseZkappPreconditionAccountStableV2 {
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
                            OrIgnore::Check(nonce) => MNonce::Check(
                                MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
                                    lower: (&nonce.lower).into(),
                                    upper: (&nonce.upper).into(),
                                },
                            ),
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
                        sequence_state: match &account.action_state {
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
                    }))
                }
                AccountPreconditions::Nonce(nonce) => MAccount::Nonce(nonce.into()),
                AccountPreconditions::Accept => MAccount::Accept,
            },
            valid_while: match &value.valid_while {
                OrIgnore::Check(valid_while) => {
                    MLength::Check(MinaBaseZkappPreconditionProtocolStateStableV1LengthA {
                        lower: (&valid_while.lower).into(),
                        upper: (&valid_while.upper).into(),
                    })
                }
                OrIgnore::Ignore => MLength::Ignore,
            },
        }
    }
}

/// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/verification_key_wire.ml#L37
fn of_vk(data: VerificationKey) -> WithHash<VerificationKey> {
    let hash = data.hash();
    WithHash { data, hash }
}

impl From<&MinaBaseAccountUpdateTStableV1> for AccountUpdate {
    fn from(value: &MinaBaseAccountUpdateTStableV1) -> Self {
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1Delegate as Delegate;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1Permissions as Perm;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1Timing as Timing;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1TokenSymbol as Symbol;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1VerificationKey as VK;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1VotingFor as Voting;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1ZkappUri as Uri;
        use MinaBaseAccountUpdateUpdateStableV1AppStateA as AppState;

        Self {
            body: zkapp_command::Body {
                public_key: value.body.public_key.clone().into_inner().into(),
                token_id: value.body.token_id.clone().into_inner().into(),
                update: zkapp_command::Update {
                    app_state: std::array::from_fn(|i| match &value.body.update.app_state[i] {
                        AppState::Set(bigint) => SetOrKeep::Set(bigint.to_field()),
                        AppState::Keep => SetOrKeep::Keep,
                    }),
                    delegate: match &value.body.update.delegate {
                        Delegate::Set(v) => SetOrKeep::Set(v.into()),
                        Delegate::Keep => SetOrKeep::Keep,
                    },
                    verification_key: match &value.body.update.verification_key {
                        VK::Set(vk) => SetOrKeep::Set(of_vk((&**vk).into())),
                        VK::Keep => SetOrKeep::Keep,
                    },
                    permissions: match &value.body.update.permissions {
                        Perm::Set(perms) => SetOrKeep::Set((&**perms).into()),
                        Perm::Keep => SetOrKeep::Keep,
                    },
                    zkapp_uri: match &value.body.update.zkapp_uri {
                        Uri::Set(s) => SetOrKeep::Set(s.try_into().unwrap()),
                        Uri::Keep => SetOrKeep::Keep,
                    },
                    token_symbol: match &value.body.update.token_symbol {
                        Symbol::Set(s) => SetOrKeep::Set((&s.0).try_into().unwrap()),
                        Symbol::Keep => SetOrKeep::Keep,
                    },
                    timing: match &value.body.update.timing {
                        Timing::Set(timing) => SetOrKeep::Set((&**timing).into()),
                        Timing::Keep => SetOrKeep::Keep,
                    },
                    voting_for: match &value.body.update.voting_for {
                        Voting::Set(bigint) => SetOrKeep::Set(VotingFor(bigint.to_field())),
                        Voting::Keep => SetOrKeep::Keep,
                    },
                },
                balance_change: Signed::<Amount> {
                    magnitude: value.body.balance_change.magnitude.clone().into(),
                    sgn: value.body.balance_change.sgn.clone().into(),
                },
                increment_nonce: value.body.increment_nonce,
                events: zkapp_command::Events(
                    value
                        .body
                        .events
                        .0
                        .iter()
                        .map(|e| zkapp_command::Event(e.iter().map(|e| e.to_field()).collect()))
                        .collect(),
                ),
                actions: zkapp_command::Actions(
                    value
                        .body
                        .actions
                        .0
                        .iter()
                        .map(|e| zkapp_command::Event(e.iter().map(|e| e.to_field()).collect()))
                        .collect(),
                ),
                call_data: value.body.call_data.to_field(),
                preconditions: (&value.body.preconditions).into(),
                use_full_commitment: value.body.use_full_commitment,
                authorization_kind: match &value.body.authorization_kind {
                    mina_p2p_messages::v2::MinaBaseAccountUpdateAuthorizationKindStableV1::NoneGiven => AuthorizationKind::NoneGiven,
                    mina_p2p_messages::v2::MinaBaseAccountUpdateAuthorizationKindStableV1::Signature => AuthorizationKind::Signature,
                    mina_p2p_messages::v2::MinaBaseAccountUpdateAuthorizationKindStableV1::Proof(hash) => AuthorizationKind::Proof(hash.to_field()),
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
                    rx: signature.0.to_field(),
                    s: signature.1.to_field()
            }),
                mina_p2p_messages::v2::MinaBaseControlStableV2::NoneGiven => zkapp_command::Control::NoneGiven,
            },
        }
    }
}

/// Notes: childs
impl From<&Vec<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA>>
    for CallForest<AccountUpdate>
{
    fn from(value: &Vec<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA>) -> Self {
        use ark_ff::Zero;

        Self(
            value
                .iter()
                .map(|update| WithStackHash {
                    elt: zkapp_command::Tree {
                        account_update: (&update.elt.account_update).into(),
                        account_update_digest: Fp::zero(), // replaced later
                        calls: (&update.elt.calls).into(),
                    },
                    stack_hash: Fp::zero(), // replaced later
                })
                .collect(),
        )
    }
}

/// Notes: root
impl From<&Vec<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA>>
    for CallForest<AccountUpdate>
{
    fn from(value: &Vec<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA>) -> Self {
        use ark_ff::Zero;

        let values = value
            .iter()
            .map(|update| WithStackHash {
                elt: zkapp_command::Tree {
                    account_update: (&update.elt.account_update).into(),
                    account_update_digest: Fp::zero(), // replaced later in `of_wire`
                    calls: (&update.elt.calls).into(),
                },
                stack_hash: Fp::zero(), // replaced later in `of_wire`
            })
            .collect();

        // https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_command.ml#L1113-L1115

        let mut call_forest = CallForest(values);
        call_forest.of_wire(value);

        call_forest
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
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1TokenSymbol as Symbol;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1VerificationKey as VK;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1VotingFor as Voting;
        use mina_p2p_messages::v2::MinaBaseAccountUpdateUpdateStableV1ZkappUri as Uri;
        use MinaBaseAccountUpdateUpdateStableV1AppStateA as AppState;

        Self {
            body: MinaBaseAccountUpdateBodyStableV1 {
                public_key: (&value.body.public_key).into(),
                token_id: (&value.body.token_id).into(),
                update: MinaBaseAccountUpdateUpdateStableV1 {
                    app_state: PaddedSeq(std::array::from_fn(|i| match &value.body.update.app_state[i] {
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
                        SetOrKeep::Set(s) => Uri::Set(s.into()),
                        SetOrKeep::Keep => Uri::Keep,
                    },
                    token_symbol: match &value.body.update.token_symbol {
                        SetOrKeep::Set(s) => Symbol::Set(MinaBaseZkappAccountZkappUriStableV1(s.into())),
                        SetOrKeep::Keep => Symbol::Keep,
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
                balance_change: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount {
                    magnitude: value.body.balance_change.magnitude.into(),
                    sgn: (&value.body.balance_change.sgn).into(),
                },
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
                zkapp_command::Control::Signature(sig) => mina_p2p_messages::v2::MinaBaseControlStableV2::Signature(sig.into()),
                zkapp_command::Control::NoneGiven => mina_p2p_messages::v2::MinaBaseControlStableV2::NoneGiven,
            },
        }
    }
}

/// Childs
impl From<&CallForest<AccountUpdate>>
    for Vec<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesAACallsA>
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
    for Vec<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA>
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
        wired
    }
}

impl From<&MinaBaseFeeTransferSingleStableV2> for SingleFeeTransfer {
    fn from(value: &MinaBaseFeeTransferSingleStableV2) -> Self {
        Self {
            receiver_pk: (&value.receiver_pk).into(),
            fee: Fee::from_u64(value.fee.as_u64()),
            fee_token: (&*value.fee_token).into(),
        }
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

impl From<&MinaBaseFeeTransferStableV2> for FeeTransfer {
    fn from(value: &MinaBaseFeeTransferStableV2) -> Self {
        use super::scan_state::transaction_snark::OneOrTwo::{One, Two};

        match value {
            MinaBaseFeeTransferStableV2::One(ft) => FeeTransfer(One(ft.into())),
            MinaBaseFeeTransferStableV2::Two((a, b)) => FeeTransfer(Two((a.into(), b.into()))),
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

impl From<&MinaBaseSignedCommandStableV2> for SignedCommand {
    fn from(cmd: &MinaBaseSignedCommandStableV2) -> Self {
        Self {
            payload: transaction_logic::signed_command::SignedCommandPayload {
                common: transaction_logic::signed_command::Common {
                    fee: (&cmd.payload.common.fee).into(),
                    fee_payer_pk: (&cmd.payload.common.fee_payer_pk).into(),
                    nonce: (&cmd.payload.common.nonce).into(),
                    valid_until: (&cmd.payload.common.valid_until).into(),
                    memo: (&cmd.payload.common.memo).into(),
                },
                body: match &cmd.payload.body {
                    MinaBaseSignedCommandPayloadBodyStableV2::Payment(payload) => {
                        transaction_logic::signed_command::Body::Payment(PaymentPayload {
                            source_pk: (&payload.source_pk).into(),
                            receiver_pk: (&payload.receiver_pk).into(),
                            amount: payload.amount.clone().into(),
                        })
                    }
                    MinaBaseSignedCommandPayloadBodyStableV2::StakeDelegation(
                        MinaBaseStakeDelegationStableV1::SetDelegate {
                            delegator,
                            new_delegate,
                        },
                    ) => transaction_logic::signed_command::Body::StakeDelegation(
                        StakeDelegationPayload::SetDelegate {
                            delegator: delegator.into(),
                            new_delegate: new_delegate.into(),
                        },
                    ),
                },
            },
            signer: (&cmd.signer).into(),
            signature: (&cmd.signature).into(),
        }
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
                            source_pk: (&payload.source_pk).into(),
                            receiver_pk: (&payload.receiver_pk).into(),
                            amount: payload.amount.into(),
                        },
                    ),
                    crate::scan_state::transaction_logic::signed_command::Body::StakeDelegation(
                        StakeDelegationPayload::SetDelegate {
                            delegator,
                            new_delegate,
                        },
                    ) => MinaBaseSignedCommandPayloadBodyStableV2::StakeDelegation(
                        MinaBaseStakeDelegationStableV1::SetDelegate {
                            delegator: delegator.into(),
                            new_delegate: new_delegate.into(),
                        },
                    ),
                },
            },
            signer: (&cmd.signer).into(),
            signature: (&cmd.signature).into(),
        }
    }
}

impl From<&MinaBaseZkappCommandTStableV1WireStableV1> for zkapp_command::ZkAppCommand {
    fn from(cmd: &MinaBaseZkappCommandTStableV1WireStableV1) -> Self {
        Self {
            fee_payer: (&cmd.fee_payer).into(),
            account_updates: (&cmd.account_updates).into(),
            memo: (&cmd.memo).into(),
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

impl From<&TransactionSnarkScanStateTransactionWithWitnessStableV2> for TransactionWithWitness {
    fn from(value: &TransactionSnarkScanStateTransactionWithWitnessStableV2) -> Self {
        use mina_p2p_messages::v2::MinaTransactionLogicTransactionAppliedVaryingStableV2::*;
        use mina_p2p_messages::v2::MinaTransactionLogicTransactionAppliedCommandAppliedStableV2::*;
        use mina_p2p_messages::v2::MinaTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV2::*;
        // use mina_p2p_messages::v2::TransactionSnarkPendingCoinbaseStackStateInitStackStableV1::{Base, Merge};
        use mina_p2p_messages::v2::MinaStateSnarkedLedgerStatePendingCoinbaseStackStateInitStackStableV1::{Base, Merge};
        use crate::scan_state::scan_state::transaction_snark::InitStack;
        use transaction_applied::signed_command_applied;

        Self {
            transaction_with_info: TransactionApplied {
                previous_hash: value.transaction_with_info.previous_hash.to_field(),
                varying: match &value.transaction_with_info.varying {
                    Command(cmd) => match cmd {
                        SignedCommand(cmd) => transaction_applied::Varying::Command(
                            transaction_applied::CommandApplied::SignedCommand(Box::new(
                                transaction_applied::SignedCommandApplied {
                                    common: transaction_applied::signed_command_applied::Common {
                                        user_command: WithStatus {
                                            data: (&cmd.common.user_command.data).into(),
                                            status: (&cmd.common.user_command.status).into(),
                                        },
                                    },
                                    body: match &cmd.body {
                                        Payment { new_accounts } => {
                                            signed_command_applied::Body::Payments {
                                                new_accounts: new_accounts
                                                    .iter()
                                                    .cloned()
                                                    .map(Into::into)
                                                    .collect(),
                                            }
                                        }
                                        StakeDelegation { previous_delegate } => {
                                            signed_command_applied::Body::StakeDelegation {
                                                previous_delegate: previous_delegate
                                                    .as_ref()
                                                    .map(|d| d.into()),
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
                                            let id: AccountId = id.into();
                                            // TODO: Don't clone here
                                            let account: Option<Account> =
                                                account_opt.as_ref().map(|acc| acc.clone().into());

                                            (id, account)
                                        })
                                        .collect(),
                                    command: WithStatus {
                                        data: (&cmd.command.data).into(),
                                        status: (&cmd.command.status).into(),
                                    },
                                    new_accounts: cmd.new_accounts.iter().map(Into::into).collect(),
                                },
                            )),
                        ),
                    },
                    FeeTransfer(ft) => transaction_applied::Varying::FeeTransfer(
                        transaction_applied::FeeTransferApplied {
                            fee_transfer: WithStatus {
                                data: (&ft.fee_transfer.data).into(),
                                status: (&ft.fee_transfer.status).into(),
                            },
                            new_accounts: ft.new_accounts.iter().map(Into::into).collect(),
                            burned_tokens: ft.burned_tokens.clone().into(),
                        },
                    ),
                    Coinbase(cb) => transaction_applied::Varying::Coinbase(transaction_applied::CoinbaseApplied {
                        coinbase: WithStatus {
                            data: crate::scan_state::transaction_logic::Coinbase {
                                receiver: (&cb.coinbase.data.receiver).into(),
                                amount: cb.coinbase.data.amount.clone().into(),
                                fee_transfer: cb.coinbase.data.fee_transfer.as_ref().map(|ft| {
                                    crate::scan_state::transaction_logic::CoinbaseFeeTransfer {
                                        receiver_pk: (&ft.receiver_pk).into(),
                                        fee: Fee::from_u64(ft.fee.as_u64()),
                                    }
                                }),
                            },
                            status: (&cb.coinbase.status).into(),
                        },
                        new_accounts: cb.new_accounts.iter().map(Into::into).collect(),
                        burned_tokens: cb.burned_tokens.clone().into(),
                    }),
                },
            },
            state_hash: {
                let (state, body) = &value.state_hash;
                (state.to_field(), body.to_field())
            },
            statement: (&*value.statement).into(),
            init_stack: match &value.init_stack {
                Base(base) => InitStack::Base(pending_coinbase::Stack {
                    data: pending_coinbase::CoinbaseStack(base.data.to_field()),
                    state: pending_coinbase::StateStack {
                        init: base.state.init.to_field(),
                        curr: base.state.curr.to_field(),
                    },
                }),
                Merge => InitStack::Merge,
            },
            first_pass_ledger_witness: (&value.first_pass_ledger_witness).into(),
            second_pass_ledger_witness: (&value.second_pass_ledger_witness).into(),
            block_global_slot: Slot::from_u32(value.block_global_slot.as_u32()),
        }
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
            pending_coinbase_stack: MinaBasePendingCoinbaseStackVersionedStableV1 {
                data: MinaBasePendingCoinbaseCoinbaseStackStableV1(
                    value.pending_coinbase_stack.data.0.into(),
                ),
                state: MinaBasePendingCoinbaseStateStackStableV1 {
                    init: MinaBasePendingCoinbaseStackHashStableV1(
                        value.pending_coinbase_stack.state.init.into(),
                    ),
                    curr: MinaBasePendingCoinbaseStackHashStableV1(
                        value.pending_coinbase_stack.state.curr.into(),
                    ),
                },
            },
            local_state: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 {
                stack_frame: MinaBaseStackFrameStableV1(value.local_state.stack_frame.into()),
                call_stack: MinaBaseCallStackDigestStableV1(value.local_state.call_stack.into()),
                transaction_commitment: value.local_state.transaction_commitment.into(),
                full_transaction_commitment: value.local_state.full_transaction_commitment.into(),
                token_id: (&value.local_state.token_id).into(),
                excess: MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount {
                    magnitude: value.local_state.excess.magnitude.into(),
                    sgn: (&value.local_state.excess.sgn).into(),
                },
                supply_increase:
                    MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount {
                        magnitude: value.local_state.supply_increase.magnitude.into(),
                        sgn: (&value.local_state.supply_increase.sgn).into(),
                    },
                ledger: {
                    let hash = MinaBaseLedgerHash0StableV1(value.local_state.ledger.into());
                    hash.into()
                },
                success: value.local_state.success,
                account_update_index: UnsignedExtendedUInt32StableV1(
                    (value.local_state.account_update_index.0 as i32).into(),
                ),
                failure_status_tbl: MinaBaseTransactionStatusFailureCollectionStableV1(
                    value
                        .local_state
                        .failure_status_tbl
                        .iter()
                        .map(|s| s.iter().map(|s| s.into()).collect())
                        .collect(),
                ),
                will_succeed: value.local_state.will_succeed,
            },
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

pub fn to_ledger_hash(value: &Fp) -> mina_p2p_messages::v2::LedgerHash {
    let hash = MinaBaseLedgerHash0StableV1(value.into());
    hash.into()
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
                                    let account_opt = account_opt.as_ref().map(|acc| acc.clone().into());
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

                (state.into(), MinaBaseStateBodyHashStableV1(body.into()))
            },
            statement: (&value.statement).into(),
            init_stack: match &value.init_stack {
                InitStack::Base(base) => Base(MinaBasePendingCoinbaseStackVersionedStableV1 {
                    data: MinaBasePendingCoinbaseCoinbaseStackStableV1(base.data.0.into()),
                    state: MinaBasePendingCoinbaseStateStackStableV1 {
                        init: MinaBasePendingCoinbaseStackHashStableV1(base.state.init.into()),
                        curr: MinaBasePendingCoinbaseStackHashStableV1(base.state.curr.into()),
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

impl From<&TransactionSnarkStableV2> for TransactionSnark<SokDigest> {
    fn from(value: &TransactionSnarkStableV2) -> Self {
        Self {
            statement: (&value.statement).into(),
            proof: Rc::new(value.proof.clone()),
        }
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

impl From<&LedgerProofProdStableV2> for LedgerProof {
    fn from(value: &LedgerProofProdStableV2) -> Self {
        Self((&value.0).into())
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

impl From<&MinaBaseSokMessageStableV1> for SokMessage {
    fn from(value: &MinaBaseSokMessageStableV1) -> Self {
        Self {
            fee: (&value.fee).into(),
            prover: (&value.prover).into(),
        }
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

impl From<&TransactionSnarkScanStateLedgerProofWithSokMessageStableV2>
    for LedgerProofWithSokMessage
{
    fn from(value: &TransactionSnarkScanStateLedgerProofWithSokMessageStableV2) -> Self {
        Self {
            proof: (&value.0).into(),
            sok_message: (&value.1).into(),
        }
    }
}

impl binprot::BinProtWrite for LedgerProofWithSokMessage {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let p2p: TransactionSnarkScanStateLedgerProofWithSokMessageStableV2 = self.into();
        p2p.binprot_write(w)
    }
}

impl From<&MinaBaseUserCommandStableV2> for transaction_logic::valid::UserCommand {
    fn from(value: &MinaBaseUserCommandStableV2) -> Self {
        match value {
            MinaBaseUserCommandStableV2::ZkappCommand(_) => todo!(),
            MinaBaseUserCommandStableV2::SignedCommand(cmd) => {
                Self::SignedCommand(Box::new(SignedCommand {
                    payload: transaction_logic::signed_command::SignedCommandPayload {
                        common: transaction_logic::signed_command::Common {
                            fee: (&cmd.payload.common.fee).into(),
                            fee_payer_pk: (&cmd.payload.common.fee_payer_pk).into(),
                            nonce: (&cmd.payload.common.nonce).into(),
                            valid_until: (&cmd.payload.common.valid_until).into(),
                            memo: (&cmd.payload.common.memo).into(),
                        },
                        body: match &cmd.payload.body {
                            MinaBaseSignedCommandPayloadBodyStableV2::Payment(payment) => {
                                transaction_logic::signed_command::Body::Payment(PaymentPayload {
                                    source_pk: (&payment.source_pk).into(),
                                    receiver_pk: (&payment.receiver_pk).into(),
                                    amount: payment.amount.clone().into(),
                                })
                            }
                            MinaBaseSignedCommandPayloadBodyStableV2::StakeDelegation(
                                delegation,
                            ) => {
                                let MinaBaseStakeDelegationStableV1::SetDelegate {
                                    delegator,
                                    new_delegate,
                                } = &delegation;

                                transaction_logic::signed_command::Body::StakeDelegation(
                                    StakeDelegationPayload::SetDelegate {
                                        delegator: delegator.into(),
                                        new_delegate: new_delegate.into(),
                                    },
                                )
                            }
                        },
                    },
                    signer: (&cmd.signer).into(),
                    signature: (&cmd.signature).into(),
                }))
            }
        }
    }
}
