use std::collections::{BTreeMap, HashMap};

use ark_ff::Zero;
use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::{
    scan_state::{conv::to_ledger_hash, currency::Slot, transaction_logic::AccountState},
    Account, AccountId, AccountIndex, Address, AddressIterator, BaseLedger, Direction,
    HashesMatrix, Mask, MerklePath, TreeVersion, V2,
};

/// This is used only to serialize, to get the same order as OCaml
type InsertedNumber = u32;

#[derive(Clone, Debug)]
pub struct SparseLedger<K, V> {
    values: BTreeMap<AccountIndex, V>,
    indexes: HashMap<K, (Address, InsertedNumber)>,
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

    fn current_inserted_number(&self) -> InsertedNumber {
        self.indexes.len().try_into().unwrap()
    }

    fn get_or_initialize_exn(
        &self,
        account_id: &AccountId,
        addr: &Address,
    ) -> (AccountState, Account) {
        let mut account = self.get_exn(addr).clone();

        if account.public_key == CompressedPubKey::empty() {
            let public_key = account_id.public_key.clone();
            let token_id = account_id.token_id.clone();

            // Only allow delegation if this account is for the default token.
            let delegate = if token_id.is_default() {
                Some(public_key.clone())
            } else {
                None
            };

            account.delegate = delegate;
            account.public_key = public_key;
            account.token_id = token_id;

            (AccountState::Added, account)
        } else {
            (AccountState::Existed, account)
        }
    }

    pub fn has_locked_tokens_exn(&self, global_slot: Slot, account_id: AccountId) -> bool {
        let addr = self.find_index_exn(account_id.clone());
        let (_, account) = self.get_or_initialize_exn(&account_id, &addr);
        account.has_locked_tokens(global_slot)
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
        self.indexes
            .insert(account_id, (addr, self.current_inserted_number()));
        self.values.insert(index, account);
    }

    fn get_index(&self, account_id: &AccountId) -> Option<&Address> {
        self.indexes.get(account_id).map(|(addr, _)| addr)
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
        self.get_index(&key).cloned().unwrap()
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
        self.get_index(account_id).cloned()
    }
}

impl From<&SparseLedger<AccountId, Account>>
    for mina_p2p_messages::v2::MinaBaseSparseLedgerBaseStableV2
{
    fn from(value: &SparseLedger<AccountId, Account>) -> Self {
        use mina_p2p_messages::v2::MinaBaseAccountBinableArgStableV2;
        use mina_p2p_messages::v2::MinaBaseAccountIdStableV2;
        use mina_p2p_messages::v2::MinaBaseSparseLedgerBaseStableV2Tree;

        assert!(value.hashes_matrix.get(&Address::root()).is_some());

        let mut indexes: Vec<_> = value
            .indexes
            .iter()
            .map(|(id, addr)| {
                let index: AccountIndex = addr.0.to_index();
                let index: u32 = index.as_u64().try_into().unwrap();
                let index: mina_p2p_messages::number::Int32 = (index as i32).into();

                let id: MinaBaseAccountIdStableV2 = id.clone().into();

                (id, (index, addr.1))
            })
            .collect();

        indexes.sort_by_key(|(_, (_, n))| *n);
        indexes.reverse();

        let indexes: Vec<_> = indexes
            .into_iter()
            .map(|(id, (addr, _))| (id, addr))
            .collect();

        fn build_tree(
            addr: Address,
            matrix: &HashesMatrix,
            ledger_depth: usize,
            values: &BTreeMap<AccountIndex, Account>,
        ) -> MinaBaseSparseLedgerBaseStableV2Tree {
            if addr.length() == ledger_depth {
                let account_index = addr.to_index();
                let account = values.get(&account_index).unwrap();
                let account: MinaBaseAccountBinableArgStableV2 = account.clone().into();

                return MinaBaseSparseLedgerBaseStableV2Tree::Account(Box::new(account));
            }

            let child_left = addr.child_left();
            let child_right = addr.child_right();

            let is_left = matrix.get(&child_left).is_some();
            let is_right = matrix.get(&child_right).is_some();

            if is_left && is_right {
                let hash = matrix.get(&addr).unwrap();
                let left_node = build_tree(child_left, matrix, ledger_depth, values);
                let right_node = build_tree(child_right, matrix, ledger_depth, values);

                MinaBaseSparseLedgerBaseStableV2Tree::Node(
                    to_ledger_hash(hash),
                    Box::new(left_node),
                    Box::new(right_node),
                )
            } else if is_left {
                build_tree(child_left, matrix, ledger_depth, values)
            } else if is_right {
                build_tree(child_right, matrix, ledger_depth, values)
            } else {
                let hash = matrix.get(&addr).unwrap();
                MinaBaseSparseLedgerBaseStableV2Tree::Hash(to_ledger_hash(hash))
            }
        }

        let tree = build_tree(
            Address::root(),
            &value.hashes_matrix,
            value.depth,
            &value.values,
        );

        let depth: u32 = value.depth.try_into().unwrap();

        Self {
            indexes,
            depth: (depth as i32).into(),
            tree,
        }
    }
}

