//! Tests for apply_transaction_first_pass with zkApp command transactions
//!
//! Run with: cargo test --test test_transaction_logic_first_pass_zkapp
//!
//! Tests the first pass of two-phase transaction application for zkApp
//! commands, covering:
//! - Successful zkApp command with single account update
//! - zkApp command with fee payer insufficient balance
//! - zkApp command with invalid fee payer nonce
//! - zkApp command from nonexistent fee payer

use ark_ff::Zero;
use mina_core::constants::ConstraintConstants;
use mina_curves::pasta::Fp;
use mina_signer::Signature;
use mina_tree::{
    scan_state::{
        currency::{Amount, Balance, Fee, Length, Magnitude, Nonce, Sgn, Signed, Slot},
        transaction_logic::{
            protocol_state::{EpochData, EpochLedger, ProtocolStateView},
            transaction_partially_applied::apply_transaction_first_pass,
            zkapp_command::{
                Account, AccountPreconditions, AccountUpdate, Actions, AuthorizationKind, Body,
                CallForest, Control, Events, FeePayer, FeePayerBody, MayUseToken, Numeric,
                Preconditions, Tree, Update, WithStackHash, ZkAppCommand, ZkAppPreconditions,
            },
            Memo, Transaction, UserCommand,
        },
    },
    Account as LedgerAccount, AccountId, BaseLedger, Database, Mask, MutableFp, TokenId,
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
    let alice_account =
        LedgerAccount::create_with(alice_id.clone(), Balance::from_u64(1_000_000_000));
    ledger
        .get_or_create_account(alice_id, alice_account)
        .unwrap();

    // Create Bob's account
    let bob_id = AccountId::new(bob, Default::default());
    let bob_account = LedgerAccount::create_with(bob_id.clone(), Balance::from_u64(500_000_000));
    ledger.get_or_create_account(bob_id, bob_account).unwrap();

    ledger
}

fn create_simple_zkapp_command(
    fee_payer_pk: &mina_signer::CompressedPubKey,
    account_update_pk: &mina_signer::CompressedPubKey,
    fee: u64,
    nonce: u32,
) -> ZkAppCommand {
    // Create fee payer body
    let fee_payer_body = FeePayerBody {
        public_key: fee_payer_pk.clone(),
        fee: Fee::from_u64(fee),
        valid_until: Some(Slot::max()),
        nonce: Nonce::from_u32(nonce),
    };

    // Create fee payer with a dummy signature
    let fee_payer = FeePayer {
        body: fee_payer_body,
        authorization: Signature::dummy(),
    };

    // Create an account update body with no balance change
    let account_update_body = Body {
        public_key: account_update_pk.clone(),
        token_id: TokenId::default(),
        update: Update::noop(),
        balance_change: Signed {
            magnitude: Amount::zero(),
            sgn: Sgn::Pos,
        },
        increment_nonce: false,
        events: Events::empty(),
        actions: Actions::empty(),
        call_data: Fp::zero(),
        preconditions: Preconditions {
            network: ZkAppPreconditions::accept(),
            account: AccountPreconditions(Account::accept()),
            valid_while: Numeric::Ignore,
        },
        use_full_commitment: false,
        implicit_account_creation_fee: false,
        may_use_token: MayUseToken::No,
        authorization_kind: AuthorizationKind::NoneGiven,
    };

    // Create account update
    let account_update = AccountUpdate {
        body: account_update_body,
        authorization: Control::NoneGiven,
    };

    // Create a tree with the account update and empty calls
    let tree = Tree {
        account_update,
        account_update_digest: MutableFp::new(Fp::zero()),
        calls: CallForest::new(),
    };

    // Wrap tree in WithStackHash
    let tree_with_hash = WithStackHash {
        elt: tree,
        stack_hash: MutableFp::new(Fp::zero()),
    };

    // Create call forest with the tree
    let call_forest = CallForest(vec![tree_with_hash]);

    // Ensure hashes are computed
    call_forest.ensure_hashed();

    // Create the zkApp command
    ZkAppCommand {
        fee_payer,
        account_updates: call_forest,
        memo: Memo::empty(),
    }
}

