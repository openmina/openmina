//! Tests for apply_transaction_first_pass
//!
//! Run with: cargo test --test test_transaction_logic_first_pass
//!
//! Tests the first pass of two-phase transaction application, covering:
//! - Successful payment transactions
//! - Payment creating receiver account
//! - Insufficient balance errors
//! - Invalid nonce errors
//! - Nonexistent fee payer errors

use ark_ff::Zero;
use mina_core::constants::ConstraintConstants;
use mina_curves::pasta::Fp;
use mina_tree::{
    scan_state::{
        currency::{Amount, Balance, Fee, Length, Magnitude, Nonce, Slot},
        transaction_logic::{
            protocol_state::{EpochData, EpochLedger, ProtocolStateView},
            signed_command::{Body, Common, PaymentPayload, SignedCommand, SignedCommandPayload},
            transaction_partially_applied::apply_transaction_first_pass,
            Memo, Transaction, UserCommand,
        },
    },
    Account, AccountId, BaseLedger, Database, Mask,
};

fn dummy_epoch_data() -> EpochData<Fp> {
    EpochData {
        ledger: EpochLedger {
            hash: Fp::zero(),
            total_currency: Amount::zero(),
        },
        seed: Fp::zero(),
        start_checkpoint: Fp::zero(),
        lock_checkpoint: Fp::zero(),
        epoch_length: Length::from_u32(0),
    }
}

fn test_constraint_constants() -> ConstraintConstants {
    ConstraintConstants {
        sub_windows_per_window: 11,
        ledger_depth: 15,
        work_delay: 2,
        block_window_duration_ms: 180_000,
        transaction_capacity_log_2: 7,
        pending_coinbase_depth: 5,
        coinbase_amount: 720_000_000_000,
        supercharged_coinbase_factor: 2,
        account_creation_fee: 1_000_000_000,
        fork: None,
    }
}

fn create_test_ledger() -> Mask {
    let db = Database::create(15);
    let mut ledger = Mask::new_root(db);
    let alice = mina_signer::PubKey::from_address(
        "B62qmnY6m4c6bdgSPnQGZriSaj9vuSjsfh6qkveGTsFX3yGA5ywRaja",
    )
    .unwrap()
    .into_compressed();
    let bob = mina_signer::PubKey::from_address(
        "B62qjVQLxt9nYMWGn45mkgwYfcz8e8jvjNCBo11VKJb7vxDNwv5QLPS",
    )
    .unwrap()
    .into_compressed();

    // Create Alice's account with balance
    let alice_id = AccountId::new(alice, Default::default());
    let alice_account = Account::create_with(alice_id.clone(), Balance::from_u64(1_000_000_000));
    ledger
        .get_or_create_account(alice_id, alice_account)
        .unwrap();

    // Create Bob's account
    let bob_id = AccountId::new(bob, Default::default());
    let bob_account = Account::create_with(bob_id.clone(), Balance::from_u64(500_000_000));
    ledger.get_or_create_account(bob_id, bob_account).unwrap();

    ledger
}

fn create_payment(
    from_pk: &mina_signer::CompressedPubKey,
    to_pk: &mina_signer::CompressedPubKey,
    amount: u64,
    fee: u64,
    nonce: u32,
) -> SignedCommand {
    let payload = SignedCommandPayload {
        common: Common {
            fee: Fee::from_u64(fee),
            fee_payer_pk: from_pk.clone(),
            nonce: Nonce::from_u32(nonce),
            valid_until: Slot::max(),
            memo: Memo::empty(),
        },
        body: Body::Payment(PaymentPayload {
            receiver_pk: to_pk.clone(),
            amount: Amount::from_u64(amount),
        }),
    };

    SignedCommand {
        payload,
        signer: from_pk.clone(),
        signature: mina_signer::Signature::dummy(),
    }
}