impl From<&mina_p2p_messages::v2::MinaBaseSparseLedgerBaseStableV2>
    for SparseLedger<AccountId, Account>
{
    fn from(value: &mina_p2p_messages::v2::MinaBaseSparseLedgerBaseStableV2) -> Self {
        use mina_p2p_messages::v2::MinaBaseSparseLedgerBaseStableV2Tree;
        use mina_p2p_messages::v2::MinaBaseSparseLedgerBaseStableV2Tree::{Account, Hash, Node};

        fn build_matrix(
            matrix: &mut HashesMatrix,
            addr: Address,
            node: &MinaBaseSparseLedgerBaseStableV2Tree,
            values: &mut BTreeMap<AccountIndex, crate::Account>,
        ) {
            match node {
                Account(account) => {
                    // TODO: Don't clone the account here
                    values.insert(addr.to_index(), (**account).clone().into());
                }
                Hash(hash) => {
                    matrix.set(&addr, hash.to_field());
                }
                Node(hash, left, right) => {
                    matrix.set(&addr, hash.to_field());
                    build_matrix(matrix, addr.child_left(), left, values);
                    build_matrix(matrix, addr.child_right(), right, values);
                }
            }
        }

        let depth = value.depth.as_u32() as usize;
        let mut indexes = HashMap::with_capacity(value.indexes.len());
        let mut hashes_matrix = HashesMatrix::new(depth);
        let mut values = BTreeMap::new();

        for (index, (account_id, account_index)) in value.indexes.iter().enumerate() {
            let account_id: AccountId = account_id.into();
            let account_index = AccountIndex::from(account_index.as_u32() as usize);

            let addr = Address::from_index(account_index, depth);
            let number: InsertedNumber = index.try_into().unwrap();
            indexes.insert(account_id, (addr, number));
        }

        build_matrix(
            &mut hashes_matrix,
            Address::root(),
            &value.tree,
            &mut values,
        );

        Self {
            values,
            indexes,
            hashes_matrix,
            depth,
        }
    }
}

/// Trait used in transaction logic, on the ledger witness (`SparseLedger`), or on mask
///
/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/ledger_intf.ml
/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml
pub trait LedgerIntf {
    type Location: Clone + std::fmt::Debug;

    fn get(&self, addr: &Self::Location) -> Option<Account>;
    fn location_of_account(&self, account_id: &AccountId) -> Option<Self::Location>;
    fn set(&mut self, addr: &Self::Location, account: Account);
    fn get_or_create(
        &mut self,
        account_id: &AccountId,
    ) -> Result<(AccountState, Account, Self::Location), String>;
    fn create_new_account(&mut self, account_id: AccountId, account: Account) -> Result<(), ()>;
    fn remove_accounts_exn(&mut self, account_ids: &[AccountId]);
    fn merkle_root(&mut self) -> Fp;
    fn empty(depth: usize) -> Self;
    fn create_masked(&self) -> Self;
    fn apply_mask(&self, mask: Self);
}

impl LedgerIntf for SparseLedger<AccountId, Account> {
    type Location = Address;

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml#L58
    fn get(&self, addr: &Self::Location) -> Option<Account> {
        let account = self.get(addr)?;

        if account.public_key == CompressedPubKey::empty() {
            None
        } else {
            Some(account.clone())
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml#L66
    fn location_of_account(&self, account_id: &AccountId) -> Option<Self::Location> {
        let addr = self.get_index(account_id)?;
        let account = self.get(addr)?;

        if account.public_key == CompressedPubKey::empty() {
            None
        } else {
            Some(addr.clone())
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml#L75
    fn set(&mut self, addr: &Self::Location, account: Account) {
        self.set_exn(addr.clone(), account);
    }

    /// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml#L96
    fn get_or_create(
        &mut self,
        account_id: &AccountId,
    ) -> Result<(AccountState, Account, Self::Location), String> {
        let addr = self
            .get_index(account_id)
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
        let addr = self.get_index(&account_id).ok_or(())?;
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
