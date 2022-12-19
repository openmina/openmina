use mina_hasher::Fp;
use mina_signer::{CompressedPubKey, Signature as SS};

use crate::{
    scan_state::currency::Magnitude, Account, AccountId, Address, BaseLedger, GetOrCreated,
    ReceiptChainHash, Timing, TokenId,
};

use self::{
    protocol_state::ProtocolStateView,
    signed_command::{SignedCommand, SignedCommandPayload},
    transaction_union_payload::TransactionUnionPayload,
    zkapp_command::AccountNonce,
};

use super::{
    currency::{Amount, Balance, Fee},
    scan_state::ConstraintConstants,
};

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/transaction_status.ml#L9
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum TransactionFailure {
    Predicate,
    Source_not_present,
    Receiver_not_present,
    Amount_insufficient_to_create_account,
    Cannot_pay_creation_fee_in_token,
    Source_insufficient_balance,
    Source_minimum_balance_violation,
    Receiver_already_exists,
    Token_owner_not_caller,
    Overflow,
    Global_excess_overflow,
    Local_excess_overflow,
    Local_supply_increase_overflow,
    Global_supply_increase_overflow,
    Signed_command_on_zkapp_account,
    Zkapp_account_not_present,
    Update_not_permitted_balance,
    Update_not_permitted_timing_existing_account,
    Update_not_permitted_delegate,
    Update_not_permitted_app_state,
    Update_not_permitted_verification_key,
    Update_not_permitted_sequence_state,
    Update_not_permitted_zkapp_uri,
    Update_not_permitted_token_symbol,
    Update_not_permitted_permissions,
    Update_not_permitted_nonce,
    Update_not_permitted_voting_for,
    Zkapp_command_replay_check_failed,
    Fee_payer_nonce_must_increase,
    Fee_payer_must_be_signed,
    Account_balance_precondition_unsatisfied,
    Account_nonce_precondition_unsatisfied,
    Account_receipt_chain_hash_precondition_unsatisfied,
    Account_delegate_precondition_unsatisfied,
    Account_sequence_state_precondition_unsatisfied,
    Account_app_state_precondition_unsatisfied(i64),
    Account_proved_state_precondition_unsatisfied,
    Account_is_new_precondition_unsatisfied,
    Protocol_state_precondition_unsatisfied,
    Incorrect_nonce,
    Invalid_fee_excess,
    Cancelled,
}

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/transaction_status.ml#L452
#[derive(Debug, Clone)]
pub enum TransactionStatus {
    Applied,
    Failed(TransactionFailure),
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
}

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/fee_transfer.ml#L19
#[derive(Debug, Clone)]
pub struct FeeTransfer {
    receiver_pk: CompressedPubKey,
    fee: Fee,
    fee_token: TokenId,
}

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/coinbase.ml#L17
#[derive(Debug, Clone)]
pub struct Coinbase {
    receiver: CompressedPubKey,
    amount: Amount,
    fee_transfer: FeeTransfer,
}

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signature.mli#L11
#[derive(Debug, Clone)]
pub struct Signature((Fp, (Fp, Fp)));

pub type Memo = Vec<u8>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slot(pub(super) u32);

impl Slot {
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Index(pub(super) u32);

pub mod signed_command {
    use crate::AccountId;

    use super::{zkapp_command::AccountNonce, *};

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.ml#L75
    #[derive(Debug, Clone)]
    pub struct Common {
        pub fee: Fee,
        pub fee_token: TokenId,
        pub fee_payer_pk: CompressedPubKey,
        pub nonce: AccountNonce,
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
            self.payload.common.valid_until.clone()
        }

        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.ml#L322
        pub fn fee_payer(&self) -> AccountId {
            let public_key = self.payload.common.fee_payer_pk.clone();
            AccountId::new(public_key, TokenId::default())
        }

        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.ml#L318
        pub fn fee_token(&self) -> TokenId {
            TokenId::default()
        }

        pub fn fee(&self) -> Fee {
            self.payload.common.fee.clone()
        }

        pub fn nonce(&self) -> AccountNonce {
            self.payload.common.nonce.clone()
        }
    }
}

