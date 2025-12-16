use super::{
    zkapp_command, Account, AccountId, Amount, Balance, Fee, Memo, Nonce, TokenId,
    VerificationKeyWire,
};
use crate::{
    gen_keypair,
    scan_state::{currency::Magnitude, parallel_scan::ceil_log2},
    sparse_ledger::LedgerIntf,
    AuthRequired, BaseLedger, Mask, Permissions, VerificationKey, ZkAppAccount,
    TXN_VERSION_CURRENT,
};
use mina_curves::pasta::Fp;
use mina_signer::{CompressedPubKey, Keypair};
use rand::Rng;
use std::collections::{HashMap, HashSet};

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

/// OCaml reference: src/lib/transaction_logic/mina_transaction_logic.ml L:2285-2285
/// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
/// Last verified: 2025-10-10
#[derive(Debug)]
pub struct InitLedger(pub Vec<(Keypair, u64)>);

/// OCaml reference: src/lib/transaction_logic/mina_transaction_logic.ml L:2351-2356
/// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
/// Last verified: 2025-10-10
#[derive(Debug)]
pub struct TransactionSpec {
    pub fee: Fee,
    pub sender: (Keypair, Nonce),
    pub receiver: CompressedPubKey,
    pub amount: Amount,
}

/// OCaml reference: src/lib/transaction_logic/mina_transaction_logic.ml L:2407
/// Commit: 5da42ccd72e791f164d4d200cf1ce300262873b3
/// Last verified: 2025-10-10
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
                set_verification_key: crate::SetVerificationKey {
                    auth: Either,
                    txn_version: TXN_VERSION_CURRENT,
                },
                set_zkapp_uri: Either,
                edit_action_state: Either,
                set_token_symbol: Either,
                increment_nonce: Either,
                set_voting_for: Either,
                set_timing: Either,
            };

            let zkapp = if zkapp {
                let zkapp = ZkAppAccount {
                    verification_key: Some(VerificationKeyWire::new(
                        crate::dummy::trivial_verification_key(),
                    )),
                    ..Default::default()
                };

                Some(zkapp.into())
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
    account.zkapp = Some(
        ZkAppAccount {
            verification_key: Some(VerificationKeyWire::new(vk)),
            ..Default::default()
        }
        .into(),
    );
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
