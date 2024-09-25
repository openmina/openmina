use std::{collections::HashMap, rc::Rc};

use mina_signer::Keypair;
use rand::Rng;

use crate::{
    gen_keypair,
    scan_state::{
        currency::{Balance, Magnitude},
        transaction_logic::{
            for_tests::HashableCompressedPubKey,
            valid,
            zkapp_command::{self, verifiable},
        },
    },
    util, Account, AccountId, AuthRequired, BaseLedger, Mask, MyCowMut, Permissions, TokenId,
    VerificationKey, VerificationKeyWire, ZkAppAccount, TXN_VERSION_CURRENT,
};

use super::{
    zkapp_command::GenZkappCommandParams, Failure, Role, LEDGER_DEPTH, MAX_ACCOUNT_UPDATES,
    MAX_TOKEN_UPDATES, MINIMUM_USER_COMMAND_FEE,
};

fn zkapp_command_with_ledger(
    num_keypairs: Option<usize>,
    max_account_updates: Option<usize>,
    max_token_updates: Option<usize>,
    account_state_tbl: Option<&mut HashMap<AccountId, (Account, Role)>>,
    vk: Option<VerificationKeyWire>,
    failure: Option<&Failure>,
) -> (
    valid::UserCommand,
    Keypair,
    HashMap<HashableCompressedPubKey, Keypair>,
    Mask,
) {
    let mut rng = rand::thread_rng();

    // Need a fee payer keypair, a keypair for the "balancing" account (so that the balance changes
    // sum to zero), and max_account_updates * 2 keypairs, because all the other zkapp_command
    // might be new and their accounts not in the ledger; or they might all be old and in the ledger

    // We'll put the fee payer account and max_account_updates accounts in the
    // ledger, and have max_account_updates keypairs available for new accounts
    let max_account_updates = max_account_updates.unwrap_or(MAX_ACCOUNT_UPDATES);
    let max_token_updates = max_token_updates.unwrap_or(MAX_TOKEN_UPDATES);
    let num_keypairs =
        num_keypairs.unwrap_or((max_account_updates * 2) + (max_token_updates * 3) + 2);

    let keypairs: Vec<Keypair> = (0..num_keypairs).map(|_| gen_keypair()).collect();

    let keymap: HashMap<HashableCompressedPubKey, Keypair> = keypairs
        .iter()
        .map(|kp| {
            let compressed = kp.public.into_compressed();
            (HashableCompressedPubKey(compressed), kp.clone())
        })
        .collect();

    let num_keypairs_in_ledger = num_keypairs / 2;
    let keypairs_in_ledger = util::take(&keypairs, num_keypairs_in_ledger);

    let account_ids: Vec<AccountId> = keypairs_in_ledger
        .iter()
        .map(|Keypair { public, .. }| {
            AccountId::create(public.into_compressed(), TokenId::default())
        })
        .collect();

    let verification_key = vk.clone().unwrap_or_else(|| {
        let dummy_vk = VerificationKey::dummy();
        VerificationKeyWire::new((*dummy_vk).clone())
    });

    let balances: Vec<Balance> = {
        let min_cmd_fee = MINIMUM_USER_COMMAND_FEE;

        let min_balance = {
            let balance = min_cmd_fee.as_u64() + 100_000_000_000_000_000;
            Balance::from_u64(balance)
        };

        // max balance to avoid overflow when adding deltas
        let max_balance = {
            let max_bal = Balance::of_mina_string_exn("2000000000.0");

            assert_eq!(max_bal.as_u64(), 2000000000000000000);

            min_balance
                .checked_add(&max_bal)
                .expect("zkapp_command_with_ledger: overflow for max_balance")
        };

        (0..num_keypairs_in_ledger)
            .map(move |_| {
                let balance = rng.gen_range(min_balance.as_u64()..max_balance.as_u64());
                Balance::from_u64(balance)
            })
            .collect()
    };

    let account_ids_and_balances: Vec<(AccountId, Balance)> =
        account_ids.iter().cloned().zip(balances).collect();

    let snappify_account = |mut account: Account| {
        let permissions = Permissions {
            edit_state: AuthRequired::Either,
            send: AuthRequired::Either,
            set_delegate: AuthRequired::Either,
            set_permissions: AuthRequired::Either,
            set_verification_key: crate::SetVerificationKey {
                auth: AuthRequired::Either,
                txn_version: TXN_VERSION_CURRENT,
            },
            set_zkapp_uri: AuthRequired::Either,
            edit_action_state: AuthRequired::Either,
            set_token_symbol: AuthRequired::Either,
            increment_nonce: AuthRequired::Either,
            set_voting_for: AuthRequired::Either,
            set_timing: AuthRequired::Either,
            //receive: AuthRequired::Either,
            ..Permissions::user_default()
        };

        let verification_key = Some(verification_key.clone());
        let zkapp = Some(
            ZkAppAccount {
                verification_key,
                ..ZkAppAccount::default()
            }
            .into(),
        );

        account.zkapp = zkapp;
        account.permissions = permissions;

        account
    };

    // half zkApp accounts, half non-zkApp accounts
    let accounts =
        account_ids_and_balances
            .iter()
            .enumerate()
            .map(|(ndx, (account_id, balance))| {
                let account = Account::create_with(account_id.clone(), *balance);
                if ndx % 2 == 0 {
                    account
                } else {
                    snappify_account(account)
                }
            });

    let fee_payer_keypair = keypairs.first().unwrap();

    let mut ledger = Mask::create(LEDGER_DEPTH);

    account_ids.iter().zip(accounts).for_each(|(id, account)| {
        let res = ledger
            .get_or_create_account(id.clone(), account)
            .expect("zkapp_command: error adding account for account id");
        assert!(
            matches!(res, crate::GetOrCreated::Added(_)),
            "zkapp_command: account for account id already exists"
        );
    });

    // to keep track of account states across transactions
    let mut account_state_tbl = match account_state_tbl {
        Some(account_state_tbl) => MyCowMut::Borrow(account_state_tbl),
        None => MyCowMut::Own(HashMap::new()),
    };
    let account_state_tbl = Some(&mut *account_state_tbl);

    let zkapp_command =
        super::zkapp_command::gen_zkapp_command_from(super::zkapp_command::GenZkappCommandParams {
            failure,
            max_account_updates: Some(max_account_updates),
            max_token_updates: Some(max_token_updates),
            fee_payer_keypair,
            keymap: &keymap,
            account_state_tbl,
            ledger: ledger.clone(),
            protocol_state_view: None,
            vk: vk.as_ref(),
            global_slot: None,
        });

    use crate::scan_state::transaction_logic::TransactionStatus::Applied;

    let zkapp_command =
        zkapp_command::valid::to_valid(zkapp_command, &Applied, |hash, account_id| {
            verifiable::find_vk_via_ledger(ledger.clone(), hash, account_id)
        })
        .unwrap();
    let user_command = valid::UserCommand::ZkAppCommand(Box::new(zkapp_command));

    // include generated ledger in result
    (user_command, fee_payer_keypair.clone(), keymap, ledger)
}

/// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_generators/user_command_generators.ml#L146
pub fn sequence_zkapp_command_with_ledger(
    max_account_updates: Option<usize>,
    max_token_updates: Option<usize>,
    length: Option<usize>,
    vk: Option<VerificationKeyWire>,
    failure: Option<&Failure>,
) -> (
    Vec<(
        valid::UserCommand,
        Rc<Keypair>,
        Rc<HashMap<HashableCompressedPubKey, Keypair>>,
    )>,
    Mask,
) {
    let mut rng = rand::thread_rng();

    let length = length.unwrap_or_else(|| rng.gen::<usize>() % 100);
    let max_account_updates = max_account_updates.unwrap_or(MAX_ACCOUNT_UPDATES);
    let max_token_updates = max_token_updates.unwrap_or(MAX_TOKEN_UPDATES);

    let num_keypairs = length * max_account_updates * 2;

    // Keep track of account states across multiple zkapp_command transaction
    let mut account_state_tbl = HashMap::<AccountId, (Account, Role)>::with_capacity(64);

    let num_keypairs = Some(num_keypairs);
    let max_account_updates = Some(max_account_updates);
    let max_token_updates = Some(max_token_updates);
    // let account_state_tbl = Some(&mut account_state_tbl);

    let (zkapp_command, fee_payer_keypair, keymap, ledger) = zkapp_command_with_ledger(
        num_keypairs,
        max_account_updates,
        max_token_updates,
        Some(&mut account_state_tbl),
        vk.clone(),
        failure,
    );

    let fee_payer_keypair = Rc::new(fee_payer_keypair);
    let keymap = Rc::new(keymap);

    let mut commands = Vec::with_capacity(length);

    commands.push((
        zkapp_command,
        Rc::clone(&fee_payer_keypair),
        Rc::clone(&keymap),
    ));

    (0..length.saturating_sub(1)).for_each(|_| {
        let zkapp_command = super::zkapp_command::gen_zkapp_command_from(GenZkappCommandParams {
            failure,
            max_account_updates,
            max_token_updates,
            fee_payer_keypair: &fee_payer_keypair,
            keymap: &keymap,
            account_state_tbl: Some(&mut account_state_tbl),
            ledger: ledger.clone(),
            protocol_state_view: None,
            vk: vk.as_ref(),
            global_slot: None,
        });

        use crate::scan_state::transaction_logic::TransactionStatus::Applied;
        let zkapp_command =
            zkapp_command::valid::to_valid(zkapp_command, &Applied, |hash, account_id| {
                verifiable::find_vk_via_ledger(ledger.clone(), hash, account_id)
            })
            .unwrap();
        let zkapp_command = valid::UserCommand::ZkAppCommand(Box::new(zkapp_command));

        commands.push((
            zkapp_command,
            Rc::clone(&fee_payer_keypair),
            Rc::clone(&keymap),
        ));
    });

    (commands, ledger)
}
