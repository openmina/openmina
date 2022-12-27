use std::collections::{BTreeMap, HashMap};

use mina_hasher::Fp;

use crate::{
    Account, AccountId, AccountIndex, Address, AddressIterator, Direction, HashesMatrix,
    MerklePath, TreeVersion, V2,
};

#[derive(Clone)]
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

    fn get(&self, addr: Address) -> Option<&Account> {
        assert_eq!(addr.length(), self.depth);

        let index = addr.to_index();
        self.values.get(&index)
    }

    pub fn get_exn(&self, addr: Address) -> &Account {
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

        let hash = match self.get(addr.clone()) {
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
}
