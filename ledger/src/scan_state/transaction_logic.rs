use std::collections::BTreeMap;

use ark_ff::Zero;
use itertools::{FoldWhile, Itertools};
use mina_hasher::{create_kimchi, Fp};
use mina_p2p_messages::binprot;
use mina_p2p_messages::v2::{MinaBaseUserCommandStableV2, MinaTransactionTransactionStableV2};
use mina_signer::CompressedPubKey;

use crate::proofs::witness::Witness;
use crate::scan_state::transaction_logic::transaction_partially_applied::FullyApplied;
use crate::scan_state::zkapp_logic;
use crate::{hash_with_kimchi, ControlTag, Inputs};
use crate::{
    scan_state::transaction_logic::transaction_applied::{CommandApplied, Varying},
    sparse_ledger::{LedgerIntf, SparseLedger},
    Account, AccountId, ReceiptChainHash, Timing, TokenId, VerificationKey,
};

use self::zkapp_command::{AccessedOrNot, Numeric};
use self::{
    local_state::{CallStack, LocalStateEnv, StackFrame},
    protocol_state::{GlobalState, ProtocolStateView},
    signed_command::{SignedCommand, SignedCommandPayload},
    transaction_applied::{
        signed_command_applied::{self, SignedCommandApplied},
        TransactionApplied, ZkappCommandApplied,
    },
    transaction_union_payload::TransactionUnionPayload,
    zkapp_command::{
        AccountPreconditions, AccountUpdate, WithHash, ZkAppCommand, ZkAppPreconditions,
    },
};

use super::currency::SlotSpan;
use super::zkapp_logic::ZkAppCommandElt;
use super::{
    currency::{Amount, Balance, Fee, Index, Length, Magnitude, Nonce, Signed, Slot},
    fee_excess::FeeExcess,
    scan_state::{transaction_snark::OneOrTwo, ConstraintConstants},
    zkapp_logic::{Handler, StartData},
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
    UpdateNotPermittedAccess,
    UpdateNotPermittedTiming,
    UpdateNotPermittedDelegate,
    UpdateNotPermittedAppState,
    UpdateNotPermittedVerificationKey,
    UpdateNotPermittedActionState,
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
    AccountActionStatePreconditionUnsatisfied,
    AccountAppStatePreconditionUnsatisfied(i64),
    AccountProvedStatePreconditionUnsatisfied,
    AccountIsNewPreconditionUnsatisfied,
    ProtocolStatePreconditionUnsatisfied,
    UnexpectedVerificationKeyHash,
    ValidWhilePreconditionUnsatisfied,
    IncorrectNonce,
    InvalidFeeExcess,
    Cancelled,
}

impl ToString for TransactionFailure {
    fn to_string(&self) -> String {
        match self {
            Self::Predicate => "Predicate".to_string(),
            Self::SourceNotPresent => "Source_not_present".to_string(),
            Self::ReceiverNotPresent => "Receiver_not_present".to_string(),
            Self::AmountInsufficientToCreateAccount => {
                "Amount_insufficient_to_create_account".to_string()
            }
            Self::CannotPayCreationFeeInToken => "Cannot_pay_creation_fee_in_token".to_string(),
            Self::SourceInsufficientBalance => "Source_insufficient_balance".to_string(),
            Self::SourceMinimumBalanceViolation => "Source_minimum_balance_violation".to_string(),
            Self::ReceiverAlreadyExists => "Receiver_already_exists".to_string(),
            Self::TokenOwnerNotCaller => "Token_owner_not_caller".to_string(),
            Self::Overflow => "Overflow".to_string(),
            Self::GlobalExcessOverflow => "Global_excess_overflow".to_string(),
            Self::LocalExcessOverflow => "Local_excess_overflow".to_string(),
            Self::LocalSupplyIncreaseOverflow => "Local_supply_increase_overflow".to_string(),
            Self::GlobalSupplyIncreaseOverflow => "Global_supply_increase_overflow".to_string(),
            Self::SignedCommandOnZkappAccount => "Signed_command_on_zkapp_account".to_string(),
            Self::ZkappAccountNotPresent => "Zkapp_account_not_present".to_string(),
            Self::UpdateNotPermittedBalance => "Update_not_permitted_balance".to_string(),
            Self::UpdateNotPermittedAccess => "Update_not_permitted_access".to_string(),
            Self::UpdateNotPermittedTiming => "Update_not_permitted_timing".to_string(),
            Self::UpdateNotPermittedDelegate => "update_not_permitted_delegate".to_string(),
            Self::UpdateNotPermittedAppState => "Update_not_permitted_app_state".to_string(),
            Self::UpdateNotPermittedVerificationKey => {
                "Update_not_permitted_verification_key".to_string()
            }
            Self::UpdateNotPermittedActionState => "Update_not_permitted_action_state".to_string(),
            Self::UpdateNotPermittedZkappUri => "Update_not_permitted_zkapp_uri".to_string(),
            Self::UpdateNotPermittedTokenSymbol => "Update_not_permitted_token_symbol".to_string(),
            Self::UpdateNotPermittedPermissions => "Update_not_permitted_permissions".to_string(),
            Self::UpdateNotPermittedNonce => "Update_not_permitted_nonce".to_string(),
            Self::UpdateNotPermittedVotingFor => "Update_not_permitted_voting_for".to_string(),
            Self::ZkappCommandReplayCheckFailed => "Zkapp_command_replay_check_failed".to_string(),
            Self::FeePayerNonceMustIncrease => "Fee_payer_nonce_must_increase".to_string(),
            Self::FeePayerMustBeSigned => "Fee_payer_must_be_signed".to_string(),
            Self::AccountBalancePreconditionUnsatisfied => {
                "Account_balance_precondition_unsatisfied".to_string()
            }
            Self::AccountNoncePreconditionUnsatisfied => {
                "Account_nonce_precondition_unsatisfied".to_string()
            }
            Self::AccountReceiptChainHashPreconditionUnsatisfied => {
                "Account_receipt_chain_hash_precondition_unsatisfied".to_string()
            }
            Self::AccountDelegatePreconditionUnsatisfied => {
                "Account_delegate_precondition_unsatisfied".to_string()
            }
            Self::AccountActionStatePreconditionUnsatisfied => {
                "Account_action_state_precondition_unsatisfied".to_string()
            }
            Self::AccountAppStatePreconditionUnsatisfied(i) => {
                format!("Account_app_state_{}_precondition_unsatisfied", i)
            }
            Self::AccountProvedStatePreconditionUnsatisfied => {
                "Account_proved_state_precondition_unsatisfied".to_string()
            }
            Self::AccountIsNewPreconditionUnsatisfied => {
                "Account_is_new_precondition_unsatisfied".to_string()
            }
            Self::ProtocolStatePreconditionUnsatisfied => {
                "Protocol_state_precondition_unsatisfied".to_string()
            }
            Self::IncorrectNonce => "Incorrect_nonce".to_string(),
            Self::InvalidFeeExcess => "Invalid_fee_excess".to_string(),
            Self::Cancelled => "Cancelled".to_string(),
            Self::UnexpectedVerificationKeyHash => "Unexpected_verification_key_hash".to_string(),
            Self::ValidWhilePreconditionUnsatisfied => {
                "Valid_while_precondition_unsatisfied".to_string()
            }
        }
    }
}

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/transaction_status.ml#L452
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionStatus {
    Applied,
    Failed(Vec<Vec<TransactionFailure>>),
}

impl TransactionStatus {
    pub fn is_applied(&self) -> bool {
        matches!(self, Self::Applied)
    }
}

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/with_status.ml#L6
#[derive(Debug, Clone, PartialEq)]
pub struct WithStatus<T> {
    pub data: T,
    pub status: TransactionStatus,
}

impl<T> WithStatus<T> {
    pub fn applied(data: T) -> Self {
        Self {
            data,
            status: TransactionStatus::Applied,
        }
    }

    pub fn failed(data: T, failures: Vec<Vec<TransactionFailure>>) -> Self {
        Self {
            data,
            status: TransactionStatus::Failed(failures),
        }
    }

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

impl<T> GenericCommand for WithStatus<T>
where
    T: GenericCommand,
{
    fn fee(&self) -> Fee {
        self.data.fee()
    }

    fn forget(&self) -> UserCommand {
        self.data.forget()
    }
}

pub mod valid {
    use super::*;

    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub struct VerificationKeyHash(pub Fp);

    pub type SignedCommand = super::signed_command::SignedCommand;

    #[derive(Clone, Debug, PartialEq)]
    pub enum UserCommand {
        SignedCommand(Box<SignedCommand>),
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
                    Err(format!(
                        "Cannot combine single fee transfers with incompatible tokens: {:?} <> {:?}",
                        one, two
                    ))
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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

    /// https://github.com/MinaProtocol/mina/blob/f6756507ff7380a691516ce02a3cf7d9d32915ae/src/lib/mina_base/coinbase.ml#L76
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
                // The substraction result is ignored here
                .map(|_| *amount)
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

    /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/coinbase.ml#L51
    pub fn account_access_statuses(
        &self,
        status: &TransactionStatus,
    ) -> Vec<(AccountId, zkapp_command::AccessedOrNot)> {
        let access_status = match status {
            TransactionStatus::Applied => zkapp_command::AccessedOrNot::Accessed,
            TransactionStatus::Failed(_) => zkapp_command::AccessedOrNot::NotAccessed,
        };

        let mut ids = Vec::with_capacity(2);

        ids.push((self.receiver(), access_status.clone()));

        if let Some(fee_transfer) = self.fee_transfer.as_ref() {
            ids.push((fee_transfer.receiver(), access_status));
        };

        ids
    }

    /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/coinbase.ml#L61
    pub fn accounts_referenced(&self) -> Vec<AccountId> {
        self.account_access_statuses(&TransactionStatus::Applied)
            .into_iter()
            .map(|(id, _status)| id)
            .collect()
    }
}

/// 0th byte is a tag to distinguish digests from other data
/// 1st byte is length, always 32 for digests
/// bytes 2 to 33 are data, 0-right-padded if length is less than 32
///
#[derive(Clone, PartialEq)]
pub struct Memo(pub [u8; 34]);

impl std::fmt::Debug for Memo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::staged_ledger::hash::OCamlString;

        // Display like OCaml
        // Example: "\000 \014WQ\192&\229C\178\232\171.\176`\153\218\161\209\229\223Gw\143w\135\250\171E\205\241/\227\168"

        f.write_fmt(format_args!("\"{}\"", self.0.to_ocaml_str()))
    }
}

impl Memo {
    const TAG_INDEX: usize = 0;
    const LENGTH_INDEX: usize = 1;

    const DIGEST_TAG: u8 = 0x00;
    const BYTES_TAG: u8 = 0x01;

    const DIGEST_LENGTH: usize = 32; // Blake2.digest_size_in_bytes
    const DIGEST_LENGTH_BYTE: u8 = Self::DIGEST_LENGTH as u8;

    /// +2 for tag and length bytes
    const MEMO_LENGTH: usize = Self::DIGEST_LENGTH + 2;

    const MAX_INPUT_LENGTH: usize = Self::DIGEST_LENGTH;

    const MAX_DIGESTIBLE_STRING_LENGTH: usize = 1000;

    pub fn to_bits(&self) -> [bool; std::mem::size_of::<Self>() * 8] {
        use crate::proofs::witness::legacy_input::BitsIterator;

        const NBYTES: usize = 34;
        const NBITS: usize = NBYTES * 8;
        assert_eq!(std::mem::size_of::<Self>(), NBYTES);

        let mut iter = BitsIterator {
            index: 0,
            number: self.0,
        }
        .take(NBITS);
        std::array::from_fn(|_| iter.next().unwrap())
    }

    pub fn hash(&self) -> Fp {
        use mina_hasher::ROInput as LegacyInput;

        let inputs = LegacyInput::new();
        let inputs = inputs.append_bytes(&self.0);

        use mina_hasher::{Hashable, Hasher, ROInput};

        #[derive(Clone)]
        struct MyInput(LegacyInput);

        impl Hashable for MyInput {
            type D = ();

            fn to_roinput(&self) -> ROInput {
                self.0.clone()
            }

            fn domain_string(_: Self::D) -> Option<String> {
                Some("MinaZkappMemo".to_string())
            }
        }

        let mut hasher = create_kimchi::<MyInput>(());
        hasher.update(&MyInput(inputs));
        hasher.digest()
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// https://github.com/MinaProtocol/mina/blob/3a78f0e0c1343d14e2729c8b00205baa2ec70c93/src/lib/mina_base/signed_command_memo.ml#L151
    pub fn dummy() -> Self {
        // TODO
        Self([0; 34])
    }

    pub fn empty() -> Self {
        let mut array = [0; 34];
        array[0] = 1;
        Self(array)
    }

    /// Example:
    /// "\000 \014WQ\192&\229C\178\232\171.\176`\153\218\161\209\229\223Gw\143w\135\250\171E\205\241/\227\168"
    #[cfg(test)]
    pub fn from_ocaml_str(s: &str) -> Self {
        use crate::staged_ledger::hash::OCamlString;

        Self(<[u8; 34]>::from_ocaml_str(s))
    }

    pub fn with_number(number: usize) -> Self {
        let s = format!("{:034}", number);
        assert_eq!(s.len(), 34);
        Self(s.into_bytes().try_into().unwrap())
    }

    /// https://github.com/MinaProtocol/mina/blob/d7dad23d8ea2052f515f5d55d187788fe0701c7f/src/lib/mina_base/signed_command_memo.ml#L103
    fn create_by_digesting_string_exn(s: &str) -> Self {
        if s.len() > Self::MAX_DIGESTIBLE_STRING_LENGTH {
            panic!("Too_long_digestible_string");
        }

        let mut memo = [0; 34];
        memo[Self::TAG_INDEX] = Self::DIGEST_TAG;
        memo[Self::LENGTH_INDEX] = Self::DIGEST_LENGTH_BYTE;

        use blake2::{
            digest::{Update, VariableOutput},
            Blake2bVar,
        };
        let mut hasher = Blake2bVar::new(32).expect("Invalid Blake2bVar output size");
        hasher.update(s.as_bytes());
        hasher.finalize_variable(&mut memo[2..]).unwrap();

        Self(memo)
    }

    /// https://github.com/MinaProtocol/mina/blob/d7dad23d8ea2052f515f5d55d187788fe0701c7f/src/lib/mina_base/signed_command_memo.ml#L193
    pub fn gen() -> Self {
        use rand::distributions::{Alphanumeric, DistString};
        let random_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 50);

        Self::create_by_digesting_string_exn(&random_string)
    }
}

pub mod signed_command {

    use mina_signer::Signature;

    use crate::{decompress_pk, scan_state::currency::Slot, AccountId};

    use super::{zkapp_command::AccessedOrNot, *};

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.ml#L75
    #[derive(Debug, Clone, PartialEq)]
    pub struct Common {
        pub fee: Fee,
        pub fee_payer_pk: CompressedPubKey,
        pub nonce: Nonce,
        pub valid_until: Slot,
        pub memo: Memo,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct PaymentPayload {
        pub receiver_pk: CompressedPubKey,
        pub amount: Amount,
    }

    /// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/mina_base/stake_delegation.ml#L11
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum StakeDelegationPayload {
        SetDelegate { new_delegate: CompressedPubKey },
    }

    impl StakeDelegationPayload {
        /// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/mina_base/stake_delegation.ml#L35
        pub fn receiver(&self) -> AccountId {
            let Self::SetDelegate { new_delegate } = self;
            AccountId::new(new_delegate.clone(), TokenId::default())
        }

