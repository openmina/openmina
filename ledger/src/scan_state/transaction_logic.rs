use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::{
    scan_state::{currency::Magnitude, transaction_logic::transaction_applied::Varying},
    staged_ledger::sparse_ledger::{LedgerIntf, SparseLedger},
    Account, AccountId, BaseLedger, ReceiptChainHash, Timing, ToInputs, TokenId, VerificationKey,
};

use self::{
    protocol_state::ProtocolStateView,
    signed_command::{SignedCommand, SignedCommandPayload},
    transaction_applied::TransactionApplied,
    transaction_union_payload::TransactionUnionPayload,
    valid::VerificationKeyHash,
    zkapp_command::Nonce,
};

use super::{
    currency::{Amount, Balance, Fee, Signed},
    fee_excess::FeeExcess,
    scan_state::{transaction_snark::OneOrTwo, ConstraintConstants},
};

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/transaction_status.ml#L9
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionFailure {
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
    LocalSupplyIncreaseOverflow,
    GlobalSupplyIncreaseOverflow,
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
    ZkappCommandReplayCheckFailed,
    FeePayerNonceMustIncrease,
    FeePayerMustBeSigned,
    AccountBalancePreconditionUnsatisfied,
    AccountNoncePreconditionUnsatisfied,
    AccountReceiptChainHashPreconditionUnsatisfied,
    AccountDelegatePreconditionUnsatisfied,
    AccountSequenceStatePreconditionUnsatisfied,
    AccountAppStatePreconditionUnsatisfied(i64),
    AccountProvedStatePreconditionUnsatisfied,
    AccountIsNewPreconditionUnsatisfied,
    ProtocolStatePreconditionUnsatisfied,
    IncorrectNonce,
    InvalidFeeExcess,
    Cancelled,
}

pub fn single_failure() -> Vec<Vec<TransactionFailure>> {
    vec![vec![TransactionFailure::UpdateNotPermittedBalance]]
}

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/transaction_status.ml#L452
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionStatus {
    Applied,
    Failed(Vec<Vec<TransactionFailure>>),
}

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/with_status.ml#L6
#[derive(Debug, Clone)]
pub struct WithStatus<T> {
    pub data: T,
    pub status: TransactionStatus,
}

impl<T> WithStatus<T> {
    pub fn map<F, R>(&self, fun: F) -> WithStatus<R>
    where
        F: Fn(&T) -> R,
    {
        WithStatus {
            data: fun(&self.data),
            status: self.status.clone(),
        }
    }

    pub fn into_map<F, R>(self, fun: F) -> WithStatus<R>
    where
        F: Fn(T) -> R,
    {
        WithStatus {
            data: fun(self.data),
            status: self.status,
        }
    }
}

pub trait GenericCommand {
    fn fee(&self) -> Fee;
    fn forget(&self) -> UserCommand;
}

pub trait GenericTransaction: Sized {
    fn is_fee_transfer(&self) -> bool;
    fn is_coinbase(&self) -> bool;
    fn is_command(&self) -> bool;
}

pub mod valid {
    use super::*;

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub struct VerificationKeyHash(pub Fp);

    #[derive(Clone, Debug)]
    pub enum UserCommand {
        SignedCommand(Box<super::signed_command::SignedCommand>),
        ZkAppCommand(Box<super::zkapp_command::valid::ZkAppCommand>),
    }

    impl UserCommand {
        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/user_command.ml#L277
        pub fn forget_check(&self) -> super::UserCommand {
            match self {
                UserCommand::SignedCommand(cmd) => super::UserCommand::SignedCommand(cmd.clone()),
                UserCommand::ZkAppCommand(cmd) => {
                    super::UserCommand::ZkAppCommand(Box::new(cmd.zkapp_command.clone()))
                }
            }
        }
    }

    impl GenericCommand for UserCommand {
        fn fee(&self) -> Fee {
            match self {
                UserCommand::SignedCommand(cmd) => cmd.fee(),
                UserCommand::ZkAppCommand(cmd) => cmd.zkapp_command.fee(),
            }
        }

        fn forget(&self) -> super::UserCommand {
            match self {
                UserCommand::SignedCommand(cmd) => super::UserCommand::SignedCommand(cmd.clone()),
                UserCommand::ZkAppCommand(cmd) => {
                    super::UserCommand::ZkAppCommand(Box::new(cmd.zkapp_command.clone()))
                }
            }
        }
    }

    impl GenericTransaction for Transaction {
        fn is_fee_transfer(&self) -> bool {
            matches!(self, Transaction::FeeTransfer(_))
        }
        fn is_coinbase(&self) -> bool {
            matches!(self, Transaction::Coinbase(_))
        }
        fn is_command(&self) -> bool {
            matches!(self, Transaction::Command(_))
        }
    }

    #[derive(Debug, derive_more::From)]
    pub enum Transaction {
        Command(UserCommand),
        FeeTransfer(super::FeeTransfer),
        Coinbase(super::Coinbase),
    }

    impl Transaction {
        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/transaction/transaction.ml#L61
        pub fn forget(&self) -> super::Transaction {
            match self {
                Transaction::Command(cmd) => super::Transaction::Command(cmd.forget_check()),
                Transaction::FeeTransfer(ft) => super::Transaction::FeeTransfer(ft.clone()),
                Transaction::Coinbase(cb) => super::Transaction::Coinbase(cb.clone()),
            }
        }
    }
}

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/fee_transfer.ml#L19
#[derive(Debug, Clone)]
pub struct SingleFeeTransfer {
    pub receiver_pk: CompressedPubKey,
    pub fee: Fee,
    pub fee_token: TokenId,
}

impl SingleFeeTransfer {
    pub fn receiver(&self) -> AccountId {
        AccountId {
            public_key: self.receiver_pk.clone(),
            token_id: self.fee_token.clone(),
        }
    }

    pub fn create(receiver_pk: CompressedPubKey, fee: Fee, fee_token: TokenId) -> Self {
        Self {
            receiver_pk,
            fee,
            fee_token,
        }
    }
}

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/fee_transfer.ml#L68
#[derive(Debug, Clone)]
pub struct FeeTransfer(pub(super) OneOrTwo<SingleFeeTransfer>);

impl std::ops::Deref for FeeTransfer {
    type Target = OneOrTwo<SingleFeeTransfer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FeeTransfer {
    pub fn fee_tokens(&self) -> impl Iterator<Item = &TokenId> {
        self.0.iter().map(|fee_transfer| &fee_transfer.fee_token)
    }

    pub fn receiver_pks(&self) -> impl Iterator<Item = &CompressedPubKey> {
        self.0.iter().map(|fee_transfer| &fee_transfer.receiver_pk)
    }

    pub fn receivers(&self) -> impl Iterator<Item = AccountId> + '_ {
        self.0.iter().map(|fee_transfer| AccountId {
            public_key: fee_transfer.receiver_pk.clone(),
            token_id: fee_transfer.fee_token.clone(),
        })
    }

