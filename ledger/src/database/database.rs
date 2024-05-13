use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::{
    account::{Account, AccountId, TokenId},
    address::Address,
    base::{AccountIndex, BaseLedger, GetOrCreated, MerklePath, Uuid},
    // tree::{Database, DatabaseError},
    tree_version::V2,
    TreeVersion,
};

use crate::HashesMatrix;

use super::database_impl::DatabaseImpl;

#[derive(Debug, PartialEq, Eq)]
pub enum DatabaseError {
    OutOfLeaves,
}

#[derive(Clone, Debug)]
pub struct Database<T: TreeVersion> {
    // Using a mutex for now but this can be replaced with a RefCell
    pub inner: Arc<Mutex<DatabaseImpl<T>>>,
}

// #[derive(Debug)]
// pub enum UnregisterBehavior {
//     Check,
//     Recursive,
//     IPromiseIAmReparentingThisDatabase,
// }

impl Database<V2> {
    pub fn with<F, R>(&self, fun: F) -> R
    where
        F: FnOnce(&mut DatabaseImpl<V2>) -> R,
    {
        let mut inner = self.inner.try_lock().expect("lock failed");
        fun(&mut inner)
    }
}

impl Database<V2> {
    pub fn create_with_dir(depth: u8, dir_name: Option<PathBuf>) -> Self {
        let db = DatabaseImpl::<V2>::create_with_dir(depth, dir_name);

        Self {
            inner: Arc::new(Mutex::new(db)),
        }
    }

    pub fn create(depth: u8) -> Self {
        Self::create_with_dir(depth, None)
    }

    pub fn root_hash(&mut self) -> Fp {
        self.with(|this| this.root_hash())
    }

    // Do not use
    pub fn naccounts(&self) -> usize {
        self.with(|this| this.naccounts())
    }

    pub fn create_checkpoint(&self, directory_name: String) {
        self.with(|this| this.create_checkpoint(directory_name))
    }

    pub fn make_checkpoint(&self, directory_name: String) {
        self.with(|this| this.make_checkpoint(directory_name))
    }

    pub fn clone_db(&self, directory_name: PathBuf) -> Self {
        let db = self.with(|this| this.clone_db(directory_name));
        Self {
            inner: Arc::new(Mutex::new(db)),
        }
    }

    pub fn get_cached_hash(&self, addr: &Address) -> Option<Fp> {
        self.with(|this| this.get_cached_hash(addr))
    }

    pub fn set_cached_hash(&mut self, addr: &Address, hash: Fp) {
        self.with(|this| this.set_cached_hash(addr, hash))
    }

    pub fn empty_hash_at_height(&mut self, height: usize) -> Fp {
        self.with(|this| this.empty_hash_at_height(height))
    }

    pub fn invalidate_hashes(&mut self, account_index: AccountIndex) {
        self.with(|this| this.invalidate_hashes(account_index))
    }

    pub fn transfert_hashes(&mut self, hashes: HashesMatrix) {
        self.with(|this| this.transfert_hashes(hashes))
    }

    #[cfg(test)]
    pub fn test_matrix(&self) -> HashesMatrix {
        self.with(|this| this.hashes_matrix.clone())
        // match self {
        //     Root { database, .. } => database,
        //     Unattached { hashes, .. } | Attached { hashes, .. } => hashes.clone(),
        // }
    }
}

impl BaseLedger for Database<V2> {
    fn to_list(&self) -> Vec<Account> {
        self.with(|this| this.to_list())
    }

    fn iter<F>(&self, fun: F)
    where
        F: FnMut(&Account),
    {
        self.with(|this| this.iter(fun))
    }

    fn fold<B, F>(&self, init: B, fun: F) -> B
    where
        F: FnMut(B, &Account) -> B,
    {
        self.with(|this| this.fold(init, fun))
    }

    fn fold_with_ignored_accounts<B, F>(&self, ignoreds: HashSet<AccountId>, init: B, fun: F) -> B
    where
        F: FnMut(B, &Account) -> B,
    {
        self.with(|this| this.fold_with_ignored_accounts(ignoreds, init, fun))
    }

    fn fold_until<B, F>(&self, init: B, fun: F) -> B
    where
        F: FnMut(B, &Account) -> std::ops::ControlFlow<B, B>,
    {
        self.with(|this| this.fold_until(init, fun))
    }

    fn accounts(&self) -> HashSet<AccountId> {
        self.with(|this| this.accounts())
    }

    fn token_owner(&self, token_id: TokenId) -> Option<AccountId> {
        self.with(|this| this.token_owner(token_id))
    }

    fn token_owners(&self) -> HashSet<AccountId> {
        self.with(|this| this.token_owners())
    }

    fn tokens(&self, public_key: CompressedPubKey) -> HashSet<TokenId> {
        self.with(|this| this.tokens(public_key))
    }

    fn location_of_account(&self, account_id: &AccountId) -> Option<Address> {
        self.with(|this| this.location_of_account(account_id))
    }

    fn location_of_account_batch(
        &self,
        account_ids: &[AccountId],
    ) -> Vec<(AccountId, Option<Address>)> {
        self.with(|this| this.location_of_account_batch(account_ids))
    }

    fn get_or_create_account(
        &mut self,
        account_id: AccountId,
        account: Account,
    ) -> Result<GetOrCreated, DatabaseError> {
        self.with(|this| this.get_or_create_account(account_id, account))
    }

    fn close(&self) {
        // Drop self
    }

    fn last_filled(&self) -> Option<Address> {
        self.with(|this| this.last_filled())
    }

    fn get_uuid(&self) -> Uuid {
        self.with(|this| this.get_uuid())
    }

    fn get_directory(&self) -> Option<PathBuf> {
        self.with(|this| this.get_directory())
    }

    fn get_account_hash(&mut self, account_index: AccountIndex) -> Option<Fp> {
        self.with(|this| this.get_account_hash(account_index))
    }

    fn get(&self, addr: Address) -> Option<Box<Account>> {
        self.with(|this| this.get(addr))
    }

