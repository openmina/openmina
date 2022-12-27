use std::collections::{BTreeMap, HashMap};

use ark_ff::Zero;
use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::{
    scan_state::transaction_logic::AccountState, Account, AccountId, AccountIndex, Address,
    AddressIterator, BaseLedger, Direction, HashesMatrix, Mask, MerklePath, TreeVersion, V2,
};

#[derive(Clone, Debug)]
pub struct SparseLedger<K, V> {
    values: BTreeMap<AccountIndex, V>,
    indexes: HashMap<K, Address>,
    hashes_matrix: HashesMatrix,
    depth: usize,
}

impl SparseLedger<AccountId, Account> {
    pub fn create(depth: usize, root_hash: Fp) -> Self {
        let mut hashes_matrix = HashesMatrix::new(depth);
        hashes_matrix.set(&Address::root(), root_hash);

        Self {
            values: BTreeMap::new(),
            indexes: HashMap::new(),
            depth,
            hashes_matrix,
        }
    }

    pub fn of_ledger_subset_exn(oledger: Mask, keys: &[AccountId]) -> Self {
        use crate::GetOrCreated::{Added, Existed};

        let mut ledger = oledger.copy();
        let mut sparse = Self::create(
            ledger.depth() as usize,
            BaseLedger::merkle_root(&mut ledger),
        );

        for key in keys {
            match BaseLedger::location_of_account(&ledger, key) {
                Some(addr) => {
                    let account = BaseLedger::get(&ledger, addr.clone()).unwrap();
                    let merkle_path = ledger.merkle_path(addr);
                    sparse.add_path(&merkle_path, key.clone(), account);
                }
                None => {
                    let addr = match ledger
                        .get_or_create_account(key.clone(), Account::empty())
                        .unwrap()
                    {
                        Added(addr) => addr,
                        Existed(_) => panic!("create_empty for a key already present"),
                    };

                    let merkle_path = ledger.merkle_path(addr);
                    sparse.add_path(&merkle_path, key.clone(), Account::empty());
                }
            }
        }

        assert_eq!(BaseLedger::merkle_root(&mut ledger), sparse.merkle_root());

        sparse
    }

    pub fn iteri<F>(&self, fun: F)
    where
        F: Fn(Address, &Account),
    {
        let addr = |index: &AccountIndex| Address::from_index(index.clone(), self.depth);

        for (index, value) in &self.values {
            fun(addr(index), value);
        }
    }

    pub fn add_path(
        &mut self,
        merkle_path: &[MerklePath],
        account_id: AccountId,
        account: Account,
    ) {
        assert_eq!(self.depth, merkle_path.len());

        let mut set_hash = |addr: Address, hash: &Fp| {
            if let Some(prev_hash) = self.hashes_matrix.get(&addr) {
                assert_eq!(prev_hash, hash);
                return;
            };

            self.hashes_matrix.set(&addr, *hash);
        };

        let mut addr = Address::root();

        for path in merkle_path.iter().rev() {
            addr = match path {
                MerklePath::Left(right) => {
                    set_hash(addr.child_right(), right);
                    addr.child_left()
                }
                MerklePath::Right(left) => {
                    set_hash(addr.child_left(), left);
                    addr.child_right()
                }
            }
        }

        let index = addr.to_index();
        self.indexes.insert(account_id, addr);
        self.values.insert(index, account);
    }

    fn get(&self, addr: &Address) -> Option<&Account> {
        assert_eq!(addr.length(), self.depth);

        let index = addr.to_index();
        self.values.get(&index)
    }

    pub fn get_exn(&self, addr: &Address) -> &Account {
        self.get(addr).unwrap()
    }

    pub fn set_exn(&mut self, addr: Address, value: Account) {
        assert_eq!(addr.length(), self.depth);

        let index = addr.to_index();
        self.values.insert(index, value);

        self.hashes_matrix.invalidate_hashes(addr.to_index());
    }

    pub fn find_index_exn(&self, key: AccountId) -> Address {
        self.indexes.get(&key).cloned().unwrap()
    }

    pub fn path_exn(&mut self, addr: Address) -> Vec<MerklePath> {
        let mut merkle_path = Vec::with_capacity(addr.length());
        let mut path_to_addr = addr.into_iter();
        let root = Address::root();

        self.emulate_tree_to_get_path(root, &mut path_to_addr, &mut merkle_path);

        merkle_path
    }

    fn get_value_hash(&mut self, addr: Address) -> Fp {
        if let Some(hash) = self.hashes_matrix.get(&addr) {
            return *hash;
        }

        let hash = match self.get(&addr) {
            Some(value) => V2::hash_leaf(value),
            None => V2::empty_hash_at_depth(0),
        };

        self.hashes_matrix.set(&addr, hash);

        hash
    }

    fn get_node_hash(&mut self, addr: &Address, left: Fp, right: Fp) -> Fp {
        if let Some(hash) = self.hashes_matrix.get(addr) {
            return *hash;
        };

        let depth_in_tree = self.depth - addr.length();

        let hash = V2::hash_node(depth_in_tree - 1, left, right);
        self.hashes_matrix.set(addr, hash);
        hash
    }

