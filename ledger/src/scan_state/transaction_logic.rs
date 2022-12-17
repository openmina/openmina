use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::TokenId;

use super::currency::{Amount, Fee};

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

pub mod signed_command {
    use crate::Slot;

    use super::*;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.ml#L75
    #[derive(Debug, Clone)]
    pub struct Common {
        fee: Fee,
        fee_token: TokenId,
        fee_payer_pk: CompressedPubKey,
        nonce: u32,
        valid_until: Slot,
        memo: Memo,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/payment_payload.ml#L40
    #[derive(Debug, Clone)]
    pub struct PaymentPayload {
        source_pk: CompressedPubKey,
        receiver_pk: CompressedPubKey,
        amount: Amount,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/stake_delegation.ml#L11
    #[derive(Debug, Clone)]
    pub enum StakeDelegation {
        SetDelegate {
            delegator: CompressedPubKey,
            new_delegate: CompressedPubKey,
        },
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.mli#L24
    #[derive(Debug, Clone)]
    pub enum Body {
        Payment(PaymentPayload),
        StakeDelegation(StakeDelegation),
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.mli#L165
    #[derive(Debug, Clone)]
    pub struct SignedCommandPayload {
        common: Common,
        body: Body,
    }

    #[derive(Debug, Clone)]
    pub struct SignedCommand {
        payload: SignedCommandPayload,
        signer: CompressedPubKey,
        signature: Signature,
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
    #[derive(Debug, Clone)]
    pub struct Numeric<T>(OrIgnore<ClosedInterval<T>>);

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/block_time/intf.ml#L55
    // TODO: Not sure if it's `u64`, but OCaml has methods `of_int64` and `to_in64`
    #[derive(Debug, Clone)]
    pub struct BlockTime(u64);

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_numbers/length.mli#L2
    #[derive(Debug, Clone)]
    pub struct Length(u32);

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
    #[derive(Debug, Clone)]
    pub struct AccountNonce(u32);

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
        hash: Fp,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L345
    #[derive(Debug, Clone)]
    pub struct CallForest(Vec<WithStackHash>);

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
