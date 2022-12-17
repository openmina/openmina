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

pub type WithStatus<T> = Result<T, TransactionFailure>;

pub struct FeeTransfer {
    receiver_pk: CompressedPubKey,
    fee: Fee,
    fee_token: TokenId,
}

pub struct Coinbase {
    receiver: CompressedPubKey,
    amount: Amount,
    fee_transfer: FeeTransfer,
}

/// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signature.mli#L11
pub struct Signature((Fp, (Fp, Fp)));

pub type Memo = Vec<u8>;

pub mod signed_command {
    use crate::Slot;

    use super::*;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.ml#L75
    pub struct Common {
        fee: Fee,
        fee_token: TokenId,
        fee_payer_pk: CompressedPubKey,
        nonce: u32,
        valid_until: Slot,
        memo: Memo,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/payment_payload.ml#L40
    pub struct PaymentPayload {
        source_pk: CompressedPubKey,
        receiver_pk: CompressedPubKey,
        amount: Amount,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/stake_delegation.ml#L11
    pub enum StakeDelegation {
        SetDelegate {
            delegator: CompressedPubKey,
            new_delegate: CompressedPubKey,
        },
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.mli#L24
    pub enum Body {
        Payment(PaymentPayload),
        StakeDelegation(StakeDelegation),
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.mli#L165
    pub struct SignedCommandPayload {
        common: Common,
        body: Body,
    }

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
    pub struct Events(Vec<Vec<Fp>>);

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_basic.ml#L100
    pub enum SetOrKeep<T> {
        Set(T),
        Kepp,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L319
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
    pub struct ClosedInterval<T> {
        lower: T,
        upper: T,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_basic.ml#L232
    pub enum OrIgnore<T> {
        Check(T),
        Ignore,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L439
    pub type Hash<T> = OrIgnore<T>;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L298
    pub type EqData<T> = OrIgnore<T>;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L178
    pub struct Numeric<T>(OrIgnore<ClosedInterval<T>>);

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/block_time/intf.ml#L55
    // TODO: Not sure if it's `u64`, but OCaml has methods `of_int64` and `to_in64`
    pub struct BlockTime(u64);

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_numbers/length.mli#L2
    pub struct Length(u32);

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/epoch_ledger.ml#L9
    pub struct EpochLedger {
        hash: Hash<Fp>,
        total_currency: Numeric<Amount>,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L797
    pub struct EpochData {
        ledger: EpochLedger,
        seed: Hash<Fp>,
        start_checkpoint: Hash<Fp>,
        lock_checkpoint: Hash<Fp>,
        epoch_length: Numeric<Length>,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_precondition.ml#L977
    pub struct ZkAppPreconditions {
        snarked_ledger_hash: Hash<Fp>,
        timestamp: Numeric<BlockTime>,
        blockchain_length: Numeric<Length>,
        min_window_density: Numeric<Length>,
        last_vrf_output: (), // TODO: It's not defined yet in OCAml
        total_current: Numeric<Amount>,
        global_slot_since_hard_fork: Numeric<Slot>,
        global_slot_since_genesis: Numeric<Slot>,
        staking_epoch_data: EpochData,
        next_epoch_data: EpochData,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_numbers/account_nonce.mli#L2
    pub struct AccountNonce(u32);

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
    pub enum AccountPreconditions {
        Full(Box<Account>),
        Nonce(AccountNonce),
        Accept,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L758
    pub struct Preconditions {
        network: ZkAppPreconditions,
        account: AccountPreconditions,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L27
    pub enum AuthorizationKind {
        NoneGiven,
        Signature,
        Proof,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L955
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
    pub struct SideLoadedProof {
        // Side_loaded.Proof
        // TODO: Not sure what type this is yet...
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/control.ml#L11
    pub enum Control {
        Proof(SideLoadedProof),
        Signature(Signature),
        NoneGiven,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/account_update.ml#L1437
    pub struct AccountUpdate {
        body: Body,
        authorization: Control,
    }

    // Digest.Account_update.Stable.V1.t = Fp
    // Digest.Forest.Stable.V1.t = Fp

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L49
    pub struct Tree {
        account_update: AccountUpdate,
        account_update_digest: Fp,
        calls: Vec<WithStackHash>,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/with_stack_hash.ml#L6
    pub struct WithStackHash {
        elt: Tree,
        hash: Fp,
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L345
    pub struct CallForest(Vec<WithStackHash>);

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/zkapp_command.ml#L959
    pub struct ZkAppCommand {
        fee_payer: CompressedPubKey,
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

mod transaction_applied {
    use mina_p2p_messages::v2::MinaTransactionLogicTransactionAppliedStableV2;

    use super::*;

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L639
    fn transaction(
        transaction: &MinaTransactionLogicTransactionAppliedStableV2,
    ) -> WithStatus<Transaction> {
        todo!()
    }

    // let transaction : t -> Transaction.t With_status.t =
    //  fun { varying; _ } ->
    //   match varying with
    //   | Command (Signed_command uc) ->
    //       With_status.map uc.common.user_command ~f:(fun cmd ->
    //           Transaction.Command (User_command.Signed_command cmd) )
    //   | Command (Zkapp_command s) ->
    //       With_status.map s.command ~f:(fun c ->
    //           Transaction.Command (User_command.Zkapp_command c) )
    //   | Fee_transfer f ->
    //       With_status.map f.fee_transfer ~f:(fun f ->
    //           Transaction.Fee_transfer f )
    //   | Coinbase c ->
    //       With_status.map c.coinbase ~f:(fun c -> Transaction.Coinbase c)
}
