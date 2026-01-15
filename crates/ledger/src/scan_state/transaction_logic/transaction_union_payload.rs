//! Unified transaction representation for SNARK circuits
//!
//! This module provides a single, unified structure ([`TransactionUnion`]) that
//! can represent all transaction types (payments, stake delegations, fee
//! transfers, coinbase) for SNARK circuit processing. This enables efficient
//! proof generation by using a single circuit design regardless of the specific
//! transaction type.
//!
//! # Transaction Union
//!
//! The [`TransactionUnion`] type encodes all transaction variants using a
//! tagged union approach:
//!
//! - [`Common`]: Fields present in all transactions (fee, nonce, memo, etc.)
//! - [`Body`]: Transaction-specific fields with interpretation based on [`Tag`]
//! - [`Tag`]: Discriminates between Payment, StakeDelegation, FeeTransfer, and
//!   Coinbase
//!
//! # Field Interpretation
//!
//! Fields in [`Body`] are interpreted differently based on the [`Tag`] value:
//!
//! - **Payment**: `source_pk` and `receiver_pk` are sender and recipient
//! - **Stake Delegation**: `receiver_pk` is the new delegate
//! - **Fee Transfer**: `receiver_pk` is the fee recipient, `amount` is the fee
//! - **Coinbase**: `receiver_pk` is the block producer, `amount` is the reward
//!
//! # Receipt Chain Hash
//!
//! This module also provides functions for computing receipt chain hashes,
//! which commit to the sequence of transactions applied to an account:
//!
//! - [`cons_signed_command_payload`]: Updates receipt chain hash for signed
//!   commands
//! - [`cons_zkapp_command_commitment`]: Updates receipt chain hash for zkApp
//!   commands
//! - [`checked_cons_signed_command_payload`]: Checked version for use in
//!   circuits
//!
//! # Timing and Vesting
//!
//! The module implements timing validation for timed (vested) accounts:
//!
//! - [`validate_timing`]: Ensures timing constraints are met for an account
//!   deduction
//! - [`validate_nonces`]: Validates transaction nonce matches account nonce
//! - [`account_check_timing`]: Checks timing status for an account
//! - [`timing_error_to_user_command_status`]: Converts timing errors to
//!   transaction failures
//!
//! Timed accounts have a minimum balance that decreases over time according to
//! a vesting schedule. When the minimum balance reaches zero, the account
//! automatically becomes untimed.
//!
//! # Account Helpers
//!
//! Utility functions for account operations:
//!
//! - [`get_with_location`]: Retrieves an account or creates a placeholder for
//!   new accounts
//! - [`ExistingOrNew`]: Indicates whether an account exists or is newly created
//! - [`add_amount`]/[`sub_amount`]: Safe balance arithmetic with
//!   overflow/underflow checking

