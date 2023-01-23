use std::{
    collections::HashSet,
    ops::{ControlFlow, Deref},
    path::PathBuf,
    sync::atomic::AtomicU64,
};

use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::{
    account::{Account, AccountId, TokenId},
    address::Address,
    database::DatabaseError,
    scan_state::transaction_logic::AccountState,
    staged_ledger::sparse_ledger::LedgerIntf,
};

pub type Uuid = String;

static UUID_GENERATOR: AtomicU64 = AtomicU64::new(0);

pub fn next_uuid() -> Uuid {
    // use uuid::Uuid;

    uuid::Uuid::new_v4().to_string()

    // "a".to_string()
    // UUID_GENERATOR.fetch_add(1, Ordering::AcqRel)
}

#[derive(PartialEq, Eq)]
pub enum MerklePath {
    Left(Fp),
    Right(Fp),
}

impl MerklePath {
    pub fn hash(&self) -> &Fp {
        match self {
            MerklePath::Left(h) => h,
            MerklePath::Right(h) => h,
        }
    }
}

impl std::fmt::Debug for MerklePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left(arg0) => f.debug_tuple("Left").field(&arg0.to_string()).finish(),
            Self::Right(arg0) => f.debug_tuple("Right").field(&arg0.to_string()).finish(),
        }
    }
}

pub trait BaseLedger {
    /// list of accounts in the ledger
    fn to_list(&self) -> Vec<Account>;

    /// iterate over all indexes and accounts
    fn iter<F>(&self, fun: F)
    where
        F: FnMut(&Account);

    /// fold over accounts in the ledger, passing the Merkle address
    fn fold<B, F>(&self, init: B, fun: F) -> B
    where
        F: FnMut(B, &Account) -> B;

    /// the set of [account_id]s are ledger elements to skip during the fold,
    /// because they're in a mask
    fn fold_with_ignored_accounts<B, F>(&self, ignoreds: HashSet<AccountId>, init: B, fun: F) -> B
    where
        F: FnMut(B, &Account) -> B;

    /// fold until `fun` returns `ControlFlow::Stop`
    fn fold_until<B, F>(&self, init: B, fun: F) -> B
    where
        F: FnMut(B, &Account) -> ControlFlow<B, B>;

    /// set of account ids associated with accounts
    fn accounts(&self) -> HashSet<AccountId>;

    /// Get the account id that owns a token.
    fn token_owner(&self, token_id: TokenId) -> Option<AccountId>;

    /// Get the set of all accounts which own a token.
    fn token_owners(&self) -> HashSet<AccountId>;

    /// Get all of the tokens for which a public key has accounts.
    fn tokens(&self, public_key: CompressedPubKey) -> HashSet<TokenId>;

    fn location_of_account(&self, account_id: &AccountId) -> Option<Address>;

    fn location_of_account_batch(
        &self,
        account_ids: &[AccountId],
    ) -> Vec<(AccountId, Option<Address>)>;

    /// This may return an error if the ledger is full.
    fn get_or_create_account(
        &mut self,
        account_id: AccountId,
        account: Account,
    ) -> Result<GetOrCreated, DatabaseError>;

    /// the ledger should not be used after calling [close]
    fn close(&self);

    /// for account locations in the ledger, the last (rightmost) filled location
    fn last_filled(&self) -> Option<Address>;

    fn get_uuid(&self) -> Uuid;

    /// return Some [directory] for ledgers that use a file system, else None
    fn get_directory(&self) -> Option<PathBuf>;

    fn get(&self, addr: Address) -> Option<Account>;

    fn get_batch(&self, addr: &[Address]) -> Vec<(Address, Option<Account>)>;

    fn set(&mut self, addr: Address, account: Account);

    fn set_batch(&mut self, list: &[(Address, Account)]);

    fn get_at_index(&self, index: AccountIndex) -> Option<Account>;

    fn set_at_index(&mut self, index: AccountIndex, account: Account) -> Result<(), ()>;

    fn index_of_account(&self, account_id: AccountId) -> Option<AccountIndex>;

    /// meant to be a fast operation: the root hash is stored, rather
    /// than calculated dynamically
    fn merkle_root(&mut self) -> Fp;

    fn merkle_path(&mut self, addr: Address) -> Vec<MerklePath>;

    fn merkle_path_at_index(&mut self, index: AccountIndex) -> Vec<MerklePath>;

    fn remove_accounts(&mut self, ids: &[AccountId]);

    /// Triggers when the ledger has been detached and should no longer be
    /// accessed.
    fn detached_signal(&mut self);

