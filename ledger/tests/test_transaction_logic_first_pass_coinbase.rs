//! Tests for apply_transaction_first_pass with coinbase transactions
//!
//! Run with: cargo test --test test_transaction_logic_first_pass_coinbase --release
//!
//! Tests the first pass of two-phase transaction application for coinbase
//! rewards, covering:
//! - Successful coinbase without fee transfer
//! - Successful coinbase with fee transfer to different account
//! - Coinbase with fee transfer to nonexistent account (creates account)
//! - Coinbase with fee transfer to same account (fee transfer should be
//!   removed)
//! - Coinbase creating a new account

use ark_ff::Zero;
use mina_core::constants::ConstraintConstants;
use mina_curves::pasta::Fp;
use mina_tree::{
    scan_state::{
        currency::{Amount, Balance, Fee, Length, Magnitude, Nonce, Slot},
        transaction_logic::{
            protocol_state::{EpochData, EpochLedger, ProtocolStateView},
            transaction_partially_applied::apply_transaction_first_pass,
            Coinbase, CoinbaseFeeTransfer, Transaction,
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

    // Create Alice's account with balance
    let alice_id = AccountId::new(alice, Default::default());
    let alice_account = Account::create_with(alice_id.clone(), Balance::from_u64(1_000_000_000));
    ledger
        .get_or_create_account(alice_id, alice_account)
        .unwrap();

    ledger
}

#[test]
fn test_apply_coinbase_without_fee_transfer() {
    let mut ledger = create_test_ledger();

    let alice_pk = mina_signer::PubKey::from_address(
        "B62qmnY6m4c6bdgSPnQGZriSaj9vuSjsfh6qkveGTsFX3yGA5ywRaja",
    )
    .unwrap()
    .into_compressed();

    let alice_id = AccountId::new(alice_pk.clone(), Default::default());

    // Record initial state
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_before = ledger.get(alice_location).unwrap();
    let initial_alice_balance = alice_before.balance;

    // Create a coinbase of 720 MINA to Alice with no fee transfer
    let coinbase_amount = Amount::from_u64(720_000_000_000);
    let coinbase = Coinbase::create(coinbase_amount, alice_pk.clone(), None).unwrap();

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
        &Transaction::Coinbase(coinbase),
    );

    assert!(result.is_ok());

    // Verify ledger state changes
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_after = ledger.get(alice_location).unwrap();

    // Verify Alice's balance increased by coinbase amount
    let expected_alice_balance = initial_alice_balance.add_amount(coinbase_amount).unwrap();
    assert_eq!(
        alice_after.balance, expected_alice_balance,
        "Alice's balance should increase by coinbase amount"
    );

    // Verify Alice's nonce unchanged (coinbase doesn't affect nonces)
    assert_eq!(
        alice_after.nonce, alice_before.nonce,
        "Alice's nonce should remain unchanged"
    );
}

#[test]
fn test_apply_coinbase_with_fee_transfer() {
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

    // Create Bob's account
    let bob_id = AccountId::new(bob_pk.clone(), Default::default());
    let bob_account = Account::create_with(bob_id.clone(), Balance::from_u64(500_000_000));
    ledger
        .get_or_create_account(bob_id.clone(), bob_account)
        .unwrap();

    // Record initial state
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_before = ledger.get(alice_location).unwrap();
    let bob_location = ledger.location_of_account(&bob_id).unwrap();
    let bob_before = ledger.get(bob_location).unwrap();
    let initial_alice_balance = alice_before.balance;
    let initial_bob_balance = bob_before.balance;

    // Create a coinbase of 720 MINA to Alice with a 10 MINA fee transfer to Bob
    let coinbase_amount = Amount::from_u64(720_000_000_000);
    let fee_transfer_amount = Fee::from_u64(10_000_000_000);
    let fee_transfer = CoinbaseFeeTransfer::create(bob_pk.clone(), fee_transfer_amount);
    let coinbase = Coinbase::create(coinbase_amount, alice_pk.clone(), Some(fee_transfer)).unwrap();

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
        &Transaction::Coinbase(coinbase),
    );

    assert!(result.is_ok());

    // Verify ledger state changes
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_after = ledger.get(alice_location).unwrap();
    let bob_location = ledger.location_of_account(&bob_id).unwrap();
    let bob_after = ledger.get(bob_location).unwrap();

    // Verify Alice's balance increased by (coinbase amount - fee transfer amount)
    // The fee transfer is deducted from the coinbase reward
    let coinbase_after_fee_transfer = coinbase_amount
        .checked_sub(&Amount::of_fee(&fee_transfer_amount))
        .unwrap();
    let expected_alice_balance = initial_alice_balance
        .add_amount(coinbase_after_fee_transfer)
        .unwrap();
    assert_eq!(
        alice_after.balance, expected_alice_balance,
        "Alice's balance should increase by coinbase minus fee transfer"
    );

    // Verify Bob's balance increased by fee transfer amount
    let expected_bob_balance = initial_bob_balance
        .add_amount(Amount::of_fee(&fee_transfer_amount))
        .unwrap();
    assert_eq!(
        bob_after.balance, expected_bob_balance,
        "Bob's balance should increase by fee transfer amount"
    );

    // Verify nonces unchanged
    assert_eq!(
        alice_after.nonce, alice_before.nonce,
        "Alice's nonce should remain unchanged"
    );
    assert_eq!(
        bob_after.nonce, bob_before.nonce,
        "Bob's nonce should remain unchanged"
    );
}