    fn get_batch(&self, addr: &[Address]) -> Vec<(Address, Option<Box<Account>>)> {
        self.with(|this| this.get_batch(addr))
    }

    fn set(&mut self, addr: Address, account: Box<Account>) {
        self.with(|this| this.set(addr, account))
    }

    fn set_batch(&mut self, list: &[(Address, Box<Account>)]) {
        self.with(|this| this.set_batch(list))
    }

    fn get_at_index(&self, index: AccountIndex) -> Option<Box<Account>> {
        self.with(|this| this.get_at_index(index))
    }

    fn set_at_index(&mut self, index: AccountIndex, account: Box<Account>) -> Result<(), ()> {
        self.with(|this| this.set_at_index(index, account))
    }

    fn index_of_account(&self, account_id: AccountId) -> Option<AccountIndex> {
        self.with(|this| this.index_of_account(account_id))
    }

    fn merkle_root(&mut self) -> Fp {
        self.with(|this| this.merkle_root())
    }

    fn merkle_path(&mut self, addr: Address) -> Vec<MerklePath> {
        self.with(|this| this.merkle_path(addr))
    }

    fn merkle_path_at_index(&mut self, index: AccountIndex) -> Vec<MerklePath> {
        self.with(|this| this.merkle_path_at_index(index))
    }

    fn remove_accounts(&mut self, ids: &[AccountId]) {
        self.with(|this| this.remove_accounts(ids))
    }

    fn detached_signal(&mut self) {
        self.with(|this| this.detached_signal())
    }

    fn depth(&self) -> u8 {
        self.with(|this| this.depth())
    }

    fn num_accounts(&self) -> usize {
        self.with(|this| this.num_accounts())
    }

    fn merkle_path_at_addr(&mut self, addr: Address) -> Vec<MerklePath> {
        self.with(|this| this.merkle_path_at_addr(addr))
    }

    fn get_inner_hash_at_addr(&mut self, addr: Address) -> Result<Fp, String> {
        self.with(|this| this.get_inner_hash_at_addr(addr))
    }

    fn set_inner_hash_at_addr(&mut self, addr: Address, hash: Fp) -> Result<(), ()> {
        self.with(|this| this.set_inner_hash_at_addr(addr, hash))
    }

    fn set_all_accounts_rooted_at(
        &mut self,
        addr: Address,
        accounts: &[Box<Account>],
    ) -> Result<(), ()> {
        self.with(|this| this.set_all_accounts_rooted_at(addr, accounts))
    }

    fn get_all_accounts_rooted_at(&self, addr: Address) -> Option<Vec<(Address, Box<Account>)>> {
        self.with(|this| this.get_all_accounts_rooted_at(addr))
    }

    fn make_space_for(&mut self, space: usize) {
        self.with(|this| this.make_space_for(space))
    }

    fn commit(&mut self) {
        // no-op
    }
}

#[cfg(test)]
mod tests {
    use ark_ff::One;
    use o1_utils::FieldHelpers;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use crate::tree_version::{account_empty_legacy_hash, V1};

    use super::*;

    // #[test]
    // fn test_legacy_db() {
    //     let two: usize = 2;

    //     for depth in 2..15 {
    //         let mut db = Database::<V1>::create(depth);

    //         for _ in 0..two.pow(depth as u32) {
    //             db.create_account((), AccountLegacy::create()).unwrap();
    //         }

    //         let naccounts = db.naccounts();
    //         assert_eq!(naccounts, two.pow(depth as u32));

    //         assert_eq!(
    //             db.create_account((), AccountLegacy::create()).unwrap_err(),
    //             DatabaseError::OutOfLeaves
    //         );

    //         elog!("depth={:?} naccounts={:?}", depth, naccounts);
    //     }
    // }

    #[test]
    fn test_matrix() {
        const DEPTH: usize = 4;

        let mut matrix = HashesMatrix::new(DEPTH);
        let one = Fp::one();

        for index in 0..16 {
            let account_index = AccountIndex::from(index);
            let addr = Address::from_index(account_index, DEPTH);
            matrix.set(&addr, one);

            elog!("{:?} MATRIX {:#?}", index + 1, matrix);
        }

        let addr = Address::root();

        matrix.set(&addr, one);
        elog!("{:?} MATRIX {:#?}", "root", matrix);

        matrix.set(&addr.child_left(), one);
        elog!("{:?} MATRIX {:#?}", "root", matrix);
        matrix.set(&addr.child_right(), one);
        elog!("{:?} MATRIX {:#?}", "root", matrix);

        matrix.set(&addr.child_left().child_left(), one);
        elog!("{:?} MATRIX {:#?}", "root", matrix);
        matrix.set(&addr.child_left().child_right(), one);
        elog!("{:?} MATRIX {:#?}", "root", matrix);
        matrix.set(&addr.child_right().child_left(), one);
        elog!("{:?} MATRIX {:#?}", "root", matrix);
        matrix.set(&addr.child_right().child_right(), one);
        elog!("{:?} MATRIX {:#?}", "root", matrix);
    }

    #[test]
    fn test_db_v2() {
        let two: usize = 2;

        for depth in 2..15 {
            let mut db = Database::<V2>::create(depth);

            for _ in 0..two.pow(depth as u32) {
                let account = Account::rand();
                let id = account.id();
                db.get_or_create_account(id, account).unwrap();
            }

            let naccounts = db.naccounts();
            assert_eq!(naccounts, two.pow(depth as u32));

            let account = Account::create();
            let id = account.id();
            assert_eq!(
                db.get_or_create_account(id, account).unwrap_err(),
                DatabaseError::OutOfLeaves
            );

            elog!("depth={:?} naccounts={:?}", depth, naccounts);
        }
    }

    // RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals" wasm-pack test --release --chrome -- -Z build-std=std,panic_abort -- hashing
    #[cfg(target_family = "wasm")]
    #[test]
    fn test_hashing_tree_with_web_workers() {
        use web_sys::console;

        use std::time::Duration;
        use wasm_thread as thread;

        use crate::account;

        let mut msg = format!("hello");

        const NACCOUNTS: u64 = 1_000;
        const NTHREADS: usize = 8;

        let mut accounts = (0..NACCOUNTS).map(|_| Account::rand()).collect::<Vec<_>>();

        use wasm_bindgen::prelude::*;

        fn perf_to_duration(amt: f64) -> std::time::Duration {
            let secs = (amt as u64) / 1_000;
            let nanos = (((amt as u64) % 1_000) as u32) * 1_000_000;
            std::time::Duration::new(secs, nanos)
        }

        #[cfg(target_arch = "wasm32")]
        #[wasm_bindgen(inline_js = r#"
export function performance_now() {
  return performance.now();
}"#)]
        extern "C" {
            fn performance_now() -> f64;
        }

        thread::spawn(move || {
            console::time_with_label("threads");
            console::log_1(&format!("hello from first thread {:?}", thread::current().id()).into());

            let start = performance_now();

            let mut joins = Vec::with_capacity(NTHREADS);

            for _ in 0..NTHREADS {
                let accounts = accounts.split_off(accounts.len() - (NACCOUNTS as usize / NTHREADS));

                let join = thread::spawn(move || {
                    console::log_1(
                        &format!("hello from thread {:?}", thread::current().id()).into(),
                    );

                    let hash = accounts.iter().map(|a| a.hash()).collect::<Vec<_>>();

                    console::log_1(
                        &format!("ending from thread {:?}", thread::current().id()).into(),
                    );

                    hash.len()
                });

                joins.push(join);
            }

            let nhashes: usize = joins.into_iter().map(|j| j.join().unwrap()).sum();

            assert_eq!(nhashes, NACCOUNTS as usize);

            let end = performance_now();

            console::log_1(
                &format!(
                    "nhashes={:?} nthreads={:?} time={:?}",
                    nhashes,
                    NTHREADS,
                    perf_to_duration(end - start)
                )
                .into(),
            );
            console::time_end_with_label("threads");
        });
    }