    // Following methods from Syncable_intf

    fn depth(&self) -> u8;

    fn num_accounts(&self) -> usize;

    fn merkle_path_at_addr(&mut self, addr: Address) -> Vec<MerklePath>;

    fn get_inner_hash_at_addr(&mut self, addr: Address) -> Result<Fp, ()>;

    fn set_inner_hash_at_addr(&mut self, addr: Address, hash: Fp) -> Result<(), ()>;

    fn set_all_accounts_rooted_at(&mut self, addr: Address, accounts: &[Account])
        -> Result<(), ()>;

    fn set_batch_accounts(&mut self, list: &[(Address, Account)]) {
        Self::set_batch(self, list)
    }

    /// Get all of the accounts that are in a subtree of the underlying Merkle
    /// tree rooted at `address`. The accounts are ordered by their addresses.
    fn get_all_accounts_rooted_at(&self, addr: Address) -> Option<Vec<(Address, Account)>>;

    fn make_space_for(&mut self, space: usize);

    // Following are internal methods, they might be better in a private trait
    fn get_account_hash(&mut self, account_index: AccountIndex) -> Option<Fp>;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountIndex(pub u64);

impl AccountIndex {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl PartialEq<u64> for AccountIndex {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl PartialEq<usize> for AccountIndex {
    fn eq(&self, other: &usize) -> bool {
        let other: u64 = (*other).try_into().unwrap();
        self.0 == other
    }
}

impl From<usize> for AccountIndex {
    fn from(n: usize) -> Self {
        Self(n.try_into().unwrap())
    }
}

#[derive(Clone, Debug)]
pub enum GetOrCreated {
    Added(Address),
    Existed(Address),
}

impl GetOrCreated {
    pub fn addr(self) -> Address {
        match self {
            GetOrCreated::Added(addr) => addr,
            GetOrCreated::Existed(addr) => addr,
        }
    }
}

impl Deref for GetOrCreated {
    type Target = Address;

    fn deref(&self) -> &Self::Target {
        match self {
            GetOrCreated::Added(addr) => addr,
            GetOrCreated::Existed(addr) => addr,
        }
    }
}

impl<T> LedgerIntf for T
where
    T: BaseLedger,
{
    type Location = Address;

    fn get(&self, addr: &Address) -> Option<Account> {
        BaseLedger::get(self, addr.clone())
    }

    fn location_of_account(&self, account_id: &AccountId) -> Option<Address> {
        BaseLedger::location_of_account(self, account_id)
    }

    fn set(&mut self, addr: &Address, account: Account) {
        BaseLedger::set(self, addr.clone(), account)
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_ledger/ledger.ml#L311
    fn get_or_create(
        &mut self,
        account_id: &AccountId,
    ) -> Result<(AccountState, Account, Address), String> {
        let location = BaseLedger::get_or_create_account(
            self,
            account_id.clone(),
            Account::initialize(account_id),
        )
        .map_err(|e| format!("{:?}", e))?;

        let action = match location {
            GetOrCreated::Added(_) => AccountState::Added,
            GetOrCreated::Existed(_) => AccountState::Existed,
        };

        let addr = location.addr();
        let account = BaseLedger::get(self, addr.clone()).ok_or_else(|| {
            "get_or_create: Account was not found in the ledger after creation".to_string()
        })?;

        Ok((action, account, addr))
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_ledger/ledger.ml#L304
    fn create_new_account(&mut self, account_id: AccountId, account: Account) -> Result<(), ()> {
        match BaseLedger::get_or_create_account(self, account_id, account).unwrap() {
            GetOrCreated::Added(_) => {}
            GetOrCreated::Existed(_) => panic!(),
        }
        Ok(())
    }

    fn remove_accounts_exn(&mut self, account_ids: &[AccountId]) {
        BaseLedger::remove_accounts(self, account_ids)
    }

    fn merkle_root(&mut self) -> Fp {
        BaseLedger::merkle_root(self)
    }

    fn empty(_depth: usize) -> Self {
        todo!()
    }

    fn create_masked(&self) -> Self {
        todo!()
    }

    fn apply_mask(&self, _mask: Self) {
        todo!()
    }

    fn account_locations(&self) -> Vec<Self::Location> {
        let mut addrs: Vec<Address> = self
            .accounts()
            .into_iter()
            .map(|account_id| self.location_of_account(&account_id).unwrap())
            .collect();

        addrs.sort_by_key(Address::to_index);

        addrs
    }
}