/// Test coinbase with fee transfer to a nonexistent account.
///
/// The coinbase receiver exists, but the fee transfer receiver doesn't exist.
/// The fee transfer should create the receiver account, deducting the account
/// creation fee from the fee transfer amount.
///
/// Ledger state:
/// - Coinbase receiver gets coinbase_amount - fee_transfer_amount
/// - Fee transfer receiver account created with fee_transfer_amount -
///   account_creation_fee
#[test]
fn test_apply_coinbase_with_fee_transfer_creates_account() {
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

    // Verify Bob's account does not exist before the transaction
    assert!(
        ledger.location_of_account(&bob_id).is_none(),
        "Bob's account should not exist before transaction"
    );

    // Record Alice's initial state
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_before = ledger.get(alice_location).unwrap();
    let initial_alice_balance = alice_before.balance;

    // Create a coinbase of 720 MINA to Alice with a 10 MINA fee transfer to Bob
    // (who doesn't exist yet)
    let coinbase_amount = Amount::from_u64(720_000_000_000);
    let fee_transfer_amount = Fee::from_u64(10_000_000_000);
    let fee_transfer = CoinbaseFeeTransfer::create(bob_pk.clone(), fee_transfer_amount);
    let coinbase = Coinbase::create(coinbase_amount, alice_pk.clone(), Some(fee_transfer)).unwrap();

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
        &Transaction::Coinbase(coinbase),
    );

    assert!(result.is_ok());

    // Verify Bob's account was created
    let bob_location = ledger.location_of_account(&bob_id);
    assert!(
        bob_location.is_some(),
        "Bob's account should exist after transaction"
    );

    // Verify ledger state changes
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_after = ledger.get(alice_location).unwrap();
    let bob_account = ledger.get(bob_location.unwrap()).unwrap();

    // Verify Alice's balance increased by (coinbase amount - fee transfer amount)
    let coinbase_after_fee_transfer = coinbase_amount
        .checked_sub(&Amount::of_fee(&fee_transfer_amount))
        .unwrap();
    let expected_alice_balance = initial_alice_balance
        .add_amount(coinbase_after_fee_transfer)
        .unwrap();
    assert_eq!(
        alice_after.balance, expected_alice_balance,
        "Alice's balance should increase by coinbase minus fee transfer"
    );

    // Verify Bob's balance equals fee transfer amount minus account creation fee
    let account_creation_fee = constraint_constants.account_creation_fee;
    let expected_bob_balance = Balance::from_u64(
        Amount::of_fee(&fee_transfer_amount)
            .as_u64()
            .saturating_sub(account_creation_fee),
    );
    assert_eq!(
        bob_account.balance, expected_bob_balance,
        "Bob's balance should equal fee transfer minus account creation fee"
    );

    // Verify nonces
    assert_eq!(
        alice_after.nonce, alice_before.nonce,
        "Alice's nonce should remain unchanged"
    );
    assert_eq!(
        bob_account.nonce,
        Nonce::zero(),
        "Bob's nonce should be 0 for new account"
    );
}