    #[cfg(target_family = "wasm")]
    #[test]
    fn test_hashing_tree() {
        use web_sys::console;

        const NACCOUNTS: u64 = 1_000;

        console::time_with_label("generate random accounts");

        let mut db = Database::<V2>::create(20);

        console::log_1(&format!("{:?} accounts in nodejs", NACCOUNTS).into());

        let accounts = (0..NACCOUNTS).map(|_| Account::rand()).collect::<Vec<_>>();

        for (index, mut account) in accounts.into_iter().enumerate() {
            account.token_id = TokenId::from(index as u64);
            let id = account.id();
            db.get_or_create_account(id, account).unwrap();
        }

        console::time_end_with_label("generate random accounts");
        assert_eq!(db.naccounts(), NACCOUNTS as usize);

        console::time_with_label("compute merkle root");
        db.merkle_root();

        console::time_end_with_label("compute merkle root");
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn test_hashing_tree() {
        const NACCOUNTS: u64 = 1_000;

        let now = std::time::Instant::now();
        let mut db = Database::<V2>::create(20);

        elog!("{:?} accounts natively", NACCOUNTS);

        let accounts = (0..NACCOUNTS).map(|_| Account::rand()).collect::<Vec<_>>();

        for (index, mut account) in accounts.into_iter().enumerate() {
            account.token_id = TokenId::from(index as u64);
            let id = account.id();
            db.get_or_create_account(id, account).unwrap();
        }

        elog!("generate random accounts {:?}", now.elapsed());
        assert_eq!(db.naccounts(), NACCOUNTS as usize);

        let now = std::time::Instant::now();
        db.merkle_root();
        elog!("compute merkle root {:?}", now.elapsed());
    }

    #[test]
    fn test_legacy_hash_empty() {
        let account_empty_hash = account_empty_legacy_hash();
        assert_eq!(
            account_empty_hash.to_hex(),
            "70ccdba14f829608e59a37ed98ffcaeef06dad928d568a9adbde13e3dd104a20"
        );

        for (height, s) in [
            (
                0,
                "70ccdba14f829608e59a37ed98ffcaeef06dad928d568a9adbde13e3dd104a20",
            ),
            (
                5,
                "4590712e4bd873ba93d01b665940e0edc48db1a7c90859948b7799f45a443b15",
            ),
            (
                10,
                "ba083b16b757794c81233d4ebf1ab000ba4a174a8174c1e8ee8bf0846ec2e10d",
            ),
            (
                11,
                "5d65e7d5f4c5441ac614769b913400aa3201f3bf9c0f33441dbf0a33a1239822",
            ),
            (
                100,
                "0e4ecb6104658cf8c06fca64f7f1cb3b0f1a830ab50c8c7ed9de544b8e6b2530",
            ),
            (
                2000,
                "b05105f8281f75efaf3c6b324563685c8be3a01b1c7d3f314ae733d869d95209",
            ),
        ] {
            let hash = V1::empty_hash_at_height(height);
            assert_eq!(hash.to_hex(), s, "invalid hash at depth={:?}", height);
        }
    }

    #[test]
    fn test_hash_empty() {
        let heights = [0, 5, 10, 11, 100, 2000];

        let hexs = [
            "f3ee39f42a7b2cac196c8eb1c9fe00f853678c920c0c9ce3724c0b7fe911c731",
            "d305ebed68f3d4ff16cfc9c6857c274bfbd4a5e83db6fb26e009f75711005524",
            "bfae9c6290bcc9cf282889c6b880e4eac236d4e50b22395639731e1465939915",
            "def7de4e2f2f13aa638f5671eccf9367ab9d39ae49ed683fca0169b33b31c416",
            "145494b371b318e94e1976e4b67768a6b1793841b47a6269996354e93d5c603c",
            "00fb3327a47dc00f9b04e5c5c59af2851501cb9e602a03908f1ec7b04453540c",
        ];

        let result: Vec<_> = heights
            .iter()
            .map(|height| V2::empty_hash_at_height(*height).to_hex())
            .collect();

        assert_eq!(result, hexs);

        let account_empty_hash = Account::empty().hash();
        assert_eq!(account_empty_hash.to_hex(), hexs[0]);
    }

    // /// An empty tree produces the same hash than a tree full of empty accounts
    // #[test]
    // fn test_root_hash_v2() {
    //     let mut db = Database::<V2>::create(4);
    //     for _ in 0..16 {
    //         db.create_account((), Account::empty()).unwrap();
    //     }
    //     assert_eq!(
    //         db.create_account((), Account::empty()).unwrap_err(),
    //         DatabaseError::OutOfLeaves
    //     );
    //     let hash = db.root_hash();
    //     elog!("ROOT_HASH={:?}", hash.to_string());
    //     assert_eq!(
    //         hash.to_hex(),
    //         "169bada2f4bb2ea2b8189f47cf2b665e3e0fb135233242ae1b52794eb3fe7924"
    //     );

    //     let mut db = Database::<V2>::create(4);
    //     for _ in 0..1 {
    //         db.create_account((), Account::empty()).unwrap();
    //     }
    //     let hash = db.root_hash();
    //     assert_eq!(
    //         hash.to_hex(),
    //         "169bada2f4bb2ea2b8189f47cf2b665e3e0fb135233242ae1b52794eb3fe7924"
    //     );

    //     let db = Database::<V2>::create(4);
    //     let hash = db.root_hash();
    //     assert_eq!(
    //         hash.to_hex(),
    //         "169bada2f4bb2ea2b8189f47cf2b665e3e0fb135233242ae1b52794eb3fe7924"
    //     );
    // }

    /// Accounts inserted in a different order produce different root hash
    #[test]
    fn test_root_hash_different_orders() {
        let mut db = Database::<V2>::create(4);

        let accounts = (0..16).map(|_| Account::rand()).collect::<Vec<_>>();

        for account in &accounts {
            db.get_or_create_account(account.id(), account.clone())
                .unwrap();
        }
        let root_hash_1 = db.merkle_root();

        let mut db = Database::<V2>::create(4);
        for account in accounts.iter().rev() {
            db.get_or_create_account(account.id(), account.clone())
                .unwrap();
        }
        let root_hash_2 = db.merkle_root();

        // Different orders, different root hash
        assert_ne!(root_hash_1, root_hash_2);

        let mut db = Database::<V2>::create(4);
        for account in accounts {
            db.get_or_create_account(account.id(), account).unwrap();
        }
        let root_hash_3 = db.merkle_root();

        // Same orders, same root hash
        assert_eq!(root_hash_1, root_hash_3);
    }

    // /// An empty tree produces the same hash than a tree full of empty accounts
    // #[test]
    // fn test_root_hash_legacy() {
    //     let mut db = Database::<V1>::create(4);
    //     for _ in 0..16 {
    //         db.create_account((), AccountLegacy::empty()).unwrap();
    //     }
    //     assert_eq!(
    //         db.create_account((), AccountLegacy::empty()).unwrap_err(),
    //         DatabaseError::OutOfLeaves
    //     );
    //     let hash = db.root_hash();
    //     assert_eq!(
    //         hash.to_hex(),
    //         "2db7d27130b6fe46b95541a70bc69ac51d9ea02825f7a7ab41ec4c414989421e"
    //     );

    //     let mut db = Database::<V1>::create(4);
    //     for _ in 0..1 {
    //         db.create_account((), AccountLegacy::empty()).unwrap();
    //     }
    //     let hash = db.root_hash();
    //     assert_eq!(
    //         hash.to_hex(),
    //         "2db7d27130b6fe46b95541a70bc69ac51d9ea02825f7a7ab41ec4c414989421e"
    //     );

    //     let db = Database::<V1>::create(4);
    //     let hash = db.root_hash();
    //     assert_eq!(
    //         hash.to_hex(),
    //         "2db7d27130b6fe46b95541a70bc69ac51d9ea02825f7a7ab41ec4c414989421e"
    //     );
    // }
}

#[cfg(test)]
mod tests_ocaml {
    use std::ops::ControlFlow;

    use o1_utils::FieldHelpers;
    use rand::Rng;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use crate::scan_state::currency::Balance;

    use super::*;

    // "add and retrieve an account"
    #[test]
    fn test_add_retrieve_account() {
        let mut db = Database::<V2>::create(4);

        let account = Account::rand();
        let location = db
            .get_or_create_account(account.id(), account.clone())
            .unwrap();
        let get_account = db.get(location.addr()).unwrap();

        assert_eq!(account, *get_account);
    }

    // "accounts are atomic"
    #[test]
    fn test_accounts_are_atomic() {
        let mut db = Database::<V2>::create(4);

        let account = Box::new(Account::rand());
        let location: Address = db
            .get_or_create_account(account.id(), *account.clone())
            .unwrap()
            .addr();

        db.set(location.clone(), account.clone());
        let loc = db.location_of_account(&account.id()).unwrap();

        assert_eq!(location, loc);
        assert_eq!(db.get(location), db.get(loc));
    }

    // "length"
    #[test]
    fn test_lengths() {
        for naccounts in 50..100 {
            let mut db = Database::<V2>::create(10);
            let mut unique = HashSet::with_capacity(naccounts);

            for _ in 0..naccounts {
                let account = loop {
                    let account = Account::rand();
                    if unique.insert(account.id()) {
                        break account;
                    }
                };

                db.get_or_create_account(account.id(), account).unwrap();
            }

            assert_eq!(db.num_accounts(), naccounts);
        }
    }

    // "get_or_create_acount does not update an account if key already""
    #[test]
    fn test_no_update_if_exist() {
        let mut db = Database::<V2>::create(10);

        let mut account1 = Account::rand();
        account1.balance = Balance::from_u64(100);

        let location1 = db
            .get_or_create_account(account1.id(), account1.clone())
            .unwrap();

        let mut account2 = account1;
        account2.balance = Balance::from_u64(200);

        let location2 = db
            .get_or_create_account(account2.id(), account2.clone())
            .unwrap();

        let addr1: Address = location1.clone().addr();
        let addr2: Address = location2.clone().addr();

        assert_eq!(addr1, addr2);
        assert!(matches!(location2, GetOrCreated::Existed(_)));
        assert_ne!(*db.get(location1.addr()).unwrap(), account2);
    }