#[test]
fn test_apply_payment_success() {
    let mut ledger = create_test_ledger();

    let alice_pk = mina_signer::PubKey::from_address(
        "B62qmnY6m4c6bdgSPnQGZriSaj9vuSjsfh6qkveGTsFX3yGA5ywRaja",
    )
    .unwrap()
    .into_compressed();
    let bob_pk = mina_signer::PubKey::from_address(
        "B62qjVQLxt9nYMWGn45mkgwYfcz8e8jvjNCBo11VKJb7vxDNwv5QLPS",
    )
    .unwrap()
    .into_compressed();

    let alice_id = AccountId::new(alice_pk.clone(), Default::default());
    let bob_id = AccountId::new(bob_pk.clone(), Default::default());

    // Record initial state
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_before = ledger.get(alice_location).unwrap();
    let bob_location = ledger.location_of_account(&bob_id).unwrap();
    let bob_before = ledger.get(bob_location).unwrap();

    let initial_alice_balance = alice_before.balance;
    let initial_bob_balance = bob_before.balance;
    let initial_alice_nonce = alice_before.nonce;
    let initial_alice_receipt_hash = alice_before.receipt_chain_hash;

    let amount = 100_000_000;
    let fee = 10_000_000;
    let nonce = 0;
    let payment = create_payment(&alice_pk, &bob_pk, amount, fee, nonce);

    let constraint_constants = &test_constraint_constants();
    let state_view = ProtocolStateView {
        snarked_ledger_hash: Fp::zero(),
        blockchain_length: Length::from_u32(0),
        min_window_density: Length::from_u32(0),
        total_currency: Amount::zero(),
        global_slot_since_genesis: Slot::from_u32(0),
        staking_epoch_data: dummy_epoch_data(),
        next_epoch_data: dummy_epoch_data(),
    };
    let result = apply_transaction_first_pass(
        constraint_constants,
        Slot::from_u32(0),
        &state_view,
        &mut ledger,
        &Transaction::Command(UserCommand::SignedCommand(Box::new(payment))),
    );

    assert!(result.is_ok());

    // Verify ledger state changes
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_after = ledger.get(alice_location).unwrap();
    let bob_location = ledger.location_of_account(&bob_id).unwrap();
    let bob_after = ledger.get(bob_location).unwrap();

    // Verify Alice's balance decreased by fee + payment amount
    let expected_alice_balance = initial_alice_balance
        .sub_amount(Amount::from_u64(fee))
        .unwrap()
        .sub_amount(Amount::from_u64(amount))
        .unwrap();
    assert_eq!(
        alice_after.balance, expected_alice_balance,
        "Alice's balance should decrease by fee + payment amount"
    );

    // Verify Alice's nonce incremented
    assert_eq!(
        alice_after.nonce,
        initial_alice_nonce.incr(),
        "Alice's nonce should be incremented"
    );

    // Verify Alice's receipt chain hash updated
    assert_ne!(
        alice_after.receipt_chain_hash, initial_alice_receipt_hash,
        "Alice's receipt chain hash should be updated"
    );

    // Verify Bob's balance increased by payment amount
    let expected_bob_balance = initial_bob_balance
        .add_amount(Amount::from_u64(amount))
        .unwrap();
    assert_eq!(
        bob_after.balance, expected_bob_balance,
        "Bob's balance should increase by payment amount"
    );

    // Verify Bob's nonce unchanged (he's the receiver, not sender)
    assert_eq!(
        bob_after.nonce, bob_before.nonce,
        "Bob's nonce should not change"
    );
}