pub mod zkapp_command {
    use crate::{
        scan_state::currency::{Balance, Signed},
        AuthRequired, Permissions, Slot, Timing, TokenSymbol, VerificationKey,
    };

    use super::*;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L834
    #[derive(Debug, Clone)]
    pub struct Events(Vec<Vec<Fp>>);

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_basic.ml#L100
    #[derive(Debug, Clone)]
    pub enum SetOrKeep<T> {
        Set(T),
        Kepp,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L319
    #[derive(Debug, Clone)]
    pub struct Update {
        app_state: [SetOrKeep<Fp>; 8],
        delegate: SetOrKeep<CompressedPubKey>,
        verification_key: SetOrKeep<VerificationKey>,
        permissions: SetOrKeep<Permissions<AuthRequired>>,
        zkapp_uri: SetOrKeep<String>,
        token_symbol: SetOrKeep<TokenSymbol>,
        timing: SetOrKeep<Timing>,
        voting_for: SetOrKeep<Fp>,
    }

    // TODO: This could be std::ops::Range ?
    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L23
    #[derive(Debug, Clone)]
    pub struct ClosedInterval<T> {
        lower: T,
        upper: T,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_basic.ml#L232
    #[derive(Debug, Clone)]
    pub enum OrIgnore<T> {
        Check(T),
        Ignore,
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

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_numbers/length.mli#L2
    #[derive(Debug, Clone)]
    pub struct Length(pub(super) u32);

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/epoch_ledger.ml#L9
    #[derive(Debug, Clone)]
    pub struct EpochLedger {
        hash: Hash<Fp>,
        total_currency: Numeric<Amount>,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L797
    #[derive(Debug, Clone)]
    pub struct EpochData {
        ledger: EpochLedger,
        seed: Hash<Fp>,
        start_checkpoint: Hash<Fp>,
        lock_checkpoint: Hash<Fp>,
        epoch_length: Numeric<Length>,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L977
    #[derive(Debug, Clone)]
    pub struct ZkAppPreconditions {
        snarked_ledger_hash: Hash<Fp>,
        timestamp: Numeric<BlockTime>,
        blockchain_length: Numeric<Length>,
        min_window_density: Numeric<Length>,
        last_vrf_output: (), // It's not defined in OCAml
        total_currency: Numeric<Amount>,
        global_slot_since_hard_fork: Numeric<Slot>,
        global_slot_since_genesis: Numeric<Slot>,
        staking_epoch_data: EpochData,
        next_epoch_data: EpochData,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_numbers/account_nonce.mli#L2
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct AccountNonce(pub(super) u32);

    impl AccountNonce {
        pub fn is_zero(&self) -> bool {
            self.0 == 0
        }