    // "get_or_create_account t account = location_of_account account.key"
    #[test]
    fn test_location_of_account() {
        for naccounts in 50..100 {
            let mut db = Database::<V2>::create(10);

            for _ in 0..naccounts {
                let account = Account::rand();

                let account_id = account.id();
                let location = db
                    .get_or_create_account(account_id.clone(), account)
                    .unwrap();
                let addr: Address = location.addr();

                assert_eq!(addr, db.location_of_account(&account_id).unwrap());
            }
        }
    }

    // "set_inner_hash_at_addr_exn(address,hash);
    //  get_inner_hash_at_addr_exn(address) = hash"
    #[test]
    fn test_set_inner_hash() {
        // TODO
    }

    fn create_full_db(depth: usize) -> Database<V2> {
        let mut db = Database::<V2>::create(depth as u8);

        for _ in 0..2u64.pow(depth as u32) {
            let account = Account::rand();
            db.get_or_create_account(account.id(), account).unwrap();
        }

        db
    }

    // "set_inner_hash_at_addr_exn(address,hash);
    //  get_inner_hash_at_addr_exn(address) = hash"
    #[test]
    fn test_get_set_all_same_root_hash() {
        let mut db = create_full_db(7);

        let merkle_root1 = db.merkle_root();
        let root = Address::root();

        let accounts = db.get_all_accounts_rooted_at(root.clone()).unwrap();
        let accounts = accounts.into_iter().map(|acc| acc.1).collect::<Vec<_>>();
        db.set_all_accounts_rooted_at(root, &accounts).unwrap();

        let merkle_root2 = db.merkle_root();

        assert_eq!(merkle_root1, merkle_root2);
    }

    // "set_inner_hash_at_addr_exn(address,hash);
    //  get_inner_hash_at_addr_exn(address) = hash"
    #[test]
    fn test_set_batch_accounts_change_root_hash() {
        const DEPTH: usize = 7;

        for _ in 0..5 {
            let mut db = create_full_db(DEPTH);

            let addr = Address::rand_nonleaf(DEPTH);
            let children = addr.iter_children(DEPTH);
            let accounts = children
                .map(|addr| (addr, Box::new(Account::rand())))
                .collect::<Vec<_>>();

            let merkle_root1 = db.merkle_root();
            elog!("naccounts={:?}", accounts.len());
            db.set_batch_accounts(&accounts);
            let merkle_root2 = db.merkle_root();

            assert_ne!(merkle_root1, merkle_root2);
        }
    }

    // "We can retrieve accounts by their by key after using
    //  set_batch_accounts""
    #[test]
    fn test_retrieve_account_after_set_batch() {
        const DEPTH: usize = 7;

        let mut db = Database::<V2>::create(DEPTH as u8);

        let mut addr = Address::root();
        for _ in 0..63 {
            let account = Account::rand();
            addr = db
                .get_or_create_account(account.id(), account)
                .unwrap()
                .addr();
        }

        let last_location = db.last_filled().unwrap();
        assert_eq!(addr, last_location);

        let mut accounts = Vec::with_capacity(2u64.pow(DEPTH as u32) as usize);

        while let Some(next_addr) = addr.next() {
            accounts.push((next_addr.clone(), Box::new(Account::rand())));
            addr = next_addr;
        }

        db.set_batch_accounts(&accounts);

        for (addr, account) in &accounts {
            let account_id = account.id();
            let location = db.location_of_account(&account_id).unwrap();
            let queried_account = db.get(location.clone()).unwrap();

            assert_eq!(*addr, location);
            assert_eq!(account, &queried_account);
        }

        let expected_last_location = last_location.to_index().0 + accounts.len() as u64;
        let actual_last_location = db.last_filled().unwrap().to_index().0;

        assert_eq!(expected_last_location, actual_last_location);
    }

    // "If the entire database is full,
    //  set_all_accounts_rooted_at_exn(address,accounts);get_all_accounts_rooted_at_exn(address)
    //  = accounts"
    #[test]
    fn test_set_accounts_rooted_equal_get_accounts_rooted() {
        const DEPTH: usize = 7;

        let mut db = create_full_db(DEPTH);

        for _ in 0..5 {
            let addr = Address::rand_nonleaf(DEPTH);
            let children = addr.iter_children(DEPTH);
            let accounts = children
                .map(|_| Box::new(Account::rand()))
                .collect::<Vec<_>>();

            db.set_all_accounts_rooted_at(addr.clone(), &accounts)
                .unwrap();
            let list = db
                .get_all_accounts_rooted_at(addr)
                .unwrap()
                .into_iter()
                .map(|(_, acc)| acc)
                .collect::<Vec<_>>();

            assert!(!accounts.is_empty());
            assert_eq!(accounts, list);
        }
    }

    // "create_empty doesn't modify the hash"
    #[test]
    fn test_create_empty_doesnt_modify_hash() {
        const DEPTH: usize = 7;

        let mut db = Database::<V2>::create(DEPTH as u8);

        let start_hash = db.merkle_root();

        let account = Account::empty();
        assert!(matches!(
            db.get_or_create_account(account.id(), account).unwrap(),
            GetOrCreated::Added(_)
        ));

        assert_eq!(start_hash, db.merkle_root());
    }

    // "get_at_index_exn t (index_of_account_exn t public_key) =
    // account"
    #[test]
    fn test_get_indexed() {
        const DEPTH: usize = 7;
        const NACCOUNTS: usize = 2u64.pow(DEPTH as u32) as usize;

        let mut db = Database::<V2>::create(DEPTH as u8);
        let mut accounts = Vec::with_capacity(NACCOUNTS);

        for _ in 0..NACCOUNTS {
            let account = Account::rand();
            accounts.push(account.clone());
            db.get_or_create_account(account.id(), account).unwrap();
        }

        for account in accounts {
            let account_id = account.id();
            let index_of_account = db.index_of_account(account_id).unwrap();
            let indexed_account = db.get_at_index(index_of_account).unwrap();
            assert_eq!(account, *indexed_account);
        }
    }

