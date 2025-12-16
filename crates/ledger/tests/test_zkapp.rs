// Run this test with:
// cargo test --package mina-tree --test test_zkapp

use ark_ff::Zero;
use mina_curves::pasta::Fp;
use mina_signer::Signature;
use mina_tree::{
    scan_state::{
        currency::{Amount, Fee, Magnitude, Nonce, Sgn, Signed, Slot},
        transaction_logic::{
            zkapp_command::{
                Account, AccountPreconditions, AccountUpdate, Actions, AuthorizationKind, Body,
                CallForest, Control, Events, FeePayer, FeePayerBody, MayUseToken, Numeric,
                Preconditions, Tree, Update, WithStackHash, ZkAppCommand, ZkAppPreconditions,
            },
            Memo,
        },
    },
    MutableFp, TokenId,
};

#[test]
fn test_zkapp_command_creation() {
    // Create fee payer account
    let fee_payer_pk = mina_signer::PubKey::from_address(
        "B62qmnY6m4c6bdgSPnQGZriSaj9vuSjsfh6qkveGTsFX3yGA5ywRaja",
    )
    .unwrap()
    .into_compressed();

    // Create fee payer body
    let fee_payer_body = FeePayerBody {
        public_key: fee_payer_pk.clone(),
        fee: Fee::from_u64(1000000),
        valid_until: Some(Slot::from_u32(100000)),
        nonce: Nonce::from_u32(0),
    };

    // Create fee payer with a dummy signature
    let fee_payer = FeePayer {
        body: fee_payer_body,
        authorization: Signature::dummy(),
    };

    // Create an account update body
    let account_update_pk = mina_signer::PubKey::from_address(
        "B62qjVQLxt9nYMWGn45mkgwYfcz8e8jvjNCBo11VKJb7vxDNwv5QLPS",
    )
    .unwrap()
    .into_compressed();

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
    let zkapp_command = ZkAppCommand {
        fee_payer,
        account_updates: call_forest,
        memo: Memo::empty(),
    };

    // Verify basic properties
    assert_eq!(zkapp_command.fee(), Fee::from_u64(1000000));
    assert_eq!(zkapp_command.fee_payer().public_key, fee_payer_pk);
    assert_eq!(zkapp_command.fee_token(), TokenId::default());

    // Verify account updates - access the inner vector directly
    assert_eq!(zkapp_command.account_updates.0.len(), 1);
    assert_eq!(
        zkapp_command.account_updates.0[0]
            .elt
            .account_update
            .body
            .public_key,
        account_update_pk
    );
}
