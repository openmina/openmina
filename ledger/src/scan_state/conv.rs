use mina_p2p_messages::v2::{
    CurrencyAmountStableV1, CurrencyFeeStableV1, LedgerProofProdStableV2,
    MinaBaseAccountIdDigestStableV1, MinaBaseAccountUpdateFeePayerStableV1,
    MinaBaseAccountUpdatePreconditionsStableV1, MinaBaseAccountUpdateTWireStableV1,
    MinaBaseAccountUpdateUpdateStableV1AppStateA, MinaBaseAccountUpdateUpdateTimingInfoStableV1,
    MinaBaseFeeExcessStableV1, MinaBaseFeeExcessStableV1Fee, MinaBaseLedgerHash0StableV1,
    MinaBasePendingCoinbaseStackVersionedStableV1, MinaBaseSokMessageDigestStableV1,
    MinaBaseSokMessageStableV1, MinaBaseTransactionStatusFailureCollectionStableV1,
    MinaBaseTransactionStatusFailureStableV2, MinaBaseTransactionStatusStableV2,
    MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA,
    MinaBaseZkappPreconditionProtocolStateEpochDataStableV1,
    MinaBaseZkappPreconditionProtocolStateStableV1Length,
    MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount, SgnStableV1,
    TransactionSnarkScanStateLedgerProofWithSokMessageStableV2,
    TransactionSnarkScanStateTransactionWithWitnessStableV2, TransactionSnarkStableV2,
    TransactionSnarkStatementStableV2, TransactionSnarkStatementWithSokStableV2,
    TransactionSnarkStatementWithSokStableV2Source,
    UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
};

use crate::{
    array_into, array_into_with,
    scan_state::transaction_logic::{
        zkapp_command::{self, CallForest},
        WithStatus,
    },
    Account, AccountId, Timing,
};

use super::{
    currency::{Amount, Balance, Fee, Sgn, Signed},
    fee_excess::FeeExcess,
    pending_coinbase,
    scan_state::transaction_snark::{
        LedgerProof, LedgerProofWithSokMessage, Registers, SokMessage, Statement, TransactionSnark,
        TransactionWithWitness,
    },
    transaction_logic::{
        local_state::LocalState,
        transaction_applied::{self, TransactionApplied},
        zkapp_command::{AccountUpdate, FeePayer, FeePayerBody, Nonce, SetOrKeep, WithStackHash},
        Index, Signature, Slot, TransactionFailure, TransactionStatus,
    },
};

impl From<CurrencyAmountStableV1> for Amount {
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

impl From<&MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount>
    for Signed<Amount>
{
    fn from(
        value: &MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount,
    ) -> Self {
        Self {
            magnitude: value.magnitude.into(),
            sgn: value.sgn.0.into(),
        }
    }
}

impl From<&Signed<Amount>>
    for MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1SignedAmount
{
    fn from(value: &Signed<Amount>) -> Self {
        Self {
            magnitude: value.magnitude.into(),
            sgn: ((&value.sgn).into(),),
        }
    }
}

impl From<&CurrencyFeeStableV1> for Fee {
    fn from(value: &CurrencyFeeStableV1) -> Self {
        Self(value.as_u64())
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
            sgn: value.sgn.0.into(),
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
            sgn: ((&value.sgn).into(),),
        }
    }
}

impl From<&MinaBaseFeeExcessStableV1> for FeeExcess {
    fn from(value: &MinaBaseFeeExcessStableV1) -> Self {
        Self {
            fee_token_l: (&value.fee_token_l.0).into(),
            fee_excess_l: (&value.fee_excess_l).into(),
            fee_token_r: (&value.fee_token_r.0).into(),
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
            P2P::UpdateNotPermittedTimingExistingAccount => {
                Self::UpdateNotPermittedTimingExistingAccount
            }
            P2P::UpdateNotPermittedDelegate => Self::UpdateNotPermittedDelegate,
            P2P::UpdateNotPermittedAppState => Self::UpdateNotPermittedAppState,
            P2P::UpdateNotPermittedVerificationKey => Self::UpdateNotPermittedVerificationKey,
            P2P::UpdateNotPermittedSequenceState => Self::UpdateNotPermittedSequenceState,
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
                Self::AccountSequenceStatePreconditionUnsatisfied
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
            P2P::UpdateNotPermittedTimingExistingAccount => {
                Self::UpdateNotPermittedTimingExistingAccount
            }
            P2P::UpdateNotPermittedDelegate => Self::UpdateNotPermittedDelegate,
            P2P::UpdateNotPermittedAppState => Self::UpdateNotPermittedAppState,
            P2P::UpdateNotPermittedVerificationKey => Self::UpdateNotPermittedVerificationKey,
            P2P::UpdateNotPermittedSequenceState => Self::UpdateNotPermittedSequenceState,
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
        }
    }
}