/// Test payment with insufficient balance for the payment amount.
///
/// Even though the fee can be paid, the payment amount exceeds the remaining
/// balance after fee deduction. The transaction returns an error but the fee
/// has already been charged to ensure the network is compensated.
///
/// Ledger state: Fee charged, nonce incremented, receipt chain hash updated.
/// Payment amount NOT transferred.
#[test]
fn test_apply_payment_insufficient_balance() {
    let mut ledger = create_test_ledger();

    let alice_pk = mina_signer::PubKey::from_address(
        "B62qmnY6m4c6bdgSPnQGZriSaj9vuSjsfh6qkveGTsFX3yGA5ywRaja",
    )
    .unwrap()
    .into_compressed();
    let bob_pk = mina_signer::PubKey::from_address(
        "B62qjVQLxt9nYMWGn45mkgwYfcz8e8jvjNCBo11VKJb7vxDNwv5QLPS",
    )
    .unwrap()
    .into_compressed();

    let alice_id = AccountId::new(alice_pk.clone(), Default::default());
    let bob_id = AccountId::new(bob_pk.clone(), Default::default());

    // Record initial state
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_before = ledger.get(alice_location).unwrap();
    let bob_location = ledger.location_of_account(&bob_id).unwrap();
    let bob_before = ledger.get(bob_location).unwrap();
    let initial_alice_balance = alice_before.balance;
    let initial_alice_nonce = alice_before.nonce;
    let initial_alice_receipt_hash = alice_before.receipt_chain_hash;
    let initial_bob_balance = bob_before.balance;

    let amount = 2_000_000_000; // More than Alice's balance
    let fee = 10_000_000;
    let nonce = 0;
    let payment = create_payment(&alice_pk, &bob_pk, amount, fee, nonce);

    let constraint_constants = &test_constraint_constants();
    let state_view = ProtocolStateView {
        snarked_ledger_hash: Fp::zero(),
        blockchain_length: Length::from_u32(0),
        min_window_density: Length::from_u32(0),
        total_currency: Amount::zero(),
        global_slot_since_genesis: Slot::from_u32(0),
        staking_epoch_data: dummy_epoch_data(),
        next_epoch_data: dummy_epoch_data(),
    };
    let result = apply_transaction_first_pass(
        constraint_constants,
        Slot::from_u32(0),
        &state_view,
        &mut ledger,
        &Transaction::Command(UserCommand::SignedCommand(Box::new(payment))),
    );

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Source_insufficient_balance");

    // Verify ledger state: fee charged but payment not transferred
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_after = ledger.get(alice_location).unwrap();
    let bob_location = ledger.location_of_account(&bob_id).unwrap();
    let bob_after = ledger.get(bob_location).unwrap();

    // Fee was charged
    let expected_alice_balance = initial_alice_balance
        .sub_amount(Amount::from_u64(fee))
        .unwrap();
    assert_eq!(
        alice_after.balance, expected_alice_balance,
        "Alice's balance should decrease by fee only"
    );

    // Nonce was incremented
    assert_eq!(
        alice_after.nonce,
        initial_alice_nonce.incr(),
        "Alice's nonce should be incremented"
    );

    // Receipt chain hash was updated
    assert_ne!(
        alice_after.receipt_chain_hash, initial_alice_receipt_hash,
        "Alice's receipt chain hash should be updated"
    );

    // Payment was NOT transferred to Bob
    assert_eq!(
        bob_after.balance, initial_bob_balance,
        "Bob's balance should remain unchanged"
    );
}