    // "set_at_index_exn t index  account; get_at_index_exn t
    // index = account"
    #[test]
    fn test_set_get_indexed_equal() {
        const DEPTH: usize = 7;
        const NACCOUNTS: usize = 2u64.pow(DEPTH as u32) as usize;

        let mut db = create_full_db(DEPTH);

        for _ in 0..50 {
            let account = Box::new(Account::rand());
            let index = rand::thread_rng().gen_range(0..NACCOUNTS);
            let index = AccountIndex(index as u64);

            db.set_at_index(index, account.clone()).unwrap();
            let at_index = db.get_at_index(index).unwrap();
            assert_eq!(account, at_index);
        }
    }

    // "iter"
    #[test]
    fn test_iter() {
        const DEPTH: usize = 7;
        const NACCOUNTS: usize = 2u64.pow(DEPTH as u32) as usize;

        let mut db = Database::<V2>::create(DEPTH as u8);
        let mut accounts = Vec::with_capacity(NACCOUNTS);

        for _ in 0..NACCOUNTS {
            let account = Account::rand();
            accounts.push(account.clone());
            db.get_or_create_account(account.id(), account).unwrap();
        }

        assert_eq!(accounts, db.to_list(),)
    }

    // "Add 2^d accounts (for testing, d is small)"
    #[test]
    fn test_retrieve() {
        const DEPTH: usize = 7;
        const NACCOUNTS: usize = 2u64.pow(DEPTH as u32) as usize;

        let mut db = Database::<V2>::create(DEPTH as u8);
        let mut accounts = Vec::with_capacity(NACCOUNTS);

        for _ in 0..NACCOUNTS {
            let account = Box::new(Account::rand());
            accounts.push(account.clone());
            db.get_or_create_account(account.id(), *account).unwrap();
        }

        let retrieved = db
            .get_all_accounts_rooted_at(Address::root())
            .unwrap()
            .into_iter()
            .map(|(_, acc)| acc)
            .collect::<Vec<_>>();

        assert_eq!(accounts, retrieved);
    }

    // "removing accounts restores Merkle root"
    #[test]
    fn test_remove_restore_root_hash() {
        const DEPTH: usize = 7;
        const NACCOUNTS: usize = 2u64.pow(DEPTH as u32) as usize;

        let mut db = Database::<V2>::create(DEPTH as u8);

        let root_hash = db.merkle_root();

        let mut accounts = Vec::with_capacity(NACCOUNTS);

        for _ in 0..NACCOUNTS {
            let account = Account::rand();
            accounts.push(account.id());
            db.get_or_create_account(account.id(), account).unwrap();
        }
        assert_ne!(root_hash, db.merkle_root());

        db.remove_accounts(&accounts);
        assert_eq!(root_hash, db.merkle_root());
    }

    // "fold over account balances"
    #[test]
    fn test_fold_over_account_balance() {
        const DEPTH: usize = 7;
        const NACCOUNTS: usize = 2u64.pow(DEPTH as u32) as usize;

        let mut db = Database::<V2>::create(DEPTH as u8);
        let mut total_balance: u128 = 0;

        for _ in 0..NACCOUNTS {
            let account = Account::rand();
            total_balance += account.balance.as_u64() as u128;
            db.get_or_create_account(account.id(), account).unwrap();
        }

        let retrieved = db.fold(0u128, |acc, account| acc + account.balance.as_u64() as u128);
        assert_eq!(total_balance, retrieved);
    }

    // "fold_until over account balances"
    #[test]
    fn test_fold_until_over_account_balance() {
        const DEPTH: usize = 7;
        const NACCOUNTS: usize = 2u64.pow(DEPTH as u32) as usize;

        let mut db = Database::<V2>::create(DEPTH as u8);
        let mut total_balance: u128 = 0;
        let mut last_id: AccountId = Account::empty().id();

        for i in 0..NACCOUNTS {
            let account = Account::rand();
            if i <= 30 {
                total_balance += account.balance.as_u64() as u128;
                last_id = account.id();
            }
            db.get_or_create_account(account.id(), account).unwrap();
        }

        let retrieved = db.fold_until(0u128, |mut acc, account| {
            acc += account.balance.as_u64() as u128;

            if account.id() != last_id {
                ControlFlow::Continue(acc)
            } else {
                ControlFlow::Break(acc)
            }
        });

        assert_eq!(total_balance, retrieved);
    }