impl From<&TransactionSnarkStatementWithSokStableV2Source> for Registers {
    fn from(value: &TransactionSnarkStatementWithSokStableV2Source) -> Self {
        Self {
            ledger: value.ledger.to_field(),
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
                        value.local_state.token_id.into_inner();
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
            },
        }
    }
}

impl From<&TransactionSnarkStatementStableV2> for Statement {
    fn from(value: &TransactionSnarkStatementStableV2) -> Self {
        Self {
            source: (&value.source).into(),
            target: (&value.target).into(),
            supply_increase: (&value.supply_increase).into(),
            fee_excess: (&value.fee_excess).into(),
            sok_digest: None,
        }
    }
}

impl From<&TransactionSnarkStatementWithSokStableV2> for Statement {
    fn from(value: &TransactionSnarkStatementWithSokStableV2) -> Self {
        Self {
            source: (&value.source).into(),
            target: (&value.target).into(),
            supply_increase: (&value.supply_increase).into(),
            fee_excess: (&value.fee_excess).into(),
            sok_digest: Some(value.sok_digest.to_vec()),
        }
    }
}

impl From<&Statement> for TransactionSnarkStatementWithSokStableV2 {
    fn from(value: &Statement) -> Self {
        assert!(value.sok_digest.is_some());
        Self {
            source: (&value.source).into(),
            target: (&value.target).into(),
            supply_increase: (&value.supply_increase).into(),
            fee_excess: (&value.fee_excess).into(),
            sok_digest: MinaBaseSokMessageDigestStableV1(
                value.sok_digest.as_ref().unwrap().as_slice().into(),
            ),
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
                public_key: value.body.public_key.into_inner().into(),
                fee: Fee::from_u64(value.body.fee.as_u64()),
                valid_until: value
                    .body
                    .valid_until
                    .as_ref()
                    .map(|until| Slot::from_u32(until.as_u32())),
                nonce: Nonce::from_u32(value.body.nonce.as_u32()),
            },
            authorization: Signature((
                value.authorization.0.to_field(),
                value.authorization.1.to_field(),
            )),
        }
    }
}

impl From<&MinaBaseAccountUpdateUpdateTimingInfoStableV1> for Timing {
    fn from(t: &MinaBaseAccountUpdateUpdateTimingInfoStableV1) -> Self {
        // TODO: The account update doesn't have `Self::Untimed`
        Timing::Timed {
            initial_minimum_balance: Balance::from_u64(t.initial_minimum_balance.as_u64()),
            cliff_time: Slot::from_u32(t.cliff_time.as_u32()),
            cliff_amount: Amount::from_u64(t.cliff_amount.as_u64()),
            vesting_period: Slot::from_u32(t.vesting_period.as_u32()),
            vesting_increment: Amount::from_u64(t.vesting_increment.as_u64()),
        }
    }
}