/// Test payment with incorrect nonce.
///
/// The transaction is rejected during fee payment validation because the
/// provided nonce does not match the account's current nonce.
///
/// Ledger state: Remains unchanged (no fee charged, no nonce incremented).
#[test]
fn test_apply_payment_invalid_nonce() {
    let mut ledger = create_test_ledger();

    let alice_pk = mina_signer::PubKey::from_address(
        "B62qmnY6m4c6bdgSPnQGZriSaj9vuSjsfh6qkveGTsFX3yGA5ywRaja",
    )
    .unwrap()
    .into_compressed();
    let bob_pk = mina_signer::PubKey::from_address(
        "B62qjVQLxt9nYMWGn45mkgwYfcz8e8jvjNCBo11VKJb7vxDNwv5QLPS",
    )
    .unwrap()
    .into_compressed();

    let alice_id = AccountId::new(alice_pk.clone(), Default::default());

    // Record initial state
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_before = ledger.get(alice_location).unwrap();
    let initial_alice_balance = alice_before.balance;
    let initial_alice_nonce = alice_before.nonce;

    let amount = 100_000_000;
    let fee = 10_000_000;
    let nonce = 5; // Wrong nonce (should be 0)
    let payment = create_payment(&alice_pk, &bob_pk, amount, fee, nonce);

    let constraint_constants = &test_constraint_constants();
    let state_view = ProtocolStateView {
        snarked_ledger_hash: Fp::zero(),
        blockchain_length: Length::from_u32(0),
        min_window_density: Length::from_u32(0),
        total_currency: Amount::zero(),
        global_slot_since_genesis: Slot::from_u32(0),
        staking_epoch_data: dummy_epoch_data(),
        next_epoch_data: dummy_epoch_data(),
    };
    let result = apply_transaction_first_pass(
        constraint_constants,
        Slot::from_u32(0),
        &state_view,
        &mut ledger,
        &Transaction::Command(UserCommand::SignedCommand(Box::new(payment))),
    );

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Nonce in account Nonce(0) different from nonce in transaction Nonce(5)"
    );

    // Verify ledger state unchanged
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_after = ledger.get(alice_location).unwrap();
    assert_eq!(
        alice_after.balance, initial_alice_balance,
        "Alice's balance should remain unchanged"
    );
    assert_eq!(
        alice_after.nonce, initial_alice_nonce,
        "Alice's nonce should remain unchanged"
    );
}

/// Test payment from a nonexistent fee payer account.
///
/// The transaction is rejected during fee payment validation because the
/// fee payer account does not exist in the ledger.
///
/// Ledger state: Remains unchanged (no new account created).
#[test]
fn test_apply_payment_nonexistent_fee_payer() {
    let db = Database::create(15);
    let mut ledger = Mask::new_root(db);

    let alice_pk = mina_signer::PubKey::from_address(
        "B62qmnY6m4c6bdgSPnQGZriSaj9vuSjsfh6qkveGTsFX3yGA5ywRaja",
    )
    .unwrap()
    .into_compressed();
    let bob_pk = mina_signer::PubKey::from_address(
        "B62qjVQLxt9nYMWGn45mkgwYfcz8e8jvjNCBo11VKJb7vxDNwv5QLPS",
    )
    .unwrap()
    .into_compressed();

    let alice_id = AccountId::new(alice_pk.clone(), Default::default());

    // Verify Alice's account does not exist before the transaction
    assert!(
        ledger.location_of_account(&alice_id).is_none(),
        "Alice's account should not exist before transaction"
    );

    let amount = 100_000_000;
    let fee = 10_000_000;
    let nonce = 0;
    let payment = create_payment(&alice_pk, &bob_pk, amount, fee, nonce);

    let constraint_constants = &test_constraint_constants();
    let state_view = ProtocolStateView {
        snarked_ledger_hash: Fp::zero(),
        blockchain_length: Length::from_u32(0),
        min_window_density: Length::from_u32(0),
        total_currency: Amount::zero(),
        global_slot_since_genesis: Slot::from_u32(0),
        staking_epoch_data: dummy_epoch_data(),
        next_epoch_data: dummy_epoch_data(),
    };
    let result = apply_transaction_first_pass(
        constraint_constants,
        Slot::from_u32(0),
        &state_view,
        &mut ledger,
        &Transaction::Command(UserCommand::SignedCommand(Box::new(payment))),
    );

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "The fee-payer account does not exist");

    // Verify Alice's account still does not exist after the error
    assert!(
        ledger.location_of_account(&alice_id).is_none(),
        "Alice's account should still not exist after transaction error"
    );
}