/// Test coinbase with fee transfer to the same account.
///
/// When the coinbase receiver and fee transfer receiver are the same, the fee
/// transfer should be removed during coinbase creation, and the receiver
/// should only receive the coinbase amount (not coinbase + fee transfer).
///
/// Ledger state: Receiver gets only coinbase amount.
#[test]
fn test_apply_coinbase_with_fee_transfer_to_same_account() {
    let mut ledger = create_test_ledger();

    let alice_pk = mina_signer::PubKey::from_address(
        "B62qmnY6m4c6bdgSPnQGZriSaj9vuSjsfh6qkveGTsFX3yGA5ywRaja",
    )
    .unwrap()
    .into_compressed();

    let alice_id = AccountId::new(alice_pk.clone(), Default::default());

    // Record initial state
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_before = ledger.get(alice_location).unwrap();
    let initial_alice_balance = alice_before.balance;

    // Create a coinbase of 720 MINA to Alice with a 10 MINA fee transfer also
    // to Alice. The fee transfer should be removed.
    let coinbase_amount = Amount::from_u64(720_000_000_000);
    let fee_transfer_amount = Fee::from_u64(10_000_000_000);
    let fee_transfer = CoinbaseFeeTransfer::create(alice_pk.clone(), fee_transfer_amount);
    let coinbase = Coinbase::create(coinbase_amount, alice_pk.clone(), Some(fee_transfer)).unwrap();

    // Verify that the fee transfer was removed during creation
    assert!(
        coinbase.fee_transfer.is_none(),
        "Fee transfer should be None when receiver equals fee transfer receiver"
    );

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
        &Transaction::Coinbase(coinbase),
    );

    assert!(result.is_ok());

    // Verify ledger state changes
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_after = ledger.get(alice_location).unwrap();

    // Verify Alice's balance increased by ONLY coinbase amount (not coinbase +
    // fee transfer)
    let expected_alice_balance = initial_alice_balance.add_amount(coinbase_amount).unwrap();
    assert_eq!(
        alice_after.balance, expected_alice_balance,
        "Alice's balance should increase by only coinbase amount"
    );

    // Verify Alice's nonce unchanged
    assert_eq!(
        alice_after.nonce, alice_before.nonce,
        "Alice's nonce should remain unchanged"
    );
}

/// Test coinbase to a nonexistent account.
///
/// The receiver account does not exist, so the coinbase should create it with
/// the coinbase amount as balance.
///
/// Ledger state: New account created with the coinbase amount as balance.
#[test]
fn test_apply_coinbase_creates_account() {
    let db = Database::create(15);
    let mut ledger = Mask::new_root(db);

    let bob_pk = mina_signer::PubKey::from_address(
        "B62qjVQLxt9nYMWGn45mkgwYfcz8e8jvjNCBo11VKJb7vxDNwv5QLPS",
    )
    .unwrap()
    .into_compressed();

    let bob_id = AccountId::new(bob_pk.clone(), Default::default());

    // Verify Bob's account does not exist before the transaction
    assert!(
        ledger.location_of_account(&bob_id).is_none(),
        "Bob's account should not exist before transaction"
    );

    // Create a coinbase of 720 MINA to Bob (who doesn't exist yet)
    let coinbase_amount = Amount::from_u64(720_000_000_000);
    let coinbase = Coinbase::create(coinbase_amount, bob_pk.clone(), None).unwrap();

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
        &Transaction::Coinbase(coinbase),
    );

    assert!(result.is_ok());

    // Verify Bob's account was created
    let bob_location = ledger.location_of_account(&bob_id);
    assert!(
        bob_location.is_some(),
        "Bob's account should exist after transaction"
    );

    // Verify Bob's balance equals the coinbase amount minus account creation fee
    let bob_account = ledger.get(bob_location.unwrap()).unwrap();
    let account_creation_fee = constraint_constants.account_creation_fee;
    let expected_balance = Balance::from_u64(
        coinbase_amount
            .as_u64()
            .saturating_sub(account_creation_fee),
    );
    assert_eq!(
        bob_account.balance, expected_balance,
        "Bob's balance should equal coinbase minus account creation fee"
    );

    // Verify Bob's nonce is 0 (new account)
    assert_eq!(
        bob_account.nonce,
        Nonce::zero(),
        "Bob's nonce should be 0 for new account"
    );
}