    /// https://github.com/MinaProtocol/mina/blob/e5183ca1dde1c085b4c5d37d1d9987e24c294c32/src/lib/mina_base/fee_transfer.ml#L109
    pub fn fee_excess(&self) -> Result<FeeExcess, String> {
        let one_or_two = self.0.map(|SingleFeeTransfer { fee, fee_token, .. }| {
            (fee_token.clone(), Signed::<Fee>::of_unsigned(*fee).negate())
        });
        FeeExcess::of_one_or_two(one_or_two)
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/fee_transfer.ml#L84
    pub fn of_singles(singles: OneOrTwo<SingleFeeTransfer>) -> Result<Self, String> {
        match singles {
            OneOrTwo::One(a) => Ok(Self(OneOrTwo::One(a))),
            OneOrTwo::Two((one, two)) => {
                if one.fee_token == two.fee_token {
                    Ok(Self(OneOrTwo::Two((one, two))))
                } else {
                    // Necessary invariant for the transaction snark: we should never have
                    // fee excesses in multiple tokens simultaneously.
                    return Err(format!(
                        "Cannot combine single fee transfers with incompatible tokens: {:?} <> {:?}",
                        one, two
                    ));
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CoinbaseFeeTransfer {
    pub receiver_pk: CompressedPubKey,
    pub fee: Fee,
}

impl CoinbaseFeeTransfer {
    pub fn create(receiver_pk: CompressedPubKey, fee: Fee) -> Self {
        Self { receiver_pk, fee }
    }

    pub fn receiver(&self) -> AccountId {
        AccountId {
            public_key: self.receiver_pk.clone(),
            token_id: TokenId::default(),
        }
    }
}

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/coinbase.ml#L17
#[derive(Debug, Clone)]
pub struct Coinbase {
    pub receiver: CompressedPubKey,
    pub amount: Amount,
    pub fee_transfer: Option<CoinbaseFeeTransfer>,
}

impl Coinbase {
    fn is_valid(&self) -> bool {
        match &self.fee_transfer {
            None => true,
            Some(CoinbaseFeeTransfer { fee, .. }) => Amount::of_fee(fee) <= self.amount,
        }
    }

    pub fn create(
        amount: Amount,
        receiver: CompressedPubKey,
        fee_transfer: Option<CoinbaseFeeTransfer>,
    ) -> Result<Coinbase, String> {
        let mut this = Self {
            receiver: receiver.clone(),
            amount,
            fee_transfer,
        };

        if this.is_valid() {
            let adjusted_fee_transfer = this.fee_transfer.as_ref().and_then(|ft| {
                if receiver != ft.receiver_pk {
                    Some(ft.clone())
                } else {
                    None
                }
            });
            this.fee_transfer = adjusted_fee_transfer;
            Ok(this)
        } else {
            Err("Coinbase.create: invalid coinbase".to_string())
        }
    }

    fn expected_supply_increase(&self) -> Result<Amount, String> {
        let Self {
            amount,
            fee_transfer,
            ..
        } = self;

        match fee_transfer {
            None => Ok(*amount),
            Some(CoinbaseFeeTransfer { fee, .. }) => amount
                .checked_sub(&Amount::of_fee(fee))
                .ok_or_else(|| "Coinbase underflow".to_string()),
        }
    }

    pub fn fee_excess(&self) -> Result<FeeExcess, String> {
        self.expected_supply_increase().map(|_| FeeExcess::empty())
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/coinbase.ml#L39
    pub fn receiver(&self) -> AccountId {
        AccountId::new(self.receiver.clone(), TokenId::default())
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/coinbase.ml#L51
    pub fn accounts_accessed(&self) -> Vec<AccountId> {
        let mut ids = Vec::with_capacity(2);

        ids.push(self.receiver());

        if let Some(fee_transfer) = self.fee_transfer.as_ref() {
            ids.push(fee_transfer.receiver());
        };

        ids
    }
}

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signature.mli#L11
#[derive(Debug, Clone)]
pub struct Signature(pub(super) (Fp, Fp)); // TODO: Not sure if it's correct

#[derive(Debug, Clone, derive_more::Deref, derive_more::From)]
pub struct Memo(Vec<u8>);

#[derive(Copy, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slot(pub(super) u32);

impl ToInputs for Slot {
    fn to_inputs(&self, inputs: &mut crate::Inputs) {
        inputs.append_u32(self.0);
    }
}

impl rand::distributions::Distribution<Slot> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Slot {
        Slot(rng.next_u32())
    }
}

impl Slot {
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn zero() -> Self {
        Self(0)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn from_u32(slot: u32) -> Self {
        Self(slot)
    }

    fn min() -> Self {
        Self(0)
    }

    fn max() -> Self {
        Self(u32::MAX)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Index(pub(super) u32);

pub mod signed_command {
    use crate::{decompress_pk, AccountId};

    use super::{zkapp_command::Nonce, *};

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.ml#L75
    #[derive(Debug, Clone)]
    pub struct Common {
        pub fee: Fee,
        pub fee_token: TokenId,
        pub fee_payer_pk: CompressedPubKey,
        pub nonce: Nonce,
        pub valid_until: Slot,
        pub memo: Memo,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/payment_payload.ml#L40
    #[derive(Debug, Clone)]
    pub struct PaymentPayload {
        pub source_pk: CompressedPubKey,
        pub receiver_pk: CompressedPubKey,
        pub amount: Amount,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/stake_delegation.ml#L11
    #[derive(Debug, Clone)]
    pub enum StakeDelegationPayload {
        SetDelegate {
            delegator: CompressedPubKey,
            new_delegate: CompressedPubKey,
        },
    }

    impl StakeDelegationPayload {
        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/stake_delegation.ml#L30
        pub fn source(&self) -> AccountId {
            let Self::SetDelegate { delegator, .. } = self;
            AccountId::new(delegator.clone(), TokenId::default())
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/stake_delegation.ml#L28
        pub fn source_pk(&self) -> &CompressedPubKey {
            let Self::SetDelegate { delegator, .. } = self;
            delegator
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/stake_delegation.ml#L24
        pub fn receiver(&self) -> AccountId {
            let Self::SetDelegate { new_delegate, .. } = self;
            AccountId::new(new_delegate.clone(), TokenId::default())
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/stake_delegation.ml#L22
        pub fn receiver_pk(&self) -> &CompressedPubKey {
            let Self::SetDelegate { new_delegate, .. } = self;
            new_delegate
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.mli#L24
    #[derive(Debug, Clone)]
    pub enum Body {
        Payment(PaymentPayload),
        StakeDelegation(StakeDelegationPayload),
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.mli#L165
    #[derive(Debug, Clone)]
    pub struct SignedCommandPayload {
        pub common: Common,
        pub body: Body,
    }

    #[derive(Debug, Clone)]
    pub struct SignedCommand {
        pub payload: SignedCommandPayload,
        pub signer: CompressedPubKey,
        pub signature: Signature,
    }

    impl SignedCommand {
        pub fn valid_until(&self) -> Slot {
            self.payload.common.valid_until
        }

        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.ml#L322
        pub fn fee_payer(&self) -> AccountId {
            let public_key = self.payload.common.fee_payer_pk.clone();
            AccountId::new(public_key, TokenId::default())
        }

        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.ml#L320
        pub fn fee_payer_pk(&self) -> &CompressedPubKey {
            &self.payload.common.fee_payer_pk
        }

        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.ml#L318
        pub fn fee_token(&self) -> TokenId {
            TokenId::default()
        }

        pub fn fee(&self) -> Fee {
            self.payload.common.fee
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/signed_command_payload.ml#L243
        pub fn source(&self) -> AccountId {
            match &self.payload.body {
                Body::Payment(payload) => {
                    AccountId::new(payload.source_pk.clone(), TokenId::default())
                }
                Body::StakeDelegation(payload) => payload.source(),
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/signed_command_payload.ml#L227
        pub fn source_pk(&self) -> &CompressedPubKey {
            match &self.payload.body {
                Body::Payment(payload) => &payload.source_pk,
                Body::StakeDelegation(payload) => payload.source_pk(),
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/signed_command_payload.ml#L250
        pub fn receiver(&self) -> AccountId {
            match &self.payload.body {
                Body::Payment(payload) => {
                    AccountId::new(payload.receiver_pk.clone(), TokenId::default())
                }
                Body::StakeDelegation(payload) => payload.receiver(),
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/signed_command_payload.ml#L234
        pub fn receiver_pk(&self) -> &CompressedPubKey {
            match &self.payload.body {
                Body::Payment(payload) => &payload.receiver_pk,
                Body::StakeDelegation(payload) => payload.receiver_pk(),
            }
        }

        pub fn nonce(&self) -> Nonce {
            self.payload.common.nonce
        }

        pub fn fee_excess(&self) -> FeeExcess {
            FeeExcess::of_single((self.fee_token(), Signed::<Fee>::of_unsigned(self.fee())))
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/signed_command_payload.ml#L354
        pub fn accounts_accessed(&self, status: TransactionStatus) -> Vec<AccountId> {
            use TransactionStatus::*;

            match status {
                Applied => {
                    vec![self.fee_payer(), self.source(), self.receiver()]
                }
                Failed(_) => vec![self.fee_payer()],
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/signed_command.ml#L401
        pub fn public_keys(&self) -> [&CompressedPubKey; 3] {
            [self.fee_payer_pk(), self.source_pk(), self.receiver_pk()]
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/signed_command.ml#L407
        pub fn check_valid_keys(&self) -> bool {
            self.public_keys()
                .into_iter()
                .all(|pk| decompress_pk(pk).is_some())
        }
    }
}

pub mod zkapp_command {
    use ark_ff::Zero;
    use mina_p2p_messages::v2::{
        MinaBaseAccountUpdateTWireStableV1,
        MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA,
    };
    use static_assertions::assert_eq_size_val;

    use crate::{
        hash_noinputs, hash_with_kimchi,
        scan_state::{
            conv::AsAccountUpdateWithHash,
            currency::{Balance, Signed},
        },
        AuthRequired, Inputs, MyCow, Permissions, ToInputs, TokenSymbol, VerificationKey, ZkAppUri,
    };

    use super::*;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L834
    #[derive(Debug, Clone)]
    pub struct Events(pub Vec<Vec<Fp>>);

    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L155
    #[derive(Debug, Clone)]
    pub struct SequenceEvents(pub Events);

    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L23
    trait MakeEvents {
        const SALT_PHRASE: &'static str;
        const HASH_PREFIX: &'static str;
        const DERIVER_NAME: (); // Unused here for now

        fn events(&self) -> &Events;
    }

    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L100
    impl MakeEvents for Events {
        const SALT_PHRASE: &'static str = "MinaZkappEventsEmpty";
        const HASH_PREFIX: &'static str = "MinaZkappEvents";
        const DERIVER_NAME: () = ();
        fn events(&self) -> &Events {
            self
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L156
    impl MakeEvents for SequenceEvents {
        const SALT_PHRASE: &'static str = "MinaZkappSequenceEmpty";
        const HASH_PREFIX: &'static str = "MinaZkappSeqEvents";
        const DERIVER_NAME: () = ();
        fn events(&self) -> &Events {
            &self.0
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L52
    fn events_to_inputs<E>(e: &E, inputs: &mut Inputs)
    where
        E: MakeEvents,
    {
        fn hash_event(event: &[Fp]) -> Fp {
            hash_with_kimchi("MinaZkappEvent", event)
        }

        let init = hash_noinputs(E::SALT_PHRASE);

        let field = e.events().0.iter().rfold(init, |accum, elem| {
            hash_with_kimchi(E::HASH_PREFIX, &[accum, hash_event(elem)])
        });

        inputs.append_field(field);
    }

    impl ToInputs for Events {
        fn to_inputs(&self, inputs: &mut Inputs) {
            events_to_inputs(self, inputs);
        }
    }

    impl ToInputs for SequenceEvents {
        fn to_inputs(&self, inputs: &mut Inputs) {
            events_to_inputs(self, inputs);
        }
    }

    /// Note: It's a different one than in the normal `Account`
    ///
    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L163
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Timing {
        pub initial_minimum_balance: Balance,
        pub cliff_time: Slot,
        pub cliff_amount: Amount,
        pub vesting_period: Slot,
        pub vesting_increment: Amount,
    }

    impl Timing {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L208
        fn dummy() -> Self {
            Self {
                initial_minimum_balance: Balance::zero(),
                cliff_time: Slot::zero(),
                cliff_amount: Amount::zero(),
                vesting_period: Slot::zero(),
                vesting_increment: Amount::zero(),
            }
        }
    }

    impl ToInputs for Timing {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L199
        fn to_inputs(&self, inputs: &mut Inputs) {
            let Timing {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } = self;

            inputs.append_u64(initial_minimum_balance.as_u64());
            inputs.append_u32(cliff_time.as_u32());
            inputs.append_u64(cliff_amount.as_u64());
            inputs.append_u32(vesting_period.as_u32());
            inputs.append_u64(vesting_increment.as_u64());
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_basic.ml#L100
    #[derive(Debug, Clone)]
    pub enum SetOrKeep<T> {
        Set(T),
        Keep,
    }

    impl<T> SetOrKeep<T> {
        fn map<'a, F, U>(&'a self, fun: F) -> SetOrKeep<U>
        where
            F: FnOnce(&'a T) -> U,
        {
            match self {
                SetOrKeep::Set(v) => SetOrKeep::Set(fun(v)),
                SetOrKeep::Keep => SetOrKeep::Keep,
            }
        }
    }

    impl<T, F> ToInputs for (&SetOrKeep<T>, F)
    where
        T: ToInputs,
        F: Fn() -> T,
    {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_basic.ml#L223
        fn to_inputs(&self, inputs: &mut Inputs) {
            match &self.0 {
                SetOrKeep::Set(this) => {
                    inputs.append_bool(true);
                    this.to_inputs(inputs);
                }
                SetOrKeep::Keep => {
                    inputs.append_bool(false);
                    let default = self.1();
                    default.to_inputs(inputs);
                }
            };
        }
    }

    #[derive(Debug, Clone)]
    pub struct WithHash<T> {
        pub data: T,
        pub hash: Fp,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L319
    #[derive(Debug, Clone)]
    pub struct Update {
        pub app_state: [SetOrKeep<Fp>; 8],
        pub delegate: SetOrKeep<CompressedPubKey>,
        pub verification_key: SetOrKeep<WithHash<VerificationKey>>,
        pub permissions: SetOrKeep<Permissions<AuthRequired>>,
        pub zkapp_uri: SetOrKeep<ZkAppUri>,
        pub token_symbol: SetOrKeep<TokenSymbol>,
        pub timing: SetOrKeep<Timing>,
        pub voting_for: SetOrKeep<Fp>,
    }

    // TODO: This could be std::ops::Range ?
    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L23
    #[derive(Debug, Clone)]
    pub struct ClosedInterval<T> {
        pub lower: T,
        pub upper: T,
    }

    impl<T> ToInputs for ClosedInterval<T>
    where
        T: ToInputs,
    {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_precondition.ml#L37
        fn to_inputs(&self, inputs: &mut Inputs) {
            let ClosedInterval { lower, upper } = self;

            lower.to_inputs(inputs);
            upper.to_inputs(inputs);
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_basic.ml#L232
    #[derive(Debug, Clone)]
    pub enum OrIgnore<T> {
        Check(T),
        Ignore,
    }

    impl<T, F> ToInputs for (&OrIgnore<T>, F)
    where
        T: ToInputs,
        F: Fn() -> T,
    {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_precondition.ml#L414
        fn to_inputs(&self, inputs: &mut Inputs) {
            match &self.0 {
                OrIgnore::Check(this) => {
                    inputs.append_bool(true);
                    this.to_inputs(inputs);
                }
                OrIgnore::Ignore => {
                    inputs.append_bool(false);
                    let default = self.1();
                    default.to_inputs(inputs);
                }
            };
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L439
    pub type Hash<T> = OrIgnore<T>;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L298
    pub type EqData<T> = OrIgnore<T>;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L178
    pub type Numeric<T> = OrIgnore<ClosedInterval<T>>;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/block_time/intf.ml#L55
    // TODO: Not sure if it's `u64`, but OCaml has methods `of_int64` and `to_in64`
    #[derive(Debug, Clone)]
    pub struct BlockTime(pub(super) u64);

    impl ToInputs for BlockTime {
        fn to_inputs(&self, inputs: &mut Inputs) {
            inputs.append_u64(self.0);
        }
    }

    impl BlockTime {
        pub fn from_u64(n: u64) -> Self {
            Self(n)
        }

        pub fn as_u64(&self) -> u64 {
            self.0
        }

        fn min() -> Self {
            Self(0)
        }

        fn max() -> Self {
            Self(u64::MAX)
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_numbers/length.mli#L2
    #[derive(Debug, Clone)]
    pub struct Length(pub(super) u32);

    impl ToInputs for Length {
        fn to_inputs(&self, inputs: &mut Inputs) {
            inputs.append_u32(self.0);
        }
    }

    impl Length {
        pub fn from_u32(n: u32) -> Self {
            Self(n)
        }

        pub fn as_u32(&self) -> u32 {
            self.0
        }

        fn min() -> Self {
            Self(0)
        }

        fn max() -> Self {
            Self(u32::MAX)
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/epoch_ledger.ml#L9
    #[derive(Debug, Clone)]
    pub struct EpochLedger {
        pub hash: Hash<Fp>,
        pub total_currency: Numeric<Amount>,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L797
    #[derive(Debug, Clone)]
    pub struct EpochData {
        pub(crate) ledger: EpochLedger,
        pub seed: Hash<Fp>,
        pub start_checkpoint: Hash<Fp>,
        pub lock_checkpoint: Hash<Fp>,
        pub epoch_length: Numeric<Length>,
    }

    impl ToInputs for EpochData {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_precondition.ml#L875
        fn to_inputs(&self, inputs: &mut Inputs) {
            let EpochData {
                ledger,
                seed,
                start_checkpoint,
                lock_checkpoint,
                epoch_length,
            } = self;

            {
                let EpochLedger {
                    hash,
                    total_currency,
                } = ledger;

                inputs.append(&(hash, Fp::zero));
                inputs.append(&(total_currency, || ClosedInterval {
                    lower: Amount::min(),
                    upper: Amount::max(),
                }));
            }

            inputs.append(&(seed, Fp::zero));
            inputs.append(&(start_checkpoint, Fp::zero));
            inputs.append(&(lock_checkpoint, Fp::zero));
            inputs.append(&(epoch_length, || ClosedInterval {
                lower: Length::min(),
                upper: Length::max(),
            }));
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L977
    #[derive(Debug, Clone)]
    pub struct ZkAppPreconditions {
        pub snarked_ledger_hash: Hash<Fp>,
        pub timestamp: Numeric<BlockTime>,
        pub blockchain_length: Numeric<Length>,
        pub min_window_density: Numeric<Length>,
        pub last_vrf_output: (), // It's not defined in OCAml
        pub total_currency: Numeric<Amount>,
        pub global_slot_since_hard_fork: Numeric<Slot>,
        pub global_slot_since_genesis: Numeric<Slot>,
        pub staking_epoch_data: EpochData,
        pub next_epoch_data: EpochData,
    }

    impl ToInputs for ZkAppPreconditions {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_precondition.ml#L1052
        fn to_inputs(&self, inputs: &mut Inputs) {
            let ZkAppPreconditions {
                snarked_ledger_hash,
                timestamp,
                blockchain_length,
                min_window_density,
                last_vrf_output,
                total_currency,
                global_slot_since_hard_fork,
                global_slot_since_genesis,
                staking_epoch_data,
                next_epoch_data,
            } = &self;

            assert_eq_size_val!(*last_vrf_output, ());

            let default_length = || ClosedInterval {
                lower: Length::min(),
                upper: Length::max(),
            };

            let default_slot = || ClosedInterval {
                lower: Slot::min(),
                upper: Slot::max(),
            };

            inputs.append(&(snarked_ledger_hash, Fp::zero));
            inputs.append(&(timestamp, || ClosedInterval {
                lower: BlockTime::min(),
                upper: BlockTime::max(),
            }));

            inputs.append(&(blockchain_length, default_length));
            inputs.append(&(min_window_density, default_length));

            inputs.append(&(total_currency, || ClosedInterval {
                lower: Amount::min(),
                upper: Amount::max(),
            }));

            inputs.append(&(global_slot_since_hard_fork, default_slot));
            inputs.append(&(global_slot_since_genesis, default_slot));

            inputs.append(staking_epoch_data);
            inputs.append(next_epoch_data);
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_numbers/account_nonce.mli#L2
    #[derive(Copy, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Nonce(pub(super) u32);

    impl ToInputs for Nonce {
        fn to_inputs(&self, inputs: &mut Inputs) {
            inputs.append_u32(self.0);
        }
    }

    impl Nonce {
        pub fn is_zero(&self) -> bool {
            self.0 == 0
        }

        pub fn zero() -> Self {
            Self(0)
        }

        pub fn as_u32(&self) -> u32 {
            self.0
        }

        pub fn from_u32(nonce: u32) -> Self {
            Self(nonce)
        }

        // TODO: Not sure if OCaml wraps around here
        pub fn incr(&self) -> Self {
            Self(self.0.wrapping_add(1))
        }

        pub fn min() -> Self {
            Self(0)
        }

        pub fn max() -> Self {
            Self(u32::MAX)
        }
    }

    impl rand::distributions::Distribution<Nonce> for rand::distributions::Standard {
        fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Nonce {
            Nonce(rng.next_u32())
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L478
    #[derive(Debug, Clone)]
    pub struct Account {
        pub balance: Numeric<Balance>,
        pub nonce: Numeric<Nonce>,
        pub receipt_chain_hash: Hash<Fp>, // TODO: Should be type `ReceiptChainHash`
        pub delegate: EqData<CompressedPubKey>,
        pub state: [EqData<Fp>; 8],
        pub sequence_state: EqData<Fp>,
        pub proved_state: EqData<bool>,
        pub is_new: EqData<bool>,
    }

    impl Account {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_precondition.ml#L525
        pub fn accept() -> Self {
            Self {
                balance: Numeric::Ignore,
                nonce: Numeric::Ignore,
                receipt_chain_hash: Hash::Ignore,
                delegate: EqData::Ignore,
                state: std::array::from_fn(|_| EqData::Ignore),
                sequence_state: EqData::Ignore,
                proved_state: EqData::Ignore,
                is_new: EqData::Ignore,
            }
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L613
    #[derive(Debug, Clone)]
    pub enum AccountPreconditions {
        Full(Box<Account>),
        Nonce(Nonce),
        Accept,
    }

    impl ToInputs for AccountPreconditions {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L635
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_precondition.ml#L568
        fn to_inputs(&self, inputs: &mut Inputs) {
            let account = match self {
                AccountPreconditions::Full(account) => MyCow::Borrow(&**account),
                AccountPreconditions::Nonce(nonce) => {
                    let mut account = Account::accept();
                    account.nonce = Numeric::Check(ClosedInterval {
                        lower: *nonce,
                        upper: *nonce,
                    });
                    MyCow::Own(account)
                }
                AccountPreconditions::Accept => MyCow::Own(Account::accept()),
            };

            let Account {
                balance,
                nonce,
                receipt_chain_hash,
                delegate,
                state,
                sequence_state,
                proved_state,
                is_new,
            } = account.as_ref();

            inputs.append(&(balance, || ClosedInterval {
                lower: Balance::min(),
                upper: Balance::max(),
            }));

            inputs.append(&(nonce, || ClosedInterval {
                lower: Nonce::min(),
                upper: Nonce::max(),
            }));

            inputs.append(&(receipt_chain_hash, Fp::zero));
            inputs.append(&(delegate, CompressedPubKey::empty));

            state.iter().for_each(|s| {
                inputs.append(&(s, Fp::zero));
            });

            // https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L168
            inputs.append(&(sequence_state, || {
                hash_noinputs("MinaZkappSequenceStateEmptyElt")
            }));

            inputs.append(&(proved_state, || false));
            inputs.append(&(is_new, || false));
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L758
    #[derive(Debug, Clone)]
    pub struct Preconditions {
        pub(crate) network: ZkAppPreconditions,
        pub account: AccountPreconditions,
    }

    impl ToInputs for Preconditions {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L776
        fn to_inputs(&self, inputs: &mut Inputs) {
            let Self { network, account } = self;
            network.to_inputs(inputs);
            account.to_inputs(inputs);
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L27
    #[derive(Debug, Clone)]
    pub enum AuthorizationKind {
        NoneGiven,
        Signature,
        Proof,
    }

    impl ToInputs for AuthorizationKind {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L110
        fn to_inputs(&self, inputs: &mut Inputs) {
            // bits: [is_signed, is_proved]
            let bits = match self {
                AuthorizationKind::NoneGiven => [false, false],
                AuthorizationKind::Signature => [true, false],
                AuthorizationKind::Proof => [false, true],
            };

            for bit in bits {
                inputs.append_bool(bit);
            }
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L955
    #[derive(Debug, Clone)]
    pub struct Body {
        pub public_key: CompressedPubKey,
        pub token_id: TokenId,
        pub update: Update,
        pub balance_change: Signed<Amount>,
        pub increment_nonce: bool,
        pub events: Events,
        pub sequence_events: SequenceEvents,
        pub call_data: Fp,
        pub(crate) preconditions: Preconditions,
        pub use_full_commitment: bool,
        pub caller: TokenId,
        pub authorization_kind: AuthorizationKind,
    }

    /// Notes:
    /// The type in OCaml is this one:
    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/pickles/proof.ml#L401
    ///
    /// For now we use the type from `mina_p2p_messages`, but we need to use our own.
    /// Lots of inner types are (BigInt, Bigint) which should be replaced with `Pallas<_>` etc.
    /// Also, in OCaml it has custom `{to/from}_binable` implementation.
    ///
    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/pickles/pickles_intf.ml#L316
    pub type SideLoadedProof = mina_p2p_messages::v2::PicklesProofProofsVerifiedMaxStableV2;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/control.ml#L11
    #[derive(Debug, Clone)]
    pub enum Control {
        Proof(SideLoadedProof),
        Signature(Signature),
        NoneGiven,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L1437
    #[derive(Debug, Clone)]
    pub struct AccountUpdate {
        pub(crate) body: Body,
        pub authorization: Control,
    }

    impl ToInputs for AccountUpdate {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L1297
        fn to_inputs(&self, inputs: &mut Inputs) {
            // Only the body is used
            let Self {
                body,
                authorization: _,
            } = self;

            let Body {
                public_key,
                token_id,
                update,
                balance_change,
                increment_nonce,
                events,
                sequence_events,
                call_data,
                preconditions,
                use_full_commitment,
                caller,
                authorization_kind,
            } = body;

            inputs.append(public_key);
            inputs.append(token_id);

            // `Body::update`
            {
                let Update {
                    app_state,
                    delegate,
                    verification_key,
                    permissions,
                    zkapp_uri,
                    token_symbol,
                    timing,
                    voting_for,
                } = update;

                for state in app_state {
                    inputs.append(&(state, Fp::zero));
                }

                inputs.append(&(delegate, CompressedPubKey::empty));
                inputs.append(&(&verification_key.map(|w| w.hash), Fp::zero));
                inputs.append(&(permissions, Permissions::user_default));
                inputs.append(&(&zkapp_uri.map(Some), || Option::<&ZkAppUri>::None));
                inputs.append(&(token_symbol, TokenSymbol::default));
                inputs.append(&(timing, Timing::dummy));
                inputs.append(&(voting_for, Fp::zero));
            }

            inputs.append(balance_change);
            inputs.append(increment_nonce);
            inputs.append(events);
            inputs.append(sequence_events);
            inputs.append(call_data);
            inputs.append(preconditions);
            inputs.append(use_full_commitment);
            inputs.append(caller);
            inputs.append(authorization_kind);
        }
    }

    impl AccountUpdate {
        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/account_update.ml#L1535
        pub fn account_id(&self) -> AccountId {
            AccountId::new(self.body.public_key.clone(), self.body.token_id.clone())
        }

        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L1327
        pub fn digest(&self) -> Fp {
            let mut inputs = Inputs::new();

            self.to_inputs(&mut inputs);
            hash_with_kimchi("MinaZkappBody", &inputs.to_fields())
        }
    }

    // Digest.Account_update.Stable.V1.t = Fp
    // Digest.Forest.Stable.V1.t = Fp

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L49
    #[derive(Debug, Clone)]
    pub struct Tree<Data> {
        pub account_update: (AccountUpdate, Data),
        pub account_update_digest: Fp,
        pub calls: CallForest<Data>,
    }

    impl<Data> Tree<Data> {
        fn digest(&self) -> Fp {
            let stack_hash = match self.calls.0.first() {
                Some(e) => e.stack_hash,
                None => Fp::zero(),
            };

            // self.account_update_digest should have been updated in `CallForest::accumulate_hashes`
            assert_ne!(self.account_update_digest, Fp::zero());

            hash_with_kimchi(
                "MinaAcctUpdateNode",
                &[self.account_update_digest, stack_hash],
            )
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/with_stack_hash.ml#L6
    #[derive(Debug, Clone)]
    pub struct WithStackHash<Data> {
        pub elt: Tree<Data>,
        pub stack_hash: Fp,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L345
    #[derive(Debug, Clone)]
    pub struct CallForest<Data>(pub Vec<WithStackHash<Data>>);

    #[derive(Clone)]
    struct CallForestContext {
        caller: TokenId,
        this: TokenId,
    }

    impl<Data> CallForest<Data> {
        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/zkapp_command.ml#L68
        fn fold_impl<A, F>(&self, init: A, fun: &mut F) -> A
        where
            F: FnMut(A, &(AccountUpdate, Data)) -> A,
        {
            let mut accum = init;
            for elem in &self.0 {
                accum = fun(accum, &elem.elt.account_update);
                accum = elem.elt.calls.fold_impl(accum, fun);
            }
            accum
        }

        pub fn fold<A, F>(&self, init: A, mut fun: F) -> A
        where
            F: FnMut(A, &(AccountUpdate, Data)) -> A,
        {
            self.fold_impl(init, &mut fun)
        }

        fn map_to_impl<F, VK>(&self, fun: &F) -> CallForest<VK>
        where
            F: Fn(&(AccountUpdate, Data)) -> (AccountUpdate, VK),
        {
            CallForest::<VK>(
                self.0
                    .iter()
                    .map(|item| WithStackHash::<VK> {
                        elt: Tree::<VK> {
                            account_update: fun(&item.elt.account_update),
                            account_update_digest: item.elt.account_update_digest,
                            calls: item.elt.calls.map_to_impl(fun),
                        },
                        stack_hash: item.stack_hash,
                    })
                    .collect(),
            )
        }

        pub fn map_to<F, VK>(&self, fun: F) -> CallForest<VK>
        where
            F: Fn(&(AccountUpdate, Data)) -> (AccountUpdate, VK),
        {
            self.map_to_impl(&fun)
        }

        fn add_callers_impl<F, Update>(
            &mut self,
            wired: &[Update],
            current_context: CallForestContext,
            account_update_id: &F,
        ) where
            Update: AsAccountUpdateWithHash,
            F: Fn(&MinaBaseAccountUpdateTWireStableV1) -> TokenId,
        {
            use mina_p2p_messages::v2::MinaBaseAccountUpdateCallTypeStableV1::{
                Call, DelegateCall,
            };

            assert_eq!(self.0.len(), wired.len());

            self.0.iter_mut().zip(wired).for_each(|(elem, wired)| {
                let WithStackHash {
                    elt:
                        Tree::<Data> {
                            account_update,
                            calls,
                            ..
                        },
                    ..
                } = elem;

                let child_context = match &wired.elt().account_update.body.caller {
                    DelegateCall => current_context.clone(),
                    Call => CallForestContext {
                        caller: current_context.this.clone(),
                        this: account_update_id(&wired.elt().account_update),
                    },
                };

                account_update.0.body.caller = child_context.caller.clone();
                calls.add_callers_impl(&wired.elt().calls, child_context, account_update_id);
            });
        }

        /// Delegate_call means, preserve the current caller.
        ///
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_command.ml#L616
        pub fn add_callers<F>(
            &mut self,
            wired: &[MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA],
            null_id: TokenId,
            account_update_id: F,
        ) where
            F: Fn(&MinaBaseAccountUpdateTWireStableV1) -> TokenId,
        {
            let current_context = CallForestContext {
                caller: null_id.clone(),
                this: null_id,
            };

            self.add_callers_impl(wired, current_context, &account_update_id);
        }

        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_command.ml#L583
        pub fn accumulate_hashes<F>(&mut self, hash_account_update: &F)
        where
            F: Fn(&AccountUpdate) -> Fp,
        {
            /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_command.ml#L293
            fn cons(hash: Fp, h_tl: Fp) -> Fp {
                hash_with_kimchi("MinaAcctUpdateCons", &[hash, h_tl])
            }

            /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_command.ml#L561
            fn hash<T>(elem: Option<&WithStackHash<T>>) -> Fp {
                match elem {
                    Some(next) => next.stack_hash,
                    None => Fp::zero(),
                }
            }

            // We traverse the list in reverse here (to get same behavior as OCaml recursivity)
            // We use indexes to make the borrow checker happy

            for index in (0..self.0.len()).rev() {
                let elem = &mut self.0[index];
                let WithStackHash {
                    elt:
                        Tree::<Data> {
                            account_update,
                            account_update_digest,
                            calls,
                            ..
                        },
                    ..
                } = elem;

                calls.accumulate_hashes(hash_account_update);
                *account_update_digest = hash_account_update(&account_update.0);

                let node_hash = elem.elt.digest();
                let hash = hash(self.0.get(index + 1));

                self.0[index].stack_hash = cons(node_hash, hash);
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_command.ml#L1079
        pub fn of_wire(
            &mut self,
            wired: &[MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA],
        ) {
            self.add_callers(wired, TokenId::default(), |wired_update| {
                let public_key: CompressedPubKey = (&wired_update.body.public_key).into();
                let token_id: TokenId = (&*wired_update.body.token_id).into();

                AccountId::new(public_key, token_id).derive_token_id()
            });

            self.accumulate_hashes(&|account_update| account_update.digest());
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L1081
    #[derive(Debug, Clone)]
    pub struct FeePayerBody {
        pub public_key: CompressedPubKey,
        pub fee: Fee,
        pub valid_until: Option<Slot>,
        pub nonce: Nonce,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L1484
    #[derive(Debug, Clone)]
    pub struct FeePayer {
        pub body: FeePayerBody,
        pub authorization: Signature,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L959
    #[derive(Debug, Clone)]
    pub struct ZkAppCommand {
        pub fee_payer: FeePayer,
        pub account_updates: CallForest<()>,
        pub memo: Memo,
    }

    impl ZkAppCommand {
        pub fn fee_payer(&self) -> AccountId {
            let public_key = self.fee_payer.body.public_key.clone();
            AccountId::new(public_key, self.fee_token())
        }

        pub fn fee_token(&self) -> TokenId {
            TokenId::default()
        }

        pub fn fee(&self) -> Fee {
            self.fee_payer.body.fee
        }

        pub fn fee_excess(&self) -> FeeExcess {
            FeeExcess::of_single((self.fee_token(), Signed::<Fee>::of_unsigned(self.fee())))
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/zkapp_command.ml#L1241
        pub fn accounts_accessed(&self, status: TransactionStatus) -> Vec<AccountId> {
            use TransactionStatus::*;

            match status {
                Applied => {
                    let mut ids = self.account_updates.fold(
                        Vec::with_capacity(256),
                        |mut accum, (account_update, _)| {
                            accum.push(account_update.account_id());
                            accum
                        },
                    );
                    ids.dedup(); // TODO In Rust it should be sorted for `dedup` to work. Find a solution to this
                    ids
                }
                Failed(_) => vec![self.fee_payer()],
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/zkapp_command.ml#L1251
        pub fn accounts_referenced(&self) -> Vec<AccountId> {
            self.accounts_accessed(TransactionStatus::Applied)
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/zkapp_command.ml#L1346
        pub fn of_verifiable(verifiable: verifiable::ZkAppCommand) -> Self {
            Self {
                fee_payer: verifiable.fee_payer,
                account_updates: verifiable
                    .account_updates
                    .map_to(|(acc, _)| (acc.clone(), ())),
                memo: verifiable.memo,
            }
        }
    }

    pub mod verifiable {
        use super::*;
        use crate::{scan_state::transaction_logic::valid::VerificationKeyHash, VerificationKey};

        #[derive(Debug, Clone)]
        pub struct ZkAppCommand {
            pub fee_payer: FeePayer,
            pub account_updates: CallForest<Option<(VerificationKey, VerificationKeyHash)>>,
            pub memo: Memo,
        }
    }

    pub mod valid {
        use std::collections::HashMap;

        use crate::scan_state::transaction_logic::valid::VerificationKeyHash;

        use super::*;

        #[derive(Clone, Debug)]
        pub struct ZkAppCommand {
            pub zkapp_command: super::ZkAppCommand,
            pub verification_keys: Vec<(AccountId, VerificationKeyHash)>,
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/zkapp_command.ml#L1486
        pub fn of_verifiable(cmd: verifiable::ZkAppCommand) -> Option<ZkAppCommand> {
            use AuthorizationKind as AK;
            use Control as C;

            let mut keys = HashMap::with_capacity(256);

            cmd.account_updates.fold(Some(()), |accum, (p, vk_opt)| {
                accum?;

                match (&p.authorization, &p.body.authorization_kind) {
                    (C::NoneGiven, AK::NoneGiven)
                    | (C::Proof(_), AK::Proof)
                    | (C::Signature(_), AK::Signature) => {}
                    _ => return None,
                }

                if let C::Proof(_) = &p.authorization {
                    let (_, hash) = vk_opt.as_ref()?;
                    keys.insert(p.account_id(), hash.clone());
                };
                Some(())
            })?;

            Some(ZkAppCommand {
                zkapp_command: super::ZkAppCommand::of_verifiable(cmd),
                verification_keys: keys.into_iter().collect(),
            })
        }
    }
}

pub mod verifiable {
    use super::*;

    #[derive(Debug)]
    pub enum UserCommand {
        SignedCommand(Box<signed_command::SignedCommand>),
        ZkAppCommand(Box<zkapp_command::verifiable::ZkAppCommand>),
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/signed_command.ml#L436
    pub fn check_only_for_signature(cmd: Box<signed_command::SignedCommand>) -> valid::UserCommand {
        // TODO implement actual verification
        // https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/signed_command.ml#L396

        valid::UserCommand::SignedCommand(cmd)
    }
}

#[derive(Clone, Debug)]
pub enum UserCommand {
    SignedCommand(Box<signed_command::SignedCommand>),
    ZkAppCommand(Box<zkapp_command::ZkAppCommand>),
}

impl UserCommand {
    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/user_command.ml#L203
    pub fn accounts_accessed(&self, status: TransactionStatus) -> Vec<AccountId> {
        match self {
            UserCommand::SignedCommand(cmd) => cmd.accounts_accessed(status),
            UserCommand::ZkAppCommand(cmd) => cmd.accounts_accessed(status),
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/user_command.ml#L210
    pub fn accounts_referenced(&self) -> Vec<AccountId> {
        self.accounts_accessed(TransactionStatus::Applied)
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/user_command.ml#L192
    pub fn fee(&self) -> Fee {
        match self {
            UserCommand::SignedCommand(cmd) => cmd.fee(),
            UserCommand::ZkAppCommand(cmd) => cmd.fee(),
        }
    }

    pub fn to_verifiable(&self, ledger: &impl BaseLedger) -> verifiable::UserCommand {
        let find_vk = |acc: &zkapp_command::AccountUpdate| -> Option<VerificationKey> {
            let account_id = acc.account_id();
            let addr = ledger.location_of_account(&account_id)?;
            let account = ledger.get(addr)?;
            account.zkapp.as_ref()?.verification_key.clone()
        };

        match self {
            UserCommand::SignedCommand(cmd) => verifiable::UserCommand::SignedCommand(cmd.clone()),
            UserCommand::ZkAppCommand(cmd) => {
                let zkapp_command::ZkAppCommand {
                    fee_payer,
                    account_updates,
                    memo,
                } = &**cmd;

                let zkapp = zkapp_command::verifiable::ZkAppCommand {
                    fee_payer: fee_payer.clone(),
                    account_updates: account_updates.map_to(|(account_update, _)| {
                        let with_hash = find_vk(account_update).map(|vk| {
                            let hash = vk.hash();
                            (vk, VerificationKeyHash(hash))
                        });

                        (account_update.clone(), with_hash)
                    }),
                    memo: memo.clone(),
                };

                verifiable::UserCommand::ZkAppCommand(Box::new(zkapp))
            }
        }
    }
}

impl GenericCommand for UserCommand {
    fn fee(&self) -> Fee {
        match self {
            UserCommand::SignedCommand(cmd) => cmd.fee(),
            UserCommand::ZkAppCommand(cmd) => cmd.fee(),
        }
    }

    fn forget(&self) -> UserCommand {
        self.clone()
    }
}

impl GenericTransaction for Transaction {
    fn is_fee_transfer(&self) -> bool {
        matches!(self, Transaction::FeeTransfer(_))
    }
    fn is_coinbase(&self) -> bool {
        matches!(self, Transaction::Coinbase(_))
    }
    fn is_command(&self) -> bool {
        matches!(self, Transaction::Command(_))
    }
}

#[derive(Debug, derive_more::From)]
pub enum Transaction {
    Command(UserCommand),
    FeeTransfer(FeeTransfer),
    Coinbase(Coinbase),
}

impl Transaction {
    pub fn fee_excess(&self) -> Result<FeeExcess, String> {
        use Transaction::*;
        use UserCommand::*;

        match self {
            Command(SignedCommand(cmd)) => Ok(cmd.fee_excess()),
            Command(ZkAppCommand(cmd)) => Ok(cmd.fee_excess()),
            FeeTransfer(ft) => ft.fee_excess(),
            Coinbase(cb) => cb.fee_excess(),
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/transaction/transaction.ml#L85
    pub fn public_keys(&self) -> Vec<CompressedPubKey> {
        use Transaction::*;
        use UserCommand::*;

        let to_pks = |ids: Vec<AccountId>| ids.into_iter().map(|id| id.public_key).collect();

        match self {
            Command(SignedCommand(cmd)) => [cmd.fee_payer_pk(), cmd.source_pk(), cmd.receiver_pk()]
                .into_iter()
                .cloned()
                .collect(),
            Command(ZkAppCommand(cmd)) => to_pks(cmd.accounts_referenced()),
            FeeTransfer(ft) => ft.receiver_pks().cloned().collect(),
            Coinbase(cb) => to_pks(cb.accounts_accessed()),
        }
    }
}

pub mod transaction_applied {
    use crate::{Account, AccountId};

    use super::*;

    pub mod signed_command_applied {
        use super::*;

        #[derive(Debug, Clone)]
        pub struct Common {
            pub user_command: WithStatus<signed_command::SignedCommand>,
        }

        #[derive(Debug, Clone)]
        pub enum Body {
            Payments {
                new_accounts: Vec<AccountId>,
            },
            StakeDelegation {
                previous_delegate: Option<CompressedPubKey>,
            },
            Failed,
        }

        #[derive(Debug, Clone)]
        pub struct SignedCommandApplied {
            pub common: Common,
            pub body: Body,
        }
    }

    pub use signed_command_applied::SignedCommandApplied;

    impl SignedCommandApplied {
        pub fn new_accounts(&self) -> &[AccountId] {
            use signed_command_applied::Body::*;

            match &self.body {
                Payments { new_accounts } => new_accounts.as_slice(),
                StakeDelegation { .. } | Failed => &[],
            }
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L65
    #[derive(Debug, Clone)]
    pub struct ZkappCommandApplied {
        pub accounts: Vec<(AccountId, Option<Account>)>,
        pub command: WithStatus<zkapp_command::ZkAppCommand>,
        pub new_accounts: Vec<AccountId>,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L82
    #[derive(Debug, Clone)]
    pub enum CommandApplied {
        SignedCommand(Box<SignedCommandApplied>),
        ZkappCommand(Box<ZkappCommandApplied>),
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L96
    #[derive(Debug, Clone)]
    pub struct FeeTransferApplied {
        pub fee_transfer: WithStatus<FeeTransfer>,
        pub new_accounts: Vec<AccountId>,
        pub burned_tokens: Amount,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L112
    #[derive(Debug, Clone)]
    pub struct CoinbaseApplied {
        pub coinbase: WithStatus<Coinbase>,
        pub new_accounts: Vec<AccountId>,
        pub burned_tokens: Amount,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L142
    #[derive(Debug, Clone)]
    pub enum Varying {
        Command(CommandApplied),
        FeeTransfer(FeeTransferApplied),
        Coinbase(CoinbaseApplied),
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L142
    #[derive(Debug, Clone)]
    pub struct TransactionApplied {
        pub previous_hash: Fp,
        pub varying: Varying,
    }

    impl TransactionApplied {
        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L639
        pub fn transaction(&self) -> WithStatus<Transaction> {
            use CommandApplied::*;
            use Varying::*;

            match &self.varying {
                Command(SignedCommand(cmd)) => cmd
                    .common
                    .user_command
                    .map(|c| Transaction::Command(UserCommand::SignedCommand(Box::new(c.clone())))),
                Command(ZkappCommand(cmd)) => cmd
                    .command
                    .map(|c| Transaction::Command(UserCommand::ZkAppCommand(Box::new(c.clone())))),
                FeeTransfer(f) => f.fee_transfer.map(|f| Transaction::FeeTransfer(f.clone())),
                Coinbase(c) => c.coinbase.map(|c| Transaction::Coinbase(c.clone())),
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L662
        pub fn transaction_status(&self) -> &TransactionStatus {
            use CommandApplied::*;
            use Varying::*;

            match &self.varying {
                Command(SignedCommand(cmd)) => &cmd.common.user_command.status,
                Command(ZkappCommand(cmd)) => &cmd.command.status,
                FeeTransfer(f) => &f.fee_transfer.status,
                Coinbase(c) => &c.coinbase.status,
            }
        }

        pub fn burned_tokens(&self) -> Amount {
            match &self.varying {
                Varying::Command(_) => Amount::zero(),
                Varying::FeeTransfer(f) => f.burned_tokens,
                Varying::Coinbase(c) => c.burned_tokens,
            }
        }

        pub fn new_accounts(&self) -> &[AccountId] {
            use CommandApplied::*;
            use Varying::*;

            match &self.varying {
                Command(SignedCommand(cmd)) => cmd.new_accounts(),
                Command(ZkappCommand(cmd)) => cmd.new_accounts.as_slice(),
                FeeTransfer(f) => f.new_accounts.as_slice(),
                Coinbase(cb) => cb.new_accounts.as_slice(),
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/e5183ca1dde1c085b4c5d37d1d9987e24c294c32/src/lib/transaction_logic/mina_transaction_logic.ml#L176
        pub fn supply_increase(
            &self,
            constraint_constants: &ConstraintConstants,
        ) -> Result<Signed<Amount>, String> {
            let burned_tokens = Signed::<Amount>::of_unsigned(self.burned_tokens());

            let account_creation_fees = {
                let account_creation_fee_int = constraint_constants.account_creation_fee.as_u64();
                let num_accounts_created = self.new_accounts().len() as u64;

                // int type is OK, no danger of overflow
                let amount = account_creation_fee_int
                    .checked_mul(num_accounts_created)
                    .unwrap();
                Signed::<Amount>::of_unsigned(Amount::from_u64(amount))
            };

            let expected_supply_increase = match &self.varying {
                Varying::Coinbase(cb) => cb.coinbase.data.expected_supply_increase()?,
                _ => Amount::zero(),
            };
            let expected_supply_increase = Signed::<Amount>::of_unsigned(expected_supply_increase);

            // TODO: Make sure it's correct
            let total = [burned_tokens, account_creation_fees]
                .into_iter()
                .fold(Some(expected_supply_increase), |total, amt| {
                    amt.negate().add(&total?)
                });

            total.ok_or_else(|| "overflow".to_string())
        }
    }
}

pub mod transaction_witness {
    use mina_p2p_messages::v2::MinaStateProtocolStateBodyValueStableV2;

    use crate::scan_state::pending_coinbase::Stack;

    use super::*;

    #[derive(Debug)]
    pub struct TransactionWitness {
        pub transaction: Transaction,
        pub ledger: SparseLedger<AccountId, Account>,
        pub protocol_state_body: MinaStateProtocolStateBodyValueStableV2,
        pub init_stack: Stack,
        pub status: TransactionStatus,
    }
}

pub mod protocol_state {
    use mina_p2p_messages::v2::MinaStateProtocolStateValueStableV2;

    use super::{
        zkapp_command::{BlockTime, Length},
        *,
    };

    #[derive(Debug, Clone)]
    pub struct EpochLedger {
        hash: Fp,
        total_currency: Amount,
    }

    #[derive(Debug, Clone)]
    pub struct EpochData {
        ledger: EpochLedger,
        seed: Fp,
        start_checkpoint: Fp,
        lock_checkpoint: Fp,
        epoch_length: Length,
    }

    #[derive(Debug, Clone)]
    pub struct ProtocolStateView {
        pub snarked_ledger_hash: Fp,
        pub timestamp: BlockTime,
        pub blockchain_length: Length,
        pub min_window_density: Length,
        pub last_vrf_output: (), // It's not defined in OCAml
        pub total_currency: Amount,
        pub global_slot_since_hard_fork: Slot,
        pub global_slot_since_genesis: Slot,
        pub staking_epoch_data: EpochData,
        pub next_epoch_data: EpochData,
    }

    pub fn protocol_state_view(state: &MinaStateProtocolStateValueStableV2) -> ProtocolStateView {
        let cs = &state.body.consensus_state;
        let sed = &cs.staking_epoch_data;
        let ned = &cs.staking_epoch_data;

        ProtocolStateView {
            snarked_ledger_hash: state.body.blockchain_state.registers.ledger.to_field(),
            timestamp: BlockTime(state.body.blockchain_state.timestamp.as_u64()),
            blockchain_length: Length(cs.blockchain_length.as_u32()),
            min_window_density: Length(cs.min_window_density.as_u32()),
            last_vrf_output: (),
            total_currency: Amount(cs.total_currency.as_u64()),
            global_slot_since_hard_fork: Slot(cs.curr_global_slot.slot_number.as_u32()), // TODO: Check if it's correct
            global_slot_since_genesis: Slot(cs.global_slot_since_genesis.as_u32()),
            staking_epoch_data: EpochData {
                ledger: EpochLedger {
                    hash: sed.ledger.hash.to_field(),
                    total_currency: Amount(sed.ledger.total_currency.as_u64()),
                },
                seed: sed.seed.to_field(),
                start_checkpoint: sed.start_checkpoint.to_field(),
                lock_checkpoint: sed.lock_checkpoint.to_field(),
                epoch_length: Length(sed.epoch_length.as_u32()),
            },
            next_epoch_data: EpochData {
                ledger: EpochLedger {
                    hash: ned.ledger.hash.to_field(),
                    total_currency: Amount(ned.ledger.total_currency.as_u64()),
                },
                seed: ned.seed.to_field(),
                start_checkpoint: ned.start_checkpoint.to_field(),
                lock_checkpoint: ned.lock_checkpoint.to_field(),
                epoch_length: Length(ned.epoch_length.as_u32()),
            },
        }
    }
}

pub mod local_state {
    use ark_ff::Zero;

    use crate::{hash_with_kimchi, scan_state::currency::Signed, Inputs};

    use super::{zkapp_command::CallForest, *};

    pub struct StackFrame {
        caller: TokenId,
        caller_caller: TokenId,
        calls: CallForest<()>, // TODO
    }

    impl StackFrame {
        pub fn empty() -> Self {
            Self {
                caller: TokenId::default(),
                caller_caller: TokenId::default(),
                calls: CallForest(Vec::new()),
            }
        }

        /// TODO: this needs to be tested
        ///
        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/stack_frame.ml#L90
        pub fn hash(&self) -> Fp {
            let mut inputs = Inputs::new();

            inputs.append_field(self.caller.0);
            inputs.append_field(self.caller_caller.0);

            let field = match self.calls.0.get(0) {
                None => Fp::zero(),
                Some(call) => call.stack_hash,
            };
            inputs.append_field(field);

            hash_with_kimchi("MinaAcctUpdStckFrm", &inputs.to_fields())
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct LocalState {
        pub stack_frame: Fp,
        pub call_stack: Fp,
        pub transaction_commitment: Fp,
        pub full_transaction_commitment: Fp,
        pub token_id: TokenId,
        pub excess: Signed<Amount>,
        pub supply_increase: Signed<Amount>,
        pub ledger: Fp,
        pub success: bool,
        pub account_update_index: Index,
        pub failure_status_tbl: Vec<Vec<TransactionFailure>>,
    }

    impl LocalState {
        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_state/local_state.ml#L63
        pub fn dummy() -> Self {
            Self {
                stack_frame: StackFrame::empty().hash(),
                call_stack: Fp::zero(),
                transaction_commitment: Fp::zero(),
                full_transaction_commitment: Fp::zero(),
                token_id: TokenId::default(),
                excess: Signed::<Amount>::zero(),
                supply_increase: Signed::<Amount>::zero(),
                ledger: Fp::zero(),
                success: true,
                account_update_index: Index(0),
                failure_status_tbl: Vec::new(),
            }
        }

        pub fn empty() -> Self {
            Self::dummy()
        }
    }
}

pub fn apply_transaction<L>(
    constraint_constants: &ConstraintConstants,
    txn_state_view: &ProtocolStateView,
    ledger: &mut L,
    transaction: &Transaction,
) -> Result<TransactionApplied, String>
where
    L: LedgerIntf,
{
    use Transaction::*;
    use UserCommand::*;

    let previous_hash = ledger.merkle_root();
    let txn_global_slot = &txn_state_view.global_slot_since_genesis;

    match transaction {
        Command(SignedCommand(_cmd)) => todo!(),
        Command(ZkAppCommand(_cmd)) => todo!(),
        FeeTransfer(fee_transfer) => {
            apply_fee_transfer(constraint_constants, txn_global_slot, ledger, fee_transfer)
                .map(Varying::FeeTransfer)
        }
        Coinbase(coinbase) => {
            apply_coinbase(constraint_constants, txn_global_slot, ledger, coinbase)
                .map(Varying::Coinbase)
        }
    }
    .map(|varying| TransactionApplied {
        previous_hash,
        varying,
    })
}

/// Structure of the failure status:
///  I. No fee transfer and coinbase transfer fails: [[failure]]
///  II. With fee transfer-
///   Both fee transfer and coinbase fails:
///     [[failure-of-fee-transfer]; [failure-of-coinbase]]
///   Fee transfer succeeds and coinbase fails:
///     [[];[failure-of-coinbase]]
///   Fee transfer fails and coinbase succeeds:
///     [[failure-of-fee-transfer];[]]
///
/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L2022
fn apply_coinbase<L>(
    constraint_constants: &ConstraintConstants,
    txn_global_slot: &Slot,
    ledger: &mut L,
    coinbase: &Coinbase,
) -> Result<transaction_applied::CoinbaseApplied, String>
where
    L: LedgerIntf,
{
    let Coinbase {
        receiver,
        amount: coinbase_amount,
        fee_transfer,
    } = &coinbase;

    let (
        receiver_reward,
        new_accounts1,
        transferee_update,
        transferee_timing_prev,
        failures1,
        burned_tokens1,
    ) = match fee_transfer {
        None => (*coinbase_amount, None, None, None, vec![], Amount::zero()),
        Some(
            ft @ CoinbaseFeeTransfer {
                receiver_pk: transferee,
                fee,
            },
        ) => {
            assert_ne!(transferee, receiver);

            let transferee_id = ft.receiver();
            let fee = Amount::of_fee(fee);

            let receiver_reward = coinbase_amount
                .checked_sub(&fee)
                .ok_or_else(|| "Coinbase fee transfer too large".to_string())?;

            let (transferee_account, action, can_receive) =
                has_permission_to_receive(ledger, &transferee_id);
            let new_accounts = get_new_accounts(action, transferee_id.clone());

            let timing = update_timing_when_no_deduction(txn_global_slot, &transferee_account)?;

            let balance = {
                let amount = sub_account_creation_fee(constraint_constants, action, fee)?;
                add_amount(transferee_account.balance, amount)?
            };

            if can_receive.0 {
                let (_, mut transferee_account, transferee_location) =
                    ledger.get_or_create(&transferee_id)?;

                transferee_account.balance = balance;
                transferee_account.timing = timing;

                let timing = transferee_account.timing.clone();

                (
                    receiver_reward,
                    new_accounts,
                    Some((transferee_location, transferee_account)),
                    Some(timing),
                    vec![],
                    Amount::zero(),
                )
            } else {
                (
                    receiver_reward,
                    None,
                    None,
                    None,
                    vec![TransactionFailure::UpdateNotPermittedBalance],
                    fee,
                )
            }
        }
    };

    let receiver_id = AccountId::new(receiver.clone(), TokenId::default());
    let (receiver_account, action2, can_receive) = has_permission_to_receive(ledger, &receiver_id);
    let new_accounts2 = get_new_accounts(action2, receiver_id.clone());

    // Note: Updating coinbase receiver timing only if there is no fee transfer.
    // This is so as to not add any extra constraints in transaction snark for checking
    // "receiver" timings. This is OK because timing rules will not be violated when
    // balance increases and will be checked whenever an amount is deducted from the
    // account (#5973)

    let coinbase_receiver_timing = match transferee_timing_prev {
        None => update_timing_when_no_deduction(txn_global_slot, &receiver_account)?,
        Some(_) => receiver_account.timing.clone(),
    };

    let receiver_balance = {
        let amount = sub_account_creation_fee(constraint_constants, action2, receiver_reward)?;
        add_amount(receiver_account.balance, amount)?
    };

    let (failures2, burned_tokens2) = if can_receive.0 {
        let (_action2, mut receiver_account, receiver_location) =
            ledger.get_or_create(&receiver_id)?;

        receiver_account.balance = receiver_balance;
        receiver_account.timing = coinbase_receiver_timing;

        ledger.set(&receiver_location, receiver_account);

        (vec![], Amount::zero())
    } else {
        (
            vec![TransactionFailure::UpdateNotPermittedBalance],
            receiver_reward,
        )
    };

    if let Some((addr, account)) = transferee_update {
        ledger.set(&addr, account);
    };

    let burned_tokens = burned_tokens1
        .checked_add(&burned_tokens2)
        .ok_or_else(|| "burned tokens overflow".to_string())?;

    let failures = vec![failures1, failures2];
    let status = if failures.iter().all(Vec::is_empty) {
        TransactionStatus::Applied
    } else {
        TransactionStatus::Failed(failures)
    };

    let new_accounts: Vec<_> = [new_accounts1, new_accounts2]
        .into_iter()
        .flatten()
        .collect();

    Ok(transaction_applied::CoinbaseApplied {
        coinbase: WithStatus {
            data: coinbase.clone(),
            status,
        },
        new_accounts,
        burned_tokens,
    })
}

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L1991
fn apply_fee_transfer<L>(
    constraint_constants: &ConstraintConstants,
    txn_global_slot: &Slot,
    ledger: &mut L,
    fee_transfer: &FeeTransfer,
) -> Result<transaction_applied::FeeTransferApplied, String>
where
    L: LedgerIntf,
{
    let (new_accounts, failures, burned_tokens) = process_fee_transfer(
        ledger,
        fee_transfer,
        |action, _, balance, fee| {
            let amount = {
                let amount = Amount::of_fee(fee);
                sub_account_creation_fee(constraint_constants, action, amount)?
            };
            add_amount(balance, amount)
        },
        |account| update_timing_when_no_deduction(txn_global_slot, account),
    )?;

    let status = if failures.iter().all(Vec::is_empty) {
        TransactionStatus::Applied
    } else {
        TransactionStatus::Failed(failures)
    };

    Ok(transaction_applied::FeeTransferApplied {
        fee_transfer: WithStatus {
            data: fee_transfer.clone(),
            status,
        },
        new_accounts,
        burned_tokens,
    })
}

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L607
fn sub_account_creation_fee(
    constraint_constants: &ConstraintConstants,
    action: AccountState,
    amount: Amount,
) -> Result<Amount, String> {
    let fee = &constraint_constants.account_creation_fee;

    match action {
        AccountState::Added => {
            if let Some(amount) = amount.checked_sub(&Amount::of_fee(fee)) {
                return Ok(amount);
            }
            Err(format!(
                "Error subtracting account creation fee {:?}; transaction amount {:?} insufficient",
                fee, amount
            ))
        }
        AccountState::Existed => Ok(amount),
    }
}

fn update_timing_when_no_deduction(
    txn_global_slot: &Slot,
    account: &Account,
) -> Result<Timing, String> {
    validate_timing(account, Amount::zero(), txn_global_slot)
}

// /// TODO: Move this to the ledger
// /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_ledger/ledger.ml#L311
// fn get_or_create<L>(
//     ledger: &mut L,
//     account_id: &AccountId,
// ) -> Result<(AccountState, Account, Address), String>
// where
//     L: LedgerIntf,
// {
//     let location = ledger
//         .get_or_create_account(account_id.clone(), Account::initialize(account_id))
//         .map_err(|e| format!("{:?}", e))?;

//     let action = match location {
//         GetOrCreated::Added(_) => AccountState::Added,
//         GetOrCreated::Existed(_) => AccountState::Existed,
//     };

//     let addr = location.addr();

//     let account = ledger
//         .get(addr.clone())
//         .expect("get_or_create: Account was not found in the ledger after creation");

//     Ok((action, account, addr))
// }

fn get_new_accounts<T>(action: AccountState, data: T) -> Option<T> {
    match action {
        AccountState::Added => Some(data),
        AccountState::Existed => None,
    }
}

/// Structure of the failure status:
///  I. Only one fee transfer in the transaction (`One) and it fails:
///     [[failure]]
///  II. Two fee transfers in the transaction (`Two)-
///   Both fee transfers fail:
///     [[failure-of-first-fee-transfer]; [failure-of-second-fee-transfer]]
///   First succeeds and second one fails:
///     [[];[failure-of-second-fee-transfer]]
///   First fails and second succeeds:
///     [[failure-of-first-fee-transfer];[]]
fn process_fee_transfer<L, FunBalance, FunTiming>(
    ledger: &mut L,
    fee_transfer: &FeeTransfer,
    modify_balance: FunBalance,
    modify_timing: FunTiming,
) -> Result<(Vec<AccountId>, Vec<Vec<TransactionFailure>>, Amount), String>
where
    L: LedgerIntf,
    FunTiming: Fn(&Account) -> Result<Timing, String>,
    FunBalance: Fn(AccountState, &AccountId, Balance, &Fee) -> Result<Balance, String>,
{
    if !fee_transfer.fee_tokens().all(TokenId::is_default) {
        return Err("Cannot pay fees in non-default tokens.".to_string());
    }

    match &**fee_transfer {
        OneOrTwo::One(fee_transfer) => {
            let account_id = fee_transfer.receiver();
            let (a, action, can_receive) = has_permission_to_receive(ledger, &account_id);

            let timing = modify_timing(&a)?;
            let balance = modify_balance(action, &account_id, a.balance, &fee_transfer.fee)?;

            if can_receive.0 {
                let (_, mut account, loc) = ledger.get_or_create(&account_id)?;
                let new_accounts = get_new_accounts(action, account_id.clone());

                account.balance = balance;
                account.timing = timing;

                ledger.set(&loc, account);

                let new_accounts: Vec<_> = new_accounts.into_iter().collect();
                Ok((new_accounts, vec![], Amount::zero()))
            } else {
                Ok((vec![], single_failure(), Amount::of_fee(&fee_transfer.fee)))
            }
        }
        OneOrTwo::Two((fee_transfer1, fee_transfer2)) => {
            let account_id1 = fee_transfer1.receiver();
            let (a1, action1, can_receive1) = has_permission_to_receive(ledger, &account_id1);

            let account_id2 = fee_transfer2.receiver();

            if account_id1 == account_id2 {
                let fee = fee_transfer1
                    .fee
                    .checked_add(&fee_transfer2.fee)
                    .ok_or_else(|| "Overflow".to_string())?;

                let timing = modify_timing(&a1)?;
                let balance = modify_balance(action1, &account_id1, a1.balance, &fee)?;

                if can_receive1.0 {
                    let (_, mut a1, l1) = ledger.get_or_create(&account_id1)?;
                    let new_accounts1 = get_new_accounts(action1, account_id1);

                    a1.balance = balance;
                    a1.timing = timing;

                    ledger.set(&l1, a1);

                    let new_accounts: Vec<_> = new_accounts1.into_iter().collect();
                    Ok((new_accounts, vec![vec![], vec![]], Amount::zero()))
                } else {
                    // failure for each fee transfer single

                    Ok((
                        vec![],
                        vec![
                            vec![TransactionFailure::UpdateNotPermittedBalance],
                            vec![TransactionFailure::UpdateNotPermittedBalance],
                        ],
                        Amount::of_fee(&fee),
                    ))
                }
            } else {
                let (a2, action2, can_receive2) = has_permission_to_receive(ledger, &account_id2);

                let balance1 =
                    modify_balance(action1, &account_id1, a1.balance, &fee_transfer1.fee)?;

                // Note: Not updating the timing field of a1 to avoid additional check
                // in transactions snark (check_timing for "receiver"). This is OK
                // because timing rules will not be violated when balance increases
                // and will be checked whenever an amount is deducted from the account. (#5973)*)

                let timing2 = modify_timing(&a2)?;
                let balance2 =
                    modify_balance(action2, &account_id2, a2.balance, &fee_transfer2.fee)?;

                let (new_accounts1, failures1, burned_tokens1) = if can_receive1.0 {
                    let (_, mut a1, l1) = ledger.get_or_create(&account_id1)?;
                    let new_accounts1 = get_new_accounts(action1, account_id1);

                    a1.balance = balance1;
                    ledger.set(&l1, a1);

                    (new_accounts1, vec![], Amount::zero())
                } else {
                    (
                        None,
                        vec![TransactionFailure::UpdateNotPermittedBalance],
                        Amount::of_fee(&fee_transfer1.fee),
                    )
                };

                let (new_accounts2, failures2, burned_tokens2) = if can_receive2.0 {
                    let (_, mut a2, l2) = ledger.get_or_create(&account_id2)?;
                    let new_accounts2 = get_new_accounts(action2, account_id2);

                    a2.balance = balance2;
                    a2.timing = timing2;

                    ledger.set(&l2, a2);

                    (new_accounts2, vec![], Amount::zero())
                } else {
                    (
                        None,
                        vec![TransactionFailure::UpdateNotPermittedBalance],
                        Amount::of_fee(&fee_transfer2.fee),
                    )
                };

                let burned_tokens = burned_tokens1
                    .checked_add(&burned_tokens2)
                    .ok_or_else(|| "burned tokens overflow".to_string())?;

                let new_accounts: Vec<_> = [new_accounts1, new_accounts2]
                    .into_iter()
                    .flatten()
                    .collect();
                let failures: Vec<_> = [failures1, failures2].into_iter().collect();

                Ok((new_accounts, failures, burned_tokens))
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AccountState {
    Added,
    Existed,
}

#[derive(Debug)]
struct HasPermissionToReceive(bool);

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L1852
fn has_permission_to_receive<L>(
    ledger: &mut L,
    receiver_account_id: &AccountId,
) -> (Account, AccountState, HasPermissionToReceive)
where
    L: LedgerIntf,
{
    use crate::PermissionTo::*;
    use AccountState::*;

    let init_account = Account::initialize(receiver_account_id);

    match ledger.location_of_account(receiver_account_id) {
        None => {
            // new account, check that default permissions allow receiving
            let perm = init_account.has_permission_to(Receive);
            (init_account, Added, HasPermissionToReceive(perm))
        }
        Some(location) => match ledger.get(&location) {
            None => panic!("Ledger location with no account"),
            Some(receiver_account) => {
                let perm = receiver_account.has_permission_to(Receive);
                (receiver_account, Existed, HasPermissionToReceive(perm))
            }
        },
    }
}

fn validate_time(valid_until: &Slot, current_global_slot: &Slot) -> Result<(), String> {
    if current_global_slot <= valid_until {
        return Ok(());
    }

    Err(format!(
        "Current global slot {:?} greater than transaction expiry slot {:?}",
        current_global_slot, valid_until
    ))
}

pub fn apply_user_command_unchecked<L>(
    _constraint_constants: &ConstraintConstants,
    _txn_state_view: &ProtocolStateView,
    txn_global_slot: &Slot,
    ledger: &mut L,
    user_command: SignedCommand,
) -> Result<(), String>
where
    L: LedgerIntf,
{
    let SignedCommand {
        payload: _,
        signer: signer_pk,
        signature: _,
    } = &user_command;
    let current_global_slot = txn_global_slot;

    let valid_until = user_command.valid_until();
    validate_time(&valid_until, current_global_slot)?;

    // Fee-payer information
    let _fee_payer = user_command.fee_payer();
    let (_fee_payer_location, _fee_payer_account) =
        pay_fee(&user_command, signer_pk, ledger, current_global_slot)?;

    // TODO: The rest is implemented on the branch `transaction_fuzzer`

    Ok(())
}

pub fn apply_user_command<L>(
    constraint_constants: &ConstraintConstants,
    txn_state_view: &ProtocolStateView,
    txn_global_slot: &Slot,
    ledger: &mut L,
    user_command: SignedCommand,
) -> Result<(), String>
where
    L: LedgerIntf,
{
    apply_user_command_unchecked(
        constraint_constants,
        txn_state_view,
        txn_global_slot,
        ledger,
        user_command,
    )
}

fn pay_fee<L, Loc>(
    user_command: &SignedCommand,
    signer_pk: &CompressedPubKey,
    ledger: &mut L,
    current_global_slot: &Slot,
) -> Result<(ExistingOrNew<Loc>, Account), String>
where
    L: LedgerIntf<Location = Loc>,
{
    let nonce = user_command.nonce();
    let fee_payer = user_command.fee_payer();
    let fee_token = user_command.fee_token();

    if &fee_payer.public_key != signer_pk {
        return Err("Cannot pay fees from a public key that did not sign the transaction".into());
    }

    if fee_token != TokenId::default() {
        return Err("Cannot create transactions with fee_token different from the default".into());
    }

    pay_fee_impl(
        &user_command.payload,
        nonce,
        fee_payer,
        user_command.fee(),
        ledger,
        current_global_slot,
    )
}

fn pay_fee_impl<L, Loc>(
    command: &SignedCommandPayload,
    nonce: Nonce,
    fee_payer: AccountId,
    fee: Fee,
    ledger: &mut L,
    current_global_slot: &Slot,
) -> Result<(ExistingOrNew<Loc>, Account), String>
where
    L: LedgerIntf<Location = Loc>,
{
    // Fee-payer information
    let (location, mut account) = get_with_location(ledger, &fee_payer)?;

    if let ExistingOrNew::New = location {
        return Err("The fee-payer account does not exist".to_string());
    };

    let fee = Amount::of_fee(&fee);
    let balance = sub_amount(account.balance, fee)?;

    validate_nonces(nonce, account.nonce)?;
    let timing = validate_timing(&account, fee, current_global_slot)?;

    account.balance = balance;
    account.nonce = account.nonce.incr(); // TODO: Not sure if OCaml wraps
    account.receipt_chain_hash = cons_signed_command_payload(command, account.receipt_chain_hash);
    account.timing = timing;

    Ok((location, account))

    // in
    // ( location
    // , { account with
    //     balance
    //   ; nonce = Account.Nonce.succ account.nonce
    //   ; receipt_chain_hash =
    //       Receipt.Chain_hash.cons_signed_command_payload command
    //         account.receipt_chain_hash
    //   ; timing
    //   } )
}

pub mod transaction_union_payload {
    use mina_hasher::ROInput as LegacyInput;

    use crate::scan_state::transaction_logic::signed_command::{
        PaymentPayload, StakeDelegationPayload,
    };

    use super::*;

    struct Common {
        fee: Fee,
        fee_token: TokenId,
        fee_payer_pk: CompressedPubKey,
        nonce: Nonce,
        valid_until: Slot,
        memo: Memo,
    }

    #[derive(Clone, Debug)]
    enum Tag {
        Payment = 0,
        StakeDelegation = 1,
        CreateAccount = 2,
        MintTokens = 3,
        FeeTransfer = 4,
        Coinbase = 5,
    }

    struct Body {
        tag: Tag,
        source_pk: CompressedPubKey,
        receiver_pk: CompressedPubKey,
        token_id: TokenId,
        amount: Amount,
        token_locked: bool,
    }

    pub struct TransactionUnionPayload {
        common: Common,
        body: Body,
    }

    impl TransactionUnionPayload {
        pub fn of_user_command_payload(payload: &SignedCommandPayload) -> Self {
            use signed_command::Body::{Payment, StakeDelegation};

            Self {
                common: Common {
                    fee: payload.common.fee,
                    fee_token: TokenId::default(),
                    fee_payer_pk: payload.common.fee_payer_pk.clone(),
                    nonce: payload.common.nonce,
                    valid_until: payload.common.valid_until,
                    memo: payload.common.memo.clone(),
                },
                body: match &payload.body {
                    Payment(PaymentPayload {
                        source_pk,
                        receiver_pk,
                        amount,
                    }) => Body {
                        tag: Tag::Payment,
                        source_pk: source_pk.clone(),
                        receiver_pk: receiver_pk.clone(),
                        token_id: TokenId::default(),
                        amount: *amount,
                        token_locked: false,
                    },
                    StakeDelegation(StakeDelegationPayload::SetDelegate {
                        delegator,
                        new_delegate,
                    }) => Body {
                        tag: Tag::StakeDelegation,
                        source_pk: delegator.clone(),
                        receiver_pk: new_delegate.clone(),
                        token_id: TokenId::default(),
                        amount: Amount::zero(),
                        token_locked: false,
                    },
                },
            }
        }

        /// TODO: Needs to be tested, the order might be reversed
        ///
        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/transaction_union_payload.ml#L309
        pub fn to_input_legacy(&self) -> LegacyInput {
            let mut roi = LegacyInput::new();

            // Self.common
            {
                roi = roi.append_u64(self.common.fee.0);

                // TokenId.default
                // https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.ml#L19
                roi = roi.append_bool(true);
                for _ in 0..63 {
                    roi = roi.append_bool(false);
                }

                // fee_payer_pk
                roi = roi.append_field(self.common.fee_payer_pk.x);
                roi = roi.append_bool(self.common.fee_payer_pk.is_odd);

                // nonce
                roi = roi.append_u32(self.common.nonce.0);

                // valid_until
                roi = roi.append_u32(self.common.valid_until.0);

                // memo
                roi = roi.append_bytes(&self.common.memo.0.as_slice());
            }

            // Self.body
            {
                // tag
                let tag = self.body.tag.clone() as u8;
                for bit in [4, 2, 1] {
                    roi = roi.append_bool(tag & bit != 0);
                }

                // source_pk
                roi = roi.append_field(self.body.source_pk.x);
                roi = roi.append_bool(self.body.source_pk.is_odd);

                // receiver_pk
                roi = roi.append_field(self.body.receiver_pk.x);
                roi = roi.append_bool(self.body.receiver_pk.is_odd);

                // amount
                roi = roi.append_u64(self.body.amount.0);

                // token_locked
                roi = roi.append_bool(self.body.token_locked);
            }

            roi
        }
    }
}

/// Returns the new `receipt_chain_hash`
fn cons_signed_command_payload(
    command_payload: &SignedCommandPayload,
    last_receipt_chain_hash: ReceiptChainHash,
) -> ReceiptChainHash {
    // Note: Not sure why the use the legacy way of hashing here

    use mina_hasher::ROInput as LegacyInput;

    let union = TransactionUnionPayload::of_user_command_payload(command_payload);

    let inputs = union.to_input_legacy();
    let inputs = inputs.append_field(last_receipt_chain_hash.0);

    use mina_hasher::{create_legacy, Hashable, Hasher, ROInput};

    #[derive(Clone)]
    struct MyInput(LegacyInput);

    impl Hashable for MyInput {
        type D = ();

        fn to_roinput(&self) -> ROInput {
            self.0.clone()
        }

        fn domain_string(_: Self::D) -> Option<String> {
            Some("MinaReceiptUC".to_string())
        }
    }

    let mut hasher = create_legacy::<MyInput>(());
    hasher.update(&MyInput(inputs));
    ReceiptChainHash(hasher.digest())
}

fn validate_nonces(txn_nonce: Nonce, account_nonce: Nonce) -> Result<(), String> {
    if account_nonce == txn_nonce {
        return Ok(());
    }

    Err(format!(
        "Nonce in account {:?} different from nonce in transaction {:?}",
        account_nonce, txn_nonce,
    ))
}

fn validate_timing(
    account: &Account,
    txn_amount: Amount,
    txn_global_slot: &Slot,
) -> Result<Timing, String> {
    let (timing, _) = validate_timing_with_min_balance(account, txn_amount, txn_global_slot)?;

    Ok(timing)
}

fn validate_timing_with_min_balance(
    account: &Account,
    txn_amount: Amount,
    txn_global_slot: &Slot,
) -> Result<(Timing, MinBalance), String> {
    use TimingValidation::*;

    let (possibly_error, timing, min_balance) =
        validate_timing_with_min_balance_impl(account, txn_amount, txn_global_slot);

    match possibly_error {
        InsufficientBalance(true) => Err(format!(
            "For timed account, the requested transaction for amount {:?} \
             at global slot {:?}, the balance {:?} \
             is insufficient",
            txn_amount, txn_global_slot, account.balance
        )),
        InvalidTiming(true) => Err(format!(
            "For timed account, the requested transaction for amount {:?} \
             at global slot {:?}, applying the transaction would put the \
             balance below the calculated minimum balance of {:?}",
            txn_amount, txn_global_slot, min_balance.0
        )),
        InsufficientBalance(false) => {
            panic!("Broken invariant in validate_timing_with_min_balance'")
        }
        InvalidTiming(false) => Ok((timing, min_balance)),
    }
}

enum TimingValidation {
    InsufficientBalance(bool),
    InvalidTiming(bool),
}

#[derive(Debug)]
struct MinBalance(Balance);

fn validate_timing_with_min_balance_impl(
    account: &Account,
    txn_amount: Amount,
    txn_global_slot: &Slot,
) -> (TimingValidation, Timing, MinBalance) {
    use crate::Timing::*;
    use TimingValidation::*;

    match &account.timing {
        Untimed => {
            // no time restrictions
            match account.balance.sub_amount(txn_amount) {
                None => (
                    InsufficientBalance(true),
                    Untimed,
                    MinBalance(Balance::zero()),
                ),
                Some(_) => (
                    InsufficientBalance(false),
                    Untimed,
                    MinBalance(Balance::zero()),
                ),
            }
        }
        Timed {
            initial_minimum_balance,
            cliff_time,
            cliff_amount,
            vesting_period,
            vesting_increment,
        } => {
            let account_balance = account.balance;
            let initial_minimum_balance = initial_minimum_balance;

            let (invalid_balance, invalid_timing, curr_min_balance) =
                match account_balance.sub_amount(txn_amount) {
                    None => {
                        // NB: The [initial_minimum_balance] here is the incorrect value,
                        // but:
                        // * we don't use it anywhere in this error case; and
                        // * we don't want to waste time computing it if it will be unused.
                        (true, false, *initial_minimum_balance)
                    }
                    Some(proposed_new_balance) => {
                        let cliff_time = cliff_time;
                        let cliff_amount = cliff_amount;
                        let vesting_period = vesting_period;
                        let vesting_increment = vesting_increment;

                        let curr_min_balance = account_min_balance_at_slot(
                            *txn_global_slot,
                            *cliff_time,
                            *cliff_amount,
                            *vesting_period,
                            *vesting_increment,
                            *initial_minimum_balance,
                        );

                        if proposed_new_balance < curr_min_balance {
                            (false, true, curr_min_balance)
                        } else {
                            (false, false, curr_min_balance)
                        }
                    }
                };

            // once the calculated minimum balance becomes zero, the account becomes untimed
            let possibly_error = if invalid_balance {
                InsufficientBalance(invalid_balance)
            } else {
                InvalidTiming(invalid_timing)
            };

            if curr_min_balance > Balance::zero() {
                (
                    possibly_error,
                    account.timing.clone(),
                    MinBalance(curr_min_balance),
                )
            } else {
                (possibly_error, Untimed, MinBalance(Balance::zero()))
            }
        }
    }
}

// TODO: This should be in `account.rs`
pub fn account_min_balance_at_slot(
    global_slot: Slot,
    cliff_time: Slot,
    cliff_amount: Amount,
    vesting_period: Slot,
    vesting_increment: Amount,
    initial_minimum_balance: Balance,
) -> Balance {
    if global_slot < cliff_time {
        initial_minimum_balance
    } else if vesting_period.is_zero() {
        // If vesting period is zero then everything vests immediately at the cliff
        Balance::zero()
    } else {
        match initial_minimum_balance.sub_amount(cliff_amount) {
            None => Balance::zero(),
            Some(min_balance_past_cliff) => {
                // take advantage of fact that global slots are uint32's

                let num_periods =
                    (global_slot.as_u32() - cliff_time.as_u32()) / vesting_period.as_u32();
                let num_periods: u64 = num_periods.try_into().unwrap();

                let vesting_decrement = {
                    let vesting_increment = vesting_increment.as_u64();

                    if u64::MAX
                        .checked_div(num_periods)
                        .map(|res| matches!(res.cmp(&vesting_increment), std::cmp::Ordering::Less))
                        .unwrap_or(false)
                    {
                        // The vesting decrement will overflow, use [max_int] instead.
                        Amount::from_u64(u64::MAX)
                    } else {
                        Amount::from_u64(num_periods.checked_mul(vesting_increment).unwrap())
                    }
                };

                match min_balance_past_cliff.sub_amount(vesting_decrement) {
                    None => Balance::zero(),
                    Some(amount) => amount,
                }
            }
        }
    }
}

fn sub_amount(balance: Balance, amount: Amount) -> Result<Balance, String> {
    balance
        .sub_amount(amount)
        .ok_or_else(|| "insufficient funds".to_string())
}

fn add_amount(balance: Balance, amount: Amount) -> Result<Balance, String> {
    balance
        .add_amount(amount)
        .ok_or_else(|| "overflow".to_string())
}

pub enum ExistingOrNew<Loc> {
    Existing(Loc),
    New,
}

fn get_with_location<L, Loc>(
    ledger: &mut L,
    account_id: &AccountId,
) -> Result<(ExistingOrNew<Loc>, Account), String>
where
    L: LedgerIntf<Location = Loc>,
{
    match ledger.location_of_account(account_id) {
        Some(location) => match ledger.get(&location) {
            Some(account) => Ok((ExistingOrNew::Existing(location), account)),
            None => panic!("Ledger location with no account"),
        },
        None => Ok((
            ExistingOrNew::New,
            Account::create_with(account_id.clone(), Balance::zero()),
        )),
    }
}