use super::{
    signed_command::{
        self, PaymentPayload, SignedCommand, SignedCommandPayload, StakeDelegationPayload,
    },
    transaction_partially_applied::set_with_location,
    Coinbase, CoinbaseFeeTransfer, Memo, SingleFeeTransfer, Transaction, TransactionFailure,
    UserCommand,
};
use crate::{
    decompress_pk,
    proofs::{field::Boolean, witness::Witness},
    scan_state::{
        currency::{Amount, Balance, Fee, Index, Magnitude, Nonce, Slot},
        scan_state::transaction_snark::OneOrTwo,
    },
    sparse_ledger::LedgerIntf,
    zkapps::zkapp_logic::ZkAppCommandElt,
    Account, AccountId, AppendToInputs, ReceiptChainHash, Timing, TokenId,
};
use ark_ff::PrimeField;
use mina_curves::pasta::Fp;
use mina_hasher::{Hashable, ROInput as LegacyInput};
use mina_signer::{CompressedPubKey, NetworkId, PubKey, Signature};
use poseidon::hash::{hash_with_kimchi, params::CODA_RECEIPT_UC, Inputs};

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
    pub fn is_user_command(&self) -> Boolean {
        match self {
            Tag::Payment | Tag::StakeDelegation => Boolean::True,
            Tag::FeeTransfer | Tag::Coinbase => Boolean::False,
        }
    }

    pub fn is_payment(&self) -> Boolean {
        match self {
            Tag::Payment => Boolean::True,
            Tag::FeeTransfer | Tag::Coinbase | Tag::StakeDelegation => Boolean::False,
        }
    }

    pub fn is_stake_delegation(&self) -> Boolean {
        match self {
            Tag::StakeDelegation => Boolean::True,
            Tag::FeeTransfer | Tag::Coinbase | Tag::Payment => Boolean::False,
        }
    }

    pub fn is_fee_transfer(&self) -> Boolean {
        match self {
            Tag::FeeTransfer => Boolean::True,
            Tag::StakeDelegation | Tag::Coinbase | Tag::Payment => Boolean::False,
        }
    }

    pub fn is_coinbase(&self) -> Boolean {
        match self {
            Tag::Coinbase => Boolean::True,
            Tag::StakeDelegation | Tag::FeeTransfer | Tag::Payment => Boolean::False,
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
        let fee_token_id = self.common.fee_token.0.into_bigint().0[0];
        let token_id = self.body.token_id.0.into_bigint().0[0];

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

    // TODO: this is unused, is it needed?
    fn domain_string(network_id: NetworkId) -> Option<String> {
        // Domain strings must have length <= 20
        match network_id {
            NetworkId::MAINNET => mina_core::network::mainnet::SIGNATURE_PREFIX,
            NetworkId::TESTNET => mina_core::network::devnet::SIGNATURE_PREFIX,
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

    /// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/transaction_union_payload.ml#L309>
    pub fn to_input_legacy(&self) -> ::poseidon::hash::legacy::Inputs<Fp> {
        let mut roi = ::poseidon::hash::legacy::Inputs::new();

        // Self.common
        {
            roi.append_u64(self.common.fee.0);

            // TokenId.default
            // <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/signed_command_payload.ml#L19>
            roi.append_bool(true);
            for _ in 0..63 {
                roi.append_bool(false);
            }

            // fee_payer_pk
            roi.append_field(self.common.fee_payer_pk.x);
            roi.append_bool(self.common.fee_payer_pk.is_odd);

            // nonce
            roi.append_u32(self.common.nonce.0);

            // valid_until
            roi.append_u32(self.common.valid_until.0);

            // memo
            roi.append_bytes(&self.common.memo.0);
        }

        // Self.body
        {
            // tag
            let tag = self.body.tag.clone() as u8;
            for bit in [4, 2, 1] {
                roi.append_bool(tag & bit != 0);
            }

            // source_pk
            roi.append_field(self.body.source_pk.x);
            roi.append_bool(self.body.source_pk.is_odd);

            // receiver_pk
            roi.append_field(self.body.receiver_pk.x);
            roi.append_bool(self.body.receiver_pk.is_odd);

            // default token_id
            roi.append_u64(1);

            // amount
            roi.append_u64(self.body.amount.0);

            // token_locked
            roi.append_bool(false);
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
    /// For SNARK purposes, we inject [Transaction.t]s into a single-variant
    /// 'tagged-union' record capable of representing all the variants. We
    /// interpret the fields of this union in different ways depending on the
    /// value of the [payload.body.tag] field, which represents which variant of
    /// [Transaction.t] the value corresponds to.
    ///
    /// Sometimes we interpret fields in surprising ways in different cases to
    /// save as much space in the SNARK as possible (e.g.,
    /// [payload.body.public_key] is interpreted as the recipient of a payment,
    /// the new delegate of a stake delegation command, and a fee transfer
    /// recipient for both coinbases and fee-transfers.
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
                } = fee_transfer
                    .clone()
                    .unwrap_or_else(|| CoinbaseFeeTransfer::create(receiver.clone(), Fee::zero()));

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

/// Returns the new `receipt_chain_hash`
pub fn cons_signed_command_payload(
    command_payload: &SignedCommandPayload,
    last_receipt_chain_hash: ReceiptChainHash,
) -> ReceiptChainHash {
    // Note: Not sure why they use the legacy way of hashing here

    use poseidon::hash::legacy;

    let ReceiptChainHash(last_receipt_chain_hash) = last_receipt_chain_hash;
    let union = TransactionUnionPayload::of_user_command_payload(command_payload);

    let mut inputs = union.to_input_legacy();
    inputs.append_field(last_receipt_chain_hash);
    let hash = legacy::hash_with_kimchi(&legacy::params::CODA_RECEIPT_UC, &inputs.to_fields());

    ReceiptChainHash(hash)
}

/// Returns the new `receipt_chain_hash`
pub fn checked_cons_signed_command_payload(
    payload: &TransactionUnionPayload,
    last_receipt_chain_hash: ReceiptChainHash,
    w: &mut Witness<Fp>,
) -> ReceiptChainHash {
    use crate::proofs::transaction::{
        legacy_input::CheckedLegacyInput, transaction_snark::checked_legacy_hash,
    };
    use poseidon::hash::legacy;

    let mut inputs = payload.to_checked_legacy_input_owned(w);
    inputs.append_field(last_receipt_chain_hash.0);

    let receipt_chain_hash = checked_legacy_hash(&legacy::params::CODA_RECEIPT_UC, inputs, w);

    ReceiptChainHash(receipt_chain_hash)
}

/// prepend account_update index computed by Zkapp_command_logic.apply
///
/// <https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/receipt.ml#L66>
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

    ReceiptChainHash(hash_with_kimchi(&CODA_RECEIPT_UC, &inputs.to_fields()))
}

pub fn validate_nonces(txn_nonce: Nonce, account_nonce: Nonce) -> Result<(), String> {
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
) -> (TimingValidation<bool>, Timing) {
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
            "For timed account {}, the requested transaction for amount {:?} \
         at global slot {:?}, applying the transaction would put the \
         balance below the calculated minimum balance of {:?}",
            account.public_key.into_address(),
            txn_amount,
            txn_global_slot,
            min_balance.0
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

pub enum TimingValidation<B> {
    InsufficientBalance(B),
    InvalidTiming(B),
}

#[derive(Debug)]
struct MinBalance(Balance);

fn validate_timing_with_min_balance_impl(
    account: &Account,
    txn_amount: Amount,
    txn_global_slot: &Slot,
) -> (TimingValidation<bool>, Timing, MinBalance) {
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
            ..
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
                        let curr_min_balance = account.min_balance_at_slot(*txn_global_slot);

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

pub fn sub_amount(balance: Balance, amount: Amount) -> Result<Balance, String> {
    balance
        .sub_amount(amount)
        .ok_or_else(|| "insufficient funds".to_string())
}

pub fn add_amount(balance: Balance, amount: Amount) -> Result<Balance, String> {
    balance
        .add_amount(amount)
        .ok_or_else(|| "overflow".to_string())
}

#[derive(Clone, Debug)]
pub enum ExistingOrNew<Loc> {
    Existing(Loc),
    New,
}

pub fn get_with_location<L>(
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