        pub fn as_u32(&self) -> u32 {
            self.0
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L478
    #[derive(Debug, Clone)]
    pub struct Account {
        balance: Numeric<Balance>,
        nonce: Numeric<AccountNonce>,
        receipt_chain_hash: Hash<Fp>, // TODO: Should be type `ReceiptChainHash`
        delegate: EqData<CompressedPubKey>,
        state: [EqData<Fp>; 8],
        sequence_state: EqData<Fp>,
        proved_state: EqData<bool>,
        is_new: EqData<bool>,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L613
    #[derive(Debug, Clone)]
    pub enum AccountPreconditions {
        Full(Box<Account>),
        Nonce(AccountNonce),
        Accept,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L758
    #[derive(Debug, Clone)]
    pub struct Preconditions {
        network: ZkAppPreconditions,
        account: AccountPreconditions,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L27
    #[derive(Debug, Clone)]
    pub enum AuthorizationKind {
        NoneGiven,
        Signature,
        Proof,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L955
    #[derive(Debug, Clone)]
    pub struct Body {
        public_key: CompressedPubKey,
        token_id: TokenId,
        update: Update,
        balance_change: Signed<Amount>,
        increment_nonce: bool,
        events: Events,
        sequence_events: Events,
        call_data: Fp,
        preconditions: Preconditions,
        use_full_commitment: bool,
        caller: TokenId,
        authorization_kind: AuthorizationKind,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/pickles/pickles_intf.ml#L316
    #[derive(Debug, Clone)]
    pub struct SideLoadedProof {
        // Side_loaded.Proof
        // TODO: Not sure what type this is yet...
    }

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
        body: Body,
        authorization: Control,
    }

    // Digest.Account_update.Stable.V1.t = Fp
    // Digest.Forest.Stable.V1.t = Fp

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L49
    #[derive(Debug, Clone)]
    pub struct Tree {
        account_update: AccountUpdate,
        account_update_digest: Fp,
        calls: Vec<WithStackHash>,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/with_stack_hash.ml#L6
    #[derive(Debug, Clone)]
    pub struct WithStackHash {
        elt: Tree,
        pub stack_hash: Fp,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L345
    #[derive(Debug, Clone)]
    pub struct CallForest(pub Vec<WithStackHash>);

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L1081
    #[derive(Debug, Clone)]
    pub struct FeePayerBody {
        public_key: CompressedPubKey,
        fee: Fee,
        valid_until: Option<Slot>,
        nonce: AccountNonce,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L1484
    #[derive(Debug, Clone)]
    pub struct FeePayer {
        body: FeePayerBody,
        authorization: Signature,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L959
    #[derive(Debug, Clone)]
    pub struct ZkAppCommand {
        fee_payer: FeePayer,
        account_updates: CallForest,
        memo: Memo,
    }
}

pub enum UserCommand {
    SignedCommand(Box<signed_command::SignedCommand>),
    ZkAppCommand(Box<zkapp_command::ZkAppCommand>),
}

pub enum Transaction {
    Command(UserCommand),
    FeeTransfer(FeeTransfer),
    Coinbase(Coinbase),
}

pub mod transaction_applied {
    use crate::{Account, AccountId};

    use super::*;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L17
    #[derive(Debug, Clone)]
    pub struct SignedCommandApplied {
        user_command: WithStatus<signed_command::SignedCommand>,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L65
    #[derive(Debug, Clone)]
    pub struct ZkappCommandApplied {
        accounts: Vec<(AccountId, Option<Account>)>,
        command: WithStatus<zkapp_command::ZkAppCommand>,
        new_accounts: Vec<AccountId>,
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
        fee_transfer: WithStatus<FeeTransfer>,
        new_accounts: Vec<AccountId>,
        burned_tokens: Amount,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L112
    #[derive(Debug, Clone)]
    pub struct CoinbaseApplied {
        coinbase: WithStatus<Coinbase>,
        new_accounts: Vec<AccountId>,
        burned_tokens: Amount,
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
        previous_hash: Fp,
        varying: Varying,
    }

    impl TransactionApplied {
        /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L639
        pub fn transaction(&self) -> WithStatus<Transaction> {
            use CommandApplied::*;
            use Varying::*;

            match &self.varying {
                Command(SignedCommand(cmd)) => cmd
                    .user_command
                    .map(|c| Transaction::Command(UserCommand::SignedCommand(Box::new(c.clone())))),
                Command(ZkappCommand(cmd)) => cmd
                    .command
                    .map(|c| Transaction::Command(UserCommand::ZkAppCommand(Box::new(c.clone())))),
                FeeTransfer(f) => f.fee_transfer.map(|f| Transaction::FeeTransfer(f.clone())),
                Coinbase(c) => c.coinbase.map(|c| Transaction::Coinbase(c.clone())),
            }
        }
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
        calls: CallForest, // TODO
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

    pub struct LocalState {
        stack_frame: Fp,
        call_stack: Fp,
        transaction_commitment: Fp,
        full_transaction_commitment: Fp,
        token_id: TokenId,
        excess: Signed<Amount>,
        supply_increase: Signed<Amount>,
        ledger: Fp,
        success: bool,
        account_update_index: Index,
        failure_status_tbl: Vec<Vec<TransactionFailure>>,
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
    transaction: Transaction,
) where
    L: BaseLedger,
{
    let previous_hash = ledger.merkle_root();
    let txn_global_slot = &txn_state_view.global_slot_since_genesis;

    match transaction {
        Transaction::Command(UserCommand::SignedCommand(cmd)) => todo!(),
        Transaction::Command(UserCommand::ZkAppCommand(cmd)) => todo!(),
        Transaction::FeeTransfer(_) => todo!(),
        Transaction::Coinbase(_) => todo!(),
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
    constraint_constants: &ConstraintConstants,
    txn_state_view: &ProtocolStateView,
    txn_global_slot: &Slot,
    ledger: &mut L,
    user_command: SignedCommand,
) -> Result<(), String>
where
    L: BaseLedger,
{
    let SignedCommand {
        payload,
        signer: signer_pk,
        signature,
    } = &user_command;
    let current_global_slot = txn_global_slot;

    let valid_until = user_command.valid_until();
    validate_time(&valid_until, current_global_slot)?;

    // Fee-payer information
    let fee_payer = user_command.fee_payer();
    let (fee_payer_location, fee_payer_account) =
        pay_fee(&user_command, signer_pk, ledger, current_global_slot)?;

    // TODO: The rest is implemented on the branch `transaction_fuzzer`

    Ok(())
}

fn pay_fee<L>(
    // constraint_constants: &ConstraintConstants,
    // txn_state_view: &ProtocolStateView,
    user_command: &SignedCommand,
    signer_pk: &CompressedPubKey,
    ledger: &mut L,
    current_global_slot: &Slot,
) -> Result<(ExistingOrNew, Account), String>
where
    L: BaseLedger,
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
    nonce: AccountNonce,
    fee_payer: AccountId,
    fee: Fee,
    ledger: &mut L,
    current_global_slot: &Slot,
) -> Result<(ExistingOrNew, Account), String>
where
    L: BaseLedger,
{
    // Fee-payer information
    let (location, mut account) = get_with_location(ledger, &fee_payer)?;

    if let ExistingOrNew::New = location {
        return Err("The fee-payer account does not exist".to_string());
    };

    let fee = Amount::of_fee(&fee);
    let balance = sub_amount(Balance(account.balance), fee.clone())?;

    validate_nonces(nonce, AccountNonce(account.nonce))?;
    let timing = validate_timing(&account, fee, current_global_slot)?;

    account.balance = balance.as_u64();
    account.nonce = account.nonce.wrapping_add(1); // TODO: Not sure if OCaml wraps
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
        nonce: AccountNonce,
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
                    fee: payload.common.fee.clone(),
                    fee_token: TokenId::default(),
                    fee_payer_pk: payload.common.fee_payer_pk.clone(),
                    nonce: payload.common.nonce.clone(),
                    valid_until: payload.common.valid_until.clone(),
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
                        amount: amount.clone(),
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
                roi = roi.append_bytes(&self.common.memo);
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

fn validate_nonces(txn_nonce: AccountNonce, account_nonce: AccountNonce) -> Result<(), String> {
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
        validate_timing_with_min_balance_impl(account, txn_amount.clone(), txn_global_slot);

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

    match account.timing {
        Untimed => {
            // no time restrictions
            match Balance(account.balance).sub_amount(txn_amount) {
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
            let account_balance = Balance(account.balance);
            let initial_minimum_balance = Balance(initial_minimum_balance);

            let (invalid_balance, invalid_timing, curr_min_balance) =
                match account_balance.sub_amount(txn_amount) {
                    None => {
                        // NB: The [initial_minimum_balance] here is the incorrect value,
                        // but:
                        // * we don't use it anywhere in this error case; and
                        // * we don't want to waste time computing it if it will be unused.
                        (true, false, initial_minimum_balance)
                    }
                    Some(proposed_new_balance) => {
                        let cliff_time = Slot(cliff_time);
                        let cliff_amount = Amount(cliff_amount);
                        let vesting_period = Slot(vesting_period);
                        let vesting_increment = Amount(vesting_increment);

                        let curr_min_balance = account_min_balance_at_slot(
                            txn_global_slot.clone(),
                            cliff_time,
                            cliff_amount,
                            vesting_period,
                            vesting_increment,
                            initial_minimum_balance,
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
fn account_min_balance_at_slot(
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

//     let%bind () = validate_nonces nonce account.nonce in
//     let%map timing =
//       validate_timing ~txn_amount:fee ~txn_global_slot:current_global_slot
//         ~account
//     in
//     ( location
//     , { account with
//         balance
//       ; nonce = Account.Nonce.succ account.nonce
//       ; receipt_chain_hash =
//           Receipt.Chain_hash.cons_signed_command_payload command
//             account.receipt_chain_hash
//       ; timing
//       } )

pub enum ExistingOrNew {
    Existing(Address),
    New,
}

fn get_with_location<L>(
    ledger: &mut L,
    account_id: &AccountId,
) -> Result<(ExistingOrNew, Account), String>
where
    L: BaseLedger,
{
    match ledger.location_of_account(account_id) {
        Some(location) => match ledger.get(location.clone()) {
            Some(account) => Ok((ExistingOrNew::Existing(location), account)),
            None => panic!("Ledger location with no account"),
        },
        None => Ok((
            ExistingOrNew::New,
            Account::create_with(account_id.clone(), 0),
        )),
    }
}

//     let%map () =
//       (* TODO: Remove this check and update the transaction snark once we have
//          an exchange rate mechanism. See issue #4447.
//       *)
//       if Token_id.equal fee_token Token_id.default then return ()
//       else
//         Or_error.errorf
//           "Cannot create transactions with fee_token different from the \
//            default"
//     in
//     ()
//   in
//   let%map loc, account' =
//     pay_fee' ~command:(Signed_command_payload user_command.payload) ~nonce
//       ~fee_payer
//       ~fee:(Signed_command.fee user_command)
//       ~ledger ~current_global_slot
//   in
//   (loc, account')

// let apply_user_command_unchecked
//     ~(constraint_constants : Genesis_constants.Constraint_constants.t)
//     ~txn_global_slot ledger
//     ({ payload; signer; signature = _ } as user_command : Signed_command.t) =
//   let open Or_error.Let_syntax in
//   let signer_pk = Public_key.compress signer in
//   let current_global_slot = txn_global_slot in
//   let%bind () =
//     validate_time
//       ~valid_until:(Signed_command.valid_until user_command)
//       ~current_global_slot
//   in
//   (* Fee-payer information *)
//   let fee_payer = Signed_command.fee_payer user_command in
//   let%bind fee_payer_location, fee_payer_account =
//     pay_fee ~user_command ~signer_pk ~ledger ~current_global_slot
//   in
//   let%bind () =
//     if Account.has_permission ~to_:`Send fee_payer_account then Ok ()
//     else
//       Or_error.error_string
//         Transaction_status.Failure.(describe Update_not_permitted_balance)
//   in
//   (* Charge the fee. This must happen, whether or not the command itself
//      succeeds, to ensure that the network is compensated for processing this
//      command.
//   *)
//   let%bind () =
//     set_with_location ledger fee_payer_location fee_payer_account
//   in
//   let source = Signed_command.source user_command in
//   let receiver = Signed_command.receiver user_command in
//   let exception Reject of Error.t in
//   let ok_or_reject = function Ok x -> x | Error err -> raise (Reject err) in
//   let compute_updates () =
//     let open Result.Let_syntax in
//     (* Compute the necessary changes to apply the command, failing if any of
//        the conditions are not met.
//     *)
//     match payload.body with
//     | Stake_delegation _ ->
//         let receiver_location, _receiver_account =
//           (* Check that receiver account exists. *)
//           get_with_location ledger receiver |> ok_or_reject
//         in
//         let source_location, source_account =
//           get_with_location ledger source |> ok_or_reject
//         in
//         let%bind () =
//           if Account.has_permission ~to_:`Set_delegate source_account then
//             Ok ()
//           else Error Transaction_status.Failure.Update_not_permitted_delegate
//         in
//         let%bind () =
//           match (source_location, receiver_location) with
//           | `Existing _, `Existing _ ->
//               return ()
//           | `New, _ ->
//               Result.fail Transaction_status.Failure.Source_not_present
//           | _, `New ->
//               Result.fail Transaction_status.Failure.Receiver_not_present
//         in
//         let previous_delegate = source_account.delegate in
//         (* Timing is always valid, but we need to record any switch from
//            timed to untimed here to stay in sync with the snark.
//         *)
//         let%map timing =
//           validate_timing ~txn_amount:Amount.zero
//             ~txn_global_slot:current_global_slot ~account:source_account
//           |> Result.map_error ~f:timing_error_to_user_command_status
//         in
//         let source_account =
//           { source_account with
//             delegate = Some (Account_id.public_key receiver)
//           ; timing
//           }
//         in
//         ( [ (source_location, source_account) ]
//         , Transaction_applied.Signed_command_applied.Body.Stake_delegation
//             { previous_delegate } )
//     | Payment { amount; _ } ->
//        (* Printf.eprintf "MY_LOG.getting_location\n%!"; *)
//         let receiver_location, receiver_account =
//           get_with_location ledger receiver |> ok_or_reject
//         in
//        (* Printf.eprintf "MY_LOG.got location\n%!"; *)
//         let%bind () =
//           if Account.has_permission ~to_:`Receive receiver_account then Ok ()
//           else Error Transaction_status.Failure.Update_not_permitted_balance
//         in
//         let%bind source_location, source_account =
//           let ret =
//             if Account_id.equal source receiver then
//               (*just check if the timing needs updating*)
//               let%bind location, account =
//                 match receiver_location with
//                 | `Existing _ ->
//                     return (receiver_location, receiver_account)
//                 | `New ->
//                     Result.fail Transaction_status.Failure.Source_not_present
//               in
//               let%map timing =
//                 validate_timing ~txn_amount:amount
//                   ~txn_global_slot:current_global_slot ~account
//                 |> Result.map_error ~f:timing_error_to_user_command_status
//               in
//               (location, { account with timing })
//             else
//               let location, account =
//                 get_with_location ledger source |> ok_or_reject
//               in
//               let%bind () =
//                 match location with
//                 | `Existing _ ->
//                     return ()
//                 | `New ->
//                     Result.fail Transaction_status.Failure.Source_not_present
//               in
//               let%bind timing =
//                 validate_timing ~txn_amount:amount
//                   ~txn_global_slot:current_global_slot ~account
//                 |> Result.map_error ~f:timing_error_to_user_command_status
//               in
//               let%map balance =
//                 Result.map_error (sub_amount account.balance amount)
//                   ~f:(fun _ ->
//                     Transaction_status.Failure.Source_insufficient_balance )
//               in
//               (location, { account with timing; balance })
//           in
//           if Account_id.equal fee_payer source then
//             (* Don't process transactions with insufficient balance from the
//                fee-payer.
//             *)
//             match ret with
//             | Ok x ->
//                 Ok x
//             | Error failure ->
//                 raise
//                   (Reject
//                      (Error.createf "%s"
//                         (Transaction_status.Failure.describe failure) ) )
//           else ret
//         in
//         let%bind () =
//           if Account.has_permission ~to_:`Send source_account then Ok ()
//           else Error Transaction_status.Failure.Update_not_permitted_balance
//         in
//         (* Charge the account creation fee. *)
//         let%bind receiver_amount =
//           (* Printf.eprintf "Amount_insufficient_to_create_account HERE222\n%!" ; *)
//           match receiver_location with
//           | `Existing _ ->
//               (* Printf.eprintf *)
//               (*   "MY_LOG.apply_user_command_unchecked existing\n%!" ; *)
//               return amount
//           | `New ->
//              (* Subtract the creation fee from the transaction amount. *)

//               sub_account_creation_fee ~constraint_constants `Added amount
//               |> Result.map_error ~f:(fun _ ->
//                      Transaction_status.Failure
//                      .Amount_insufficient_to_create_account )
//         in
//         (* Printf.eprintf *)
//         (*   "MY_LOG.apply_user_command_unchecked receiver_amount=%d\n%!" *)
//         (*   (Amount.to_int receiver_amount) ; *)
//         let%map receiver_account =
//           incr_balance receiver_account receiver_amount
//         in
//         let new_accounts =
//           match receiver_location with
//           | `Existing _ ->
//               []
//           | `New ->
//               [ receiver ]
//         in
//         (* let receiver_loc = *)
//         (*   match receiver_location with *)
//         (*   | `Existing _ -> *)
//         (*       "Existing" *)
//         (*   | `New -> *)
//         (*       "New" *)
//         (* in *)
//         (* let source_loc = *)
//         (*   match source_location with *)
//         (*   | `Existing _ -> *)
//         (*       "Existing" *)
//         (*   | `New -> *)
//         (*       "New" *)
//         (* in *)
//         (* Printf.eprintf *)
//         (*   "MY_LOG.apply_user_command_unchecked applied receiver=%s source=%s!\n%!" *)
//         (*   receiver_loc source_loc ; *)
//         ( [ (receiver_location, receiver_account)
//           ; (source_location, source_account)
//           ]
//         , Transaction_applied.Signed_command_applied.Body.Payment
//             { new_accounts } )
//   in
//   match compute_updates () with
//   | Ok (located_accounts, applied_body) ->
//       (* Printf.eprintf "compute_updates ok\n%!" ; *)
//       (* Update the ledger. *)
//       let%bind () =
//         List.fold located_accounts ~init:(Ok ())
//           ~f:(fun acc (location, account) ->
//             let%bind () = acc in
//             set_with_location ledger location account )
//       in
//       let applied_common : Transaction_applied.Signed_command_applied.Common.t
//           =
//         { user_command = { data = user_command; status = Applied } }
//       in
//       (* Printf.eprintf "compute_updates ok2\n%!" ; *)
//       return
//         ( { common = applied_common; body = applied_body }
//           : Transaction_applied.Signed_command_applied.t )
//   | Error failure ->
//       (* Printf.eprintf "compute_updates err=%s\n%!" *)
//       (*   (Transaction_status.Failure.to_string failure) ; *)
//       (* Do not update the ledger. Except for the fee payer which is already updated *)
//       let applied_common : Transaction_applied.Signed_command_applied.Common.t
//           =
//         { user_command =
//             { data = user_command
//             ; status =
//                 Failed
//                   (Transaction_status.Failure.Collection.of_single_failure
//                      failure )
//             }
//         }
//       in
//       return
//         ( { common = applied_common; body = Failed }
//           : Transaction_applied.Signed_command_applied.t )
//   | exception Reject err ->
//       (* Printf.eprintf "compute_updates exception\n%!" ; *)
//       (* TODO: These transactions should never reach this stage, this error
//          should be fatal.
//       *)
//       Error err

// let apply_transaction ~constraint_constants
//     ~(txn_state_view : Zkapp_precondition.Protocol_state.View.t) ledger
//     (t : Transaction.t) =
//   let previous_hash = merkle_root ledger in
//   let txn_global_slot = txn_state_view.global_slot_since_genesis in
//   Or_error.map
//     ( match t with
//     | Command (Signed_command txn) ->
//         (* Printf.eprintf "MY_LOG.APPLY_TRANSACTION.SIGNED_COMMAND\n%!" ; *)
//         Or_error.map
//           (apply_user_command_unchecked ~constraint_constants ~txn_global_slot
//              ledger txn ) ~f:(fun applied ->
//             Transaction_applied.Varying.Command (Signed_command applied) )
//     | Command (Zkapp_command txn) ->
//         (* Printf.eprintf "MY_LOG.APPLY_TRANSACTION.ZKAPP_COMMAND\n%!" ; *)
//         Or_error.map
//           (apply_zkapp_command_unchecked ~state_view:txn_state_view
//              ~constraint_constants ledger txn ) ~f:(fun (applied, _) ->
//             Transaction_applied.Varying.Command (Zkapp_command applied) )
//     | Fee_transfer t ->
//         (* Printf.eprintf "MY_LOG.APPLY_TRANSACTION.FREE_TRANSFER\n%!" ; *)
//         Or_error.map
//           (apply_fee_transfer ~constraint_constants ~txn_global_slot ledger t)
//           ~f:(fun applied -> Transaction_applied.Varying.Fee_transfer applied)
//     | Coinbase t ->
//         (* Printf.eprintf "MY_LOG.APPLY_TRANSACTION.COINBASE\n%!" ; *)
//         Or_error.map
//           (apply_coinbase ~constraint_constants ~txn_global_slot ledger t)
//           ~f:(fun applied -> Transaction_applied.Varying.Coinbase applied) )
//     ~f:(fun varying -> { Transaction_applied.previous_hash; varying })