#[test]
fn test_apply_zkapp_command_success() {
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

    let fee = 10_000_000;
    let nonce = 0;
    let zkapp_command = create_simple_zkapp_command(&alice_pk, &bob_pk, fee, nonce);

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
        &Transaction::Command(UserCommand::ZkAppCommand(Box::new(zkapp_command))),
    );

    assert!(result.is_ok());

    // Verify ledger state changes
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_after = ledger.get(alice_location).unwrap();

    // Verify Alice's balance decreased by fee
    let expected_alice_balance = initial_alice_balance
        .sub_amount(Amount::from_u64(fee))
        .unwrap();
    assert_eq!(
        alice_after.balance, expected_alice_balance,
        "Alice's balance should decrease by fee"
    );

    // Verify Alice's nonce incremented
    assert_eq!(
        alice_after.nonce,
        initial_alice_nonce.incr(),
        "Alice's nonce should be incremented"
    );
}

/// Test zkApp command with insufficient balance for fee.
///
/// The transaction is rejected during fee payment validation because the
/// fee payer balance is less than the required fee.
///
/// Ledger state: Remains unchanged (no fee charged, no nonce incremented).
#[test]
fn test_apply_zkapp_command_insufficient_balance() {
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

    // Create Alice's account with very small balance
    let alice_id = AccountId::new(alice_pk.clone(), Default::default());
    let alice_account = LedgerAccount::create_with(alice_id.clone(), Balance::from_u64(1_000_000));
    ledger
        .get_or_create_account(alice_id.clone(), alice_account)
        .unwrap();

    // Create Bob's account
    let bob_id = AccountId::new(bob_pk.clone(), Default::default());
    let bob_account = LedgerAccount::create_with(bob_id.clone(), Balance::from_u64(500_000_000));
    ledger.get_or_create_account(bob_id, bob_account).unwrap();

    // Record initial state
    let alice_location = ledger.location_of_account(&alice_id).unwrap();
    let alice_before = ledger.get(alice_location).unwrap();
    let initial_alice_balance = alice_before.balance;
    let initial_alice_nonce = alice_before.nonce;

    let fee = 10_000_000; // More than Alice's balance
    let nonce = 0;
    let zkapp_command = create_simple_zkapp_command(&alice_pk, &bob_pk, fee, nonce);

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
        &Transaction::Command(UserCommand::ZkAppCommand(Box::new(zkapp_command))),
    );

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "[[Overflow]]");

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

/// Test zkApp command with incorrect nonce.
///
/// The transaction is rejected during fee payment validation because the
/// provided nonce does not match the fee payer's current nonce.
///
/// Ledger state: Remains unchanged (no fee charged, no nonce incremented).
#[test]
fn test_apply_zkapp_command_invalid_nonce() {
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

    let fee = 10_000_000;
    let nonce = 5; // Wrong nonce (should be 0)
    let zkapp_command = create_simple_zkapp_command(&alice_pk, &bob_pk, fee, nonce);

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
        &Transaction::Command(UserCommand::ZkAppCommand(Box::new(zkapp_command))),
    );

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "[[AccountNoncePreconditionUnsatisfied]]"
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

/// Test zkApp command from a nonexistent fee payer account.
///
/// The transaction is rejected during fee payment validation because the
/// fee payer account does not exist in the ledger.
///
/// Ledger state: Remains unchanged (no new account created).
#[test]
fn test_apply_zkapp_command_nonexistent_fee_payer() {
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

    let fee = 10_000_000;
    let nonce = 0;
    let zkapp_command = create_simple_zkapp_command(&alice_pk, &bob_pk, fee, nonce);

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
        &Transaction::Command(UserCommand::ZkAppCommand(Box::new(zkapp_command))),
    );

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "[[Overflow, AmountInsufficientToCreateAccount]]"
    );

    // Verify Alice's account still does not exist after the error
    assert!(
        ledger.location_of_account(&alice_id).is_none(),
        "Alice's account should still not exist after transaction error"
    );
}