/// Test payment that creates a new receiver account.
///
/// When the receiver account doesn't exist, a new account is created
/// automatically. The account creation fee is deducted from the payment amount,
/// not from the sender's balance.
///
/// Ledger state: Sender's balance decreased by amount + fee, receiver account
/// created with balance = amount - account_creation_fee.
#[test]
fn test_apply_payment_creates_receiver_account() {
    let db = Database::create(15);
    let mut ledger = Mask::new_root(db);

    let alice_pk = mina_signer::PubKey::from_address(
        "B62qmnY6m4c6bdgSPnQGZriSaj9vuSjsfh6qkveGTsFX3yGA5ywRaja",
    )
    .unwrap()
    .into_compressed();
    let bob_pk = mina_signer::PubKey::from_address(
        "B62qjVQLxt9nYMWGn45mkgwYfcz8e8jvjNCBo11VKJb7vxDNwv5QLPS",
    )
    .unwrap()
    .into_compressed();

    // Create only Alice's account
    let alice_id = AccountId::new(alice_pk.clone(), Default::default());
    let alice_account = Account::create_with(alice_id.clone(), Balance::from_u64(5_000_000_000));
    ledger
        .get_or_create_account(alice_id.clone(), alice_account)
        .unwrap();

    let bob_id = AccountId::new(bob_pk.clone(), Default::default());

    // Verify Bob's account does not exist before the transaction
    assert!(
        ledger.location_of_account(&bob_id).is_none(),
        "Bob's account should not exist before transaction"
    );

    // Record initial state
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_before = ledger.get(alice_location).unwrap();
    let initial_alice_balance = alice_before.balance;
    let initial_alice_nonce = alice_before.nonce;
    let initial_alice_receipt_hash = alice_before.receipt_chain_hash;

    let amount = 2_000_000_000; // 2 MINA
    let fee = 10_000_000; // 0.01 MINA
    let nonce = 0;
    let payment = create_payment(&alice_pk, &bob_pk, amount, fee, nonce);

    let constraint_constants = &test_constraint_constants();
    let account_creation_fee = constraint_constants.account_creation_fee; // 1 MINA

    let state_view = ProtocolStateView {
        snarked_ledger_hash: Fp::zero(),
        blockchain_length: Length::from_u32(0),
        min_window_density: Length::from_u32(0),
        total_currency: Amount::zero(),
        global_slot_since_genesis: Slot::from_u32(0),
        staking_epoch_data: dummy_epoch_data(),
        next_epoch_data: dummy_epoch_data(),
    };
    let result = apply_transaction_first_pass(
        constraint_constants,
        Slot::from_u32(0),
        &state_view,
        &mut ledger,
        &Transaction::Command(UserCommand::SignedCommand(Box::new(payment))),
    );

    assert!(result.is_ok());

    // Verify Alice's balance decreased by fee + payment amount
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_after = ledger.get(alice_location).unwrap();
    let expected_alice_balance = initial_alice_balance
        .sub_amount(Amount::from_u64(fee))
        .unwrap()
        .sub_amount(Amount::from_u64(amount))
        .unwrap();
    assert_eq!(
        alice_after.balance, expected_alice_balance,
        "Alice's balance should decrease by fee + payment amount"
    );

    // Verify Alice's nonce incremented
    assert_eq!(
        alice_after.nonce,
        initial_alice_nonce.incr(),
        "Alice's nonce should be incremented"
    );

    // Verify Alice's receipt chain hash updated
    assert_ne!(
        alice_after.receipt_chain_hash, initial_alice_receipt_hash,
        "Alice's receipt chain hash should be updated"
    );

    // Verify Bob's account was created
    let bob_location = ledger.location_of_account(&bob_id);
    assert!(
        bob_location.is_some(),
        "Bob's account should now exist after transaction"
    );

    // Verify Bob's balance is payment amount minus account creation fee
    let bob_location = bob_location.unwrap();
    let bob_after = ledger.get(bob_location).unwrap();
    let expected_bob_balance = Balance::from_u64(amount - account_creation_fee);
    assert_eq!(
        bob_after.balance, expected_bob_balance,
        "Bob's balance should be payment amount minus account creation fee"
    );

    // Verify Bob's nonce is 0 (new account)
    assert_eq!(
        bob_after.nonce,
        Nonce::zero(),
        "Bob's nonce should be 0 for new account"
    );
}