    #[test]
    fn test_merkle_path_long() {
        const DEPTH: usize = 4;
        const NACCOUNTS: usize = 2u64.pow(DEPTH as u32) as usize;

        let mut db = Database::<V2>::create(DEPTH as u8);

        for index in 0..NACCOUNTS / 2 {
            let mut account = Account::empty();
            account.token_id = TokenId::from(index as u64);

            // elog!("account{}={}", index, account.hash().to_hex());

            let res = db.get_or_create_account(account.id(), account).unwrap();
            assert!(matches!(res, GetOrCreated::Added(_)));
        }

        elog!("naccounts={:?}", db.last_filled());

        let expected = [
            &[
                "f3ee39f42a7b2cac196c8eb1c9fe00f853678c920c0c9ce3724c0b7fe911c731",
                "19c428e06065374dcdbb9c2773d59c14a4bc5322b821b616f5ad8b95e3fce83c",
                "277bb79d5e2b48e92cf9e3cf169eccf199e66b1b611765bdd190ad914d50f138",
                "63f92c64075ae8ef4d076fc7d6743e758513e7b958a4d7a6cb6744eda019d019",
            ][..],
            &[
                "ea3278ff7f5c0472163846755a94643f9dea2babe2d25848294fd727d83a6630",
                "19c428e06065374dcdbb9c2773d59c14a4bc5322b821b616f5ad8b95e3fce83c",
                "277bb79d5e2b48e92cf9e3cf169eccf199e66b1b611765bdd190ad914d50f138",
                "63f92c64075ae8ef4d076fc7d6743e758513e7b958a4d7a6cb6744eda019d019",
            ][..],
            &[
                "9c461cf909421302a96f3794f198bae68d14eb542cfdae3c5942d61211cdbc3c",
                "c9c043aa20b69f061a06bf27e19e6c7a7c2cb94a5022a614ebfb9209cda11527",
                "277bb79d5e2b48e92cf9e3cf169eccf199e66b1b611765bdd190ad914d50f138",
                "63f92c64075ae8ef4d076fc7d6743e758513e7b958a4d7a6cb6744eda019d019",
            ][..],
            &[
                "225381a665a6c436f0750ce907b8077ae3670b17af0acee4a3eb98569692dd10",
                "c9c043aa20b69f061a06bf27e19e6c7a7c2cb94a5022a614ebfb9209cda11527",
                "277bb79d5e2b48e92cf9e3cf169eccf199e66b1b611765bdd190ad914d50f138",
                "63f92c64075ae8ef4d076fc7d6743e758513e7b958a4d7a6cb6744eda019d019",
            ][..],
            &[
                "6224d72dc65c9b89d5717cb43b9de5a31763192dbe316f46bb0ed2486c46d50a",
                "16b9cac5d2aa4e87c46520d88df96ddb6da7e2f52e23ec406fce9a06d3654f10",
                "a4c190a90cddf828af5c87b2b9278da29fdf46bab27642c553f281c4cb25ca1f",
                "63f92c64075ae8ef4d076fc7d6743e758513e7b958a4d7a6cb6744eda019d019",
            ][..],
            &[
                "1b1edf8a2bc43639f2dd7fbffc953a5887ca8e2f57afe2474b293e00a0c8fd13",
                "16b9cac5d2aa4e87c46520d88df96ddb6da7e2f52e23ec406fce9a06d3654f10",
                "a4c190a90cddf828af5c87b2b9278da29fdf46bab27642c553f281c4cb25ca1f",
                "63f92c64075ae8ef4d076fc7d6743e758513e7b958a4d7a6cb6744eda019d019",
            ][..],
            &[
                "bd649e2c20b743b9ecc6235f06016b4aa1263ee7cd6af77afb5358c60ca9143e",
                "2580091fe2ab125a78a5d8df8c017d1a0f588b871f50a2a34cd383df3d647503",
                "a4c190a90cddf828af5c87b2b9278da29fdf46bab27642c553f281c4cb25ca1f",
                "63f92c64075ae8ef4d076fc7d6743e758513e7b958a4d7a6cb6744eda019d019",
            ][..],
            &[
                "2834b7e93942f095625b96ce9b6b709934bbdd35bc5762f6493db73dd84b242c",
                "2580091fe2ab125a78a5d8df8c017d1a0f588b871f50a2a34cd383df3d647503",
                "a4c190a90cddf828af5c87b2b9278da29fdf46bab27642c553f281c4cb25ca1f",
                "63f92c64075ae8ef4d076fc7d6743e758513e7b958a4d7a6cb6744eda019d019",
            ][..],
            &[
                "f3ee39f42a7b2cac196c8eb1c9fe00f853678c920c0c9ce3724c0b7fe911c731",
                "64ea581ca7a7aef2b78d04e4a64d0df1da1aa5c2d32a2a34c3c8d50a2c932f07",
                "358495ff9f624ba2ea37d0890b8c14079fb16e3626bed5ff0447ee10ecf5f30f",
                "c1da0c74de92bf4c1ae893b0f70524a9c6ff4a7ad754e0fc64f5e00548b4cd3d",
            ][..],
            &[
                "f3ee39f42a7b2cac196c8eb1c9fe00f853678c920c0c9ce3724c0b7fe911c731",
                "64ea581ca7a7aef2b78d04e4a64d0df1da1aa5c2d32a2a34c3c8d50a2c932f07",
                "358495ff9f624ba2ea37d0890b8c14079fb16e3626bed5ff0447ee10ecf5f30f",
                "c1da0c74de92bf4c1ae893b0f70524a9c6ff4a7ad754e0fc64f5e00548b4cd3d",
            ][..],
            &[
                "f3ee39f42a7b2cac196c8eb1c9fe00f853678c920c0c9ce3724c0b7fe911c731",
                "64ea581ca7a7aef2b78d04e4a64d0df1da1aa5c2d32a2a34c3c8d50a2c932f07",
                "358495ff9f624ba2ea37d0890b8c14079fb16e3626bed5ff0447ee10ecf5f30f",
                "c1da0c74de92bf4c1ae893b0f70524a9c6ff4a7ad754e0fc64f5e00548b4cd3d",
            ][..],
            &[
                "f3ee39f42a7b2cac196c8eb1c9fe00f853678c920c0c9ce3724c0b7fe911c731",
                "64ea581ca7a7aef2b78d04e4a64d0df1da1aa5c2d32a2a34c3c8d50a2c932f07",
                "358495ff9f624ba2ea37d0890b8c14079fb16e3626bed5ff0447ee10ecf5f30f",
                "c1da0c74de92bf4c1ae893b0f70524a9c6ff4a7ad754e0fc64f5e00548b4cd3d",
            ][..],
            &[
                "f3ee39f42a7b2cac196c8eb1c9fe00f853678c920c0c9ce3724c0b7fe911c731",
                "64ea581ca7a7aef2b78d04e4a64d0df1da1aa5c2d32a2a34c3c8d50a2c932f07",
                "358495ff9f624ba2ea37d0890b8c14079fb16e3626bed5ff0447ee10ecf5f30f",
                "c1da0c74de92bf4c1ae893b0f70524a9c6ff4a7ad754e0fc64f5e00548b4cd3d",
            ][..],
            &[
                "f3ee39f42a7b2cac196c8eb1c9fe00f853678c920c0c9ce3724c0b7fe911c731",
                "64ea581ca7a7aef2b78d04e4a64d0df1da1aa5c2d32a2a34c3c8d50a2c932f07",
                "358495ff9f624ba2ea37d0890b8c14079fb16e3626bed5ff0447ee10ecf5f30f",
                "c1da0c74de92bf4c1ae893b0f70524a9c6ff4a7ad754e0fc64f5e00548b4cd3d",
            ][..],
            &[
                "f3ee39f42a7b2cac196c8eb1c9fe00f853678c920c0c9ce3724c0b7fe911c731",
                "64ea581ca7a7aef2b78d04e4a64d0df1da1aa5c2d32a2a34c3c8d50a2c932f07",
                "358495ff9f624ba2ea37d0890b8c14079fb16e3626bed5ff0447ee10ecf5f30f",
                "c1da0c74de92bf4c1ae893b0f70524a9c6ff4a7ad754e0fc64f5e00548b4cd3d",
            ][..],
            &[
                "f3ee39f42a7b2cac196c8eb1c9fe00f853678c920c0c9ce3724c0b7fe911c731",
                "64ea581ca7a7aef2b78d04e4a64d0df1da1aa5c2d32a2a34c3c8d50a2c932f07",
                "358495ff9f624ba2ea37d0890b8c14079fb16e3626bed5ff0447ee10ecf5f30f",
                "c1da0c74de92bf4c1ae893b0f70524a9c6ff4a7ad754e0fc64f5e00548b4cd3d",
            ][..],
        ];

        let mut hashes = Vec::with_capacity(100);

        let root = Address::root();
        let nchild = root.iter_children(DEPTH);

        for child in nchild {
            let path = db.merkle_path(child);
            let path = path.iter().map(|p| p.hash().to_hex()).collect::<Vec<_>>();
            hashes.push(path);
        }

        // elog!("expected={:#?}", expected);
        // elog!("computed={:#?}", hashes);

        assert_eq!(&expected[..], hashes.as_slice());
    }