impl From<&MinaBaseZkappPreconditionProtocolStateStableV1Length>
    for zkapp_command::Numeric<zkapp_command::Length>
{
    fn from(value: &MinaBaseZkappPreconditionProtocolStateStableV1Length) -> Self {
        use zkapp_command::{ClosedInterval, Length, Numeric};
        use MinaBaseZkappPreconditionProtocolStateStableV1Length as MLength;

        match value {
            MLength::Check(length) => Numeric::Check(ClosedInterval {
                lower: Length::from_u32(length.lower.0.as_u32()),
                upper: Length::from_u32(length.upper.0.as_u32()),
            }),
            MLength::Ignore => todo!(),
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
                hash: match value.ledger.hash {
                    Hash::Check(hash) => OrIgnore::Check(hash.to_field()),
                    Hash::Ignore => OrIgnore::Ignore,
                },
                total_currency: match value.ledger.total_currency {
                    MAmount::Check(amount) => OrIgnore::Check(ClosedInterval {
                        lower: Amount::from_u64(amount.lower.0 .0.as_u64()),
                        upper: Amount::from_u64(amount.upper.0 .0.as_u64()),
                    }),
                    MAmount::Ignore => OrIgnore::Ignore,
                },
            },
            seed: match value.seed {
                Seed::Check(seed) => OrIgnore::Check(seed.to_field()),
                Seed::Ignore => OrIgnore::Ignore,
            },
            start_checkpoint: match value.start_checkpoint {
                Start::Check(start) => OrIgnore::Check(start.to_field()),
                Start::Ignore => OrIgnore::Ignore,
            },
            lock_checkpoint: match value.lock_checkpoint {
                Start::Check(start) => OrIgnore::Check(start.to_field()),
                Start::Ignore => OrIgnore::Ignore,
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
        use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1Time as Time;
        use zkapp_command::AccountPreconditions;
        use zkapp_command::{BlockTime, ClosedInterval, Length, Numeric, OrIgnore};
        use MinaBaseZkappPreconditionProtocolStateStableV1Length as MLength;

        Self {
            network: zkapp_command::ZkAppPreconditions {
                snarked_ledger_hash: match value.network.snarked_ledger_hash {
                    Ledger::Check(hash) => OrIgnore::Check(hash.to_field()),
                    Ledger::Ignore => OrIgnore::Ignore,
                },
                timestamp: match value.network.timestamp {
                    Time::Check(time) => OrIgnore::Check(ClosedInterval {
                        lower: BlockTime::from_u64(time.lower.0 .0.as_u64()),
                        upper: BlockTime::from_u64(time.upper.0 .0.as_u64()),
                    }),
                    Time::Ignore => OrIgnore::Ignore,
                },
                blockchain_length: (&value.network.blockchain_length).into(),
                min_window_density: (&value.network.min_window_density).into(),
                last_vrf_output: value.network.last_vrf_output,
                total_currency: match value.network.total_currency {
                    MAmount::Check(amount) => OrIgnore::Check(ClosedInterval {
                        lower: Amount::from_u64(amount.lower.0 .0.as_u64()),
                        upper: Amount::from_u64(amount.upper.0 .0.as_u64()),
                    }),
                    MAmount::Ignore => OrIgnore::Ignore,
                },
                global_slot_since_hard_fork: match value.network.global_slot_since_hard_fork {
                    MLength::Check(length) => Numeric::Check(ClosedInterval {
                        lower: Slot::from_u32(length.lower.0.as_u32()),
                        upper: Slot::from_u32(length.upper.0.as_u32()),
                    }),
                    MLength::Ignore => OrIgnore::Ignore,
                },
                global_slot_since_genesis: match value.network.global_slot_since_genesis {
                    MLength::Check(length) => Numeric::Check(ClosedInterval {
                        lower: Slot::from_u32(length.lower.0.as_u32()),
                        upper: Slot::from_u32(length.upper.0.as_u32()),
                    }),
                    MLength::Ignore => OrIgnore::Ignore,
                },
                staking_epoch_data: (&value.network.staking_epoch_data).into(),
                next_epoch_data: (&value.network.staking_epoch_data).into(),
            },
            account: match value.account {
                MAccount::Full(account) => {
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2Balance as MBalance;
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2Delegate as Delegate;
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2ProvedState as Proved;
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2ReceiptChainHash as Receipt;
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionAccountStableV2StateA as State;
                    use mina_p2p_messages::v2::MinaBaseZkappPreconditionProtocolStateStableV1Length as MNonce;

                    let account = &*account;
                    zkapp_command::Account {
                        balance: match account.balance {
                            MBalance::Check(balance) => OrIgnore::Check(ClosedInterval {
                                lower: Balance::from_u64(balance.lower.0.as_u64()),
                                upper: Balance::from_u64(balance.upper.0.as_u64()),
                            }),
                            MBalance::Ignore => OrIgnore::Ignore,
                        },
                        nonce: match account.nonce {
                            MNonce::Check(balance) => OrIgnore::Check(ClosedInterval {
                                lower: Nonce::from_u32(balance.lower.0.as_u32()),
                                upper: Nonce::from_u32(balance.upper.0.as_u32()),
                            }),
                            MNonce::Ignore => OrIgnore::Ignore,
                        },
                        receipt_chain_hash: match account.receipt_chain_hash {
                            Receipt::Check(hash) => OrIgnore::Check(hash.to_field()),
                            Receipt::Ignore => OrIgnore::Ignore,
                        },
                        delegate: match account.delegate {
                            Delegate::Check(delegate) => OrIgnore::Check((&delegate).into()),
                            Delegate::Ignore => OrIgnore::Ignore,
                        },
                        state: array_into_with(&account.state, |s| match s {
                            State::Check(s) => OrIgnore::Check(s.to_field()),
                            State::Ignore => OrIgnore::Ignore,
                        }),
                        sequence_state: match account.sequence_state {
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
                    }
                }
                MAccount::Nonce(nonce) => {
                    AccountPreconditions::Nonce(Nonce::from_u32(nonce.as_u32()))
                }
                MAccount::Accept => AccountPreconditions::Accept,
            },
        }
    }
}

impl From<&MinaBaseAccountUpdateTWireStableV1> for AccountUpdate {
    fn from(value: &MinaBaseAccountUpdateTWireStableV1) -> Self {
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
                public_key: value.body.public_key.into_inner().into(),
                token_id: value.body.token_id.into_inner().into(),
                update: zkapp_command::Update {
                    app_state: std::array::from_fn(|i| match value.body.update.app_state[i] {
                        AppState::Set(bigint) => SetOrKeep::Set(bigint.to_field()),
                        AppState::Keep => SetOrKeep::Kepp,
                    }),
                    delegate: match value.body.update.delegate {
                        Delegate::Set(v) => SetOrKeep::Set(v.into()),
                        Delegate::Keep => SetOrKeep::Kepp,
                    },
                    verification_key: match value.body.update.verification_key {
                        VK::Set(vk) => SetOrKeep::Set((&*vk).into()),
                        VK::Keep => SetOrKeep::Kepp,
                    },
                    permissions: match value.body.update.permissions {
                        Perm::Set(perms) => SetOrKeep::Set((&*perms).into()),
                        Perm::Keep => SetOrKeep::Kepp,
                    },
                    zkapp_uri: match value.body.update.zkapp_uri {
                        Uri::Set(s) => SetOrKeep::Set(s.try_into().unwrap()),
                        Uri::Keep => SetOrKeep::Kepp,
                    },
                    token_symbol: match value.body.update.token_symbol {
                        Symbol::Set(s) => SetOrKeep::Set(s.0.try_into().unwrap()),
                        Symbol::Keep => SetOrKeep::Kepp,
                    },
                    timing: match value.body.update.timing {
                        Timing::Set(timing) => SetOrKeep::Set((&*timing).into()),
                        Timing::Keep => SetOrKeep::Kepp,
                    },
                    voting_for: match value.body.update.voting_for {
                        Voting::Set(bigint) => SetOrKeep::Set(bigint.to_field()),
                        Voting::Keep => SetOrKeep::Kepp,
                    },
                },
                balance_change: Signed::<Amount> {
                    magnitude: value.body.balance_change.magnitude.into(),
                    sgn: value.body.balance_change.sgn.0.into(),
                },
                increment_nonce: value.body.increment_nonce,
                events: zkapp_command::Events(
                    value
                        .body
                        .events
                        .0
                        .iter()
                        .map(|e| e.iter().map(|e| e.to_field()).collect())
                        .collect(),
                ),
                sequence_events: zkapp_command::Events(
                    value
                        .body
                        .sequence_events
                        .0
                        .iter()
                        .map(|e| e.iter().map(|e| e.to_field()).collect())
                        .collect(),
                ),
                call_data: value.body.call_data.to_field(),
                preconditions: (&value.body.preconditions).into(),
                use_full_commitment: value.body.use_full_commitment,
                caller: match value.body.caller {
                    mina_p2p_messages::v2::MinaBaseAccountUpdateCallTypeStableV1::Call => todo!(),
                    mina_p2p_messages::v2::MinaBaseAccountUpdateCallTypeStableV1::DelegateCall => {
                        todo!()
                    }
                },
                authorization_kind: todo!(),
            },
            authorization: todo!(),
        }
    }
}