    fn emulate_tree_to_get_path(
        &mut self,
        addr: Address,
        path: &mut AddressIterator,
        merkle_path: &mut Vec<MerklePath>,
    ) -> Fp {
        if addr.length() == self.depth {
            return self.get_value_hash(addr);
        }

        let next_direction = path.next();

        // We go until the end of the path
        if let Some(direction) = next_direction.as_ref() {
            let child = match direction {
                Direction::Left => addr.child_left(),
                Direction::Right => addr.child_right(),
            };
            self.emulate_tree_to_get_path(child, path, merkle_path);
        };

        let mut get_child_hash = |addr: Address| match self.hashes_matrix.get(&addr) {
            Some(hash) => *hash,
            None => {
                if let Some(hash) = self.hashes_matrix.get(&addr) {
                    *hash
                } else {
                    self.emulate_tree_to_get_path(addr, path, merkle_path)
                }
            }
        };

        let left = get_child_hash(addr.child_left());
        let right = get_child_hash(addr.child_right());

        if let Some(direction) = next_direction {
            let hash = match direction {
                Direction::Left => MerklePath::Left(right),
                Direction::Right => MerklePath::Right(left),
            };
            merkle_path.push(hash);
        };

        self.get_node_hash(&addr, left, right)
    }

    pub fn merkle_root(&mut self) -> Fp {
        let root = Address::root();

        if let Some(hash) = self.hashes_matrix.get(&root) {
            return *hash;
        };

        self.emulate_tree_merkle_root(root)
    }

    pub fn emulate_tree_merkle_root(&mut self, addr: Address) -> Fp {
        let current_depth = self.depth - addr.length();

        if current_depth == 0 {
            return self.get_value_hash(addr);
        }

        let mut get_child_hash = |addr: Address| {
            if let Some(hash) = self.hashes_matrix.get(&addr) {
                *hash
            } else {
                self.emulate_tree_merkle_root(addr)
            }
        };

        let left_hash = get_child_hash(addr.child_left());
        let right_hash = get_child_hash(addr.child_right());

        self.get_node_hash(&addr, left_hash, right_hash)
    }

    fn location_of_account_impl(&self, account_id: &AccountId) -> Option<Address> {
        self.indexes.get(account_id).cloned()
    }
}

/// Trait used in transaction logic, on the ledger witness (`SparseLedger`)
///
/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/ledger_intf.ml
/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml
pub trait LedgerIntf {
    fn get(&self, addr: &Address) -> Option<Account>;
    fn location_of_account(&self, account_id: &AccountId) -> Option<Address>;
    fn set(&mut self, addr: &Address, account: Account);
    fn get_or_create(
        &mut self,
        account_id: &AccountId,
    ) -> Result<(AccountState, Account, Address), String>;
    fn create_new_account(&mut self, account_id: AccountId, account: Account) -> Result<(), ()>;
    fn remove_accounts_exn(&mut self, account_ids: &[AccountId]);
    fn merkle_root(&mut self) -> Fp;
    fn empty(depth: usize) -> Self;
    fn create_masked(&self) -> Self;
    fn apply_mask(&self, mask: Self);
}

impl LedgerIntf for SparseLedger<AccountId, Account> {
    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml#L58
    fn get(&self, addr: &Address) -> Option<Account> {
        let account = self.get(addr)?;

        if account.public_key == CompressedPubKey::empty() {
            None
        } else {
            Some(account.clone())
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml#L66
    fn location_of_account(&self, account_id: &AccountId) -> Option<Address> {
        let addr = self.indexes.get(account_id)?;
        let account = self.get(addr)?;

        if account.public_key == CompressedPubKey::empty() {
            None
        } else {
            Some(addr.clone())
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml#L75
    fn set(&mut self, addr: &Address, account: Account) {
        self.set_exn(addr.clone(), account);
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml#L96
    fn get_or_create(
        &mut self,
        account_id: &AccountId,
    ) -> Result<(AccountState, Account, Address), String> {
        let addr = self
            .indexes
            .get(account_id)
            .ok_or_else(|| "failed".to_string())?;
        let account = self.get(addr).ok_or_else(|| "failed".to_string())?;

        let addr = addr.clone();
        if account.public_key == CompressedPubKey::empty() {
            let public_key = account_id.public_key.clone();
            let mut account = account.clone();

            account.delegate = Some(public_key.clone());
            account.public_key = public_key;
            account.token_id = account_id.token_id.clone();

            self.set(&addr, account.clone());
            Ok((AccountState::Added, account, addr))
        } else {
            Ok((AccountState::Existed, account.clone(), addr))
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml#L109
    fn create_new_account(&mut self, account_id: AccountId, to_set: Account) -> Result<(), ()> {
        let addr = self.indexes.get(&account_id).ok_or(())?;
        let account = self.get(addr).ok_or(())?;

        if account.public_key == CompressedPubKey::empty() {
            let addr = addr.clone();
            self.set(&addr, to_set);
        }

        Ok(())
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml#L112
    fn remove_accounts_exn(&mut self, _account_ids: &[AccountId]) {
        unimplemented!("remove_accounts_exn: not implemented")
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml#L115
    fn merkle_root(&mut self) -> Fp {
        self.merkle_root()
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml#L142
    fn empty(depth: usize) -> Self {
        Self::create(depth, Fp::zero())
    }

    fn create_masked(&self) -> Self {
        todo!()
    }

    fn apply_mask(&self, _mask: Self) {
        todo!()
    }
}