    // "fold_until over account balances"
    #[test]
    fn test_merkle_path_test2() {
        const DEPTH: usize = 20;
        const NACCOUNTS: usize = 2u64.pow(DEPTH as u32) as usize;

        let mut db = Database::<V2>::create(DEPTH as u8);
        db.merkle_path(Address::first(20));
    }

    // "fold_until over account balances"
    // #[test]
    fn test_merkle_path_test() {
        const DEPTH: usize = 4;
        const NACCOUNTS: usize = 2u64.pow(DEPTH as u32) as usize;

        elog!("empty={}", Account::empty().hash());
        elog!("height1={}", V2::empty_hash_at_height(1));
        elog!("height2={}", V2::empty_hash_at_height(2));
        elog!("height3={}", V2::empty_hash_at_height(3));
        elog!("height4={}", V2::empty_hash_at_height(4));

        // let db = Database::<V2>::create(DEPTH as u8);
        // db.merkle_root();
        // db.merkle_path(Address::first(DEPTH));

        // elog!("WITH_ACC");

        // let mut db = Database::<V2>::create(DEPTH as u8);
        // let mut account = Account::empty();
        // account.token_symbol = "seb".to_string();
        // db.get_or_create_account(account.id(), account).unwrap();
        // db.merkle_root();

        let mut db = Database::<V2>::create(DEPTH as u8);

        // for _ in 0..NACCOUNTS {
        //     let account = Account::rand();
        //     db.get_or_create_account(account.id(), account).unwrap();
        // }

        db.merkle_root();

        db.merkle_path(Address::first(DEPTH));

        // elog!(
        //     "INNER_AT_0={}",
        //     db.get_inner_hash_at_addr(Address::try_from("0000").unwrap())
        //         .unwrap()
        // );
        // elog!(
        //     "INNER_AT_0={}",
        //     db.get_inner_hash_at_addr(Address::try_from("0001").unwrap())
        //         .unwrap()
        // );
        // elog!(
        //     "INNER_AT_0={}",
        //     db.get_inner_hash_at_addr(Address::try_from("0010").unwrap())
        //         .unwrap()
        // );
        // elog!(
        //     "INNER_AT_0={}",
        //     db.get_inner_hash_at_addr(Address::try_from("0101").unwrap())
        //         .unwrap()
        // );

        // elog!("A");
        // elog!(
        //     "INNER_AT_3={}",
        //     db.get_inner_hash_at_addr(Address::try_from("000").unwrap())
        //         .unwrap()
        // );
        // elog!(
        //     "INNER_AT_3={}",
        //     db.get_inner_hash_at_addr(Address::try_from("001").unwrap())
        //         .unwrap()
        // );
        // elog!(
        //     "INNER_AT_3={}",
        //     db.get_inner_hash_at_addr(Address::try_from("010").unwrap())
        //         .unwrap()
        // );
        // elog!(
        //     "INNER_AT_3={}",
        //     db.get_inner_hash_at_addr(Address::try_from("101").unwrap())
        //         .unwrap()
        // );

        // elog!("A");
        // elog!(
        //     "INNER_AT_2={}",
        //     db.get_inner_hash_at_addr(Address::try_from("10").unwrap())
        //         .unwrap()
        // );
        // elog!(
        //     "INNER_AT_2={}",
        //     db.get_inner_hash_at_addr(Address::try_from("01").unwrap())
        //         .unwrap()
        // );

        // elog!("A");
        // elog!(
        //     "INNER_AT_1={}",
        //     db.get_inner_hash_at_addr(Address::try_from("1").unwrap())
        //         .unwrap()
        // );
        // elog!(
        //     "INNER_AT_1={}",
        //     db.get_inner_hash_at_addr(Address::try_from("0").unwrap())
        //         .unwrap()
        // );

        // elog!("A");
        // elog!(
        //     "INNER_AT_0={}",
        //     db.get_inner_hash_at_addr(Address::try_from("").unwrap())
        //         .unwrap()
        // );
    }
}