impl From<&Vec<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA>> for CallForest<()> {
    fn from(value: &Vec<MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA>) -> Self {
        Self(
            value
                .iter()
                .map(|update| WithStackHash {
                    elt: zkapp_command::Tree {
                        account_update: (
                            AccountUpdate {
                                body: todo!(),
                                authorization: todo!(),
                            },
                            (),
                        ),
                        account_update_digest: None,
                        calls: (&update.elt.calls).into(),
                    },
                    stack_hash: None,
                })
                .collect(),
        )
    }
}

impl From<&TransactionSnarkScanStateTransactionWithWitnessStableV2> for TransactionWithWitness {
    fn from(value: &TransactionSnarkScanStateTransactionWithWitnessStableV2) -> Self {
        use mina_p2p_messages::v2::MinaTransactionLogicTransactionAppliedVaryingStableV2::*;
        use mina_p2p_messages::v2::MinaTransactionLogicTransactionAppliedCommandAppliedStableV2::*;
        use mina_p2p_messages::v2::MinaTransactionLogicTransactionAppliedSignedCommandAppliedBodyStableV2::*;
        use transaction_applied::signed_command_applied;

        Self {
            transaction_with_info: TransactionApplied {
                previous_hash: value
                    .transaction_with_info
                    .previous_hash
                    .into_inner()
                    .to_field(),
                varying: transaction_applied::Varying::Command(
                    match value.transaction_with_info.varying {
                        Command(cmd) => match cmd {
                            SignedCommand(cmd) => {
                                transaction_applied::CommandApplied::SignedCommand(Box::new(
                                    transaction_applied::SignedCommandApplied {
                                        common: todo!(),
                                        body: match cmd.body {
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
                                ))
                            }
                            ZkappCommand(cmd) => transaction_applied::CommandApplied::ZkappCommand(
                                Box::new(transaction_applied::ZkappCommandApplied {
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
                                        data: zkapp_command::ZkAppCommand {
                                            fee_payer: (&cmd.command.data.fee_payer).into(),
                                            account_updates: CallForest(
                                                cmd.command
                                                    .data
                                                    .account_updates
                                                    .iter()
                                                    .map(|upd| {
                                                        // upd.elt
                                                        todo!()
                                                    })
                                                    .collect(),
                                            ),
                                            memo: todo!(),
                                        },
                                        status: (&cmd.command.status).into(),
                                    },
                                    new_accounts: todo!(),
                                }),
                            ),
                        },
                        FeeTransfer(_) => todo!(),
                        Coinbase(_) => todo!(),
                    },
                ),
            },
            // transaction_with_info: value.transaction_with_info.clone(),
            state_hash: value.state_hash.clone(),
            statement: (&value.statement).into(),
            init_stack: value.init_stack.clone(),
            ledger_witness: todo!(), // value.ledger_witness.clone(),
        }
    }
}

impl From<&Registers> for TransactionSnarkStatementWithSokStableV2Source {
    fn from(value: &Registers) -> Self {
        Self {
            ledger: MinaBaseLedgerHash0StableV1(value.ledger.into()).into(),
            pending_coinbase_stack: value.pending_coinbase_stack.clone(),
            local_state: value.local_state.clone(),
        }
    }
}

impl From<&Statement> for TransactionSnarkStatementStableV2 {
    fn from(value: &Statement) -> Self {
        Self {
            source: (&value.source).into(),
            target: (&value.target).into(),
            supply_increase: (&value.supply_increase).into(),
            fee_excess: (&value.fee_excess).into(),
            sok_digest: (),
        }
    }
}

impl From<&TransactionWithWitness> for TransactionSnarkScanStateTransactionWithWitnessStableV2 {
    fn from(value: &TransactionWithWitness) -> Self {
        Self {
            transaction_with_info: value.transaction_with_info.clone(),
            state_hash: value.state_hash.clone(),
            statement: (&value.statement).into(),
            init_stack: value.init_stack.clone(),
            ledger_witness: todo!(), // value.ledger_witness.clone(),
        }
    }
}

impl binprot::BinProtWrite for TransactionWithWitness {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let p2p: TransactionSnarkScanStateTransactionWithWitnessStableV2 = self.into();
        p2p.binprot_write(w)
    }
}

impl From<&TransactionSnarkStableV2> for TransactionSnark {
    fn from(value: &TransactionSnarkStableV2) -> Self {
        Self {
            statement: (&value.statement).into(),
            proof: value.proof.clone(),
        }
    }
}

impl From<&TransactionSnark> for TransactionSnarkStableV2 {
    fn from(value: &TransactionSnark) -> Self {
        Self {
            statement: (&value.statement).into(),
            proof: value.proof.clone(),
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

impl binprot::BinProtWrite for LedgerProof {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let p2p: LedgerProofProdStableV2 = self.into();
        p2p.binprot_write(w)
    }
}

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