        /// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/mina_base/stake_delegation.ml#L33
        pub fn receiver_pk(&self) -> &CompressedPubKey {
            let Self::SetDelegate { new_delegate } = self;
            new_delegate
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.mli#L24
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Body {
        Payment(PaymentPayload),
        StakeDelegation(StakeDelegationPayload),
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.mli#L165
    #[derive(Debug, Clone, PartialEq)]
    pub struct SignedCommandPayload {
        pub common: Common,
        pub body: Body,
    }

    impl SignedCommandPayload {
        pub fn create(
            fee: Fee,
            fee_payer_pk: CompressedPubKey,
            nonce: Nonce,
            valid_until: Option<Slot>,
            memo: Memo,
            body: Body,
        ) -> Self {
            Self {
                common: Common {
                    fee,
                    fee_payer_pk,
                    nonce,
                    valid_until: valid_until.unwrap_or_else(Slot::max),
                    memo,
                },
                body,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct SignedCommand {
        pub payload: SignedCommandPayload,
        pub signer: CompressedPubKey, // TODO: This should be a `mina_signer::PubKey`
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

        /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/signed_command_payload.ml#L354
        pub fn account_access_statuses(
            &self,
            status: &TransactionStatus,
        ) -> Vec<(AccountId, AccessedOrNot)> {
            use AccessedOrNot::*;
            use TransactionStatus::*;

            match status {
                Applied => vec![(self.fee_payer(), Accessed), (self.receiver(), Accessed)],
                Failed(_) => vec![(self.receiver(), NotAccessed)],
            }
        }

        pub fn accounts_referenced(&self) -> Vec<AccountId> {
            self.account_access_statuses(&TransactionStatus::Applied)
                .into_iter()
                .map(|(id, _status)| id)
                .collect()
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/signed_command.ml#L401
        pub fn public_keys(&self) -> [&CompressedPubKey; 2] {
            [self.fee_payer_pk(), self.receiver_pk()]
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
    use std::sync::Arc;

    use ark_ff::{UniformRand, Zero};
    use mina_p2p_messages::v2::MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA;
    use mina_signer::Signature;
    use rand::{seq::SliceRandom, Rng};

    use crate::{
        account, dummy, gen_compressed, gen_keypair, hash_noinputs, hash_with_kimchi,
        proofs::{
            field::{Boolean, ToBoolean},
            to_field_elements::ToFieldElements,
            witness::Check,
        },
        scan_state::currency::{Balance, Length, MinMax, Sgn, Signed, Slot, SlotSpan},
        zkapps::snark::zkapp_check::InSnarkCheck,
        AuthRequired, ControlTag, Inputs, MyCow, Permissions, ToInputs, TokenSymbol,
        VerificationKey, VotingFor, ZkAppAccount, ZkAppUri,
    };

    use super::{zkapp_statement::TransactionCommitment, *};

    #[derive(Debug, Clone, PartialEq)]
    pub struct Event(pub Vec<Fp>);

    impl Event {
        pub fn hash(&self) -> Fp {
            hash_with_kimchi("MinaZkappEvent", &self.0[..])
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L834
    #[derive(Debug, Clone, PartialEq)]
    pub struct Events(pub Vec<Event>);

    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L155
    #[derive(Debug, Clone, PartialEq)]
    pub struct Actions(pub Vec<Event>);

    pub fn gen_events() -> Vec<Event> {
        let mut rng = rand::thread_rng();

        let n = rng.gen_range(0..=5);

        (0..=n)
            .map(|_| {
                let n = rng.gen_range(0..=3);
                let event = (0..=n).map(|_| Fp::rand(&mut rng)).collect();
                Event(event)
            })
            .collect()
    }

    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L23
    pub trait MakeEvents {
        const SALT_PHRASE: &'static str;
        const HASH_PREFIX: &'static str;
        const DERIVER_NAME: (); // Unused here for now

        fn events(&self) -> &[Event];
        fn empty_hash() -> Fp;
    }

    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L100
    impl MakeEvents for Events {
        const SALT_PHRASE: &'static str = "MinaZkappEventsEmpty";
        const HASH_PREFIX: &'static str = "MinaZkappEvents";
        const DERIVER_NAME: () = ();
        fn events(&self) -> &[Event] {
            self.0.as_slice()
        }
        fn empty_hash() -> Fp {
            cache_one!(Fp, events_to_field(&Events::empty()))
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L156
    impl MakeEvents for Actions {
        const SALT_PHRASE: &'static str = "MinaZkappActionsEmpty";
        const HASH_PREFIX: &'static str = "MinaZkappSeqEvents";
        const DERIVER_NAME: () = ();
        fn events(&self) -> &[Event] {
            self.0.as_slice()
        }
        fn empty_hash() -> Fp {
            cache_one!(Fp, events_to_field(&Actions::empty()))
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L52
    pub fn events_to_field<E>(e: &E) -> Fp
    where
        E: MakeEvents,
    {
        let init = hash_noinputs(E::SALT_PHRASE);

        e.events().iter().rfold(init, |accum, elem| {
            hash_with_kimchi(E::HASH_PREFIX, &[accum, elem.hash()])
        })
    }

    impl ToInputs for Events {
        fn to_inputs(&self, inputs: &mut Inputs) {
            inputs.append(&events_to_field(self));
        }
    }

    impl ToInputs for Actions {
        fn to_inputs(&self, inputs: &mut Inputs) {
            inputs.append(&events_to_field(self));
        }
    }

    impl ToFieldElements<Fp> for Events {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            events_to_field(self).to_field_elements(fields);
        }
    }

    impl ToFieldElements<Fp> for Actions {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            events_to_field(self).to_field_elements(fields);
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
        pub vesting_period: SlotSpan,
        pub vesting_increment: Amount,
    }

    impl Timing {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L208
        fn dummy() -> Self {
            Self {
                initial_minimum_balance: Balance::zero(),
                cliff_time: Slot::zero(),
                cliff_amount: Amount::zero(),
                vesting_period: SlotSpan::zero(),
                vesting_increment: Amount::zero(),
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/transaction_logic/mina_transaction_logic.ml#L1278
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L228
        pub fn of_account_timing(timing: crate::account::Timing) -> Option<Self> {
            match timing {
                crate::Timing::Untimed => None,
                crate::Timing::Timed {
                    initial_minimum_balance,
                    cliff_time,
                    cliff_amount,
                    vesting_period,
                    vesting_increment,
                } => Some(Self {
                    initial_minimum_balance,
                    cliff_time,
                    cliff_amount,
                    vesting_period,
                    vesting_increment,
                }),
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L219
        pub fn to_account_timing(self) -> crate::account::Timing {
            let Self {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } = self;

            crate::account::Timing::Timed {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            }
        }
    }

    impl ToFieldElements<Fp> for Timing {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            let Self {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } = self;

            initial_minimum_balance.to_field_elements(fields);
            cliff_time.to_field_elements(fields);
            cliff_amount.to_field_elements(fields);
            vesting_period.to_field_elements(fields);
            vesting_increment.to_field_elements(fields);
        }
    }

    impl Check<Fp> for Timing {
        fn check(&self, w: &mut Witness<Fp>) {
            let Self {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } = self;

            initial_minimum_balance.check(w);
            cliff_time.check(w);
            cliff_amount.check(w);
            vesting_period.check(w);
            vesting_increment.check(w);
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

    impl Events {
        pub fn empty() -> Self {
            Self(Vec::new())
        }

        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        pub fn push_event(acc: Fp, event: Event) -> Fp {
            hash_with_kimchi(Self::HASH_PREFIX, &[acc, event.hash()])
        }

        pub fn push_events(&self, acc: Fp) -> Fp {
            let hash = self
                .0
                .iter()
                .rfold(hash_noinputs(Self::SALT_PHRASE), |acc, e| {
                    Self::push_event(acc, e.clone())
                });
            hash_with_kimchi(Self::HASH_PREFIX, &[acc, hash])
        }
    }

    impl Actions {
        pub fn empty() -> Self {
            Self(Vec::new())
        }

        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        pub fn push_event(acc: Fp, event: Event) -> Fp {
            hash_with_kimchi(Self::HASH_PREFIX, &[acc, event.hash()])
        }

        pub fn push_events(&self, acc: Fp) -> Fp {
            let hash = self
                .0
                .iter()
                .rfold(hash_noinputs(Self::SALT_PHRASE), |acc, e| {
                    Self::push_event(acc, e.clone())
                });
            hash_with_kimchi(Self::HASH_PREFIX, &[acc, hash])
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_basic.ml#L100
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum SetOrKeep<T: Clone> {
        Set(T),
        Keep,
    }

    impl<T: Clone> SetOrKeep<T> {
        fn map<'a, F, U>(&'a self, fun: F) -> SetOrKeep<U>
        where
            F: FnOnce(&'a T) -> U,
            U: Clone,
        {
            match self {
                SetOrKeep::Set(v) => SetOrKeep::Set(fun(v)),
                SetOrKeep::Keep => SetOrKeep::Keep,
            }
        }

        pub fn into_map<F, U>(self, fun: F) -> SetOrKeep<U>
        where
            F: FnOnce(T) -> U,
            U: Clone,
        {
            match self {
                SetOrKeep::Set(v) => SetOrKeep::Set(fun(v)),
                SetOrKeep::Keep => SetOrKeep::Keep,
            }
        }

        pub fn set_or_keep(&self, x: T) -> T {
            match self {
                Self::Set(data) => data.clone(),
                Self::Keep => x,
            }
        }

        pub fn is_keep(&self) -> bool {
            match self {
                Self::Keep => true,
                Self::Set(_) => false,
            }
        }

        pub fn is_set(&self) -> bool {
            !self.is_keep()
        }

        pub fn gen<F>(mut fun: F) -> Self
        where
            F: FnMut() -> T,
        {
            let mut rng = rand::thread_rng();

            if rng.gen() {
                Self::Set(fun())
            } else {
                Self::Keep
            }
        }
    }

    impl<T, F> ToInputs for (&SetOrKeep<T>, F)
    where
        T: ToInputs,
        T: Clone,
        F: Fn() -> T,
    {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_basic.ml#L223
        fn to_inputs(&self, inputs: &mut Inputs) {
            let (set_or_keep, default_fn) = self;

            match set_or_keep {
                SetOrKeep::Set(this) => {
                    inputs.append_bool(true);
                    this.to_inputs(inputs);
                }
                SetOrKeep::Keep => {
                    inputs.append_bool(false);
                    let default = default_fn();
                    default.to_inputs(inputs);
                }
            };
        }
    }

    impl<T, F> ToFieldElements<Fp> for (&SetOrKeep<T>, F)
    where
        T: ToFieldElements<Fp>,
        T: Clone,
        F: Fn() -> T,
    {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            let (set_or_keep, default_fn) = self;

            match set_or_keep {
                SetOrKeep::Set(this) => {
                    Boolean::True.to_field_elements(fields);
                    this.to_field_elements(fields);
                }
                SetOrKeep::Keep => {
                    Boolean::False.to_field_elements(fields);
                    let default = default_fn();
                    default.to_field_elements(fields);
                }
            }
        }
    }

    impl<T, F> Check<Fp> for (&SetOrKeep<T>, F)
    where
        T: Check<Fp>,
        T: Clone,
        F: Fn() -> T,
    {
        fn check(&self, w: &mut Witness<Fp>) {
            let (set_or_keep, default_fn) = self;
            let value = match set_or_keep {
                SetOrKeep::Set(this) => MyCow::Borrow(this),
                SetOrKeep::Keep => MyCow::Own(default_fn()),
            };
            value.check(w);
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct WithHash<T> {
        pub data: T,
        pub hash: Fp,
    }

    impl<T> ToFieldElements<Fp> for WithHash<T> {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            let Self { data: _, hash } = self;
            hash.to_field_elements(fields);
        }
    }

    impl<T> std::ops::Deref for WithHash<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.data
        }
    }

    impl<T> WithHash<T> {
        pub fn of_data(data: T, hash_data: impl Fn(&T) -> Fp) -> Self {
            let hash = hash_data(&data);
            Self { data, hash }
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L319
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Update {
        pub app_state: [SetOrKeep<Fp>; 8],
        pub delegate: SetOrKeep<CompressedPubKey>,
        pub verification_key: SetOrKeep<WithHash<VerificationKey>>,
        pub permissions: SetOrKeep<Permissions<AuthRequired>>,
        pub zkapp_uri: SetOrKeep<ZkAppUri>,
        pub token_symbol: SetOrKeep<TokenSymbol>,
        pub timing: SetOrKeep<Timing>,
        pub voting_for: SetOrKeep<VotingFor>,
    }

    impl ToFieldElements<Fp> for Update {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            let Self {
                app_state,
                delegate,
                verification_key,
                permissions,
                zkapp_uri,
                token_symbol,
                timing,
                voting_for,
            } = self;

            for s in app_state {
                (s, Fp::zero).to_field_elements(fields);
            }
            (delegate, CompressedPubKey::empty).to_field_elements(fields);
            (&verification_key.map(|w| w.hash), Fp::zero).to_field_elements(fields);
            (permissions, Permissions::empty).to_field_elements(fields);
            (&zkapp_uri.map(Some), || Option::<&ZkAppUri>::None).to_field_elements(fields);
            (token_symbol, TokenSymbol::default).to_field_elements(fields);
            (timing, Timing::dummy).to_field_elements(fields);
            (voting_for, VotingFor::dummy).to_field_elements(fields);
        }
    }

    impl Update {
        /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/account_update.ml#L460
        pub fn noop() -> Self {
            Self {
                app_state: std::array::from_fn(|_| SetOrKeep::Keep),
                delegate: SetOrKeep::Keep,
                verification_key: SetOrKeep::Keep,
                permissions: SetOrKeep::Keep,
                zkapp_uri: SetOrKeep::Keep,
                token_symbol: SetOrKeep::Keep,
                timing: SetOrKeep::Keep,
                voting_for: SetOrKeep::Keep,
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/account_update.ml#L472
        pub fn dummy() -> Self {
            Self::noop()
        }

        /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/account_update.ml#L338
        pub fn gen(
            token_account: Option<bool>,
            zkapp_account: Option<bool>,
            vk: Option<&WithHash<VerificationKey>>,
            permissions_auth: Option<crate::ControlTag>,
        ) -> Self {
            let mut rng = rand::thread_rng();

            let token_account = token_account.unwrap_or(false);
            let zkapp_account = zkapp_account.unwrap_or(false);

            let app_state: [_; 8] = std::array::from_fn(|_| SetOrKeep::gen(|| Fp::rand(&mut rng)));

            let delegate = if !token_account {
                SetOrKeep::gen(|| gen_keypair().public.into_compressed())
            } else {
                SetOrKeep::Keep
            };

            let verification_key = if zkapp_account {
                SetOrKeep::gen(|| match vk {
                    None => {
                        let dummy = VerificationKey::dummy();
                        let hash = dummy.digest();
                        WithHash { data: dummy, hash }
                    }
                    Some(vk) => vk.clone(),
                })
            } else {
                SetOrKeep::Keep
            };

            let permissions = match permissions_auth {
                None => SetOrKeep::Keep,
                Some(auth_tag) => SetOrKeep::Set(Permissions::gen(auth_tag)),
            };

            let zkapp_uri = SetOrKeep::gen(|| {
                ZkAppUri::from(
                    [
                        "https://www.example.com",
                        "https://www.minaprotocol.com",
                        "https://www.gurgle.com",
                        "https://faceplant.com",
                    ]
                    .choose(&mut rng)
                    .unwrap()
                    .to_string(),
                )
            });

            let token_symbol = SetOrKeep::gen(|| {
                TokenSymbol::from(
                    ["MINA", "TOKEN1", "TOKEN2", "TOKEN3", "TOKEN4", "TOKEN5"]
                        .choose(&mut rng)
                        .unwrap()
                        .to_string(),
                )
            });

            let voting_for = SetOrKeep::gen(|| VotingFor(Fp::rand(&mut rng)));

            let timing = SetOrKeep::Keep;

            Self {
                app_state,
                delegate,
                verification_key,
                permissions,
                zkapp_uri,
                token_symbol,
                timing,
                voting_for,
            }
        }
    }

    pub trait OutSnarkCheck {
        type A;
        type B;

        /// zkapp check
        fn zcheck(&self, label: String, x: Self::B) -> Result<(), String>;
    }

    impl<T> OutSnarkCheck for T
    where
        T: Eq,
    {
        type A = T;
        type B = T;

        /// zkapp check
        fn zcheck(&self, label: String, rhs: Self::B) -> Result<(), String> {
            if *self == rhs {
                Ok(())
            } else {
                Err(format!("Equality check failed: {}", label))
            }
        }
    }

    // TODO: This could be std::ops::Range ?
    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L23
    #[derive(Debug, Clone, PartialEq)]
    pub struct ClosedInterval<T> {
        pub lower: T,
        pub upper: T,
    }

    impl<T> ClosedInterval<T>
    where
        T: MinMax,
    {
        pub fn min_max() -> Self {
            Self {
                lower: T::min(),
                upper: T::max(),
            }
        }
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

    impl<T> ToFieldElements<Fp> for ClosedInterval<T>
    where
        T: ToFieldElements<Fp>,
    {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            let ClosedInterval { lower, upper } = self;

            lower.to_field_elements(fields);
            upper.to_field_elements(fields);
        }
    }

    impl<T> Check<Fp> for ClosedInterval<T>
    where
        T: Check<Fp>,
    {
        fn check(&self, w: &mut Witness<Fp>) {
            let ClosedInterval { lower, upper } = self;
            lower.check(w);
            upper.check(w);
        }
    }

    impl<T> OutSnarkCheck for ClosedInterval<T>
    where
        T: PartialOrd + std::fmt::Debug,
    {
        type A = ClosedInterval<T>;
        type B = T;

        /// zkapp check
        fn zcheck(&self, label: String, rhs: Self::B) -> Result<(), String> {
            /*println!(
                "bounds check lower {:?} rhs {:?} upper {:?}",
                self.lower, rhs, self.upper
            );*/
            if self.lower <= rhs && rhs <= self.upper {
                Ok(())
            } else {
                Err(format!("Bounds check failed: {}", label))
            }
        }
    }

    impl<T> ClosedInterval<T>
    where
        T: PartialOrd,
    {
        pub fn is_constant(&self) -> bool {
            self.lower == self.upper
        }

        /// https://github.com/MinaProtocol/mina/blob/d7d4aa4d650eb34b45a42b29276554802683ce15/src/lib/mina_base/zkapp_precondition.ml#L30
        pub fn gen<F>(mut fun: F) -> Self
        where
            F: FnMut() -> T,
        {
            let a1 = fun();
            let a2 = fun();

            if a1 <= a2 {
                Self {
                    lower: a1,
                    upper: a2,
                }
            } else {
                Self {
                    lower: a2,
                    upper: a1,
                }
            }
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_basic.ml#L232
    #[derive(Debug, Clone, PartialEq)]
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
            let (or_ignore, default_fn) = self;

            match or_ignore {
                OrIgnore::Check(this) => {
                    inputs.append_bool(true);
                    this.to_inputs(inputs);
                }
                OrIgnore::Ignore => {
                    inputs.append_bool(false);
                    let default = default_fn();
                    default.to_inputs(inputs);
                }
            };
        }
    }

    impl<T, F> ToFieldElements<Fp> for (&OrIgnore<T>, F)
    where
        T: ToFieldElements<Fp>,
        F: Fn() -> T,
    {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            let (or_ignore, default_fn) = self;

            match or_ignore {
                OrIgnore::Check(this) => {
                    Boolean::True.to_field_elements(fields);
                    this.to_field_elements(fields);
                }
                OrIgnore::Ignore => {
                    Boolean::False.to_field_elements(fields);
                    let default = default_fn();
                    default.to_field_elements(fields);
                }
            };
        }
    }

    impl<T, F> Check<Fp> for (&OrIgnore<T>, F)
    where
        T: Check<Fp>,
        F: Fn() -> T,
    {
        fn check(&self, w: &mut Witness<Fp>) {
            let (or_ignore, default_fn) = self;
            let value = match or_ignore {
                OrIgnore::Check(this) => MyCow::Borrow(this),
                OrIgnore::Ignore => MyCow::Own(default_fn()),
            };
            value.check(w);
        }
    }

    impl<T> OrIgnore<T>
    where
        T: OutSnarkCheck<A = T>,
    {
        /// zkapp check
        pub fn zcheck(&self, label: String, rhs: T::B) -> Result<(), String> {
            // println!("[rust] check {}, {:?}", label, ret);
            match self {
                Self::Ignore => Ok(()),
                Self::Check(t) => t.zcheck(label, rhs),
            }
        }
    }

    impl<T> OrIgnore<T> {
        /// https://github.com/MinaProtocol/mina/blob/d7d4aa4d650eb34b45a42b29276554802683ce15/src/lib/mina_base/zkapp_basic.ml#L239
        pub fn gen<F>(mut fun: F) -> Self
        where
            F: FnMut() -> T,
        {
            let mut rng = rand::thread_rng();

            if rng.gen() {
                Self::Check(fun())
            } else {
                Self::Ignore
            }
        }

        pub fn map<F, V>(&self, fun: F) -> OrIgnore<V>
        where
            F: Fn(&T) -> V,
        {
            match self {
                OrIgnore::Check(v) => OrIgnore::Check(fun(v)),
                OrIgnore::Ignore => OrIgnore::Ignore,
            }
        }
    }

    impl<T> OrIgnore<ClosedInterval<T>>
    where
        T: PartialOrd,
    {
        /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/zkapp_precondition.ml#L294
        pub fn is_constant(&self) -> bool {
            match self {
                OrIgnore::Check(interval) => interval.lower == interval.upper,
                OrIgnore::Ignore => false,
            }
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L439
    pub type Hash<T> = OrIgnore<T>;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L298
    pub type EqData<T> = OrIgnore<T>;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L178
    pub type Numeric<T> = OrIgnore<ClosedInterval<T>>;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/epoch_ledger.ml#L9
    #[derive(Debug, Clone, PartialEq)]
    pub struct EpochLedger {
        pub hash: Hash<Fp>,
        pub total_currency: Numeric<Amount>,
    }

    impl EpochLedger {
        pub fn epoch_ledger(&self, t: protocol_state::EpochLedger<Fp>) -> Result<(), String> {
            self.hash.zcheck("epoch_ledger_hash".to_string(), t.hash)?;
            self.total_currency
                .zcheck("epoch_ledger_total_currency".to_string(), t.total_currency)
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L797
    #[derive(Debug, Clone, PartialEq)]
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
                inputs.append(&(total_currency, ClosedInterval::min_max));
            }

            inputs.append(&(seed, Fp::zero));
            inputs.append(&(start_checkpoint, Fp::zero));
            inputs.append(&(lock_checkpoint, Fp::zero));
            inputs.append(&(epoch_length, ClosedInterval::min_max));
        }
    }

    impl ToFieldElements<Fp> for EpochData {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
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

                (hash, Fp::zero).to_field_elements(fields);
                (total_currency, ClosedInterval::min_max).to_field_elements(fields);
            }

            (seed, Fp::zero).to_field_elements(fields);
            (start_checkpoint, Fp::zero).to_field_elements(fields);
            (lock_checkpoint, Fp::zero).to_field_elements(fields);
            (epoch_length, ClosedInterval::min_max).to_field_elements(fields);
        }
    }

    impl Check<Fp> for EpochData {
        fn check(&self, w: &mut Witness<Fp>) {
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

                (hash, Fp::zero).check(w);
                (total_currency, ClosedInterval::min_max).check(w);
            }

            (seed, Fp::zero).check(w);
            (start_checkpoint, Fp::zero).check(w);
            (lock_checkpoint, Fp::zero).check(w);
            (epoch_length, ClosedInterval::min_max).check(w);
        }
    }

    impl EpochData {
        pub fn epoch_data(
            &self,
            label: &str,
            t: protocol_state::EpochData<Fp>,
        ) -> Result<(), String> {
            self.ledger.epoch_ledger(t.ledger)?;
            // ignore seed
            self.start_checkpoint.zcheck(
                format!("{}_{}", label, "start_checkpoint"),
                t.start_checkpoint,
            )?;
            self.lock_checkpoint.zcheck(
                format!("{}_{}", label, "lock_checkpoint"),
                t.lock_checkpoint,
            )?;
            self.epoch_length
                .zcheck(format!("{}_{}", label, "epoch_length"), t.epoch_length)
        }

        pub fn gen() -> Self {
            let mut rng = rand::thread_rng();

            EpochData {
                ledger: EpochLedger {
                    hash: OrIgnore::gen(|| Fp::rand(&mut rng)),
                    total_currency: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
                },
                seed: OrIgnore::gen(|| Fp::rand(&mut rng)),
                start_checkpoint: OrIgnore::gen(|| Fp::rand(&mut rng)),
                lock_checkpoint: OrIgnore::gen(|| Fp::rand(&mut rng)),
                epoch_length: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
            }
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L977
    #[derive(Debug, Clone, PartialEq)]
    pub struct ZkAppPreconditions {
        pub snarked_ledger_hash: Hash<Fp>,
        pub blockchain_length: Numeric<Length>,
        pub min_window_density: Numeric<Length>,
        pub total_currency: Numeric<Amount>,
        pub global_slot_since_genesis: Numeric<Slot>,
        pub staking_epoch_data: EpochData,
        pub next_epoch_data: EpochData,
    }

    impl ZkAppPreconditions {
        /// zkapp check
        pub fn zcheck(&self, s: ProtocolStateView) -> Result<(), String> {
            self.snarked_ledger_hash
                .zcheck("snarker_ledger_hash".to_string(), s.snarked_ledger_hash)?;
            self.blockchain_length
                .zcheck("blockchain_length".to_string(), s.blockchain_length)?;
            self.min_window_density
                .zcheck("min_window_density".to_string(), s.min_window_density)?;
            self.total_currency
                .zcheck("total_currency".to_string(), s.total_currency)?;
            self.global_slot_since_genesis.zcheck(
                "global_slot_since_genesis".to_string(),
                s.global_slot_since_genesis,
            )?;
            self.staking_epoch_data
                .epoch_data("staking_epoch_data", s.staking_epoch_data)?;
            self.next_epoch_data
                .epoch_data("next_epoch_data", s.next_epoch_data)
        }

        pub fn checked_zcheck(&self, s: &ProtocolStateView, w: &mut Witness<Fp>) -> Boolean {
            let Self {
                snarked_ledger_hash,
                blockchain_length,
                min_window_density,
                total_currency,
                global_slot_since_genesis,
                staking_epoch_data,
                next_epoch_data,
            } = self;

            // NOTE: Here the 2nd element in the tuples is the default value of `OrIgnore`

            let epoch_data = |epoch_data: &EpochData,
                              view: &protocol_state::EpochData<Fp>,
                              w: &mut Witness<Fp>| {
                let EpochData {
                    ledger:
                        EpochLedger {
                            hash,
                            total_currency,
                        },
                    seed: _,
                    start_checkpoint,
                    lock_checkpoint,
                    epoch_length,
                } = epoch_data;
                // Reverse to match OCaml order of the list, while still executing `checked_zcheck`
                // in correct order
                [
                    (epoch_length, ClosedInterval::min_max).checked_zcheck(&view.epoch_length, w),
                    (lock_checkpoint, Fp::zero).checked_zcheck(&view.lock_checkpoint, w),
                    (start_checkpoint, Fp::zero).checked_zcheck(&view.start_checkpoint, w),
                    (total_currency, ClosedInterval::min_max)
                        .checked_zcheck(&view.ledger.total_currency, w),
                    (hash, Fp::zero).checked_zcheck(&view.ledger.hash, w),
                ]
            };

            let next_epoch_data = epoch_data(next_epoch_data, &s.next_epoch_data, w);
            let staking_epoch_data = epoch_data(staking_epoch_data, &s.staking_epoch_data, w);

            // Reverse to match OCaml order of the list, while still executing `checked_zcheck`
            // in correct order
            let bools = [
                (global_slot_since_genesis, ClosedInterval::min_max)
                    .checked_zcheck(&s.global_slot_since_genesis, w),
                (total_currency, ClosedInterval::min_max).checked_zcheck(&s.total_currency, w),
                (min_window_density, ClosedInterval::min_max)
                    .checked_zcheck(&s.min_window_density, w),
                (blockchain_length, ClosedInterval::min_max)
                    .checked_zcheck(&s.blockchain_length, w),
                (snarked_ledger_hash, Fp::zero).checked_zcheck(&s.snarked_ledger_hash, w),
            ]
            .into_iter()
            .rev()
            .chain(staking_epoch_data.into_iter().rev())
            .chain(next_epoch_data.into_iter().rev())
            .collect::<Vec<_>>();

            Boolean::all(&bools, w)
        }

        /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/zkapp_precondition.ml#L1303
        pub fn accept() -> Self {
            let epoch_data = || EpochData {
                ledger: EpochLedger {
                    hash: OrIgnore::Ignore,
                    total_currency: OrIgnore::Ignore,
                },
                seed: OrIgnore::Ignore,
                start_checkpoint: OrIgnore::Ignore,
                lock_checkpoint: OrIgnore::Ignore,
                epoch_length: OrIgnore::Ignore,
            };

            Self {
                snarked_ledger_hash: OrIgnore::Ignore,
                blockchain_length: OrIgnore::Ignore,
                min_window_density: OrIgnore::Ignore,
                total_currency: OrIgnore::Ignore,
                global_slot_since_genesis: OrIgnore::Ignore,
                staking_epoch_data: epoch_data(),
                next_epoch_data: epoch_data(),
            }
        }
    }

    impl ToInputs for ZkAppPreconditions {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_precondition.ml#L1052
        fn to_inputs(&self, inputs: &mut Inputs) {
            let ZkAppPreconditions {
                snarked_ledger_hash,
                blockchain_length,
                min_window_density,
                total_currency,
                global_slot_since_genesis,
                staking_epoch_data,
                next_epoch_data,
            } = &self;

            inputs.append(&(snarked_ledger_hash, Fp::zero));
            inputs.append(&(blockchain_length, ClosedInterval::min_max));
            inputs.append(&(min_window_density, ClosedInterval::min_max));
            inputs.append(&(total_currency, ClosedInterval::min_max));
            inputs.append(&(global_slot_since_genesis, ClosedInterval::min_max));
            inputs.append(staking_epoch_data);
            inputs.append(next_epoch_data);
        }
    }

    impl ToFieldElements<Fp> for ZkAppPreconditions {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            let Self {
                snarked_ledger_hash,
                blockchain_length,
                min_window_density,
                total_currency,
                global_slot_since_genesis,
                staking_epoch_data,
                next_epoch_data,
            } = self;

            (snarked_ledger_hash, Fp::zero).to_field_elements(fields);
            (blockchain_length, ClosedInterval::min_max).to_field_elements(fields);
            (min_window_density, ClosedInterval::min_max).to_field_elements(fields);
            (total_currency, ClosedInterval::min_max).to_field_elements(fields);
            (global_slot_since_genesis, ClosedInterval::min_max).to_field_elements(fields);
            staking_epoch_data.to_field_elements(fields);
            next_epoch_data.to_field_elements(fields);
        }
    }

    impl Check<Fp> for ZkAppPreconditions {
        fn check(&self, w: &mut Witness<Fp>) {
            let Self {
                snarked_ledger_hash,
                blockchain_length,
                min_window_density,
                total_currency,
                global_slot_since_genesis,
                staking_epoch_data,
                next_epoch_data,
            } = self;

            (snarked_ledger_hash, Fp::zero).check(w);
            (blockchain_length, ClosedInterval::min_max).check(w);
            (min_window_density, ClosedInterval::min_max).check(w);
            (total_currency, ClosedInterval::min_max).check(w);
            (global_slot_since_genesis, ClosedInterval::min_max).check(w);
            staking_epoch_data.check(w);
            next_epoch_data.check(w);
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/da6ba9a52e71d03ec6b6803b01f6d249eebc1ccb/src/lib/mina_base/zkapp_basic.ml#L401
    fn invalid_public_key() -> CompressedPubKey {
        CompressedPubKey {
            x: Fp::zero(),
            is_odd: false,
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L478
    #[derive(Debug, Clone, PartialEq)]
    pub struct Account {
        pub balance: Numeric<Balance>,
        pub nonce: Numeric<Nonce>,
        pub receipt_chain_hash: Hash<Fp>, // TODO: Should be type `ReceiptChainHash`
        pub delegate: EqData<CompressedPubKey>,
        pub state: [EqData<Fp>; 8],
        pub action_state: EqData<Fp>,
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
                action_state: EqData::Ignore,
                proved_state: EqData::Ignore,
                is_new: EqData::Ignore,
            }
        }
    }

    impl Account {
        /// zkapp check
        pub fn zcheck<F>(&self, new_account: bool, mut check: F, a: account::Account)
        where
            F: FnMut(TransactionFailure, bool),
        {
            self.zchecks(new_account, a)
                .iter()
                .for_each(|(failure, res)| check(failure.clone(), res.is_ok()))
        }

        fn zchecks(
            &self,
            new_account: bool,
            a: account::Account,
        ) -> Vec<(TransactionFailure, Result<(), String>)> {
            let zkapp = a.zkapp.unwrap_or_default();
            let mut ret = vec![
                (
                    TransactionFailure::AccountBalancePreconditionUnsatisfied,
                    self.balance.zcheck("balance".to_string(), a.balance),
                ),
                (
                    TransactionFailure::AccountNoncePreconditionUnsatisfied,
                    self.nonce.zcheck("nonce".to_string(), a.nonce),
                ),
                (
                    TransactionFailure::AccountReceiptChainHashPreconditionUnsatisfied,
                    self.receipt_chain_hash
                        .zcheck("receipt_chain_hash".to_string(), a.receipt_chain_hash.0),
                ),
                (
                    TransactionFailure::AccountDelegatePreconditionUnsatisfied,
                    self.delegate.zcheck(
                        "delegate".to_string(),
                        a.delegate.unwrap_or_else(invalid_public_key),
                    ),
                ),
                (
                    TransactionFailure::AccountActionStatePreconditionUnsatisfied,
                    match zkapp
                        .action_state
                        .iter()
                        .find(|state| self.action_state.zcheck("".to_string(), **state).is_ok())
                    {
                        None => Err("Action state mismatch".to_string()),
                        Some(_) => Ok(()),
                    },
                ),
            ];

            for (i, (c, v)) in self.state.iter().zip(zkapp.app_state.iter()).enumerate() {
                ret.push((
                    TransactionFailure::AccountAppStatePreconditionUnsatisfied(i as i64),
                    c.zcheck(format!("state[{}]", i), *v),
                ));
            }

            let mut ret2 = vec![
                (
                    TransactionFailure::AccountProvedStatePreconditionUnsatisfied,
                    self.proved_state
                        .zcheck("proved_state".to_string(), zkapp.proved_state),
                ),
                (
                    TransactionFailure::AccountIsNewPreconditionUnsatisfied,
                    self.is_new.zcheck("is_new".to_string(), new_account),
                ),
            ];

            ret.append(&mut ret2);
            ret
        }

        fn checked_zchecks(
            &self,
            account: &crate::Account,
            new_account: Boolean,
            w: &mut Witness<Fp>,
        ) -> Vec<(TransactionFailure, Boolean)> {
            use TransactionFailure::*;

            let Self {
                balance,
                nonce,
                receipt_chain_hash,
                delegate,
                state,
                action_state,
                proved_state,
                is_new,
            } = self;

            let zkapp_account = account.zkapp_or_empty();
            let is_new = is_new.map(ToBoolean::to_boolean);
            let proved_state = proved_state.map(ToBoolean::to_boolean);

            // NOTE: Here we need to execute all `checked_zcheck` in the exact same order than OCaml
            // so we execute them in reverse order (compared to OCaml): OCaml evaluates from right
            // to left.
            // We then have to reverse the resulting vector, to match OCaml resulting list.

            // NOTE 2: Here the 2nd element in the tuples is the default value of `OrIgnore`
            let mut checks: Vec<(TransactionFailure, _)> = [
                (
                    AccountIsNewPreconditionUnsatisfied,
                    (&is_new, || Boolean::False).checked_zcheck(&new_account, w),
                ),
                (
                    AccountProvedStatePreconditionUnsatisfied,
                    (&proved_state, || Boolean::False)
                        .checked_zcheck(&zkapp_account.proved_state.to_boolean(), w),
                ),
            ]
            .into_iter()
            .chain({
                let bools = state
                    .iter()
                    .zip(&zkapp_account.app_state)
                    .enumerate()
                    .rev()
                    .map(|(i, (s, account_s))| {
                        let b = (s, Fp::zero).checked_zcheck(account_s, w);
                        (AccountAppStatePreconditionUnsatisfied(i as i64), b)
                    })
                    .collect::<Vec<_>>();
                // We need to call rev here, in order to match OCaml order
                bools.into_iter().rev()
            })
            .chain([
                {
                    let bools: Vec<_> = zkapp_account
                        .action_state
                        .iter()
                        .map(|account_s| {
                            (action_state, ZkAppAccount::empty_action_state)
                                .checked_zcheck(account_s, w)
                        })
                        .collect();
                    (
                        AccountActionStatePreconditionUnsatisfied,
                        Boolean::any(&bools, w),
                    )
                },
                (
                    AccountDelegatePreconditionUnsatisfied,
                    (delegate, CompressedPubKey::empty)
                        .checked_zcheck(&*account.delegate_or_empty(), w),
                ),
                (
                    AccountReceiptChainHashPreconditionUnsatisfied,
                    (receipt_chain_hash, Fp::zero).checked_zcheck(&account.receipt_chain_hash.0, w),
                ),
                (
                    AccountNoncePreconditionUnsatisfied,
                    (nonce, ClosedInterval::min_max).checked_zcheck(&account.nonce, w),
                ),
                (
                    AccountBalancePreconditionUnsatisfied,
                    (balance, ClosedInterval::min_max).checked_zcheck(&account.balance, w),
                ),
            ])
            .collect::<Vec<_>>();

            checks.reverse();
            checks
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L613
    #[derive(Debug, Clone, PartialEq)]
    pub enum AccountPreconditions {
        Full(Box<Account>),
        Nonce(Nonce),
        Accept,
    }

    impl ToInputs for AccountPreconditions {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L635
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_precondition.ml#L568
        fn to_inputs(&self, inputs: &mut Inputs) {
            let account = self.to_full();

            let Account {
                balance,
                nonce,
                receipt_chain_hash,
                delegate,
                state,
                action_state,
                proved_state,
                is_new,
            } = account.as_ref();

            inputs.append(&(balance, ClosedInterval::min_max));
            inputs.append(&(nonce, ClosedInterval::min_max));
            inputs.append(&(receipt_chain_hash, Fp::zero));
            inputs.append(&(delegate, CompressedPubKey::empty));
            state.iter().for_each(|s| {
                inputs.append(&(s, Fp::zero));
            });
            // https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_account.ml#L168
            inputs.append(&(action_state, ZkAppAccount::empty_action_state));
            inputs.append(&(proved_state, || false));
            inputs.append(&(is_new, || false));
        }
    }

    impl ToFieldElements<Fp> for AccountPreconditions {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            let account = self.to_full();

            let Account {
                balance,
                nonce,
                receipt_chain_hash,
                delegate,
                state,
                action_state,
                proved_state,
                is_new,
            } = account.as_ref();

            (balance, ClosedInterval::min_max).to_field_elements(fields);
            (nonce, ClosedInterval::min_max).to_field_elements(fields);
            (receipt_chain_hash, Fp::zero).to_field_elements(fields);
            (delegate, CompressedPubKey::empty).to_field_elements(fields);
            state.iter().for_each(|s| {
                (s, Fp::zero).to_field_elements(fields);
            });
            (action_state, ZkAppAccount::empty_action_state).to_field_elements(fields);
            (proved_state, || false).to_field_elements(fields);
            (is_new, || false).to_field_elements(fields);
        }
    }

    impl Check<Fp> for AccountPreconditions {
        fn check(&self, w: &mut Witness<Fp>) {
            let account = self.to_full();

            let Account {
                balance,
                nonce,
                receipt_chain_hash,
                delegate,
                state,
                action_state,
                proved_state,
                is_new,
            } = account.as_ref();

            (balance, ClosedInterval::min_max).check(w);
            (nonce, ClosedInterval::min_max).check(w);
            (receipt_chain_hash, Fp::zero).check(w);
            (delegate, CompressedPubKey::empty).check(w);
            state.iter().for_each(|s| {
                (s, Fp::zero).check(w);
            });
            (action_state, ZkAppAccount::empty_action_state).check(w);
            (proved_state, || false).check(w);
            (is_new, || false).check(w);
        }
    }

    impl AccountPreconditions {
        pub fn nonce(&self) -> Numeric<Nonce> {
            match self {
                Self::Full(account) => account.nonce.clone(),
                Self::Nonce(nonce) => Numeric::Check(ClosedInterval {
                    lower: *nonce,
                    upper: *nonce,
                }),
                Self::Accept => Numeric::Ignore,
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L635
        pub fn to_full(&self) -> MyCow<Account> {
            match self {
                AccountPreconditions::Full(s) => MyCow::Borrow(&**s),
                AccountPreconditions::Nonce(nonce) => {
                    let mut account = Account::accept();
                    account.nonce = OrIgnore::Check(ClosedInterval {
                        lower: *nonce,
                        upper: *nonce,
                    });
                    MyCow::Own(account)
                }
                AccountPreconditions::Accept => MyCow::Own(Account::accept()),
            }
        }

        pub fn checked_zcheck<Fun>(
            &self,
            new_account: Boolean,
            account: &crate::Account,
            mut check: Fun,
            w: &mut Witness<Fp>,
        ) where
            Fun: FnMut(TransactionFailure, Boolean, &mut Witness<Fp>),
        {
            let this = self.to_full();
            for (failure, passed) in this.checked_zchecks(account, new_account, w) {
                check(failure, passed, w);
            }
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L758
    #[derive(Debug, Clone, PartialEq)]
    pub struct Preconditions {
        pub(crate) network: ZkAppPreconditions,
        pub account: AccountPreconditions,
        pub valid_while: Numeric<Slot>,
    }

    impl ToFieldElements<Fp> for Preconditions {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            let Self {
                network,
                account,
                valid_while,
            } = self;

            network.to_field_elements(fields);
            account.to_field_elements(fields);
            (valid_while, ClosedInterval::min_max).to_field_elements(fields);
        }
    }

    impl Check<Fp> for Preconditions {
        fn check(&self, w: &mut Witness<Fp>) {
            let Self {
                network,
                account,
                valid_while,
            } = self;

            network.check(w);
            account.check(w);
            (valid_while, ClosedInterval::min_max).check(w);
        }
    }

    impl ToInputs for Preconditions {
        /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/account_update.ml#L1148
        fn to_inputs(&self, inputs: &mut Inputs) {
            let Self {
                network,
                account,
                valid_while,
            } = self;

            inputs.append(network);
            inputs.append(account);
            inputs.append(&(valid_while, ClosedInterval::min_max));
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L27
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum AuthorizationKind {
        NoneGiven,
        Signature,
        Proof(Fp), // hash
    }

    impl AuthorizationKind {
        pub fn vk_hash(&self) -> Fp {
            match self {
                AuthorizationKind::NoneGiven | AuthorizationKind::Signature => {
                    VerificationKey::dummy().hash()
                }
                AuthorizationKind::Proof(hash) => *hash,
            }
        }

        pub fn is_proved(&self) -> bool {
            match self {
                AuthorizationKind::Proof(_) => true,
                AuthorizationKind::NoneGiven => false,
                AuthorizationKind::Signature => false,
            }
        }

        pub fn is_signed(&self) -> bool {
            match self {
                AuthorizationKind::Proof(_) => false,
                AuthorizationKind::NoneGiven => false,
                AuthorizationKind::Signature => true,
            }
        }

        fn to_structured(&self) -> ([bool; 2], Fp) {
            // bits: [is_signed, is_proved]
            let bits = match self {
                AuthorizationKind::NoneGiven => [false, false],
                AuthorizationKind::Signature => [true, false],
                AuthorizationKind::Proof(_) => [false, true],
            };
            let field = self.vk_hash();
            (bits, field)
        }
    }

    impl ToInputs for AuthorizationKind {
        /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/account_update.ml#L142
        fn to_inputs(&self, inputs: &mut Inputs) {
            let (bits, field) = self.to_structured();

            for bit in bits {
                inputs.append_bool(bit);
            }
            inputs.append_field(field);
        }
    }

    impl ToFieldElements<Fp> for AuthorizationKind {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            self.to_structured().to_field_elements(fields);
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/account_update.ml#L1311
    #[derive(Debug, Clone, PartialEq)]
    pub struct Body {
        pub public_key: CompressedPubKey,
        pub token_id: TokenId,
        pub update: Update,
        pub balance_change: Signed<Amount>,
        pub increment_nonce: bool,
        pub events: Events,
        pub actions: Actions,
        pub call_data: Fp,
        pub preconditions: Preconditions,
        pub use_full_commitment: bool,
        pub implicit_account_creation_fee: bool,
        pub may_use_token: MayUseToken,
        pub authorization_kind: AuthorizationKind,
    }

    impl ToInputs for Body {
        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L1297
        fn to_inputs(&self, inputs: &mut Inputs) {
            let Self {
                public_key,
                token_id,
                update,
                balance_change,
                increment_nonce,
                events,
                actions,
                call_data,
                preconditions,
                use_full_commitment,
                implicit_account_creation_fee,
                may_use_token,
                authorization_kind,
            } = self;

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
                inputs.append(&(permissions, Permissions::empty));
                inputs.append(&(&zkapp_uri.map(Some), || Option::<&ZkAppUri>::None));
                inputs.append(&(token_symbol, TokenSymbol::default));
                inputs.append(&(timing, Timing::dummy));
                inputs.append(&(voting_for, VotingFor::dummy));
            }

            inputs.append(balance_change);
            inputs.append(increment_nonce);
            inputs.append(events);
            inputs.append(actions);
            inputs.append(call_data);
            inputs.append(preconditions);
            inputs.append(use_full_commitment);
            inputs.append(implicit_account_creation_fee);
            inputs.append(may_use_token);
            inputs.append(authorization_kind);
        }
    }

    impl ToFieldElements<Fp> for Body {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            let Self {
                public_key,
                token_id,
                update,
                balance_change,
                increment_nonce,
                events,
                actions,
                call_data,
                preconditions,
                use_full_commitment,
                implicit_account_creation_fee,
                may_use_token,
                authorization_kind,
            } = self;

            public_key.to_field_elements(fields);
            token_id.to_field_elements(fields);
            update.to_field_elements(fields);
            balance_change.to_field_elements(fields);
            increment_nonce.to_field_elements(fields);
            events.to_field_elements(fields);
            actions.to_field_elements(fields);
            call_data.to_field_elements(fields);
            preconditions.to_field_elements(fields);
            use_full_commitment.to_field_elements(fields);
            implicit_account_creation_fee.to_field_elements(fields);
            may_use_token.to_field_elements(fields);
            authorization_kind.to_field_elements(fields);
        }
    }

    impl Check<Fp> for Body {
        fn check(&self, w: &mut Witness<Fp>) {
            let Self {
                public_key: _,
                token_id: _,
                update:
                    Update {
                        app_state: _,
                        delegate: _,
                        verification_key: _,
                        permissions: _,
                        zkapp_uri: _,
                        token_symbol,
                        timing,
                        voting_for: _,
                    },
                balance_change,
                increment_nonce: _,
                events: _,
                actions: _,
                call_data: _,
                preconditions,
                use_full_commitment: _,
                implicit_account_creation_fee: _,
                may_use_token,
                authorization_kind: _,
            } = self;

            (token_symbol, TokenSymbol::default).check(w);
            (timing, Timing::dummy).check(w);
            balance_change.check(w);

            preconditions.check(w);
            may_use_token.check(w);
        }
    }

    impl Body {
        pub fn account_id(&self) -> AccountId {
            let Self {
                public_key,
                token_id,
                ..
            } = self;
            AccountId::create(public_key.clone(), token_id.clone())
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/account_update.ml#L1284
    #[derive(Debug, Clone, PartialEq)]
    pub struct BodySimple {
        pub public_key: CompressedPubKey,
        pub token_id: TokenId,
        pub update: Update,
        pub balance_change: Signed<Amount>,
        pub increment_nonce: bool,
        pub events: Events,
        pub actions: Actions,
        pub call_data: Fp,
        pub call_depth: usize,
        pub preconditions: Preconditions,
        pub use_full_commitment: bool,
        pub implicit_account_creation_fee: bool,
        pub may_use_token: MayUseToken,
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
    pub type SideLoadedProof = Arc<mina_p2p_messages::v2::PicklesProofProofsVerifiedMaxStableV2>;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/control.ml#L11
    #[derive(Clone, PartialEq)]
    pub enum Control {
        Proof(SideLoadedProof),
        Signature(Signature),
        NoneGiven,
    }

    impl std::fmt::Debug for Control {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Proof(_) => f.debug_tuple("Proof").field(&"_").finish(),
                Self::Signature(arg0) => f.debug_tuple("Signature").field(arg0).finish(),
                Self::NoneGiven => write!(f, "NoneGiven"),
            }
        }
    }

    impl Control {
        /// https://github.com/MinaProtocol/mina/blob/d7d4aa4d650eb34b45a42b29276554802683ce15/src/lib/mina_base/control.ml#L81
        pub fn tag(&self) -> crate::ControlTag {
            match self {
                Control::Proof(_) => crate::ControlTag::Proof,
                Control::Signature(_) => crate::ControlTag::Signature,
                Control::NoneGiven => crate::ControlTag::NoneGiven,
            }
        }

        pub fn dummy_of_tag(tag: ControlTag) -> Self {
            match tag {
                ControlTag::Proof => Self::Proof(dummy::sideloaded_proof()),
                ControlTag::Signature => Self::Signature(Signature::dummy()),
                ControlTag::NoneGiven => Self::NoneGiven,
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub enum MayUseToken {
        /// No permission to use any token other than the default Mina
        /// token
        No,
        /// Has permission to use the token owned by the direct parent of
        /// this account update, which may be inherited by child account
        /// updates.
        ParentsOwnToken,
        /// Inherit the token permission available to the parent.
        InheritFromParent,
    }

    impl MayUseToken {
        pub fn parents_own_token(&self) -> bool {
            matches!(self, Self::ParentsOwnToken)
        }

        pub fn inherit_from_parent(&self) -> bool {
            matches!(self, Self::InheritFromParent)
        }

        fn to_bits(&self) -> [bool; 2] {
            // [ parents_own_token; inherit_from_parent ]
            match self {
                MayUseToken::No => [false, false],
                MayUseToken::ParentsOwnToken => [true, false],
                MayUseToken::InheritFromParent => [false, true],
            }
        }
    }

    impl ToInputs for MayUseToken {
        fn to_inputs(&self, inputs: &mut Inputs) {
            for bit in self.to_bits() {
                inputs.append_bool(bit);
            }
        }
    }

    impl ToFieldElements<Fp> for MayUseToken {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            for bit in self.to_bits() {
                bit.to_field_elements(fields);
            }
        }
    }

    impl Check<Fp> for MayUseToken {
        fn check(&self, w: &mut Witness<Fp>) {
            use crate::proofs::field::field;

            let [parents_own_token, inherit_from_parent] = self.to_bits();
            let [parents_own_token, inherit_from_parent] = [
                parents_own_token.to_boolean(),
                inherit_from_parent.to_boolean(),
            ];

            let sum = parents_own_token.to_field::<Fp>() + inherit_from_parent.to_field::<Fp>();
            let _sum_squared = field::mul(sum, sum, w);
        }
    }

    pub struct CheckAuthorizationResult<Bool> {
        pub proof_verifies: Bool,
        pub signature_verifies: Bool,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L1437
    pub type AccountUpdate = AccountUpdateSkeleton<Body>;

    #[derive(Debug, Clone, PartialEq)]
    pub struct AccountUpdateSkeleton<Body> {
        pub body: Body,
        pub authorization: Control,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L1395
    #[derive(Debug, Clone, PartialEq)]
    pub struct AccountUpdateSimple {
        pub body: BodySimple,
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

            inputs.append(body);
        }
    }

    impl AccountUpdate {
        // TODO: mainnet: MainnetZkappBody
        pub const HASH_PARAM: &'static str = "TestnetZkappBody";

        /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/account_update.ml#L1538
        /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/account_update.ml#L1465
        pub fn of_fee_payer(fee_payer: FeePayer) -> Self {
            let FeePayer {
                body:
                    FeePayerBody {
                        public_key,
                        fee,
                        valid_until,
                        nonce,
                    },
                authorization,
            } = fee_payer;

            Self {
                body: Body {
                    public_key,
                    token_id: TokenId::default(),
                    update: Update::noop(),
                    balance_change: Signed {
                        magnitude: Amount::of_fee(&fee),
                        sgn: Sgn::Neg,
                    },
                    increment_nonce: true,
                    events: Events::empty(),
                    actions: Actions::empty(),
                    call_data: Fp::zero(),
                    preconditions: Preconditions {
                        network: {
                            let mut network = ZkAppPreconditions::accept();

                            let valid_util = valid_until.unwrap_or_else(Slot::max);
                            network.global_slot_since_genesis = OrIgnore::Check(ClosedInterval {
                                lower: Slot::zero(),
                                upper: valid_util,
                            });

                            network
                        },
                        account: AccountPreconditions::Nonce(nonce),
                        valid_while: Numeric::Ignore,
                    },
                    use_full_commitment: true,
                    authorization_kind: AuthorizationKind::Signature,
                    implicit_account_creation_fee: true,
                    may_use_token: MayUseToken::No,
                },
                authorization: Control::Signature(authorization),
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/account_update.ml#L1535
        pub fn account_id(&self) -> AccountId {
            AccountId::new(self.body.public_key.clone(), self.body.token_id.clone())
        }

        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/account_update.ml#L1327
        pub fn digest(&self) -> Fp {
            // TODO: mainnet: "MainnetZkappBody"
            self.hash_with_param(AccountUpdate::HASH_PARAM)
        }

        pub fn timing(&self) -> SetOrKeep<Timing> {
            self.body.update.timing.clone()
        }

        pub fn may_use_parents_own_token(&self) -> bool {
            self.body.may_use_token.parents_own_token()
        }

        pub fn may_use_token_inherited_from_parent(&self) -> bool {
            self.body.may_use_token.inherit_from_parent()
        }

        pub fn public_key(&self) -> CompressedPubKey {
            self.body.public_key.clone()
        }

        pub fn token_id(&self) -> TokenId {
            self.body.token_id.clone()
        }

        pub fn increment_nonce(&self) -> bool {
            self.body.increment_nonce
        }

        pub fn implicit_account_creation_fee(&self) -> bool {
            self.body.implicit_account_creation_fee
        }

        // commitment and calls argument are ignored here, only used in the transaction snark
        pub fn check_authorization(
            &self,
            _will_succeed: bool,
            _commitment: Fp,
            _calls: CallForest<AccountUpdate>,
        ) -> CheckAuthorizationResult<bool> {
            match self.authorization {
                Control::Signature(_) => CheckAuthorizationResult {
                    proof_verifies: false,
                    signature_verifies: true,
                },
                Control::Proof(_) => CheckAuthorizationResult {
                    proof_verifies: true,
                    signature_verifies: false,
                },
                Control::NoneGiven => CheckAuthorizationResult {
                    proof_verifies: false,
                    signature_verifies: false,
                },
            }
        }

        pub fn permissions(&self) -> SetOrKeep<Permissions<AuthRequired>> {
            self.body.update.permissions.clone()
        }

        pub fn app_state(&self) -> [SetOrKeep<Fp>; 8] {
            self.body.update.app_state.clone()
        }

        pub fn zkapp_uri(&self) -> SetOrKeep<ZkAppUri> {
            self.body.update.zkapp_uri.clone()
        }

        /*
        pub fn token_symbol(&self) -> SetOrKeep<[u8; 6]> {
            self.body.update.token_symbol.clone()
        }
        */

        pub fn token_symbol(&self) -> SetOrKeep<TokenSymbol> {
            self.body.update.token_symbol.clone()
        }

        pub fn delegate(&self) -> SetOrKeep<CompressedPubKey> {
            self.body.update.delegate.clone()
        }

        pub fn voting_for(&self) -> SetOrKeep<VotingFor> {
            self.body.update.voting_for.clone()
        }

        pub fn verification_key(&self) -> SetOrKeep<VerificationKey> {
            self.body.update.verification_key.map(|vk| vk.data.clone())
        }

        pub fn valid_while_precondition(&self) -> OrIgnore<ClosedInterval<Slot>> {
            self.body.preconditions.valid_while.clone()
        }

        pub fn actions(&self) -> Actions {
            self.body.actions.clone()
        }

        pub fn balance_change(&self) -> Signed<Amount> {
            self.body.balance_change
        }
        pub fn use_full_commitment(&self) -> bool {
            self.body.use_full_commitment
        }

        pub fn protocol_state_precondition(&self) -> ZkAppPreconditions {
            self.body.preconditions.network.clone()
        }

        pub fn account_precondition(&self) -> AccountPreconditions {
            self.body.preconditions.account.clone()
        }

        pub fn is_proved(&self) -> bool {
            match &self.body.authorization_kind {
                AuthorizationKind::Proof(_) => true,
                AuthorizationKind::Signature | AuthorizationKind::NoneGiven => false,
            }
        }

        pub fn is_signed(&self) -> bool {
            match &self.body.authorization_kind {
                AuthorizationKind::Signature => true,
                AuthorizationKind::Proof(_) | AuthorizationKind::NoneGiven => false,
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/transaction_logic/mina_transaction_logic.ml#L1708
        pub fn verification_key_hash(&self) -> Option<Fp> {
            match &self.body.authorization_kind {
                AuthorizationKind::Proof(vk_hash) => Some(*vk_hash),
                _ => None,
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/account_update.ml#L1333
        pub fn of_simple(simple: &AccountUpdateSimple) -> Self {
            let AccountUpdateSimple {
                body:
                    BodySimple {
                        public_key,
                        token_id,
                        update,
                        balance_change,
                        increment_nonce,
                        events,
                        actions,
                        call_data,
                        call_depth: _,
                        preconditions,
                        use_full_commitment,
                        implicit_account_creation_fee,
                        may_use_token,
                        authorization_kind,
                    },
                authorization,
            } = simple.clone();

            Self {
                body: Body {
                    public_key,
                    token_id,
                    update,
                    balance_change,
                    increment_nonce,
                    events,
                    actions,
                    call_data,
                    preconditions,
                    use_full_commitment,
                    implicit_account_creation_fee,
                    may_use_token,
                    authorization_kind,
                },
                authorization,
            }
        }

        /// Usage: Random `AccountUpdate` to compare hashes with OCaml
        pub fn rand() -> Self {
            let mut rng = rand::thread_rng();
            let rng = &mut rng;

            Self {
                body: Body {
                    public_key: gen_compressed(),
                    token_id: TokenId(Fp::rand(rng)),
                    update: Update {
                        app_state: std::array::from_fn(|_| SetOrKeep::gen(|| Fp::rand(rng))),
                        delegate: SetOrKeep::gen(gen_compressed),
                        verification_key: SetOrKeep::gen(|| {
                            let vk = VerificationKey::gen();
                            let hash = vk.hash();
                            WithHash { data: vk, hash }
                        }),
                        permissions: SetOrKeep::gen(|| {
                            let auth_tag = [
                                ControlTag::NoneGiven,
                                ControlTag::Proof,
                                ControlTag::Signature,
                            ]
                            .choose(rng)
                            .unwrap();

                            Permissions::gen(*auth_tag)
                        }),
                        zkapp_uri: SetOrKeep::gen(ZkAppUri::gen),
                        token_symbol: SetOrKeep::gen(TokenSymbol::gen),
                        timing: SetOrKeep::gen(|| Timing {
                            initial_minimum_balance: rng.gen(),
                            cliff_time: rng.gen(),
                            cliff_amount: rng.gen(),
                            vesting_period: rng.gen(),
                            vesting_increment: rng.gen(),
                        }),
                        voting_for: SetOrKeep::gen(|| VotingFor(Fp::rand(rng))),
                    },
                    balance_change: Signed::gen(),
                    increment_nonce: rng.gen(),
                    events: Events(gen_events()),
                    actions: Actions(gen_events()),
                    call_data: Fp::rand(rng),
                    preconditions: Preconditions {
                        network: ZkAppPreconditions {
                            snarked_ledger_hash: OrIgnore::gen(|| Fp::rand(rng)),
                            blockchain_length: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
                            min_window_density: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
                            total_currency: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
                            global_slot_since_genesis: OrIgnore::gen(|| {
                                ClosedInterval::gen(|| rng.gen())
                            }),
                            staking_epoch_data: EpochData::gen(),
                            next_epoch_data: EpochData::gen(),
                        },
                        account: {
                            match AccountPreconditions::Accept {
                                AccountPreconditions::Full(_) => (),
                                AccountPreconditions::Nonce(_) => (),
                                AccountPreconditions::Accept => (),
                            };

                            match rng.gen_range(0..3) {
                                0 => AccountPreconditions::Accept,
                                1 => AccountPreconditions::Nonce(rng.gen()),
                                _ => AccountPreconditions::Full(Box::new(Account {
                                    balance: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
                                    nonce: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
                                    receipt_chain_hash: OrIgnore::gen(|| Fp::rand(rng)),
                                    delegate: OrIgnore::gen(gen_compressed),
                                    state: std::array::from_fn(|_| OrIgnore::gen(|| Fp::rand(rng))),
                                    action_state: OrIgnore::gen(|| Fp::rand(rng)),
                                    proved_state: OrIgnore::gen(|| rng.gen()),
                                    is_new: OrIgnore::gen(|| rng.gen()),
                                })),
                            }
                        },
                        valid_while: OrIgnore::gen(|| ClosedInterval::gen(|| rng.gen())),
                    },
                    use_full_commitment: rng.gen(),
                    implicit_account_creation_fee: rng.gen(),
                    may_use_token: {
                        match MayUseToken::No {
                            MayUseToken::No => (),
                            MayUseToken::ParentsOwnToken => (),
                            MayUseToken::InheritFromParent => (),
                        };

                        [
                            MayUseToken::No,
                            MayUseToken::InheritFromParent,
                            MayUseToken::ParentsOwnToken,
                        ]
                        .choose(rng)
                        .cloned()
                        .unwrap()
                    },
                    authorization_kind: {
                        match AuthorizationKind::NoneGiven {
                            AuthorizationKind::NoneGiven => (),
                            AuthorizationKind::Signature => (),
                            AuthorizationKind::Proof(_) => (),
                        };

                        [
                            AuthorizationKind::NoneGiven,
                            AuthorizationKind::Signature,
                            AuthorizationKind::Proof(Fp::rand(rng)),
                        ]
                        .choose(rng)
                        .cloned()
                        .unwrap()
                    },
                },
                authorization: {
                    match Control::NoneGiven {
                        Control::Proof(_) => (),
                        Control::Signature(_) => (),
                        Control::NoneGiven => (),
                    };

                    match rng.gen_range(0..3) {
                        0 => Control::NoneGiven,
                        1 => Control::Signature(Signature::dummy()),
                        _ => Control::Proof(dummy::sideloaded_proof()),
                    }
                },
            }
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L49
    #[derive(Debug, Clone, PartialEq)]
    pub struct Tree<AccUpdate: Clone> {
        pub account_update: AccUpdate,
        pub account_update_digest: Fp,
        pub calls: CallForest<AccUpdate>,
    }

    impl<AccUpdate: Clone> Tree<AccUpdate> {
        pub const HASH_PARAM: &'static str = "MinaAcctUpdateNode";

        pub fn digest(&self) -> Fp {
            let stack_hash = match self.calls.0.first() {
                Some(e) => e.stack_hash,
                None => Fp::zero(),
            };

            // self.account_update_digest should have been updated in `CallForest::accumulate_hashes`
            assert_ne!(self.account_update_digest, Fp::zero());

            hash_with_kimchi(Self::HASH_PARAM, &[self.account_update_digest, stack_hash])
        }

        fn fold<F>(&self, init: Vec<AccountId>, f: &mut F) -> Vec<AccountId>
        where
            F: FnMut(Vec<AccountId>, &AccUpdate) -> Vec<AccountId>,
        {
            self.calls.fold(f(init, &self.account_update), f)
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/with_stack_hash.ml#L6
    #[derive(Debug, Clone, PartialEq)]
    pub struct WithStackHash<AccUpdate: Clone> {
        pub elt: Tree<AccUpdate>,
        pub stack_hash: Fp,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L345
    #[derive(Debug, Clone, PartialEq)]
    pub struct CallForest<AccUpdate: Clone>(pub Vec<WithStackHash<AccUpdate>>);

    impl<Data: Clone> Default for CallForest<Data> {
        fn default() -> Self {
            Self::new()
        }
    }

    #[derive(Clone)]
    struct CallForestContext {
        caller: TokenId,
        this: TokenId,
    }

    pub const ACCOUNT_UPDATE_CONS_HASH_PARAM: &'static str = "MinaAcctUpdateCons";

    impl<AccUpdate: Clone> CallForest<AccUpdate> {
        pub fn new() -> Self {
            Self(Vec::new())
        }

        pub fn empty() -> Self {
            Self::new()
        }

        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        // In OCaml push/pop to the head is cheap because they work with lists.
        // In Rust we use vectors so we will push/pop to the tail.
        // To work with the elements as if they were in the original order we need to iterate backwards
        pub fn iter(&self) -> impl Iterator<Item = &WithStackHash<AccUpdate>> {
            self.0.iter() //.rev()
        }
        // Warning: Update this if we ever change the order
        pub fn first(&self) -> Option<&WithStackHash<AccUpdate>> {
            self.0.first()
        }
        // Warning: Update this if we ever change the order
        pub fn tail(&self) -> Option<&[WithStackHash<AccUpdate>]> {
            self.0.get(1..)
        }

        pub fn hash(&self) -> Fp {
            /*
            for x in self.0.iter() {
                println!("hash: {:?}", x.stack_hash);
            }
            */

            if let Some(x) = self.first() {
                x.stack_hash
            } else {
                Fp::zero()
            }
        }

        fn cons_tree(&self, tree: Tree<AccUpdate>) -> Self {
            let hash = tree.digest();
            let h_tl = self.hash();

            let stack_hash = hash_with_kimchi(ACCOUNT_UPDATE_CONS_HASH_PARAM, &[hash, h_tl]);
            let node = WithStackHash::<AccUpdate> {
                elt: tree,
                stack_hash,
            };
            let mut forest = Vec::with_capacity(self.0.len() + 1);
            forest.push(node);
            forest.extend(self.0.iter().cloned());

            Self(forest)
        }

        pub fn pop_exn(&self) -> ((AccUpdate, CallForest<AccUpdate>), CallForest<AccUpdate>) {
            if self.0.is_empty() {
                panic!()
            }

            let Tree::<AccUpdate> {
                account_update,
                calls,
                ..
            } = self.0[0].elt.clone();
            (
                (account_update, calls),
                CallForest(Vec::from_iter(self.0[1..].iter().cloned())),
            )
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/zkapp_command.ml#L68
        fn fold_impl<A, F>(&self, init: A, fun: &mut F) -> A
        where
            F: FnMut(A, &AccUpdate) -> A,
        {
            let mut accum = init;
            for elem in self.iter() {
                accum = fun(accum, &elem.elt.account_update);
                accum = elem.elt.calls.fold_impl(accum, fun);
            }
            accum
        }

        pub fn fold<A, F>(&self, init: A, mut fun: F) -> A
        where
            F: FnMut(A, &AccUpdate) -> A,
        {
            self.fold_impl(init, &mut fun)
        }

        fn map_to_impl<F, AnotherAccUpdate: Clone>(&self, fun: &F) -> CallForest<AnotherAccUpdate>
        where
            F: Fn(&AccUpdate) -> AnotherAccUpdate,
        {
            CallForest::<AnotherAccUpdate>(
                self.iter()
                    .map(|item| WithStackHash::<AnotherAccUpdate> {
                        elt: Tree::<AnotherAccUpdate> {
                            account_update: fun(&item.elt.account_update),
                            account_update_digest: item.elt.account_update_digest,
                            calls: item.elt.calls.map_to_impl(fun),
                        },
                        stack_hash: item.stack_hash,
                    })
                    .collect(),
            )
        }

        #[must_use]
        pub fn map_to<F, AnotherAccUpdate: Clone>(&self, fun: F) -> CallForest<AnotherAccUpdate>
        where
            F: Fn(&AccUpdate) -> AnotherAccUpdate,
        {
            self.map_to_impl(&fun)
        }

        fn map_with_trees_to_impl<F, AnotherAccUpdate: Clone>(
            &self,
            fun: &F,
        ) -> CallForest<AnotherAccUpdate>
        where
            F: Fn(&AccUpdate, &Tree<AccUpdate>) -> AnotherAccUpdate,
        {
            CallForest::<AnotherAccUpdate>(
                self.iter()
                    .map(|item| {
                        let account_update = fun(&item.elt.account_update, &item.elt);

                        WithStackHash::<AnotherAccUpdate> {
                            elt: Tree::<AnotherAccUpdate> {
                                account_update,
                                account_update_digest: item.elt.account_update_digest,
                                calls: item.elt.calls.map_with_trees_to_impl(fun),
                            },
                            stack_hash: item.stack_hash,
                        }
                    })
                    .collect(),
            )
        }

        #[must_use]
        pub fn map_with_trees_to<F, AnotherAccUpdate: Clone>(
            &self,
            fun: F,
        ) -> CallForest<AnotherAccUpdate>
        where
            F: Fn(&AccUpdate, &Tree<AccUpdate>) -> AnotherAccUpdate,
        {
            self.map_with_trees_to_impl(&fun)
        }

        fn try_map_to_impl<F, E, AnotherAccUpdate: Clone>(
            &self,
            fun: &mut F,
        ) -> Result<CallForest<AnotherAccUpdate>, E>
        where
            F: FnMut(&AccUpdate) -> Result<AnotherAccUpdate, E>,
        {
            Ok(CallForest::<AnotherAccUpdate>(
                self.iter()
                    .map(|item| {
                        Ok(WithStackHash::<AnotherAccUpdate> {
                            elt: Tree::<AnotherAccUpdate> {
                                account_update: fun(&item.elt.account_update)?,
                                account_update_digest: item.elt.account_update_digest,
                                calls: item.elt.calls.try_map_to_impl(fun)?,
                            },
                            stack_hash: item.stack_hash,
                        })
                    })
                    .collect::<Result<_, E>>()?,
            ))
        }

        pub fn try_map_to<F, E, AnotherAccUpdate: Clone>(
            &self,
            mut fun: F,
        ) -> Result<CallForest<AnotherAccUpdate>, E>
        where
            F: FnMut(&AccUpdate) -> Result<AnotherAccUpdate, E>,
        {
            self.try_map_to_impl(&mut fun)
        }

        fn to_account_updates_impl(&self, accounts: &mut Vec<AccUpdate>) {
            // TODO: Check iteration order in OCaml
            for elem in self.iter() {
                accounts.push(elem.elt.account_update.clone());
                elem.elt.calls.to_account_updates_impl(accounts);
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/zkapp_command.ml#L436
        pub fn to_account_updates(&self) -> Vec<AccUpdate> {
            let mut accounts = Vec::with_capacity(128);
            self.to_account_updates_impl(&mut accounts);
            accounts
        }

        fn to_zkapp_command_with_hashes_list_impl(&self, output: &mut Vec<(AccUpdate, Fp)>) {
            self.iter().for_each(|item| {
                let WithStackHash { elt, stack_hash } = item;
                let Tree {
                    account_update,
                    account_update_digest: _,
                    calls,
                } = elt;
                output.push((account_update.clone(), *stack_hash));
                calls.to_zkapp_command_with_hashes_list_impl(output);
            });
        }

        pub fn to_zkapp_command_with_hashes_list(&self) -> Vec<(AccUpdate, Fp)> {
            let mut output = Vec::with_capacity(128);
            self.to_zkapp_command_with_hashes_list_impl(&mut output);
            output
        }
    }

    impl CallForest<AccountUpdate> {
        pub fn cons(
            &self,
            calls: Option<CallForest<AccountUpdate>>,
            account_update: AccountUpdate,
        ) -> Self {
            let account_update_digest = account_update.digest();

            let tree = Tree::<AccountUpdate> {
                account_update,
                account_update_digest,
                calls: calls.unwrap_or_else(|| CallForest(Vec::new())),
            };
            self.cons_tree(tree)
        }

        /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_command.ml#L583
        pub fn accumulate_hashes<F>(&mut self, hash_account_update: &F)
        where
            F: Fn(&AccountUpdate) -> Fp,
        {
            /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_command.ml#L293
            fn cons(hash: Fp, h_tl: Fp) -> Fp {
                hash_with_kimchi(ACCOUNT_UPDATE_CONS_HASH_PARAM, &[hash, h_tl])
            }

            /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/zkapp_command.ml#L561
            fn hash<AccUpdate: Clone>(elem: Option<&WithStackHash<AccUpdate>>) -> Fp {
                match elem {
                    Some(next) => next.stack_hash,
                    None => Fp::zero(),
                }
            }

            // We traverse the list in reverse here (to get same behavior as OCaml recursivity)
            // Note that reverse here means 0 to last, see `CallForest::iter` for explaination
            //
            // We use indexes to make the borrow checker happy

            for index in (0..self.0.len()).rev() {
                let elem = &mut self.0[index];
                let WithStackHash {
                    elt:
                        Tree::<AccountUpdate> {
                            account_update,
                            account_update_digest,
                            calls,
                            ..
                        },
                    ..
                } = elem;

                calls.accumulate_hashes(hash_account_update);
                *account_update_digest = hash_account_update(account_update);

                let node_hash = elem.elt.digest();
                let hash = hash(self.0.get(index + 1));

                self.0[index].stack_hash = cons(node_hash, hash);
            }
        }

        pub fn accumulate_hashes_predicated(&mut self) {
            // Note: There seems to be no difference with `accumulate_hashes`
            self.accumulate_hashes(&|account_update| account_update.digest());
        }

        /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/zkapp_command.ml#L830
        pub fn of_wire(
            &mut self,
            _wired: &[MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA],
        ) {
            self.accumulate_hashes(&|account_update| account_update.digest());
        }

        /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/zkapp_command.ml#L840
        pub fn to_wire(
            &self,
            _wired: &mut [MinaBaseZkappCommandTStableV1WireStableV1AccountUpdatesA],
        ) {
            // self.remove_callers(wired);
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L1081
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FeePayerBody {
        pub public_key: CompressedPubKey,
        pub fee: Fee,
        pub valid_until: Option<Slot>,
        pub nonce: Nonce,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L1484
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FeePayer {
        pub body: FeePayerBody,
        pub authorization: Signature,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L959
    #[derive(Debug, Clone, PartialEq)]
    pub struct ZkAppCommand {
        pub fee_payer: FeePayer,
        pub account_updates: CallForest<AccountUpdate>,
        pub memo: Memo,
    }

    #[derive(Debug, Clone, PartialEq, Hash, Eq)]
    pub enum AccessedOrNot {
        Accessed,
        NotAccessed,
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

        /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/zkapp_command.ml#L997
        pub fn account_access_statuses(
            &self,
            status: &TransactionStatus,
        ) -> Vec<(AccountId, AccessedOrNot)> {
            use AccessedOrNot::*;
            use TransactionStatus::*;

            // always `Accessed` for fee payer
            let init = vec![(self.fee_payer(), Accessed)];

            let status_sym = match status {
                Applied => Accessed,
                Failed(_) => NotAccessed,
            };

            let ids = self
                .account_updates
                .fold(init, |mut accum, account_update| {
                    accum.push((account_update.account_id(), status_sym.clone()));
                    accum
                });
            // WARNING: the code previous to merging latest changes wasn't doing the "rev()" call. Check this in case of errors.
            ids.iter()
                .unique() /*.rev()*/
                .cloned()
                .collect()
        }

        /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/zkapp_command.ml#L1006
        pub fn accounts_referenced(&self) -> Vec<AccountId> {
            self.account_access_statuses(&TransactionStatus::Applied)
                .into_iter()
                .map(|(id, _status)| id)
                .collect()
        }

        /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/zkapp_command.ml#L1346
        pub fn of_verifiable(verifiable: verifiable::ZkAppCommand) -> Self {
            Self {
                fee_payer: verifiable.fee_payer,
                account_updates: verifiable.account_updates.map_to(|(acc, _)| acc.clone()),
                memo: verifiable.memo,
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/zkapp_command.ml#L1386
        pub fn account_updates_hash(&self) -> Fp {
            self.account_updates.hash()
        }

        /// https://github.com/MinaProtocol/mina/blob/02c9d453576fa47f78b2c388fb2e0025c47d991c/src/lib/mina_base/zkapp_command.ml#L989
        pub fn extract_vks(&self) -> Vec<WithHash<VerificationKey>> {
            self.account_updates
                .fold(Vec::with_capacity(256), |mut acc, p| {
                    if let SetOrKeep::Set(vk) = &p.body.update.verification_key {
                        acc.push(vk.clone());
                    };
                    acc
                })
        }

        pub fn all_account_updates(&self) -> CallForest<AccountUpdate> {
            let p = &self.fee_payer;

            let mut fee_payer = AccountUpdate::of_fee_payer(p.clone());
            fee_payer.authorization = Control::Signature(p.authorization.clone());

            self.account_updates.cons(None, fee_payer)
        }

        pub fn all_account_updates_list(&self) -> Vec<AccountUpdate> {
            let mut account_updates = Vec::with_capacity(16);
            account_updates.push(AccountUpdate::of_fee_payer(self.fee_payer.clone()));

            self.account_updates.fold(account_updates, |mut acc, u| {
                acc.push(u.clone());
                acc
            })
        }

        pub fn commitment(&self) -> TransactionCommitment {
            let account_updates_hash = self.account_updates_hash();
            TransactionCommitment::create(account_updates_hash)
        }
    }

    pub mod verifiable {
        use std::collections::HashMap;

        use super::*;
        use crate::VerificationKey;

        #[derive(Debug, Clone)]
        pub struct ZkAppCommand {
            pub fee_payer: FeePayer,
            pub account_updates: CallForest<(AccountUpdate, Option<WithHash<VerificationKey>>)>,
            pub memo: Memo,
        }

        fn ok_if_vk_hash_expected(
            got: WithHash<VerificationKey>,
            expected: Fp,
        ) -> Result<WithHash<VerificationKey>, String> {
            if got.hash == expected {
                return Ok(got);
            }
            Err(format!(
                "Expected vk hash doesn't match hash in vk we received\
                         expected: {:?}\
                         got: {:?}",
                expected, got
            ))
        }

        pub fn find_vk_via_ledger<L>(
            ledger: L,
            expected_vk_hash: Fp,
            account_id: &AccountId,
        ) -> Result<WithHash<VerificationKey>, String>
        where
            L: LedgerIntf + Clone,
        {
            let vk = ledger
                .location_of_account(account_id)
                .and_then(|location| ledger.get(&location))
                .and_then(|account| {
                    account
                        .zkapp
                        .as_ref()
                        .and_then(|zkapp| zkapp.verification_key.clone())
                });

            match vk {
                Some(vk) => {
                    // TODO: The account should contains the `WithHash<VerificationKey>`
                    let hash = vk.hash();
                    let vk = WithHash { data: vk, hash };

                    ok_if_vk_hash_expected(vk, expected_vk_hash)
                }
                None => Err(format!(
                    "No verification key found for proved account update\
                                     account_id: {:?}",
                    account_id
                )),
            }
        }

        fn check_authorization(p: &AccountUpdate) -> Result<(), String> {
            use AuthorizationKind as AK;
            use Control as C;

            match (&p.authorization, &p.body.authorization_kind) {
                (C::NoneGiven, AK::NoneGiven)
                | (C::Proof(_), AK::Proof(_))
                | (C::Signature(_), AK::Signature) => Ok(()),
                _ => Err(format!(
                    "Authorization kind does not match the authorization\
                                 expected={:#?}\
                                 got={:#?}",
                    p.body.authorization_kind, p.authorization
                )),
            }
        }

        /// Ensures that there's a verification_key available for all account_updates
        /// and creates a valid command associating the correct keys with each
        /// account_id.
        ///
        /// If an account_update replaces the verification_key (or deletes it),
        /// subsequent account_updates use the replaced key instead of looking in the
        /// ledger for the key (ie set by a previous transaction).
        pub fn create(
            zkapp: &super::ZkAppCommand,
            status: &TransactionStatus,
            find_vk: impl Fn(Fp, &AccountId) -> Result<WithHash<VerificationKey>, String>,
        ) -> Result<ZkAppCommand, String> {
            let super::ZkAppCommand {
                fee_payer,
                account_updates,
                memo,
            } = zkapp;

            let mut tbl = HashMap::with_capacity(128);
            // Keep track of the verification keys that have been set so far
            // during this transaction.
            let mut vks_overridden = HashMap::with_capacity(128);

            let account_updates = account_updates.try_map_to(|p| {
                let account_id = p.account_id();

                if let SetOrKeep::Set(vk_next) = &p.body.update.verification_key {
                    vks_overridden.insert(account_id.clone(), Some(vk_next.clone()));
                }

                check_authorization(p)?;

                match (&p.body.authorization_kind, status.is_applied()) {
                    (AuthorizationKind::Proof(vk_hash), true) => {
                        let prioritized_vk = {
                            // only lookup _past_ vk setting, ie exclude the new one we
                            // potentially set in this account_update (use the non-'
                            // vks_overrided) .

                            match vks_overridden.get(&account_id) {
                                Some(Some(vk)) => {
                                    ok_if_vk_hash_expected(vk.clone(), *vk_hash)?
                                },
                                Some(None) => {
                                    // we explicitly have erased the key
                                    return Err(format!("No verification key found for proved account \
                                                        update: the verification key was removed by a \
                                                        previous account update\
                                                        account_id={:?}", account_id));
                                }
                                None => {
                                    // we haven't set anything; lookup the vk in the fallback
                                    find_vk(*vk_hash, &account_id)?
                                },
                            }
                        };

                        tbl.insert(account_id, prioritized_vk.hash);

                        Ok((p.clone(), Some(prioritized_vk)))
                    },

                    _ => {
                        Ok((p.clone(), None))
                    }
                }
            })?;

            Ok(ZkAppCommand {
                fee_payer: fee_payer.clone(),
                account_updates,
                memo: memo.clone(),
            })
        }
    }

    pub mod valid {
        use crate::scan_state::transaction_logic::zkapp_command::verifiable::create;

        use super::*;

        #[derive(Clone, Debug, PartialEq)]
        pub struct ZkAppCommand {
            pub zkapp_command: super::ZkAppCommand,
        }

        impl ZkAppCommand {
            pub fn forget(self) -> super::ZkAppCommand {
                self.zkapp_command
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/zkapp_command.ml#L1499
        pub fn of_verifiable(cmd: verifiable::ZkAppCommand) -> ZkAppCommand {
            ZkAppCommand {
                zkapp_command: super::ZkAppCommand::of_verifiable(cmd),
            }
        }

        /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/zkapp_command.ml#L1507
        pub fn to_valid(
            zkapp_command: super::ZkAppCommand,
            status: &TransactionStatus,
            find_vk: impl Fn(Fp, &AccountId) -> Result<WithHash<VerificationKey>, String>,
        ) -> Result<ZkAppCommand, String> {
            create(&zkapp_command, status, find_vk).map(of_verifiable)
        }
    }

    pub trait Strategy {
        fn create_all(
            cmds: Vec<WithStatus<Box<ZkAppCommand>>>,
            find_vk: impl Fn(Fp, &AccountId) -> Result<WithHash<VerificationKey>, String>,
        ) -> Result<Vec<WithStatus<verifiable::ZkAppCommand>>, String>;
    }

    trait CreateAll {
        type Value;

        fn empty() -> Self;
        fn find(&self, key: Fp) -> Option<&Self::Value>;
        fn set(&mut self, key: Fp, value: Self::Value);
    }

    impl<T> Strategy for T
    where
        T: CreateAll<Value = WithHash<VerificationKey>>,
    {
        /// https://github.com/MinaProtocol/mina/blob/02c9d453576fa47f78b2c388fb2e0025c47d991c/src/lib/mina_base/zkapp_command.ml#L1346
        fn create_all(
            cmds: Vec<WithStatus<Box<ZkAppCommand>>>,
            find_vk: impl Fn(Fp, &AccountId) -> Result<WithHash<VerificationKey>, String>,
        ) -> Result<Vec<WithStatus<verifiable::ZkAppCommand>>, String> {
            let mut running_cache = Self::empty();

            Ok(cmds
                .into_iter()
                .map(|WithStatus { data: cmd, status }| {
                    let verified_cmd = verifiable::create(&cmd, &status, |vk_hash, account_id| {
                        // first we check if there's anything in the running
                        // cache within this chunk so far

                        match running_cache.find(vk_hash) {
                            None => {
                                // before falling back to the find_vk
                                find_vk(vk_hash, account_id)
                            }
                            Some(vk) => Ok(vk.clone()),
                        }
                    })
                    .unwrap();

                    for vk in cmd.extract_vks() {
                        running_cache.set(vk.hash, vk);
                    }

                    WithStatus {
                        data: verified_cmd,
                        status,
                    }
                })
                .collect())
        }
    }

    mod any {
        use std::collections::HashMap;

        use super::*;

        struct Any<T> {
            inner: std::collections::HashMap<Fp, T>,
        }

        impl<T> super::CreateAll for Any<T> {
            type Value = T;

            fn empty() -> Self {
                Self {
                    inner: HashMap::with_capacity(128),
                }
            }

            fn find(&self, key: Fp) -> Option<&Self::Value> {
                self.inner.get(&key)
            }

            fn set(&mut self, key: Fp, value: Self::Value) {
                self.inner.insert(key, value);
            }
        }
    }

    pub mod last {
        use super::*;

        pub struct Last<T> {
            inner: Option<(Fp, T)>,
        }

        impl<T> CreateAll for Last<T> {
            type Value = T;

            fn empty() -> Self {
                Self { inner: None }
            }

            fn find(&self, key: Fp) -> Option<&Self::Value> {
                match self.inner.as_ref() {
                    Some((k, value)) if k == &key => Some(value),
                    _ => None,
                }
            }

            fn set(&mut self, key: Fp, value: Self::Value) {
                self.inner = Some((key, value));
            }
        }
    }
}

pub mod zkapp_statement {
    use super::{
        zkapp_command::{CallForest, Tree, ACCOUNT_UPDATE_CONS_HASH_PARAM},
        *,
    };

    #[derive(Copy, Clone, Debug, derive_more::Deref, derive_more::From)]
    pub struct TransactionCommitment(pub Fp);

    impl TransactionCommitment {
        /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/zkapp_command.ml#L1365
        pub fn create(account_updates_hash: Fp) -> Self {
            Self(account_updates_hash)
        }

        /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/zkapp_command.ml#L1368
        pub fn create_complete(&self, memo_hash: Fp, fee_payer_hash: Fp) -> Self {
            Self(hash_with_kimchi(
                ACCOUNT_UPDATE_CONS_HASH_PARAM,
                &[memo_hash, fee_payer_hash, self.0],
            ))
        }

        pub fn empty() -> Self {
            Self(Fp::zero())
        }
    }

    impl mina_hasher::Hashable for TransactionCommitment {
        type D = mina_signer::NetworkId;

        fn to_roinput(&self) -> mina_hasher::ROInput {
            mina_hasher::ROInput::new().append_field(self.0)
        }

        fn domain_string(network_id: mina_signer::NetworkId) -> Option<String> {
            // Domain strings must have length <= 20
            match network_id {
                mina_signer::NetworkId::MAINNET => "MinaSignatureMainnet",
                mina_signer::NetworkId::TESTNET => "CodaSignature",
            }
            .to_string()
            .into()
        }
    }

    #[derive(Clone, Debug)]
    pub struct ZkappStatement {
        pub account_update: TransactionCommitment,
        pub calls: TransactionCommitment,
    }

    impl ZkappStatement {
        pub fn to_field_elements(&self) -> Vec<Fp> {
            let Self {
                account_update,
                calls,
            } = self;

            vec![**account_update, **calls]
        }

        pub fn of_tree<AccUpdate: Clone>(tree: &Tree<AccUpdate>) -> Self {
            let Tree {
                account_update: _,
                account_update_digest,
                calls,
            } = tree;

            Self {
                account_update: TransactionCommitment(*account_update_digest),
                calls: TransactionCommitment(calls.hash()),
            }
        }

        pub fn zkapp_statements_of_forest_prime<Data: Clone>(
            forest: CallForest<(AccountUpdate, Data)>,
        ) -> CallForest<(AccountUpdate, (Data, Self))> {
            forest.map_with_trees_to(|(account_update, data), tree| {
                (account_update.clone(), (data.clone(), Self::of_tree(tree)))
            })
        }

        fn zkapp_statements_of_forest(
            forest: CallForest<AccountUpdate>,
        ) -> CallForest<(AccountUpdate, Self)> {
            forest.map_with_trees_to(|account_update, tree| {
                (account_update.clone(), Self::of_tree(tree))
            })
        }
    }
}

pub mod verifiable {
    use std::ops::Neg;

    use ark_ff::{BigInteger, PrimeField};
    use mina_signer::Signer;

    use super::*;

    #[derive(Debug)]
    pub enum UserCommand {
        SignedCommand(Box<signed_command::SignedCommand>),
        ZkAppCommand(Box<zkapp_command::verifiable::ZkAppCommand>),
    }

    pub fn compressed_to_pubkey(pubkey: &CompressedPubKey) -> mina_signer::PubKey {
        // Taken from https://github.com/o1-labs/proof-systems/blob/e3fc04ce87f8695288de167115dea80050ab33f4/signer/src/pubkey.rs#L95-L106
        let mut pt = mina_signer::CurvePoint::get_point_from_x(pubkey.x, pubkey.is_odd).unwrap();

        if pt.y.into_repr().is_even() == pubkey.is_odd {
            pt.y = pt.y.neg();
        }

        assert!(pt.is_on_curve());

        // Safe now because we checked point pt is on curve
        mina_signer::PubKey::from_point_unsafe(pt)
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/signed_command.ml#L436
    pub fn check_only_for_signature(
        cmd: Box<signed_command::SignedCommand>,
    ) -> Result<valid::UserCommand, Box<signed_command::SignedCommand>> {
        // https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/signed_command.ml#L396

        let signed_command::SignedCommand {
            payload,
            signer: pubkey,
            signature,
        } = &*cmd;

        let payload = TransactionUnionPayload::of_user_command_payload(payload);
        let pubkey = compressed_to_pubkey(pubkey);

        let mut signer = mina_signer::create_legacy(mina_signer::NetworkId::TESTNET);

        if signer.verify(signature, &pubkey, &payload) {
            Ok(valid::UserCommand::SignedCommand(cmd))
        } else {
            Err(cmd)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UserCommand {
    SignedCommand(Box<signed_command::SignedCommand>),
    ZkAppCommand(Box<zkapp_command::ZkAppCommand>),
}

impl From<&UserCommand> for MinaBaseUserCommandStableV2 {
    fn from(user_command: &UserCommand) -> Self {
        match user_command {
            UserCommand::SignedCommand(signed_command) => {
                MinaBaseUserCommandStableV2::SignedCommand((&(*(signed_command.clone()))).into())
            }
            UserCommand::ZkAppCommand(zkapp_command) => {
                MinaBaseUserCommandStableV2::ZkappCommand((&(*(zkapp_command.clone()))).into())
            }
        }
    }
}

impl From<&MinaBaseUserCommandStableV2> for UserCommand {
    fn from(user_command: &MinaBaseUserCommandStableV2) -> Self {
        match user_command {
            MinaBaseUserCommandStableV2::SignedCommand(signed_command) => {
                UserCommand::SignedCommand(Box::new(signed_command.into()))
            }
            MinaBaseUserCommandStableV2::ZkappCommand(zkapp_command) => {
                UserCommand::ZkAppCommand(Box::new(zkapp_command.into()))
            }
        }
    }
}

impl binprot::BinProtWrite for UserCommand {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let p2p: MinaBaseUserCommandStableV2 = self.into();
        p2p.binprot_write(w)
    }
}

impl binprot::BinProtRead for UserCommand {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error> {
        let p2p = MinaBaseUserCommandStableV2::binprot_read(r)?;
        Ok(UserCommand::from(&p2p))
    }
}

impl UserCommand {
    /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/user_command.ml#L239
    pub fn account_access_statuses(
        &self,
        status: &TransactionStatus,
    ) -> Vec<(AccountId, AccessedOrNot)> {
        match self {
            UserCommand::SignedCommand(cmd) => cmd.account_access_statuses(status).to_vec(),
            UserCommand::ZkAppCommand(cmd) => cmd.account_access_statuses(status),
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/user_command.ml#L247
    pub fn accounts_referenced(&self) -> Vec<AccountId> {
        self.account_access_statuses(&TransactionStatus::Applied)
            .into_iter()
            .map(|(id, _status)| id)
            .collect()
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/user_command.ml#L192
    pub fn fee(&self) -> Fee {
        match self {
            UserCommand::SignedCommand(cmd) => cmd.fee(),
            UserCommand::ZkAppCommand(cmd) => cmd.fee(),
        }
    }

    pub fn extract_vks(&self) -> Vec<WithHash<VerificationKey>> {
        match self {
            UserCommand::SignedCommand(_) => vec![],
            UserCommand::ZkAppCommand(zkapp) => zkapp.extract_vks(),
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/mina_base/user_command.ml#L339
    pub fn to_valid_unsafe(self) -> valid::UserCommand {
        match self {
            UserCommand::SignedCommand(cmd) => valid::UserCommand::SignedCommand(cmd),
            UserCommand::ZkAppCommand(cmd) => {
                valid::UserCommand::ZkAppCommand(Box::new(zkapp_command::valid::ZkAppCommand {
                    zkapp_command: *cmd,
                }))
            }
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/user_command.ml#L162
    pub fn to_verifiable<F>(
        &self,
        status: &TransactionStatus,
        find_vk: F,
    ) -> Result<verifiable::UserCommand, String>
    where
        F: Fn(Fp, &AccountId) -> Result<WithHash<VerificationKey>, String>,
    {
        use verifiable::UserCommand::{SignedCommand, ZkAppCommand};
        match self {
            UserCommand::SignedCommand(cmd) => Ok(SignedCommand(cmd.clone())),
            UserCommand::ZkAppCommand(zkapp) => Ok(ZkAppCommand(Box::new(
                zkapp_command::verifiable::create(zkapp, status, find_vk)?,
            ))),
        }
    }

    pub fn to_all_verifiable<S, F>(
        ts: Vec<WithStatus<Self>>,
        find_vk: F,
    ) -> Result<Vec<WithStatus<verifiable::UserCommand>>, String>
    where
        F: Fn(Fp, &AccountId) -> Result<WithHash<VerificationKey>, String>,
        S: zkapp_command::Strategy,
    {
        // https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/mina_base/user_command.ml#L180
        use itertools::Either;

        let (izk_cmds, is_cmds) = ts
            .into_iter()
            .enumerate()
            .partition_map::<Vec<_>, Vec<_>, _, _, _>(|(i, cmd)| match cmd.data {
                UserCommand::ZkAppCommand(c) => Either::Left((
                    i,
                    WithStatus {
                        data: c,
                        status: cmd.status,
                    },
                )),
                UserCommand::SignedCommand(c) => Either::Right((
                    i,
                    WithStatus {
                        data: c,
                        status: cmd.status,
                    },
                )),
            });

        // then unzip the indices
        let (ixs, zk_cmds): (Vec<_>, Vec<_>) = izk_cmds.into_iter().unzip();

        // then we verify the zkapp commands
        let vzk_cmds = S::create_all(zk_cmds, find_vk)?;

        // rezip indices
        let ivzk_cmds: Vec<_> = ixs.into_iter().zip(vzk_cmds).collect();

        // Put them back in with a sort by index (un-partition)

        use verifiable::UserCommand::{SignedCommand, ZkAppCommand};
        let mut ivs: Vec<_> = is_cmds
            .into_iter()
            .map(|(i, cmd)| (i, cmd.into_map(SignedCommand)))
            .chain(
                ivzk_cmds
                    .into_iter()
                    .map(|(i, cmd)| (i, cmd.into_map(|cmd| ZkAppCommand(Box::new(cmd))))),
            )
            .collect();

        ivs.sort_unstable_by_key(|(i, _)| *i);

        // Drop the indices
        Ok(ivs.into_iter().unzip::<_, _, Vec<_>, _>().1)
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

#[derive(Clone, Debug, derive_more::From)]
pub enum Transaction {
    Command(UserCommand),
    FeeTransfer(FeeTransfer),
    Coinbase(Coinbase),
}

impl Transaction {
    pub fn is_zkapp(&self) -> bool {
        matches!(self, Self::Command(UserCommand::ZkAppCommand(_)))
    }

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

    /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/transaction/transaction.ml#L98
    pub fn public_keys(&self) -> Vec<CompressedPubKey> {
        use Transaction::*;
        use UserCommand::*;

        let to_pks = |ids: Vec<AccountId>| ids.into_iter().map(|id| id.public_key).collect();

        match self {
            Command(SignedCommand(cmd)) => to_pks(cmd.accounts_referenced()),
            Command(ZkAppCommand(cmd)) => to_pks(cmd.accounts_referenced()),
            FeeTransfer(ft) => ft.receiver_pks().cloned().collect(),
            Coinbase(cb) => to_pks(cb.accounts_referenced()),
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/transaction/transaction.ml#L112
    pub fn account_access_statuses(
        &self,
        status: &TransactionStatus,
    ) -> Vec<(AccountId, zkapp_command::AccessedOrNot)> {
        use Transaction::*;
        use UserCommand::*;

        match self {
            Command(SignedCommand(cmd)) => cmd.account_access_statuses(status).to_vec(),
            Command(ZkAppCommand(cmd)) => cmd.account_access_statuses(status),
            FeeTransfer(ft) => ft
                .receivers()
                .map(|account_id| (account_id, AccessedOrNot::Accessed))
                .collect(),
            Coinbase(cb) => cb.account_access_statuses(status),
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/transaction/transaction.ml#L125
    pub fn accounts_referenced(&self) -> Vec<AccountId> {
        self.account_access_statuses(&TransactionStatus::Applied)
            .into_iter()
            .map(|(id, _status)| id)
            .collect()
    }
}

impl From<&Transaction> for MinaTransactionTransactionStableV2 {
    fn from(value: &Transaction) -> Self {
        match value {
            Transaction::Command(v) => Self::Command(Box::new(v.into())),
            Transaction::FeeTransfer(v) => Self::FeeTransfer(v.into()),
            Transaction::Coinbase(v) => Self::Coinbase(v.into()),
        }
    }
}

pub mod transaction_applied {
    use crate::{Account, AccountId};

    use super::*;

    pub mod signed_command_applied {
        use super::*;

        #[derive(Debug, Clone, PartialEq)]
        pub struct Common {
            pub user_command: WithStatus<signed_command::SignedCommand>,
        }

        #[derive(Debug, Clone, PartialEq)]
        pub enum Body {
            Payments {
                new_accounts: Vec<AccountId>,
            },
            StakeDelegation {
                previous_delegate: Option<CompressedPubKey>,
            },
            Failed,
        }

        #[derive(Debug, Clone, PartialEq)]
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
    #[derive(Debug, Clone, PartialEq)]
    pub struct ZkappCommandApplied {
        pub accounts: Vec<(AccountId, Option<Box<Account>>)>,
        pub command: WithStatus<zkapp_command::ZkAppCommand>,
        pub new_accounts: Vec<AccountId>,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L82
    #[derive(Debug, Clone, PartialEq)]
    pub enum CommandApplied {
        SignedCommand(Box<SignedCommandApplied>),
        ZkappCommand(Box<ZkappCommandApplied>),
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L96
    #[derive(Debug, Clone, PartialEq)]
    pub struct FeeTransferApplied {
        pub fee_transfer: WithStatus<FeeTransfer>,
        pub new_accounts: Vec<AccountId>,
        pub burned_tokens: Amount,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L112
    #[derive(Debug, Clone, PartialEq)]
    pub struct CoinbaseApplied {
        pub coinbase: WithStatus<Coinbase>,
        pub new_accounts: Vec<AccountId>,
        pub burned_tokens: Amount,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L142
    #[derive(Debug, Clone, PartialEq)]
    pub enum Varying {
        Command(CommandApplied),
        FeeTransfer(FeeTransferApplied),
        Coinbase(CoinbaseApplied),
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L142
    #[derive(Debug, Clone, PartialEq)]
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
                    total?.add(&amt.negate())
                });

            total.ok_or_else(|| "overflow".to_string())
        }
    }
}

pub mod transaction_witness {
    use mina_p2p_messages::v2::MinaStateProtocolStateBodyValueStableV2;

    use crate::scan_state::pending_coinbase::Stack;

    use super::*;

    /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/transaction_witness/transaction_witness.ml#L55
    #[derive(Debug)]
    pub struct TransactionWitness {
        pub transaction: Transaction,
        pub first_pass_ledger: SparseLedger,
        pub second_pass_ledger: SparseLedger,
        pub protocol_state_body: MinaStateProtocolStateBodyValueStableV2,
        pub init_stack: Stack,
        pub status: TransactionStatus,
        pub block_global_slot: Slot,
    }
}

pub mod protocol_state {
    use mina_p2p_messages::v2::{self, MinaStateProtocolStateValueStableV2};

    use crate::proofs::field::FieldWitness;

    use super::*;

    #[derive(Debug, Clone)]
    pub struct EpochLedger<F: FieldWitness> {
        pub hash: F,
        pub total_currency: Amount,
    }

    #[derive(Debug, Clone)]
    pub struct EpochData<F: FieldWitness> {
        pub ledger: EpochLedger<F>,
        pub seed: F,
        pub start_checkpoint: F,
        pub lock_checkpoint: F,
        pub epoch_length: Length,
    }

    #[derive(Debug, Clone)]
    pub struct ProtocolStateView {
        pub snarked_ledger_hash: Fp,
        pub blockchain_length: Length,
        pub min_window_density: Length,
        pub total_currency: Amount,
        pub global_slot_since_genesis: Slot,
        pub staking_epoch_data: EpochData<Fp>,
        pub next_epoch_data: EpochData<Fp>,
    }

    /// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/mina_state/protocol_state.ml#L180
    pub fn protocol_state_view(state: &MinaStateProtocolStateValueStableV2) -> ProtocolStateView {
        let MinaStateProtocolStateValueStableV2 {
            previous_state_hash: _,
            body,
        } = state;

        protocol_state_body_view(body)
    }

    pub fn protocol_state_body_view(
        body: &v2::MinaStateProtocolStateBodyValueStableV2,
    ) -> ProtocolStateView {
        let cs = &body.consensus_state;
        let sed = &cs.staking_epoch_data;
        let ned = &cs.next_epoch_data;

        ProtocolStateView {
            // https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/mina_state/blockchain_state.ml#L58
            //
            snarked_ledger_hash: body
                .blockchain_state
                .ledger_proof_statement
                .target
                .first_pass_ledger
                .to_field(),
            blockchain_length: Length(cs.blockchain_length.as_u32()),
            min_window_density: Length(cs.min_window_density.as_u32()),
            total_currency: Amount(cs.total_currency.as_u64()),
            global_slot_since_genesis: (&cs.global_slot_since_genesis).into(),
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

    pub type GlobalState<L> = GlobalStateSkeleton<L, Signed<Amount>, Slot>;

    #[derive(Debug, Clone)]
    pub struct GlobalStateSkeleton<L, SignedAmount, Slot> {
        pub first_pass_ledger: L,
        pub second_pass_ledger: L,
        pub fee_excess: SignedAmount,
        pub supply_increase: SignedAmount,
        pub protocol_state: ProtocolStateView,
        /// Slot of block when the transaction is applied.
        /// NOTE: This is at least 1 slot after the protocol_state's view,
        /// which is for the *previous* slot.
        pub block_global_slot: Slot,
    }

    impl<L: LedgerIntf + Clone> GlobalState<L> {
        pub fn first_pass_ledger(&self) -> L {
            self.first_pass_ledger.create_masked()
        }

        #[must_use]
        pub fn set_first_pass_ledger(&self, should_update: bool, ledger: L) -> Self {
            let mut this = self.clone();
            if should_update {
                this.first_pass_ledger.apply_mask(ledger);
            }
            this
        }

        pub fn second_pass_ledger(&self) -> L {
            self.second_pass_ledger.create_masked()
        }

        #[must_use]
        pub fn set_second_pass_ledger(&self, should_update: bool, ledger: L) -> Self {
            let mut this = self.clone();
            if should_update {
                this.second_pass_ledger.apply_mask(ledger);
            }
            this
        }

        pub fn fee_excess(&self) -> Signed<Amount> {
            self.fee_excess
        }

        #[must_use]
        fn set_fee_excess(&self, fee_excess: Signed<Amount>) -> Self {
            let mut this = self.clone();
            this.fee_excess = fee_excess;
            this
        }

        pub fn supply_increase(&self) -> Signed<Amount> {
            self.supply_increase
        }

        #[must_use]
        pub fn set_supply_increase(&self, supply_increase: Signed<Amount>) -> Self {
            let mut this = self.clone();
            this.supply_increase = supply_increase;
            this
        }

        fn block_global_slot(&self) -> Slot {
            self.block_global_slot
        }
    }
}

pub mod local_state {
    use std::{cell::RefCell, rc::Rc};

    use ark_ff::Zero;

    use crate::{
        hash_with_kimchi,
        proofs::{
            field::{field, Boolean, ToBoolean},
            numbers::nat::CheckedNat,
            to_field_elements::ToFieldElements,
        },
        scan_state::currency::{Index, Signed},
        zkapps::intefaces::{
            CallStackInterface, IndexInterface, SignedAmountInterface, StackFrameInterface,
        },
        Inputs, ToInputs,
    };

    use super::{zkapp_command::CallForest, *};

    #[derive(Debug, Clone)]
    pub struct StackFrame {
        pub caller: TokenId,
        pub caller_caller: TokenId,
        pub calls: CallForest<AccountUpdate>, // TODO
    }

    // https://github.com/MinaProtocol/mina/blob/78535ae3a73e0e90c5f66155365a934a15535779/src/lib/transaction_snark/transaction_snark.ml#L1081
    #[derive(Debug, Clone)]
    pub struct StackFrameCheckedFrame {
        pub caller: TokenId,
        pub caller_caller: TokenId,
        pub calls: WithHash<CallForest<AccountUpdate>>,
        /// Hack until we have proper cvar
        pub is_default: bool,
    }

    impl ToFieldElements<Fp> for StackFrameCheckedFrame {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            let Self {
                caller,
                caller_caller,
                calls,
                is_default: _,
            } = self;

            // calls.hash().to_field_elements(fields);
            calls.hash.to_field_elements(fields);
            caller_caller.to_field_elements(fields);
            caller.to_field_elements(fields);
        }
    }

    enum LazyValueInner<T, D> {
        Value(T),
        Fun(Box<dyn FnOnce(&mut D) -> T>),
        None,
    }

    impl<T, D> Default for LazyValueInner<T, D> {
        fn default() -> Self {
            Self::None
        }
    }

    pub struct LazyValue<T, D> {
        value: Rc<RefCell<LazyValueInner<T, D>>>,
    }

    impl<T, D> Clone for LazyValue<T, D> {
        fn clone(&self) -> Self {
            Self {
                value: Rc::clone(&self.value),
            }
        }
    }

    impl<T: std::fmt::Debug, D> std::fmt::Debug for LazyValue<T, D> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let v = self.try_get();
            f.debug_struct("LazyValue").field("value", &v).finish()
        }
    }

    impl<T, D> LazyValue<T, D> {
        pub fn make<F>(fun: F) -> Self
        where
            F: FnOnce(&mut D) -> T + 'static,
        {
            Self {
                value: Rc::new(RefCell::new(LazyValueInner::Fun(Box::new(fun)))),
            }
        }

        fn get_impl(&self) -> std::cell::Ref<'_, T> {
            use std::cell::Ref;

            let inner = self.value.borrow();
            Ref::map(inner, |inner| {
                let LazyValueInner::Value(value) = inner else {
                    panic!("invalid state");
                };
                value
            })
        }

        /// Returns the value when it already has been "computed"
        pub fn try_get(&self) -> Option<std::cell::Ref<'_, T>> {
            let inner = self.value.borrow();

            match &*inner {
                LazyValueInner::Value(_) => {}
                LazyValueInner::Fun(_) => return None,
                LazyValueInner::None => panic!("invalid state"),
            }

            Some(self.get_impl())
        }

        pub fn get(&self, data: &mut D) -> std::cell::Ref<'_, T> {
            let v = self.value.borrow();

            if let LazyValueInner::Fun(_) = &*v {
                std::mem::drop(v);

                let LazyValueInner::Fun(fun) = self.value.take() else {
                    panic!("invalid state");
                };

                let data = fun(data);
                self.value.replace(LazyValueInner::Value(data));
            };

            self.get_impl()
        }
    }

    #[derive(Clone, Debug)]
    pub struct WithLazyHash<T> {
        pub data: T,
        hash: LazyValue<Fp, Witness<Fp>>,
    }

    impl<T> WithLazyHash<T> {
        pub fn new<F>(data: T, fun: F) -> Self
        where
            F: FnOnce(&mut Witness<Fp>) -> Fp + 'static,
        {
            Self {
                data,
                hash: LazyValue::make(fun),
            }
        }

        pub fn hash(&self, w: &mut Witness<Fp>) -> Fp {
            *self.hash.get(w)
        }
    }

    impl<T> std::ops::Deref for WithLazyHash<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.data
        }
    }

    impl<T> ToFieldElements<Fp> for WithLazyHash<T> {
        fn to_field_elements(&self, fields: &mut Vec<Fp>) {
            let hash = self.hash.try_get().expect("hash hasn't been computed yet");
            hash.to_field_elements(fields)
        }
    }

    // https://github.com/MinaProtocol/mina/blob/78535ae3a73e0e90c5f66155365a934a15535779/src/lib/transaction_snark/transaction_snark.ml#L1083
    pub type StackFrameChecked = WithLazyHash<StackFrameCheckedFrame>;

    impl Default for StackFrame {
        fn default() -> Self {
            StackFrame {
                caller: TokenId::default(),
                caller_caller: TokenId::default(),
                calls: CallForest::new(),
            }
        }
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

        pub fn digest(&self) -> Fp {
            self.hash()
        }

        pub fn unhash(&self, _h: Fp, w: &mut Witness<Fp>) -> StackFrameChecked {
            let v = self.exists_elt(w);
            v.hash(w);
            v
        }

        pub fn exists_elt(&self, w: &mut Witness<Fp>) -> StackFrameChecked {
            // We decompose this way because of OCaml evaluation order
            let calls = WithHash {
                data: self.calls.clone(),
                hash: w.exists(self.calls.hash()),
            };
            let caller_caller = w.exists(self.caller_caller.clone());
            let caller = w.exists(self.caller.clone());

            let frame = StackFrameCheckedFrame {
                caller,
                caller_caller,
                calls,
                is_default: false,
            };

            StackFrameChecked::of_frame(frame)
        }
    }

    impl StackFrameCheckedFrame {
        pub fn hash(&self, w: &mut Witness<Fp>) -> Fp {
            let mut inputs = Inputs::new();

            inputs.append(&self.caller);
            inputs.append(&self.caller_caller.0);
            inputs.append(&self.calls.hash);

            let fields = inputs.to_fields();

            if self.is_default {
                use crate::proofs::witness::transaction_snark::checked_hash3;
                checked_hash3("MinaAcctUpdStckFrm", &fields, w)
            } else {
                use crate::proofs::witness::transaction_snark::checked_hash;
                checked_hash("MinaAcctUpdStckFrm", &fields, w)
            }
        }
    }

    impl StackFrameChecked {
        pub fn of_frame(frame: StackFrameCheckedFrame) -> Self {
            // TODO: Don't clone here
            let frame2 = frame.clone();
            let hash = LazyValue::make(move |w: &mut Witness<Fp>| frame2.hash(w));

            Self { data: frame, hash }
        }
    }

    #[derive(Debug, Clone)]
    pub struct CallStack(pub Vec<StackFrame>);

    impl Default for CallStack {
        fn default() -> Self {
            Self::new()
        }
    }

    impl CallStack {
        pub fn new() -> Self {
            CallStack(Vec::new())
        }

        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        pub fn iter(&self) -> impl Iterator<Item = &StackFrame> {
            self.0.iter().rev()
        }

        pub fn push(&self, stack_frame: &StackFrame) -> Self {
            let mut ret = self.0.clone();
            ret.push(stack_frame.clone());
            Self(ret)
        }

        pub fn pop(&self) -> Option<(StackFrame, CallStack)> {
            let mut ret = self.0.clone();
            ret.pop().map(|frame| (frame, Self(ret)))
        }

        pub fn pop_exn(&self) -> (StackFrame, CallStack) {
            let mut ret = self.0.clone();
            if let Some(frame) = ret.pop() {
                (frame, Self(ret))
            } else {
                panic!()
            }
        }
    }

    /// NOTE: It looks like there are different instances of the polymorphic LocalEnv type
    /// One with concrete types for the stack frame, call stack, and ledger. Created from the Env
    /// And the other with their hashes. To differentiate them I renamed the first LocalStateEnv
    /// Maybe a better solution is to keep the LocalState name and put it under a different module
    pub type LocalStateEnv<L> = LocalStateSkeleton<
        L,                            // ledger
        StackFrame,                   // stack_frame
        CallStack,                    // call_stack
        ReceiptChainHash,             // commitments
        Signed<Amount>,               // excess & supply_increase
        Vec<Vec<TransactionFailure>>, // failure_status_tbl
        bool,                         // success & will_succeed
        Index,                        // account_update_index
    >;

    #[derive(Debug, Clone)]
    pub struct LocalStateSkeleton<
        L: LedgerIntf + Clone,
        StackFrame: StackFrameInterface,
        CallStack: CallStackInterface,
        TC,
        SignedAmount: SignedAmountInterface,
        FailuresTable,
        Bool,
        Index: IndexInterface,
    > {
        pub stack_frame: StackFrame,
        pub call_stack: CallStack,
        pub transaction_commitment: TC,
        pub full_transaction_commitment: TC,
        pub excess: SignedAmount,
        pub supply_increase: SignedAmount,
        pub ledger: L,
        pub success: Bool,
        pub account_update_index: Index,
        // TODO: optimize by reversing the insertion order
        pub failure_status_tbl: FailuresTable,
        pub will_succeed: Bool,
    }

    impl<L: LedgerIntf + Clone> LocalStateEnv<L> {
        pub fn add_new_failure_status_bucket(&self) -> Self {
            let mut failure_status_tbl = self.failure_status_tbl.clone();
            failure_status_tbl.insert(0, Vec::new());
            Self {
                failure_status_tbl,
                ..self.clone()
            }
        }

        pub fn add_check(&self, failure: TransactionFailure, b: bool) -> Self {
            let failure_status_tbl = if let false = b {
                let mut failure_status_tbl = self.failure_status_tbl.clone();
                failure_status_tbl[0].insert(0, failure);
                failure_status_tbl
            } else {
                self.failure_status_tbl.clone()
            };

            Self {
                failure_status_tbl,
                success: self.success && b,
                ..self.clone()
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct LocalState {
        pub stack_frame: Fp,
        pub call_stack: Fp,
        pub transaction_commitment: Fp,
        pub full_transaction_commitment: Fp,
        pub excess: Signed<Amount>,
        pub supply_increase: Signed<Amount>,
        pub ledger: Fp,
        pub success: bool,
        pub account_update_index: Index,
        pub failure_status_tbl: Vec<Vec<TransactionFailure>>,
        pub will_succeed: bool,
    }

    impl ToInputs for LocalState {
        /// https://github.com/MinaProtocol/mina/blob/4e0b324912017c3ff576704ee397ade3d9bda412/src/lib/mina_state/local_state.ml#L116
        fn to_inputs(&self, inputs: &mut Inputs) {
            let Self {
                stack_frame,
                call_stack,
                transaction_commitment,
                full_transaction_commitment,
                excess,
                supply_increase,
                ledger,
                success,
                account_update_index,
                failure_status_tbl: _,
                will_succeed,
            } = self;

            inputs.append(stack_frame);
            inputs.append(call_stack);
            inputs.append(transaction_commitment);
            inputs.append(full_transaction_commitment);
            inputs.append(excess);
            inputs.append(supply_increase);
            inputs.append(ledger);
            inputs.append(account_update_index);
            inputs.append(success);
            inputs.append(will_succeed);
        }
    }

    impl LocalState {
        /// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/mina_state/local_state.ml#L65
        pub fn dummy() -> Self {
            Self {
                stack_frame: StackFrame::empty().hash(),
                call_stack: Fp::zero(),
                transaction_commitment: Fp::zero(),
                full_transaction_commitment: Fp::zero(),
                excess: Signed::<Amount>::zero(),
                supply_increase: Signed::<Amount>::zero(),
                ledger: Fp::zero(),
                success: true,
                account_update_index: <Index as Magnitude>::zero(),
                failure_status_tbl: Vec::new(),
                will_succeed: true,
            }
        }

        pub fn empty() -> Self {
            Self::dummy()
        }

        pub fn equal_without_ledger(&self, other: &Self) -> bool {
            let Self {
                stack_frame,
                call_stack,
                transaction_commitment,
                full_transaction_commitment,
                excess,
                supply_increase,
                ledger: _,
                success,
                account_update_index,
                failure_status_tbl,
                will_succeed,
            } = self;

            stack_frame == &other.stack_frame
                && call_stack == &other.call_stack
                && transaction_commitment == &other.transaction_commitment
                && full_transaction_commitment == &other.full_transaction_commitment
                && excess == &other.excess
                && supply_increase == &other.supply_increase
                && success == &other.success
                && account_update_index == &other.account_update_index
                && failure_status_tbl == &other.failure_status_tbl
                && will_succeed == &other.will_succeed
        }

        pub fn checked_equal_prime(&self, other: &Self, w: &mut Witness<Fp>) -> [Boolean; 11] {
            let Self {
                stack_frame,
                call_stack,
                transaction_commitment,
                full_transaction_commitment,
                excess,
                supply_increase,
                ledger,
                success,
                account_update_index,
                failure_status_tbl: _,
                will_succeed,
            } = self;

            // { stack_frame : 'stack_frame
            // ; call_stack : 'call_stack
            // ; transaction_commitment : 'comm
            // ; full_transaction_commitment : 'comm
            // ; excess : 'signed_amount
            // ; supply_increase : 'signed_amount
            // ; ledger : 'ledger
            // ; success : 'bool
            // ; account_update_index : 'length
            // ; failure_status_tbl : 'failure_status_tbl
            // ; will_succeed : 'bool
            // }

            let mut alls = [
                field::equal(*stack_frame, other.stack_frame, w),
                field::equal(*call_stack, other.call_stack, w),
                field::equal(*transaction_commitment, other.transaction_commitment, w),
                field::equal(
                    *full_transaction_commitment,
                    other.full_transaction_commitment,
                    w,
                ),
                excess
                    .to_checked::<Fp>()
                    .equal(&other.excess.to_checked(), w),
                supply_increase
                    .to_checked::<Fp>()
                    .equal(&other.supply_increase.to_checked(), w),
                field::equal(*ledger, other.ledger, w),
                success.to_boolean().equal(&other.success.to_boolean(), w),
                account_update_index
                    .to_checked::<Fp>()
                    .equal(&other.account_update_index.to_checked(), w),
                Boolean::True,
                will_succeed
                    .to_boolean()
                    .equal(&other.will_succeed.to_boolean(), w),
            ];
            alls.reverse();
            alls
        }
    }
}

// let equal' (t1 : t) (t2 : t) =
//   let ( ! ) f x y = Impl.run_checked (f x y) in
//   let f eq acc f = Core_kernel.Field.(eq (get f t1) (get f t2)) :: acc in
//   Mina_transaction_logic.Zkapp_command_logic.Local_state.Fields.fold ~init:[]
//     ~stack_frame:(f Stack_frame.Digest.Checked.equal)
//     ~call_stack:(f Call_stack_digest.Checked.equal)
//     ~transaction_commitment:(f Field.equal)
//     ~full_transaction_commitment:(f Field.equal)
//     ~excess:(f !Currency.Amount.Signed.Checked.equal)
//     ~supply_increase:(f !Currency.Amount.Signed.Checked.equal)
//     ~ledger:(f !Ledger_hash.equal_var) ~success:(f Impl.Boolean.equal)
//     ~account_update_index:(f !Mina_numbers.Index.Checked.equal)
//     ~failure_status_tbl:(f (fun () () -> Impl.Boolean.true_))
//     ~will_succeed:(f Impl.Boolean.equal)

pub enum Eff<L: LedgerIntf + Clone> {
    CheckValidWhilePrecondition(Numeric<Slot>, GlobalState<L>),
    CheckAccountPrecondition(AccountUpdate, Account, bool, LocalStateEnv<L>),
    CheckProtocolStatePrecondition(ZkAppPreconditions, GlobalState<L>),
    InitAccount(AccountUpdate, Account),
}

pub struct Env<L: LedgerIntf + Clone> {
    account_update: AccountUpdate,
    zkapp_command: ZkAppCommand,
    account: Account,
    ledger: L,
    amount: Amount,
    signed_amount: Signed<Amount>,
    bool: bool,
    token_id: TokenId,
    global_state: GlobalState<L>,
    local_state: LocalStateEnv<L>,
    protocol_state_precondition: ZkAppPreconditions,
    valid_while_precondition: Numeric<Slot>,
    transaction_commitment: Fp,
    full_transaction_commitment: Fp,
    field: Fp,
    failure: Option<TransactionFailure>,
}

pub enum PerformResult<L: LedgerIntf + Clone> {
    Bool(bool),
    LocalState(LocalStateEnv<L>),
    Account(Box<Account>),
}

impl<L: LedgerIntf + Clone> PerformResult<L> {
    pub fn to_bool(self) -> bool {
        match self {
            PerformResult::Bool(v) => v,
            _ => panic!("Not a bool"),
        }
    }
}

impl<L> Env<L>
where
    L: LedgerIntf + Clone,
{
    pub fn perform(eff: Eff<L>) -> PerformResult<L> {
        match eff {
            Eff::CheckValidWhilePrecondition(valid_while, global_state) => PerformResult::Bool(
                valid_while
                    .zcheck(
                        "valid_while_precondition".to_string(),
                        global_state.block_global_slot,
                    )
                    .is_ok(),
            ),
            Eff::CheckProtocolStatePrecondition(pred, global_state) => {
                PerformResult::Bool(pred.zcheck(global_state.protocol_state).is_ok())
            }
            Eff::CheckAccountPrecondition(account_update, account, new_account, local_state) => {
                let local_state = match account_update.body.preconditions.account {
                    AccountPreconditions::Accept => local_state,
                    AccountPreconditions::Nonce(n) => local_state.add_check(
                        TransactionFailure::AccountNoncePreconditionUnsatisfied,
                        account.nonce == n,
                    ),
                    AccountPreconditions::Full(precondition_account) => {
                        let mut _local_state = local_state;
                        let check = |failure, b| {
                            _local_state = _local_state.add_check(failure, b);
                        };
                        precondition_account.zcheck(new_account, check, account);
                        _local_state
                    }
                };
                PerformResult::LocalState(local_state)
            }
            Eff::InitAccount(_account_update, a) => PerformResult::Account(Box::new(a)),
        }
    }
}

// fn step_all<L>(
//     constraint_constants: &ConstraintConstants,
//     f: &impl Fn(
//         Option<(LocalStateEnv<L>, Signed<Fee>)>,
//         (GlobalState<L>, LocalStateEnv<L>),
//     ) -> Option<(LocalStateEnv<L>, Signed<Fee>)>,
//     h: fn(Eff<L>) -> PerformResult<L>,
//     user_acc: Option<(LocalStateEnv<L>, Signed<Fee>)>,
//     (g_state, l_state): (GlobalState<L>, LocalStateEnv<L>),
// ) -> Result<
//     (
//         Option<(LocalStateEnv<L>, Signed<Fee>)>,
//         Vec<Vec<TransactionFailure>>,
//     ),
//     String,
// >
// where
//     L: LedgerIntf + Clone,
// {
//     if l_state.stack_frame.calls.is_empty() {
//         Ok((user_acc, l_state.failure_status_tbl))
//     } else {
//         let states = apply(
//             constraint_constants,
//             IsStart::No,
//             Handler { perform: h },
//             (g_state, l_state),
//         );
//         step_all(
//             constraint_constants,
//             f,
//             h,
//             f(user_acc, states.clone()),
//             states,
//         )
//     }
// }

fn step_all<A, L>(
    constraint_constants: &ConstraintConstants,
    f: &impl Fn(A, (GlobalState<L>, LocalStateEnv<L>)) -> A,
    handler: &Handler<L>,
    user_acc: A,
    (g_state, l_state): (GlobalState<L>, LocalStateEnv<L>),
) -> Result<(A, Vec<Vec<TransactionFailure>>), String>
where
    L: LedgerIntf + Clone,
{
    if l_state.stack_frame.calls.is_empty() {
        Ok((user_acc, l_state.failure_status_tbl))
    } else {
        let states = zkapp_logic::step(constraint_constants, handler, (g_state, l_state))?;
        step_all(
            constraint_constants,
            f,
            handler,
            f(user_acc, states.clone()),
            states,
        )
    }
}

/// apply zkapp command fee payer's while stubbing out the second pass ledger
/// CAUTION: If you use the intermediate local states, you MUST update the
///   [will_succeed] field to [false] if the [status] is [Failed].*)
pub fn apply_zkapp_command_first_pass_aux<A, F, L>(
    constraint_constants: &ConstraintConstants,
    global_slot: Slot,
    state_view: &ProtocolStateView,
    init: A,
    f: F,
    fee_excess: Option<Signed<Amount>>,
    supply_increase: Option<Signed<Amount>>,
    ledger: &mut L,
    command: &ZkAppCommand,
) -> Result<(ZkappCommandPartiallyApplied<L>, A), String>
where
    L: LedgerIntf + Clone,
    F: Fn(A, (GlobalState<L>, LocalStateEnv<L>)) -> A,
{
    let fee_excess = fee_excess.unwrap_or_else(Signed::zero);
    let supply_increase = supply_increase.unwrap_or_else(Signed::zero);

    let previous_hash = ledger.merkle_root();
    let original_first_pass_account_states = {
        let id = command.fee_payer();
        let location = {
            let loc = ledger.location_of_account(&id);
            let account = loc.as_ref().and_then(|loc| ledger.get(loc));
            loc.zip(account)
        };

        vec![(id, location)]
    };

    let perform = |eff: Eff<L>| Env::perform(eff);

    let initial_state = (
        GlobalState {
            protocol_state: state_view.clone(),
            first_pass_ledger: ledger.clone(),
            second_pass_ledger: {
                // We stub out the second_pass_ledger initially, and then poke the
                // correct value in place after the first pass is finished.
                L::empty(0)
            },
            fee_excess,
            supply_increase,
            block_global_slot: global_slot,
        },
        LocalStateEnv {
            stack_frame: StackFrame::default(),
            call_stack: CallStack::new(),
            transaction_commitment: ReceiptChainHash(Fp::zero()),
            full_transaction_commitment: ReceiptChainHash(Fp::zero()),
            excess: Signed::<Amount>::zero(),
            supply_increase,
            ledger: L::empty(0),
            success: true,
            account_update_index: Index::zero(),
            failure_status_tbl: Vec::new(),
            will_succeed: true,
        },
    );

    let user_acc = f(init, initial_state.clone());
    let account_updates = command.all_account_updates();

    let (global_state, local_state) = {
        zkapp_logic::start(
            constraint_constants,
            StartData {
                account_updates,
                memo_hash: command.memo.hash(),
                // It's always valid to set this value to true, and it will
                // have no effect outside of the snark.
                will_succeed: true,
            },
            &Handler { perform },
            initial_state,
        )?
    };

    let command = command.clone();
    let constraint_constants = constraint_constants.clone();
    let state_view = state_view.clone();

    let res = ZkappCommandPartiallyApplied {
        command,
        previous_hash,
        original_first_pass_account_states,
        constraint_constants,
        state_view,
        global_state,
        local_state,
    };

    Ok((res, user_acc))
}

fn apply_zkapp_command_first_pass<L>(
    constraint_constants: &ConstraintConstants,
    global_slot: Slot,
    state_view: &ProtocolStateView,
    fee_excess: Option<Signed<Amount>>,
    supply_increase: Option<Signed<Amount>>,
    ledger: &mut L,
    command: &ZkAppCommand,
) -> Result<ZkappCommandPartiallyApplied<L>, String>
where
    L: LedgerIntf + Clone,
{
    let (partial_stmt, _user_acc) = apply_zkapp_command_first_pass_aux(
        constraint_constants,
        global_slot,
        state_view,
        None,
        |_acc, state| Some(state),
        fee_excess,
        supply_increase,
        ledger,
        command,
    )?;

    Ok(partial_stmt)
}

pub fn apply_zkapp_command_second_pass_aux<A, F, L>(
    constraint_constants: &ConstraintConstants,
    init: A,
    f: F,
    ledger: &mut L,
    c: ZkappCommandPartiallyApplied<L>,
) -> Result<(ZkappCommandApplied, A), String>
where
    L: LedgerIntf + Clone,
    F: Fn(A, (GlobalState<L>, LocalStateEnv<L>)) -> A,
{
    let perform = |eff: Eff<L>| Env::perform(eff);

    let original_account_states: Vec<_> = {
        // get the original states of all the accounts in each pass.
        // If an account updated in the first pass is referenced in account
        // updates, then retain the value before first pass application*)

        let accounts_referenced = c.command.accounts_referenced();

        let mut account_states = BTreeMap::<AccountId, Option<_>>::new();

        let referenced = accounts_referenced.into_iter().map(|id| {
            let location = {
                let loc = ledger.location_of_account(&id);
                let account = loc.as_ref().and_then(|loc| ledger.get(loc));
                loc.zip(account)
            };
            (id, location)
        });

        c.original_first_pass_account_states
            .into_iter()
            .chain(referenced)
            .for_each(|(id, acc_opt)| {
                use std::collections::btree_map::Entry::Vacant;

                if let Vacant(entry) = account_states.entry(id) {
                    entry.insert(acc_opt);
                };
            });

        account_states.into_iter().collect()
    };

    let mut account_states_after_fee_payer = {
        // To check if the accounts remain unchanged in the event the transaction
        // fails. First pass updates will remain even if the transaction fails to
        // apply zkapp account updates*)

        c.command.accounts_referenced().into_iter().map(|id| {
            let loc = ledger.location_of_account(&id);
            let a = loc.as_ref().and_then(|loc| ledger.get(loc));

            match a {
                Some(a) => (id, Some((loc.unwrap(), a))),
                None => (id, None),
            }
        })
    };

    let accounts = || {
        original_account_states
            .iter()
            .map(|(id, account)| (id.clone(), account.as_ref().map(|(_loc, acc)| acc.clone())))
            .collect::<Vec<_>>()
    };

    // Warning(OCaml): This is an abstraction leak / hack.
    // Here, we update global second pass ledger to be the input ledger, and
    // then update the local ledger to be the input ledger *IF AND ONLY IF*
    // there are more transaction segments to be processed in this pass.

    // TODO(OCaml): Remove this, and uplift the logic into the call in staged ledger.

    let global_state = GlobalState {
        second_pass_ledger: ledger.clone(),
        ..c.global_state
    };

    let local_state = {
        if c.local_state.stack_frame.calls.is_empty() {
            // Don't mess with the local state; we've already finished the
            // transaction after the fee payer.
            c.local_state
        } else {
            // Install the ledger that should already be in the local state, but
            // may not be in some situations depending on who the caller is.
            LocalStateEnv {
                ledger: global_state.second_pass_ledger(),
                ..c.local_state
            }
        }
    };

    let start = (global_state, local_state);

    let (user_acc, reversed_failure_status_tbl) = step_all(
        constraint_constants,
        &f,
        &Handler { perform },
        f(init, start.clone()),
        start,
    )?;

    let failure_status_tbl = reversed_failure_status_tbl
        .into_iter()
        .rev()
        .collect::<Vec<_>>();

    let account_ids_originally_not_in_ledger =
        original_account_states
            .iter()
            .filter_map(|(acct_id, loc_and_acct)| {
                if loc_and_acct.is_none() {
                    Some(acct_id)
                } else {
                    None
                }
            });

    let successfully_applied = failure_status_tbl.concat().is_empty();

    // if the zkapp command fails in at least 1 account update,
    // then all the account updates would be cancelled except
    // the fee payer one
    let failure_status_tbl = if successfully_applied {
        failure_status_tbl
    } else {
        failure_status_tbl
            .into_iter()
            .enumerate()
            .map(|(idx, fs)| {
                if idx > 0 && fs.is_empty() {
                    vec![TransactionFailure::Cancelled]
                } else {
                    fs
                }
            })
            .collect()
    };

    // accounts not originally in ledger, now present in ledger
    let new_accounts = account_ids_originally_not_in_ledger
        .filter(|acct_id| ledger.location_of_account(acct_id).is_some())
        .cloned()
        .collect::<Vec<_>>();

    let new_accounts_is_empty = new_accounts.is_empty();

    let valid_result = Ok((
        ZkappCommandApplied {
            accounts: accounts(),
            command: WithStatus {
                data: c.command,
                status: if successfully_applied {
                    TransactionStatus::Applied
                } else {
                    TransactionStatus::Failed(failure_status_tbl)
                },
            },
            new_accounts,
        },
        user_acc,
    ));

    if successfully_applied {
        valid_result
    } else {
        let other_account_update_accounts_unchanged = account_states_after_fee_payer
            .fold_while(true, |acc, (_, loc_opt)| match loc_opt {
                Some((loc, a)) => match ledger.get(&loc) {
                    Some(a_) if !(a == a_) => FoldWhile::Done(false),
                    _ => FoldWhile::Continue(acc),
                },
                _ => FoldWhile::Continue(acc),
            })
            .into_inner();

        // Other zkapp_command failed, therefore, updates in those should not get applied
        if new_accounts_is_empty && other_account_update_accounts_unchanged {
            valid_result
        } else {
            Err("Zkapp_command application failed but new accounts created or some of the other account_update updates applied".to_string())
        }
    }
}

fn apply_zkapp_command_second_pass<L>(
    constraint_constants: &ConstraintConstants,
    ledger: &mut L,
    c: ZkappCommandPartiallyApplied<L>,
) -> Result<ZkappCommandApplied, String>
where
    L: LedgerIntf + Clone,
{
    let (x, _) =
        apply_zkapp_command_second_pass_aux(constraint_constants, (), |a, _| a, ledger, c)?;
    Ok(x)
}

fn apply_zkapp_command_unchecked_aux<A, F, L>(
    constraint_constants: &ConstraintConstants,
    global_slot: Slot,
    state_view: &ProtocolStateView,
    init: A,
    f: F,
    fee_excess: Option<Signed<Amount>>,
    supply_increase: Option<Signed<Amount>>,
    ledger: &mut L,
    command: &ZkAppCommand,
) -> Result<(ZkappCommandApplied, A), String>
where
    L: LedgerIntf + Clone,
    F: Fn(A, (GlobalState<L>, LocalStateEnv<L>)) -> A,
{
    let (partial_stmt, user_acc) = apply_zkapp_command_first_pass_aux(
        constraint_constants,
        global_slot,
        state_view,
        init,
        &f,
        fee_excess,
        supply_increase,
        ledger,
        command,
    )?;

    apply_zkapp_command_second_pass_aux(constraint_constants, user_acc, f, ledger, partial_stmt)
}

fn apply_zkapp_command_unchecked<L>(
    constraint_constants: &ConstraintConstants,
    global_slot: Slot,
    state_view: &ProtocolStateView,
    ledger: &mut L,
    command: &ZkAppCommand,
) -> Result<(ZkappCommandApplied, (LocalStateEnv<L>, Signed<Amount>)), String>
where
    L: LedgerIntf + Clone,
{
    let zkapp_partially_applied: ZkappCommandPartiallyApplied<L> = apply_zkapp_command_first_pass(
        constraint_constants,
        global_slot,
        state_view,
        None,
        None,
        ledger,
        command,
    )?;

    let (account_update_applied, state_res) = apply_zkapp_command_second_pass_aux(
        constraint_constants,
        None,
        |_acc, (global_state, local_state)| Some((local_state, global_state.fee_excess)),
        ledger,
        zkapp_partially_applied,
    )?;

    Ok((account_update_applied, state_res.unwrap()))
}

pub mod transaction_partially_applied {
    use super::{
        transaction_applied::{CoinbaseApplied, FeeTransferApplied},
        *,
    };

    #[derive(Clone, Debug)]
    pub struct ZkappCommandPartiallyApplied<L: LedgerIntf + Clone> {
        pub command: ZkAppCommand,
        pub previous_hash: Fp,
        pub original_first_pass_account_states:
            Vec<(AccountId, Option<(L::Location, Box<Account>)>)>,
        pub constraint_constants: ConstraintConstants,
        pub state_view: ProtocolStateView,
        pub global_state: GlobalState<L>,
        pub local_state: LocalStateEnv<L>,
    }

    #[derive(Clone, Debug)]
    pub struct FullyApplied<T> {
        pub previous_hash: Fp,
        pub applied: T,
    }

    #[derive(Clone, Debug)]
    pub enum TransactionPartiallyApplied<L: LedgerIntf + Clone> {
        SignedCommand(FullyApplied<SignedCommandApplied>),
        ZkappCommand(ZkappCommandPartiallyApplied<L>),
        FeeTransfer(FullyApplied<FeeTransferApplied>),
        Coinbase(FullyApplied<CoinbaseApplied>),
    }

    impl<L: LedgerIntf + Clone> TransactionPartiallyApplied<L> {
        pub fn command(self) -> Transaction {
            use Transaction as T;

            match self {
                Self::SignedCommand(s) => T::Command(UserCommand::SignedCommand(Box::new(
                    s.applied.common.user_command.data,
                ))),
                Self::ZkappCommand(z) => T::Command(UserCommand::ZkAppCommand(Box::new(z.command))),
                Self::FeeTransfer(ft) => T::FeeTransfer(ft.applied.fee_transfer.data),
                Self::Coinbase(cb) => T::Coinbase(cb.applied.coinbase.data),
            }
        }
    }
}

use transaction_partially_applied::{TransactionPartiallyApplied, ZkappCommandPartiallyApplied};

pub fn apply_transaction_first_pass<L>(
    constraint_constants: &ConstraintConstants,
    global_slot: Slot,
    txn_state_view: &ProtocolStateView,
    ledger: &mut L,
    transaction: &Transaction,
) -> Result<TransactionPartiallyApplied<L>, String>
where
    L: LedgerIntf + Clone,
{
    use Transaction::*;
    use UserCommand::*;

    let previous_hash = ledger.merkle_root();
    let txn_global_slot = &txn_state_view.global_slot_since_genesis;

    match transaction {
        Command(SignedCommand(cmd)) => apply_user_command(
            constraint_constants,
            txn_state_view,
            txn_global_slot,
            ledger,
            cmd,
        )
        .map(|applied| {
            TransactionPartiallyApplied::SignedCommand(FullyApplied {
                previous_hash,
                applied,
            })
        }),
        Command(ZkAppCommand(txn)) => apply_zkapp_command_first_pass(
            constraint_constants,
            global_slot,
            txn_state_view,
            None,
            None,
            ledger,
            txn,
        )
        .map(TransactionPartiallyApplied::ZkappCommand),
        FeeTransfer(fee_transfer) => {
            apply_fee_transfer(constraint_constants, txn_global_slot, ledger, fee_transfer).map(
                |applied| {
                    TransactionPartiallyApplied::FeeTransfer(FullyApplied {
                        previous_hash,
                        applied,
                    })
                },
            )
        }
        Coinbase(coinbase) => {
            apply_coinbase(constraint_constants, txn_global_slot, ledger, coinbase).map(|applied| {
                TransactionPartiallyApplied::Coinbase(FullyApplied {
                    previous_hash,
                    applied,
                })
            })
        }
    }
}

pub fn apply_transaction_second_pass<L>(
    constraint_constants: &ConstraintConstants,
    ledger: &mut L,
    partial_transaction: TransactionPartiallyApplied<L>,
) -> Result<TransactionApplied, String>
where
    L: LedgerIntf + Clone,
{
    use TransactionPartiallyApplied as P;

    match partial_transaction {
        P::SignedCommand(FullyApplied {
            previous_hash,
            applied,
        }) => Ok(TransactionApplied {
            previous_hash,
            varying: Varying::Command(CommandApplied::SignedCommand(Box::new(applied))),
        }),
        P::ZkappCommand(partially_applied) => {
            // TODO(OCaml): either here or in second phase of apply, need to update the
            // prior global state statement for the fee payer segment to add the
            // second phase ledger at the end

            let previous_hash = partially_applied.previous_hash;
            let applied =
                apply_zkapp_command_second_pass(constraint_constants, ledger, partially_applied)?;

            Ok(TransactionApplied {
                previous_hash,
                varying: Varying::Command(CommandApplied::ZkappCommand(Box::new(applied))),
            })
        }
        P::FeeTransfer(FullyApplied {
            previous_hash,
            applied,
        }) => Ok(TransactionApplied {
            previous_hash,
            varying: Varying::FeeTransfer(applied),
        }),
        P::Coinbase(FullyApplied {
            previous_hash,
            applied,
        }) => Ok(TransactionApplied {
            previous_hash,
            varying: Varying::Coinbase(applied),
        }),
    }
}

pub fn apply_transactions<L>(
    constraint_constants: &ConstraintConstants,
    global_slot: Slot,
    txn_state_view: &ProtocolStateView,
    ledger: &mut L,
    txns: &[Transaction],
) -> Result<Vec<TransactionApplied>, String>
where
    L: LedgerIntf + Clone,
{
    let first_pass: Vec<_> = txns
        .iter()
        .map(|txn| {
            apply_transaction_first_pass(
                constraint_constants,
                global_slot,
                txn_state_view,
                ledger,
                txn,
            )
        })
        .collect::<Result<Vec<TransactionPartiallyApplied<_>>, _>>()?;

    first_pass
        .into_iter()
        .map(|partial_transaction| {
            apply_transaction_second_pass(constraint_constants, ledger, partial_transaction)
        })
        .collect()
}

struct FailureCollection {
    inner: Vec<Vec<TransactionFailure>>,
}

/// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/transaction_logic/mina_transaction_logic.ml#L2197C1-L2210C53
impl FailureCollection {
    fn empty() -> Self {
        Self {
            inner: Vec::default(),
        }
    }

    fn no_failure() -> Vec<TransactionFailure> {
        vec![]
    }

    /// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/transaction_logic/mina_transaction_logic.ml#L2204
    fn single_failure() -> Self {
        Self {
            inner: vec![vec![TransactionFailure::UpdateNotPermittedBalance]],
        }
    }

    fn update_failed() -> Vec<TransactionFailure> {
        vec![TransactionFailure::UpdateNotPermittedBalance]
    }

    /// https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/transaction_logic/mina_transaction_logic.ml#L2208
    fn append_entry(list: Vec<TransactionFailure>, mut s: Self) -> Self {
        if s.inner.is_empty() {
            Self { inner: vec![list] }
        } else {
            s.inner.insert(1, list);
            s
        }
    }

    fn is_empty(&self) -> bool {
        self.inner.iter().all(Vec::is_empty)
    }

    fn take(self) -> Vec<Vec<TransactionFailure>> {
        self.inner
    }
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
        None => (
            *coinbase_amount,
            None,
            None,
            None,
            FailureCollection::empty(),
            Amount::zero(),
        ),
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
                    FailureCollection::append_entry(
                        FailureCollection::no_failure(),
                        FailureCollection::empty(),
                    ),
                    Amount::zero(),
                )
            } else {
                (
                    receiver_reward,
                    None,
                    None,
                    None,
                    FailureCollection::single_failure(),
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

    let (failures, burned_tokens2) = if can_receive.0 {
        let (_action2, mut receiver_account, receiver_location) =
            ledger.get_or_create(&receiver_id)?;

        receiver_account.balance = receiver_balance;
        receiver_account.timing = coinbase_receiver_timing;

        ledger.set(&receiver_location, receiver_account);

        (
            FailureCollection::append_entry(FailureCollection::no_failure(), failures1),
            Amount::zero(),
        )
    } else {
        (
            FailureCollection::append_entry(FailureCollection::update_failed(), failures1),
            receiver_reward,
        )
    };

    if let Some((addr, account)) = transferee_update {
        ledger.set(&addr, account);
    };

    let burned_tokens = burned_tokens1
        .checked_add(&burned_tokens2)
        .ok_or_else(|| "burned tokens overflow".to_string())?;

    let status = if failures.is_empty() {
        TransactionStatus::Applied
    } else {
        TransactionStatus::Failed(failures.take())
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

    let status = if failures.is_empty() {
        TransactionStatus::Applied
    } else {
        TransactionStatus::Failed(failures.take())
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
) -> Result<(Vec<AccountId>, FailureCollection, Amount), String>
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
                Ok((new_accounts, FailureCollection::empty(), Amount::zero()))
            } else {
                Ok((
                    vec![],
                    FailureCollection::single_failure(),
                    Amount::of_fee(&fee_transfer.fee),
                ))
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
                    Ok((new_accounts, FailureCollection::empty(), Amount::zero()))
                } else {
                    // failure for each fee transfer single

                    Ok((
                        vec![],
                        FailureCollection::append_entry(
                            FailureCollection::update_failed(),
                            FailureCollection::single_failure(),
                        ),
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

                let (new_accounts1, failures, burned_tokens1) = if can_receive1.0 {
                    let (_, mut a1, l1) = ledger.get_or_create(&account_id1)?;
                    let new_accounts1 = get_new_accounts(action1, account_id1);

                    a1.balance = balance1;
                    ledger.set(&l1, a1);

                    (
                        new_accounts1,
                        FailureCollection::append_entry(
                            FailureCollection::no_failure(),
                            FailureCollection::empty(),
                        ),
                        Amount::zero(),
                    )
                } else {
                    (
                        None,
                        FailureCollection::single_failure(),
                        Amount::of_fee(&fee_transfer1.fee),
                    )
                };

                let (new_accounts2, failures, burned_tokens2) = if can_receive2.0 {
                    let (_, mut a2, l2) = ledger.get_or_create(&account_id2)?;
                    let new_accounts2 = get_new_accounts(action2, account_id2);

                    a2.balance = balance2;
                    a2.timing = timing2;

                    ledger.set(&l2, a2);

                    (
                        new_accounts2,
                        FailureCollection::append_entry(FailureCollection::no_failure(), failures),
                        Amount::zero(),
                    )
                } else {
                    (
                        None,
                        FailureCollection::append_entry(
                            FailureCollection::update_failed(),
                            failures,
                        ),
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
) -> (Box<Account>, AccountState, HasPermissionToReceive)
where
    L: LedgerIntf,
{
    use crate::PermissionTo::*;
    use AccountState::*;

    let init_account = Account::initialize(receiver_account_id);

    match ledger.location_of_account(receiver_account_id) {
        None => {
            // new account, check that default permissions allow receiving
            let perm = init_account.has_permission_to(ControlTag::NoneGiven, Receive);
            (Box::new(init_account), Added, HasPermissionToReceive(perm))
        }
        Some(location) => match ledger.get(&location) {
            None => panic!("Ledger location with no account"),
            Some(receiver_account) => {
                let perm = receiver_account.has_permission_to(ControlTag::NoneGiven, Receive);
                (receiver_account, Existed, HasPermissionToReceive(perm))
            }
        },
    }
}

pub fn validate_time(valid_until: &Slot, current_global_slot: &Slot) -> Result<(), String> {
    if current_global_slot <= valid_until {
        return Ok(());
    }

    Err(format!(
        "Current global slot {:?} greater than transaction expiry slot {:?}",
        current_global_slot, valid_until
    ))
}

pub fn is_timed(a: &Account) -> bool {
    matches!(&a.timing, Timing::Timed { .. })
}

pub fn set_with_location<L>(
    ledger: &mut L,
    location: &ExistingOrNew<L::Location>,
    account: Box<Account>,
) -> Result<(), String>
where
    L: LedgerIntf,
{
    match location {
        ExistingOrNew::Existing(location) => {
            ledger.set(location, account);
            Ok(())
        }
        ExistingOrNew::New => ledger
            .create_new_account(account.id(), *account)
            .map_err(|_| "set_with_location".to_string()),
    }
}

pub struct Updates<Location> {
    pub located_accounts: Vec<(ExistingOrNew<Location>, Box<Account>)>,
    pub applied_body: signed_command_applied::Body,
}

pub fn compute_updates<L>(
    constraint_constants: &ConstraintConstants,
    receiver: AccountId,
    ledger: &mut L,
    current_global_slot: &Slot,
    user_command: &SignedCommand,
    fee_payer: &AccountId,
    fee_payer_account: &Account,
    fee_payer_location: &ExistingOrNew<L::Location>,
    reject_command: &mut bool,
) -> Result<Updates<L::Location>, TransactionFailure>
where
    L: LedgerIntf,
{
    match &user_command.payload.body {
        signed_command::Body::StakeDelegation(_) => {
            let (receiver_location, _) = get_with_location(ledger, &receiver).unwrap();

            if let ExistingOrNew::New = receiver_location {
                return Err(TransactionFailure::ReceiverNotPresent);
            }
            if !fee_payer_account.has_permission_to_set_delegate() {
                return Err(TransactionFailure::UpdateNotPermittedDelegate);
            }

            let previous_delegate = fee_payer_account.delegate.clone();

            // Timing is always valid, but we need to record any switch from
            // timed to untimed here to stay in sync with the snark.
            let fee_payer_account = {
                let timing = timing_error_to_user_command_status(validate_timing(
                    fee_payer_account,
                    Amount::zero(),
                    current_global_slot,
                ))?;

                Box::new(Account {
                    delegate: Some(receiver.public_key.clone()),
                    timing,
                    ..fee_payer_account.clone()
                })
            };

            Ok(Updates {
                located_accounts: vec![(fee_payer_location.clone(), fee_payer_account)],
                applied_body: signed_command_applied::Body::StakeDelegation { previous_delegate },
            })
        }
        signed_command::Body::Payment(payment) => {
            let get_fee_payer_account = || {
                let balance = fee_payer_account
                    .balance
                    .sub_amount(payment.amount)
                    .ok_or(TransactionFailure::SourceInsufficientBalance)?;

                let timing = timing_error_to_user_command_status(validate_timing(
                    fee_payer_account,
                    payment.amount,
                    current_global_slot,
                ))?;

                Ok(Box::new(Account {
                    balance,
                    timing,
                    ..fee_payer_account.clone()
                }))
            };

            let fee_payer_account = match get_fee_payer_account() {
                Ok(fee_payer_account) => fee_payer_account,
                Err(e) => {
                    // OCaml throw an exception when an error occurs here
                    // Here in Rust we set `reject_command` to differentiate the 3 cases (Ok, Err, exception)
                    //
                    // https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/transaction_logic/mina_transaction_logic.ml#L962

                    // Don't accept transactions with insufficient balance from the fee-payer.
                    // TODO(OCaml): eliminate this condition and accept transaction with failed status
                    *reject_command = true;
                    return Err(e);
                }
            };

            let (receiver_location, mut receiver_account) = if fee_payer == &receiver {
                (fee_payer_location.clone(), fee_payer_account.clone())
            } else {
                get_with_location(ledger, &receiver).unwrap()
            };

            if !fee_payer_account.has_permission_to_send() {
                return Err(TransactionFailure::UpdateNotPermittedBalance);
            }

            if !receiver_account.has_permission_to_receive() {
                return Err(TransactionFailure::UpdateNotPermittedBalance);
            }

            let receiver_amount = match &receiver_location {
                ExistingOrNew::Existing(_) => payment.amount,
                ExistingOrNew::New => {
                    match payment
                        .amount
                        .checked_sub(&Amount::of_fee(&constraint_constants.account_creation_fee))
                    {
                        Some(amount) => amount,
                        None => return Err(TransactionFailure::AmountInsufficientToCreateAccount),
                    }
                }
            };

            let balance = match receiver_account.balance.add_amount(receiver_amount) {
                Some(balance) => balance,
                None => return Err(TransactionFailure::Overflow),
            };

            let new_accounts = match receiver_location {
                ExistingOrNew::New => vec![receiver.clone()],
                ExistingOrNew::Existing(_) => vec![],
            };

            receiver_account.balance = balance;

            let updated_accounts = if fee_payer == &receiver {
                // [receiver_account] at this point has all the updates
                vec![(receiver_location, receiver_account)]
            } else {
                vec![
                    (receiver_location, receiver_account),
                    (fee_payer_location.clone(), fee_payer_account),
                ]
            };

            Ok(Updates {
                located_accounts: updated_accounts,
                applied_body: signed_command_applied::Body::Payments { new_accounts },
            })
        }
    }
}

pub fn apply_user_command_unchecked<L>(
    constraint_constants: &ConstraintConstants,
    _txn_state_view: &ProtocolStateView,
    txn_global_slot: &Slot,
    ledger: &mut L,
    user_command: &SignedCommand,
) -> Result<SignedCommandApplied, String>
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
    let fee_payer = user_command.fee_payer();
    let (fee_payer_location, fee_payer_account) =
        pay_fee(user_command, signer_pk, ledger, current_global_slot)?;

    if !fee_payer_account.has_permission_to_send() {
        return Err(TransactionFailure::UpdateNotPermittedBalance.to_string());
    }
    if !fee_payer_account.has_permission_to_increment_nonce() {
        return Err(TransactionFailure::UpdateNotPermittedNonce.to_string());
    }

    // Charge the fee. This must happen, whether or not the command itself
    // succeeds, to ensure that the network is compensated for processing this
    // command.
    set_with_location(ledger, &fee_payer_location, fee_payer_account.clone())?;

    let receiver = user_command.receiver();

    let mut reject_command = false;

    match compute_updates(
        constraint_constants,
        receiver,
        ledger,
        current_global_slot,
        user_command,
        &fee_payer,
        &fee_payer_account,
        &fee_payer_location,
        &mut reject_command,
    ) {
        Ok(Updates {
            located_accounts,
            applied_body,
        }) => {
            for (location, account) in located_accounts {
                set_with_location(ledger, &location, account)?;
            }

            Ok(SignedCommandApplied {
                common: signed_command_applied::Common {
                    user_command: WithStatus::<SignedCommand> {
                        data: user_command.clone(),
                        status: TransactionStatus::Applied,
                    },
                },
                body: applied_body,
            })
        }
        Err(failure) if !reject_command => Ok(SignedCommandApplied {
            common: signed_command_applied::Common {
                user_command: WithStatus::<SignedCommand> {
                    data: user_command.clone(),
                    status: TransactionStatus::Failed(vec![vec![failure]]),
                },
            },
            body: signed_command_applied::Body::Failed,
        }),
        Err(failure) => {
            // This case occurs when an exception is throwned in OCaml
            // https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/transaction_logic/mina_transaction_logic.ml#L964
            assert!(reject_command);
            Err(failure.to_string())
        }
    }
}

pub fn apply_user_command<L>(
    constraint_constants: &ConstraintConstants,
    txn_state_view: &ProtocolStateView,
    txn_global_slot: &Slot,
    ledger: &mut L,
    user_command: &SignedCommand,
) -> Result<SignedCommandApplied, String>
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

pub fn pay_fee<L, Loc>(
    user_command: &SignedCommand,
    signer_pk: &CompressedPubKey,
    ledger: &mut L,
    current_global_slot: &Slot,
) -> Result<(ExistingOrNew<Loc>, Box<Account>), String>
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

fn pay_fee_impl<L>(
    command: &SignedCommandPayload,
    nonce: Nonce,
    fee_payer: AccountId,
    fee: Fee,
    ledger: &mut L,
    current_global_slot: &Slot,
) -> Result<(ExistingOrNew<L::Location>, Box<Account>), String>
where
    L: LedgerIntf,
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
    use ark_ff::PrimeField;
    use mina_hasher::{Hashable, ROInput as LegacyInput};
    use mina_signer::{NetworkId, PubKey, Signature};

    use crate::{
        decompress_pk,
        scan_state::transaction_logic::signed_command::{PaymentPayload, StakeDelegationPayload},
    };

    use super::*;

    #[derive(Clone)]
    pub struct Common {
        pub fee: Fee,
        pub fee_token: TokenId,
        pub fee_payer_pk: CompressedPubKey,
        pub nonce: Nonce,
        pub valid_until: Slot,
        pub memo: Memo,
    }

    #[derive(Clone, Debug)]
    pub enum Tag {
        Payment = 0,
        StakeDelegation = 1,
        FeeTransfer = 2,
        Coinbase = 3,
    }

    impl Tag {
        pub fn is_user_command(&self) -> bool {
            match self {
                Tag::Payment | Tag::StakeDelegation => true,
                Tag::FeeTransfer | Tag::Coinbase => false,
            }
        }

        pub fn is_payment(&self) -> bool {
            match self {
                Tag::Payment => true,
                Tag::FeeTransfer | Tag::Coinbase | Tag::StakeDelegation => false,
            }
        }

        pub fn is_stake_delegation(&self) -> bool {
            match self {
                Tag::StakeDelegation => true,
                Tag::FeeTransfer | Tag::Coinbase | Tag::Payment => false,
            }
        }

        pub fn is_fee_transfer(&self) -> bool {
            match self {
                Tag::FeeTransfer => true,
                Tag::StakeDelegation | Tag::Coinbase | Tag::Payment => false,
            }
        }

        pub fn is_coinbase(&self) -> bool {
            match self {
                Tag::Coinbase => true,
                Tag::StakeDelegation | Tag::FeeTransfer | Tag::Payment => false,
            }
        }

        pub fn to_bits(&self) -> [bool; 3] {
            let tag = self.clone() as u8;
            let mut bits = [false; 3];
            for (index, bit) in [4, 2, 1].iter().enumerate() {
                bits[index] = tag & bit != 0;
            }
            bits
        }

        pub fn to_untagged_bits(&self) -> [bool; 5] {
            let mut is_payment = false;
            let mut is_stake_delegation = false;
            let mut is_fee_transfer = false;
            let mut is_coinbase = false;
            let mut is_user_command = false;

            match self {
                Tag::Payment => {
                    is_payment = true;
                    is_user_command = true;
                }
                Tag::StakeDelegation => {
                    is_stake_delegation = true;
                    is_user_command = true;
                }
                Tag::FeeTransfer => is_fee_transfer = true,
                Tag::Coinbase => is_coinbase = true,
            }

            [
                is_payment,
                is_stake_delegation,
                is_fee_transfer,
                is_coinbase,
                is_user_command,
            ]
        }
    }

    #[derive(Clone)]
    pub struct Body {
        pub tag: Tag,
        pub source_pk: CompressedPubKey,
        pub receiver_pk: CompressedPubKey,
        pub token_id: TokenId,
        pub amount: Amount,
    }

    #[derive(Clone)]
    pub struct TransactionUnionPayload {
        pub common: Common,
        pub body: Body,
    }

    impl Hashable for TransactionUnionPayload {
        type D = NetworkId;

        fn to_roinput(&self) -> LegacyInput {
            /*
                Payment transactions only use the default token-id value 1.
                The old transaction format encoded the token-id as an u64,
                however zkApps encode the token-id as a Fp.

                For testing/fuzzing purposes we want the ability to encode
                arbitrary values different from the default token-id, for this
                we will extract the LS u64 of the token-id.
            */
            let fee_token_id = self.common.fee_token.0.into_repr().0[0];
            let token_id = self.body.token_id.0.into_repr().0[0];

            let mut roi = LegacyInput::new()
                .append_field(self.common.fee_payer_pk.x)
                .append_field(self.body.source_pk.x)
                .append_field(self.body.receiver_pk.x)
                .append_u64(self.common.fee.as_u64())
                .append_u64(fee_token_id)
                .append_bool(self.common.fee_payer_pk.is_odd)
                .append_u32(self.common.nonce.as_u32())
                .append_u32(self.common.valid_until.as_u32())
                .append_bytes(&self.common.memo.0);

            let tag = self.body.tag.clone() as u8;
            for bit in [4, 2, 1] {
                roi = roi.append_bool(tag & bit != 0);
            }

            roi.append_bool(self.body.source_pk.is_odd)
                .append_bool(self.body.receiver_pk.is_odd)
                .append_u64(token_id)
                .append_u64(self.body.amount.as_u64())
                .append_bool(false) // Used to be `self.body.token_locked`
        }

        fn domain_string(network_id: NetworkId) -> Option<String> {
            // Domain strings must have length <= 20
            match network_id {
                NetworkId::MAINNET => "MinaSignatureMainnet",
                NetworkId::TESTNET => "CodaSignature",
            }
            .to_string()
            .into()
        }
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
                        receiver_pk,
                        amount,
                    }) => Body {
                        tag: Tag::Payment,
                        source_pk: payload.common.fee_payer_pk.clone(),
                        receiver_pk: receiver_pk.clone(),
                        token_id: TokenId::default(),
                        amount: *amount,
                    },
                    StakeDelegation(StakeDelegationPayload::SetDelegate { new_delegate }) => Body {
                        tag: Tag::StakeDelegation,
                        source_pk: payload.common.fee_payer_pk.clone(),
                        receiver_pk: new_delegate.clone(),
                        token_id: TokenId::default(),
                        amount: Amount::zero(),
                    },
                },
            }
        }

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
                roi = roi.append_bytes(&self.common.memo.0);
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

                // default token_id
                roi = roi.append_u64(1);

                // amount
                roi = roi.append_u64(self.body.amount.0);

                // token_locked
                roi = roi.append_bool(false);
            }

            roi
        }
    }

    pub struct TransactionUnion {
        pub payload: TransactionUnionPayload,
        pub signer: PubKey,
        pub signature: Signature,
    }

    impl TransactionUnion {
        /// For SNARK purposes, we inject [Transaction.t]s into a single-variant 'tagged-union' record capable of
        /// representing all the variants. We interpret the fields of this union in different ways depending on
        /// the value of the [payload.body.tag] field, which represents which variant of [Transaction.t] the value
        /// corresponds to.
        ///
        /// Sometimes we interpret fields in surprising ways in different cases to save as much space in the SNARK as possible (e.g.,
        /// [payload.body.public_key] is interpreted as the recipient of a payment, the new delegate of a stake
        /// delegation command, and a fee transfer recipient for both coinbases and fee-transfers.
        pub fn of_transaction(tx: &Transaction) -> Self {
            match tx {
                Transaction::Command(cmd) => {
                    let UserCommand::SignedCommand(cmd) = cmd else {
                        unreachable!();
                    };

                    let SignedCommand {
                        payload,
                        signer,
                        signature,
                    } = cmd.as_ref();

                    TransactionUnion {
                        payload: TransactionUnionPayload::of_user_command_payload(payload),
                        signer: decompress_pk(signer).unwrap(),
                        signature: signature.clone(),
                    }
                }
                Transaction::Coinbase(Coinbase {
                    receiver,
                    amount,
                    fee_transfer,
                }) => {
                    let CoinbaseFeeTransfer {
                        receiver_pk: other_pk,
                        fee: other_amount,
                    } = fee_transfer.clone().unwrap_or_else(|| {
                        CoinbaseFeeTransfer::create(receiver.clone(), Fee::zero())
                    });

                    let signer = decompress_pk(&other_pk).unwrap();
                    let payload = TransactionUnionPayload {
                        common: Common {
                            fee: other_amount,
                            fee_token: TokenId::default(),
                            fee_payer_pk: other_pk.clone(),
                            nonce: Nonce::zero(),
                            valid_until: Slot::max(),
                            memo: Memo::empty(),
                        },
                        body: Body {
                            source_pk: other_pk,
                            receiver_pk: receiver.clone(),
                            token_id: TokenId::default(),
                            amount: *amount,
                            tag: Tag::Coinbase,
                        },
                    };

                    TransactionUnion {
                        payload,
                        signer,
                        signature: Signature::dummy(),
                    }
                }
                Transaction::FeeTransfer(tr) => {
                    let two = |SingleFeeTransfer {
                                   receiver_pk: pk1,
                                   fee: fee1,
                                   fee_token,
                               },
                               SingleFeeTransfer {
                                   receiver_pk: pk2,
                                   fee: fee2,
                                   fee_token: token_id,
                               }| {
                        let signer = decompress_pk(&pk2).unwrap();
                        let payload = TransactionUnionPayload {
                            common: Common {
                                fee: fee2,
                                fee_token,
                                fee_payer_pk: pk2.clone(),
                                nonce: Nonce::zero(),
                                valid_until: Slot::max(),
                                memo: Memo::empty(),
                            },
                            body: Body {
                                source_pk: pk2,
                                receiver_pk: pk1,
                                token_id,
                                amount: Amount::of_fee(&fee1),
                                tag: Tag::FeeTransfer,
                            },
                        };

                        TransactionUnion {
                            payload,
                            signer,
                            signature: Signature::dummy(),
                        }
                    };

                    match tr.0.clone() {
                        OneOrTwo::One(t) => {
                            let other = SingleFeeTransfer::create(
                                t.receiver_pk.clone(),
                                Fee::zero(),
                                t.fee_token.clone(),
                            );
                            two(t, other)
                        }
                        OneOrTwo::Two((t1, t2)) => two(t1, t2),
                    }
                }
            }
        }
    }
}

/// Returns the new `receipt_chain_hash`
pub fn cons_signed_command_payload(
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

/// Returns the new `receipt_chain_hash`
pub fn checked_cons_signed_command_payload(
    payload: &TransactionUnionPayload,
    last_receipt_chain_hash: ReceiptChainHash,
    w: &mut Witness<Fp>,
) -> ReceiptChainHash {
    use crate::proofs::witness::legacy_input::CheckedLegacyInput;
    use crate::proofs::witness::transaction_snark::checked_legacy_hash;

    let mut inputs = payload.to_checked_legacy_input_owned(w);
    inputs.append_field(last_receipt_chain_hash.0);

    let receipt_chain_hash = checked_legacy_hash("MinaReceiptUC", inputs, w);

    ReceiptChainHash(receipt_chain_hash)
}

/// prepend account_update index computed by Zkapp_command_logic.apply
///
/// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/receipt.ml#L66
pub fn cons_zkapp_command_commitment(
    index: Index,
    e: ZkAppCommandElt,
    receipt_hash: &ReceiptChainHash,
) -> ReceiptChainHash {
    let ZkAppCommandElt::ZkAppCommandCommitment(x) = e;

    let mut inputs = Inputs::new();

    inputs.append(&index);
    inputs.append_field(x.0);
    inputs.append(receipt_hash);

    ReceiptChainHash(hash_with_kimchi("MinaReceiptUC", &inputs.to_fields()))
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

pub fn validate_timing(
    account: &Account,
    txn_amount: Amount,
    txn_global_slot: &Slot,
) -> Result<Timing, String> {
    let (timing, _) = validate_timing_with_min_balance(account, txn_amount, txn_global_slot)?;

    Ok(timing)
}

pub fn account_check_timing(
    txn_global_slot: &Slot,
    account: &Account,
) -> (TimingValidation, Timing) {
    let (invalid_timing, timing, _) =
        validate_timing_with_min_balance_impl(account, Amount::from_u64(0), txn_global_slot);
    // TODO: In OCaml the returned Timing is actually converted to None/Some(fields of Timing structure)
    (invalid_timing, timing)
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

pub fn timing_error_to_user_command_status(
    timing_result: Result<Timing, String>,
) -> Result<Timing, TransactionFailure> {
    match timing_result {
        Ok(timing) => Ok(timing),
        Err(err_str) => {
            /*
                HACK: we are matching over the full error string instead
                of including an extra tag string to the Err variant
            */
            if err_str.contains("minimum balance") {
                return Err(TransactionFailure::SourceMinimumBalanceViolation);
            }

            if err_str.contains("is insufficient") {
                return Err(TransactionFailure::SourceInsufficientBalance);
            }

            panic!("Unexpected timed account validation error")
        }
    }
}

pub enum TimingValidation {
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
                Some(_) => (InvalidTiming(false), Untimed, MinBalance(Balance::zero())),
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
    vesting_period: SlotSpan,
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

#[derive(Clone, Debug)]
pub enum ExistingOrNew<Loc> {
    Existing(Loc),
    New,
}

fn get_with_location<L>(
    ledger: &mut L,
    account_id: &AccountId,
) -> Result<(ExistingOrNew<L::Location>, Box<Account>), String>
where
    L: LedgerIntf,
{
    match ledger.location_of_account(account_id) {
        Some(location) => match ledger.get(&location) {
            Some(account) => Ok((ExistingOrNew::Existing(location), account)),
            None => panic!("Ledger location with no account"),
        },
        None => Ok((
            ExistingOrNew::New,
            Box::new(Account::create_with(account_id.clone(), Balance::zero())),
        )),
    }
}

pub fn get_account<L>(
    ledger: &mut L,
    account_id: AccountId,
) -> (Box<Account>, ExistingOrNew<L::Location>)
where
    L: LedgerIntf,
{
    let (loc, account) = get_with_location(ledger, &account_id).unwrap();
    (account, loc)
}

pub fn set_account<'a, L>(
    l: &'a mut L,
    (a, loc): (Box<Account>, &ExistingOrNew<L::Location>),
) -> &'a mut L
where
    L: LedgerIntf,
{
    set_with_location(l, loc, a).unwrap();
    l
}

#[cfg(test)]
pub mod for_tests {
    use std::collections::{HashMap, HashSet};

    use mina_signer::Keypair;
    use rand::Rng;
    // use o1_utils::math::ceil_log2;

    use crate::{
        gen_keypair, scan_state::parallel_scan::ceil_log2, AuthRequired, BaseLedger, Mask,
        Permissions, ZkAppAccount,
    };

    use super::*;

    const MIN_INIT_BALANCE: u64 = 8000000000;
    const MAX_INIT_BALANCE: u64 = 8000000000000;
    const NUM_ACCOUNTS: u64 = 10;
    const NUM_TRANSACTIONS: u64 = 10;
    const DEPTH: u64 = ceil_log2(NUM_ACCOUNTS + NUM_TRANSACTIONS);

    /// Use this for tests only
    /// Hashmaps are not deterministic
    #[derive(Debug, PartialEq, Eq)]
    pub struct HashableKeypair(pub Keypair);

    impl std::hash::Hash for HashableKeypair {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            let compressed = self.0.public.into_compressed();
            HashableCompressedPubKey(compressed).hash(state);
        }
    }

    /// Use this for tests only
    /// Hashmaps are not deterministic
    #[derive(Clone, Debug, Eq, derive_more::From)]
    pub struct HashableCompressedPubKey(pub CompressedPubKey);

    impl PartialEq for HashableCompressedPubKey {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    impl std::hash::Hash for HashableCompressedPubKey {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.0.x.hash(state);
            self.0.is_odd.hash(state);
        }
    }

    impl PartialOrd for HashableCompressedPubKey {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            match self.0.x.partial_cmp(&other.0.x) {
                Some(core::cmp::Ordering::Equal) => {}
                ord => return ord,
            };
            self.0.is_odd.partial_cmp(&other.0.is_odd)
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/transaction_logic/mina_transaction_logic.ml#L2194
    #[derive(Debug)]
    pub struct InitLedger(pub Vec<(Keypair, u64)>);

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/transaction_logic/mina_transaction_logic.ml#L2230
    #[derive(Debug)]
    pub struct TransactionSpec {
        pub fee: Fee,
        pub sender: (Keypair, Nonce),
        pub receiver: CompressedPubKey,
        pub amount: Amount,
    }

    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/transaction_logic/mina_transaction_logic.ml#L2283
    #[derive(Debug)]
    pub struct TestSpec {
        pub init_ledger: InitLedger,
        pub specs: Vec<TransactionSpec>,
    }

    impl InitLedger {
        pub fn init(&self, zkapp: Option<bool>, ledger: &mut impl LedgerIntf) {
            let zkapp = zkapp.unwrap_or(true);

            self.0.iter().for_each(|(kp, amount)| {
                let (_tag, mut account, loc) = ledger
                    .get_or_create(&AccountId::new(
                        kp.public.into_compressed(),
                        TokenId::default(),
                    ))
                    .unwrap();

                use AuthRequired::Either;
                let permissions = Permissions {
                    edit_state: Either,
                    access: AuthRequired::None,
                    send: Either,
                    receive: AuthRequired::None,
                    set_delegate: Either,
                    set_permissions: Either,
                    set_verification_key: Either,
                    set_zkapp_uri: Either,
                    edit_action_state: Either,
                    set_token_symbol: Either,
                    increment_nonce: Either,
                    set_voting_for: Either,
                    set_timing: Either,
                };

                let zkapp = if zkapp {
                    let zkapp = ZkAppAccount {
                        // TODO: Hash is `Fp::zero` here
                        verification_key: Some(crate::dummy::trivial_verification_key()),
                        ..Default::default()
                    };

                    Some(zkapp)
                } else {
                    None
                };

                account.balance = Balance::from_u64(*amount);
                account.permissions = permissions;
                account.zkapp = zkapp;

                ledger.set(&loc, account);
            });
        }

        pub fn gen() -> Self {
            let mut rng = rand::thread_rng();

            let mut tbl = HashSet::with_capacity(256);

            let init = (0..NUM_ACCOUNTS)
                .map(|_| {
                    let kp = loop {
                        let keypair = gen_keypair();
                        let compressed = keypair.public.into_compressed();
                        if !tbl.contains(&HashableCompressedPubKey(compressed)) {
                            break keypair;
                        }
                    };

                    let amount = rng.gen_range(MIN_INIT_BALANCE..MAX_INIT_BALANCE);
                    tbl.insert(HashableCompressedPubKey(kp.public.into_compressed()));
                    (kp, amount)
                })
                .collect();

            Self(init)
        }
    }

    impl TransactionSpec {
        pub fn gen(init_ledger: &InitLedger, nonces: &mut HashMap<HashableKeypair, Nonce>) -> Self {
            let mut rng = rand::thread_rng();

            let pk = |(kp, _): (Keypair, u64)| kp.public.into_compressed();

            let receiver_is_new: bool = rng.gen();

            let mut gen_index = || rng.gen_range(0..init_ledger.0.len().checked_sub(1).unwrap());

            let receiver_index = if receiver_is_new {
                None
            } else {
                Some(gen_index())
            };

            let receiver = match receiver_index {
                None => gen_keypair().public.into_compressed(),
                Some(i) => pk(init_ledger.0[i].clone()),
            };

            let sender = {
                let i = match receiver_index {
                    None => gen_index(),
                    Some(j) => loop {
                        let i = gen_index();
                        if i != j {
                            break i;
                        }
                    },
                };
                init_ledger.0[i].0.clone()
            };

            let nonce = nonces
                .get(&HashableKeypair(sender.clone()))
                .cloned()
                .unwrap();

            let amount = Amount::from_u64(rng.gen_range(1_000_000..100_000_000));
            let fee = Fee::from_u64(rng.gen_range(1_000_000..100_000_000));

            let old = nonces.get_mut(&HashableKeypair(sender.clone())).unwrap();
            *old = old.incr();

            Self {
                fee,
                sender: (sender, nonce),
                receiver,
                amount,
            }
        }
    }

    impl TestSpec {
        fn mk_gen(num_transactions: Option<u64>) -> TestSpec {
            let num_transactions = num_transactions.unwrap_or(NUM_TRANSACTIONS);

            let init_ledger = InitLedger::gen();

            let mut map = init_ledger
                .0
                .iter()
                .map(|(kp, _)| (HashableKeypair(kp.clone()), Nonce::zero()))
                .collect();

            let specs = (0..num_transactions)
                .map(|_| TransactionSpec::gen(&init_ledger, &mut map))
                .collect();

            Self { init_ledger, specs }
        }

        pub fn gen() -> Self {
            Self::mk_gen(Some(NUM_TRANSACTIONS))
        }
    }

    #[derive(Debug)]
    pub struct UpdateStatesSpec {
        pub fee: Fee,
        pub sender: (Keypair, Nonce),
        pub fee_payer: Option<(Keypair, Nonce)>,
        pub receivers: Vec<(CompressedPubKey, Amount)>,
        pub amount: Amount,
        pub zkapp_account_keypairs: Vec<Keypair>,
        pub memo: Memo,
        pub new_zkapp_account: bool,
        pub snapp_update: zkapp_command::Update,
        // Authorization for the update being performed
        pub current_auth: AuthRequired,
        pub actions: Vec<Vec<Fp>>,
        pub events: Vec<Vec<Fp>>,
        pub call_data: Fp,
        pub preconditions: Option<zkapp_command::Preconditions>,
    }

    pub fn trivial_zkapp_account(
        permissions: Option<Permissions<AuthRequired>>,
        vk: VerificationKey,
        pk: CompressedPubKey,
    ) -> Account {
        let id = AccountId::new(pk, TokenId::default());
        let mut account = Account::create_with(id, Balance::from_u64(1_000_000_000_000_000));
        account.permissions = permissions.unwrap_or_else(Permissions::user_default);
        account.zkapp = Some(ZkAppAccount {
            verification_key: Some(vk),
            ..Default::default()
        });
        account
    }

    pub fn create_trivial_zkapp_account(
        permissions: Option<Permissions<AuthRequired>>,
        vk: VerificationKey,
        ledger: &mut Mask,
        pk: CompressedPubKey,
    ) {
        let id = AccountId::new(pk.clone(), TokenId::default());
        let account = trivial_zkapp_account(permissions, vk, pk);
        assert!(BaseLedger::location_of_account(ledger, &id).is_none());
        ledger.get_or_create_account(id, account).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use o1_utils::FieldHelpers;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use super::{
        signed_command::{Body, Common, PaymentPayload},
        *,
    };

    fn pub_key(address: &str) -> CompressedPubKey {
        mina_signer::PubKey::from_address(address)
            .unwrap()
            .into_compressed()
    }

    /// Test using same values as here:
    /// https://github.com/MinaProtocol/mina/blob/3a78f0e0c1343d14e2729c8b00205baa2ec70c93/src/lib/mina_base/receipt.ml#L136
    #[test]
    fn test_cons_receipt_hash_ocaml() {
        let from = pub_key("B62qr71UxuyKpkSKYceCPsjw14nuaeLwWKZdMqaBMPber5AAF6nkowS");
        let to = pub_key("B62qnvGVnU7FXdy8GdkxL7yciZ8KattyCdq5J6mzo5NCxjgQPjL7BTH");

        let common = Common {
            fee: Fee::from_u64(9758327274353182341),
            fee_payer_pk: from,
            nonce: Nonce::from_u32(1609569868),
            valid_until: Slot::from_u32(2127252111),
            memo: Memo([
                1, 32, 101, 26, 225, 104, 115, 118, 55, 102, 76, 118, 108, 78, 114, 50, 0, 115,
                110, 108, 53, 75, 109, 112, 50, 110, 88, 97, 76, 66, 76, 81, 235, 79,
            ]),
        };

        let body = Body::Payment(PaymentPayload {
            receiver_pk: to,
            amount: Amount::from_u64(1155659205107036493),
        });

        let tx = SignedCommandPayload { common, body };

        let prev = "4918218371695029984164006552208340844155171097348169027410983585063546229555";
        let prev_receipt_chain_hash = ReceiptChainHash(Fp::from_str(prev).unwrap());

        let next = "11119245469205697592341599081188990695704663506019727849135180468159777463297";
        let next_receipt_chain_hash = ReceiptChainHash(Fp::from_str(next).unwrap());

        let result = cons_signed_command_payload(&tx, prev_receipt_chain_hash);
        assert_eq!(result, next_receipt_chain_hash);
    }

    #[test]
    fn test_receipt_hash_update() {
        let from = pub_key("B62qmnY6m4c6bdgSPnQGZriSaj9vuSjsfh6qkveGTsFX3yGA5ywRaja");
        let to = pub_key("B62qjVQLxt9nYMWGn45mkgwYfcz8e8jvjNCBo11VKJb7vxDNwv5QLPS");

        let common = Common {
            fee: Fee::from_u64(14500000),
            fee_payer_pk: from,
            nonce: Nonce::from_u32(15),
            valid_until: Slot::from_u32(-1i32 as u32),
            memo: Memo([
                1, 7, 84, 104, 101, 32, 49, 48, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]),
        };

        let body = Body::Payment(PaymentPayload {
            receiver_pk: to,
            amount: Amount::from_u64(2354000000),
        });

        let tx = SignedCommandPayload { common, body };

        let mut prev =
            hex::decode("09ac04c9965b885acfc9c54141dbecfc63b2394a4532ea2c598d086b894bfb14")
                .unwrap();
        prev.reverse();
        let prev_receipt_chain_hash = ReceiptChainHash(Fp::from_bytes(&prev).unwrap());

        let mut next =
            hex::decode("0735169b96af4385c7345c94d4d65f83823309e95f72752d3f9d84f4282a53ac")
                .unwrap();
        next.reverse();
        let next_receipt_chain_hash = ReceiptChainHash(Fp::from_bytes(&next).unwrap());

        let result = cons_signed_command_payload(&tx, prev_receipt_chain_hash);
        assert_eq!(result, next_receipt_chain_hash);
    }
}
