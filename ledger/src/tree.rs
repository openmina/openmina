use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Debug,
    ops::ControlFlow,
    path::PathBuf,
};

use crate::{
    account::{Account, AccountId, AccountLegacy, TokenId},
    address::{Address, AddressIterator, Direction},
    base::{AccountIndex, BaseLedger, GetOrCreated},
    tree_version::{TreeVersion, V1, V2},
};
use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

#[derive(Clone, Debug)]
enum NodeOrLeaf<T: TreeVersion> {
    Leaf(Leaf<T>),
    Node(Node<T>),
}

#[derive(Clone, Debug)]
struct Node<T: TreeVersion> {
    left: Option<Box<NodeOrLeaf<T>>>,
    right: Option<Box<NodeOrLeaf<T>>>,
}

impl<T: TreeVersion> Default for Node<T> {
    fn default() -> Self {
        Self {
            left: None,
            right: None,
        }
    }
}

#[derive(Clone, Debug)]
struct Leaf<T: TreeVersion> {
    account: Option<Box<T::Account>>,
}

#[derive(Debug)]
pub struct Database<T: TreeVersion> {
    root: Option<NodeOrLeaf<T>>,
    id_to_addr: HashMap<AccountId, Address>,
    depth: u8,
    last_location: Option<Address>,
    naccounts: usize,
}

impl NodeOrLeaf<V2> {
    fn go_to(&self, path: AddressIterator) -> Option<&Self> {
        let mut node_or_leaf = self;

        for direction in path {
            let node = match node_or_leaf {
                NodeOrLeaf::Node(node) => node,
                NodeOrLeaf::Leaf(_) => return None,
            };

            let child = match direction {
                Direction::Left => &node.left,
                Direction::Right => &node.right,
            };

            let child = match child {
                Some(child) => child,
                None => return None,
            };

            node_or_leaf = &*child;
        }

        Some(node_or_leaf)
    }

    fn go_to_mut(&mut self, path: AddressIterator) -> Option<&mut Self> {
        let mut node_or_leaf = self;

        for direction in path {
            let node = match node_or_leaf {
                NodeOrLeaf::Node(node) => node,
                NodeOrLeaf::Leaf(_) => return None,
            };

            let child = match direction {
                Direction::Left => &mut node.left,
                Direction::Right => &mut node.right,
            };

            let child = match child {
                Some(child) => child,
                None => return None,
            };

            node_or_leaf = &mut *child;
        }

        Some(node_or_leaf)
    }

    fn get_on_path(&self, path: AddressIterator) -> Option<&Account> {
        let node_or_leaf = self.go_to(path)?;

        match node_or_leaf {
            NodeOrLeaf::Leaf(leaf) => Some(leaf.account.as_ref()?),
            NodeOrLeaf::Node(_) => None,
        }
    }

    fn get_mut_leaf_on_path(&mut self, path: AddressIterator) -> Option<&mut Leaf<V2>> {
        let node_or_leaf = self.go_to_mut(path)?;

        match node_or_leaf {
            NodeOrLeaf::Leaf(leaf) => Some(leaf),
            NodeOrLeaf::Node(_) => None,
        }
    }
}

impl<T: TreeVersion> NodeOrLeaf<T> {
    fn add_account_on_path(&mut self, account: T::Account, path: AddressIterator) {
        let mut node_or_leaf = self;

        for direction in path {
            let node = match node_or_leaf {
                NodeOrLeaf::Node(node) => node,
                NodeOrLeaf::Leaf(_) => panic!("Expected node"),
            };

            let child = match direction {
                Direction::Left => &mut node.left,
                Direction::Right => &mut node.right,
            };

            let child = match child {
                Some(child) => child,
                None => {
                    *child = Some(Box::new(NodeOrLeaf::Node(Node::default())));
                    child.as_mut().unwrap()
                }
            };

            node_or_leaf = &mut *child;
        }

        *node_or_leaf = NodeOrLeaf::Leaf(Leaf {
            account: Some(Box::new(account)),
        });
    }

    fn hash(&self, depth: Option<usize>) -> Fp {
        let node = match self {
            NodeOrLeaf::Node(node) => node,
            NodeOrLeaf::Leaf(leaf) => {
                return match leaf.account.as_ref() {
                    Some(account) => T::hash_leaf(account),
                    None => T::empty_hash_at_depth(depth.unwrap()),
                }
            }
        };

        let depth = match depth {
            Some(depth) => depth,
            None => panic!("invalid depth"),
        };

        let left_hash = match &node.left {
            Some(left) => left.hash(depth.checked_sub(1)),
            None => T::empty_hash_at_depth(depth),
        };

        let right_hash = match &node.right {
            Some(right) => right.hash(depth.checked_sub(1)),
            None => T::empty_hash_at_depth(depth),
        };

        T::hash_node(depth, left_hash, right_hash)
    }

    fn iter_recursive<F>(&self, fun: &mut F) -> ControlFlow<()>
    where
        F: FnMut(&T::Account) -> ControlFlow<()>,
    {
        match self {
            NodeOrLeaf::Leaf(leaf) => match leaf.account.as_ref() {
                Some(account) => fun(account),
                None => ControlFlow::Continue(()),
            },
            NodeOrLeaf::Node(node) => {
                if let Some(left) = node.left.as_ref() {
                    left.iter_recursive(fun)?;
                };
                if let Some(right) = node.right.as_ref() {
                    right.iter_recursive(fun)?;
                };
                ControlFlow::Continue(())
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DatabaseError {
    OutOfLeaves,
}

impl Database<V2> {
    pub fn create_account(
        &mut self,
        account_id: AccountId,
        account: Account,
    ) -> Result<GetOrCreated, DatabaseError> {
        if self.root.is_none() {
            self.root = Some(NodeOrLeaf::Node(Node::default()));
        }

        if let Some(addr) = self.id_to_addr.get(&account_id).cloned() {
            return Ok(GetOrCreated::Existed(addr));
        }

        let location = match self.last_location.as_ref() {
            Some(last) => last.next().ok_or(DatabaseError::OutOfLeaves)?,
            None => Address::first(self.depth as usize),
        };

        let root = self.root.as_mut().unwrap();
        root.add_account_on_path(account, location.iter());

        self.last_location = Some(location.clone());
        self.naccounts += 1;

        self.id_to_addr.insert(account_id, location.clone());

        Ok(GetOrCreated::Added(location))
    }
}

impl Database<V1> {
    pub fn create_account(
        &mut self,
        _account_id: (),
        account: AccountLegacy,
    ) -> Result<Address, DatabaseError> {
        if self.root.is_none() {
            self.root = Some(NodeOrLeaf::Node(Node::default()));
        }

        let location = match self.last_location.as_ref() {
            Some(last) => last.next().ok_or(DatabaseError::OutOfLeaves)?,
            None => Address::first(self.depth as usize),
        };

        let root = self.root.as_mut().unwrap();
        let path_iter = location.clone().into_iter();
        root.add_account_on_path(account, path_iter);

        self.last_location = Some(location.clone());
        self.naccounts += 1;

        Ok(location)
    }
}

impl<T: TreeVersion> Database<T> {
    pub fn create(depth: u8) -> Self {
        assert!((1..0xfe).contains(&depth));

        let max_naccounts = 2u64.pow(depth.min(25) as u32);

        Self {
            depth,
            root: None,
            last_location: None,
            naccounts: 0,
            id_to_addr: HashMap::with_capacity(max_naccounts as usize),
        }
    }

    pub fn root_hash(&self) -> Fp {
        println!("naccounts={:?}", self.naccounts);
        match self.root.as_ref() {
            Some(root) => root.hash(Some(self.depth as usize - 1)),
            None => T::empty_hash_at_depth(self.depth as usize),
        }
    }

    pub fn naccounts(&self) -> usize {
        let mut naccounts = 0;

        if let Some(root) = self.root.as_ref() {
            self.naccounts_recursive(root, &mut naccounts)
        };

        naccounts
    }

    fn naccounts_recursive(&self, elem: &NodeOrLeaf<T>, naccounts: &mut usize) {
        match elem {
            NodeOrLeaf::Leaf(_) => *naccounts += 1,
            NodeOrLeaf::Node(node) => {
                if let Some(left) = node.left.as_ref() {
                    self.naccounts_recursive(left, naccounts);
                };
                if let Some(right) = node.right.as_ref() {
                    self.naccounts_recursive(right, naccounts);
                };
            }
        }
    }
}

impl BaseLedger for Database<V2> {
    fn to_list(&self) -> Vec<Account> {
        let root = match self.root.as_ref() {
            Some(root) => root,
            None => return Vec::new(),
        };

        let mut accounts = Vec::with_capacity(100);

        root.iter_recursive(&mut |acc| {
            accounts.push(acc.clone());
            ControlFlow::Continue(())
        });

        accounts
    }

    fn iter<F>(&self, mut fun: F)
    where
        F: FnMut(&Account),
    {
        let root = match self.root.as_ref() {
            Some(root) => root,
            None => return,
        };

        root.iter_recursive(&mut |acc| {
            fun(acc);
            ControlFlow::Continue(())
        });
    }

    fn fold<B, F>(&self, init: B, mut fun: F) -> B
    where
        F: FnMut(B, &Account) -> B,
    {
        let root = match self.root.as_ref() {
            Some(root) => root,
            None => return init,
        };

        let mut accum = Some(init);
        root.iter_recursive(&mut |acc| {
            let res = fun(accum.take().unwrap(), acc);
            accum = Some(res);
            ControlFlow::Continue(())
        });

        accum.unwrap()
    }

    fn fold_with_ignored_accounts<B, F>(
        &self,
        ignoreds: HashSet<AccountId>,
        init: B,
        mut fun: F,
    ) -> B
    where
        F: FnMut(B, &Account) -> B,
    {
        self.fold(init, |accum, acc| {
            let account_id = acc.id();

            if !ignoreds.contains(&account_id) {
                fun(accum, acc)
            } else {
                accum
            }
        })
    }

    fn fold_until<B, F>(&self, init: B, mut fun: F) -> B
    where
        F: FnMut(B, &Account) -> Option<B>,
    {
        let root = match self.root.as_ref() {
            Some(root) => root,
            None => return init,
        };

        let mut accum = Some(init);
        root.iter_recursive(&mut |acc| {
            let res = match fun(accum.take().unwrap(), acc) {
                Some(res) => res,
                None => return ControlFlow::Break(()),
            };

            accum = Some(res);
            ControlFlow::Continue(())
        });

        accum.unwrap()
    }

    fn accounts(&self) -> HashSet<AccountId> {
        self.id_to_addr.keys().cloned().collect()
    }

    fn token_owner(&self, token: TokenId) -> Option<AccountId> {
        let root = self.root.as_ref()?;
        let mut account_id = None;

        root.iter_recursive(&mut |acc| {
            if acc.token_id == token {
                account_id = Some(acc.id());
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });

        account_id
    }

    // TODO: Not sure if it's a correct impl, ocaml seems to keep an index
    fn token_owners(&self) -> HashSet<AccountId> {
        let root = match self.root.as_ref() {
            Some(root) => root,
            None => return HashSet::default(),
        };

        let mut tokens = HashMap::with_capacity(self.naccounts);

        root.iter_recursive(&mut |acc| {
            let token = acc.token_id.clone();
            let id = acc.id();

            tokens.insert(token, id);

            ControlFlow::Continue(())
        });

        tokens.into_values().collect()
    }

    fn tokens(&self, public_key: CompressedPubKey) -> HashSet<TokenId> {
        let root = match self.root.as_ref() {
            Some(root) => root,
            None => return HashSet::default(),
        };

        let mut set = HashSet::with_capacity(self.naccounts);

        root.iter_recursive(&mut |acc| {
            if acc.public_key == public_key {
                set.insert(acc.token_id.clone());
            }

            ControlFlow::Continue(())
        });

        set
    }

    fn location_of_account(&self, account_id: AccountId) -> Option<Address> {
        self.id_to_addr.get(&account_id).cloned()
    }

    fn location_of_account_batch(
        &self,
        account_ids: &[AccountId],
    ) -> Vec<(AccountId, Option<Address>)> {
        account_ids
            .iter()
            .map(|acc_id| {
                let addr = self.id_to_addr.get(acc_id).cloned();
                (acc_id.clone(), addr)
            })
            .collect()
    }

    fn get_or_create_account(
        &mut self,
        account_id: AccountId,
        account: Account,
    ) -> Result<GetOrCreated, DatabaseError> {
        self.create_account(account_id, account)
    }

    fn close(self) {
        // Drop
    }

    fn last_filled(&self) -> Option<Address> {
        self.last_location.clone()
    }

    fn get_uuid(&self) -> crate::base::Uuid {
        todo!()
    }

    fn get_directory(&self) -> Option<PathBuf> {
        None
    }

    fn get(&self, addr: Address) -> Option<Account> {
        self.root.as_ref()?.get_on_path(addr.into_iter()).cloned()
    }

    fn get_batch(&self, addr: &[Address]) -> Vec<(Address, Option<Account>)> {
        let root = match self.root.as_ref() {
            Some(root) => Cow::Borrowed(root),
            None => Cow::Owned(NodeOrLeaf::Node(Node::default())),
        };

        addr.iter()
            .map(|addr| (addr.clone(), root.get_on_path(addr.iter()).cloned()))
            .collect()
    }

    fn set(&mut self, addr: Address, account: Account) {
        if self.root.is_none() {
            self.root = Some(NodeOrLeaf::Node(Node::default()));
        }

        let id = account.id();
        let root = self.root.as_mut().unwrap();

        // Remove account at the address and it's index
        if let Some(account) = root.get_on_path(addr.iter()) {
            let id = account.id();
            self.id_to_addr.remove(&id);
        } else {
            self.naccounts += 1;
        }

        root.add_account_on_path(account, addr.iter());
        self.id_to_addr.insert(id, addr);

        // TODO: Should it modify Self::last_location ?
    }

    fn set_batch(&mut self, list: &[(Address, Account)]) {
        for (addr, account) in list {
            self.set(addr.clone(), account.clone());
        }
    }

    fn get_at_index(&self, index: AccountIndex) -> Option<Account> {
        let addr = Address::from_index(index, self.depth as usize);
        self.get(addr)
    }

    fn set_at_index(&mut self, index: AccountIndex, account: Account) -> Result<(), ()> {
        let addr = Address::from_index(index, self.depth as usize);
        self.set(addr, account);
        Ok(())
    }

    fn index_of_account(&self, account_id: AccountId) -> Option<AccountIndex> {
        self.id_to_addr.get(&account_id).map(Address::to_index)
    }

    fn merkle_root(&self) -> Fp {
        self.root_hash()
    }

    fn merkle_path(&self, addr: Address) -> AddressIterator {
        addr.into_iter()
    }

    fn merkle_path_at_index(&self, index: AccountIndex) -> Option<AddressIterator> {
        let addr = Address::from_index(index, self.depth as usize);
        Some(addr.into_iter())
    }

    fn remove_accounts(&mut self, ids: &[AccountId]) {
        let root = match self.root.as_mut() {
            Some(root) => root,
            None => return,
        };

        for addr in ids.iter().filter_map(|id| self.id_to_addr.get(id)) {
            let leaf = match root.get_mut_leaf_on_path(addr.iter()) {
                Some(leaf) => leaf,
                None => continue,
            };
            leaf.account = None;
        }
    }

    fn detached_signal(&mut self) {
        todo!()
    }

    fn depth(&self) -> u8 {
        self.depth.try_into().unwrap()
    }

    fn num_accounts(&self) -> usize {
        self.naccounts
    }

    fn merkle_path_at_addr(&self, addr: Address) -> Option<AddressIterator> {
        Some(self.merkle_path(addr))
    }

    fn get_inner_hash_at_addr(&self, addr: Address) -> Result<Fp, ()> {
        let root = match self.root.as_ref() {
            Some(root) => root,
            None => todo!(),
        };

        // TODO: See how ocaml behaves when the address is at a non-created branch

        let node_or_leaf = match root.go_to(addr.into_iter()) {
            Some(node_or_leaf) => node_or_leaf,
            None => todo!(),
        };

        let hash = node_or_leaf.hash(Some(self.depth as usize));
        Ok(hash)
    }

    fn set_inner_hash_at_addr(&mut self, addr: Address, hash: Fp) -> Result<(), ()> {
        // No-op for now, because we don't store the hashes anywhere
        Ok(())
    }

    fn set_all_accounts_rooted_at(
        &mut self,
        addr: Address,
        accounts: &[Account],
    ) -> Result<(), ()> {
        if addr.length() > self.depth as usize {
            return Err(());
        }

        for (child_addr, account) in addr.iter_children(self.depth as usize).zip(accounts) {
            self.set(child_addr, account.clone());
        }

        Ok(())
    }

    fn get_all_accounts_rooted_at(&self, addr: Address) -> Option<Vec<(Address, Account)>> {
        if addr.length() > self.depth as usize {
            return None;
        }

        let root = match self.root.as_ref() {
            Some(root) => root,
            None => return None,
        };

        let mut accounts = Vec::with_capacity(1000); // TODO: compute nchildren
        for child_addr in addr.iter_children(self.depth as usize) {
            let account = match root.get_on_path(child_addr.iter()).cloned() {
                Some(account) => account,
                None => continue,
            };
            accounts.push((child_addr, account));
        }

        if accounts.is_empty() {
            None
        } else {
            Some(accounts)
        }
    }

    fn make_space_for(&mut self, _space: usize) {
        // No op, we're in memory
    }
}

#[cfg(test)]
mod tests {
    use o1_utils::FieldHelpers;

    use crate::{
        account::{Account, AccountLegacy},
        tree_version::{account_empty_legacy_hash, V1, V2},
    };

    use super::*;

    #[test]
    fn test_legacy_db() {
        let two: usize = 2;

        for depth in 2..15 {
            let mut db = Database::<V1>::create(depth);

            for _ in 0..two.pow(depth as u32) {
                db.create_account((), AccountLegacy::create()).unwrap();
            }

            let naccounts = db.naccounts();
            assert_eq!(naccounts, two.pow(depth as u32));

            assert_eq!(
                db.create_account((), AccountLegacy::create()).unwrap_err(),
                DatabaseError::OutOfLeaves
            );

            println!("depth={:?} naccounts={:?}", depth, naccounts);
        }
    }

    #[test]
    fn test_db_v2() {
        let two: usize = 2;

        for depth in 2..15 {
            let mut db = Database::<V2>::create(depth);

            for _ in 0..two.pow(depth as u32) {
                let account = Account::rand();
                let id = account.id();
                db.create_account(id, account).unwrap();
            }

            let naccounts = db.naccounts();
            assert_eq!(naccounts, two.pow(depth as u32));

            let account = Account::create();
            let id = account.id();
            assert_eq!(
                db.create_account(id, account).unwrap_err(),
                DatabaseError::OutOfLeaves
            );

            println!("depth={:?} naccounts={:?}", depth, naccounts);
        }
    }

    #[test]
    fn test_legacy_hash_empty() {
        let account_empty_hash = account_empty_legacy_hash();
        assert_eq!(
            account_empty_hash.to_hex(),
            "70ccdba14f829608e59a37ed98ffcaeef06dad928d568a9adbde13e3dd104a20"
        );

        for (depth, s) in [
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
            let hash = V1::empty_hash_at_depth(depth);
            assert_eq!(hash.to_hex(), s, "invalid hash at depth={:?}", depth);
        }
    }

    #[test]
    fn test_hash_empty() {
        let account_empty_hash = Account::empty().hash();
        assert_eq!(
            account_empty_hash.to_hex(),
            "976de129aebe3a7a4a6127bafad8fba19b75ae2517854133013d0f1ab87c2904"
        );

        for (depth, s) in [
            (
                0,
                "976de129aebe3a7a4a6127bafad8fba19b75ae2517854133013d0f1ab87c2904",
            ),
            (
                5,
                "ab4bda63c3c9edf4deb113f2993724a1599a5588421530a9a862f5dbdbeded06",
            ),
            (
                10,
                "d753d0d1dc1211d97c903c53c5eb62a49bc370ddf63870aa26bfade7b47b5102",
            ),
            (
                11,
                "eab73d282c56c799bd42b18eb92fab18a90dcfac48c8866e19e2902d850b3731",
            ),
            (
                100,
                "3ec0aa90fa11f39482d347b18032d2292b3673807d5b4c6fc2aa73b98d875a2f",
            ),
            (
                2000,
                "031a2618a9592787596642ba88bfc502236221d0981facd2f3caf8648336ca12",
            ),
        ] {
            let hash = V2::empty_hash_at_depth(depth);
            assert_eq!(hash.to_hex(), s, "invalid hash at depth={:?}", depth);
        }
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
    //     println!("ROOT_HASH={:?}", hash.to_string());
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

    /// An empty tree produces the same hash than a tree full of empty accounts
    #[test]
    fn test_root_hash_legacy() {
        let mut db = Database::<V1>::create(4);
        for _ in 0..16 {
            db.create_account((), AccountLegacy::empty()).unwrap();
        }
        assert_eq!(
            db.create_account((), AccountLegacy::empty()).unwrap_err(),
            DatabaseError::OutOfLeaves
        );
        let hash = db.root_hash();
        assert_eq!(
            hash.to_hex(),
            "2db7d27130b6fe46b95541a70bc69ac51d9ea02825f7a7ab41ec4c414989421e"
        );

        let mut db = Database::<V1>::create(4);
        for _ in 0..1 {
            db.create_account((), AccountLegacy::empty()).unwrap();
        }
        let hash = db.root_hash();
        assert_eq!(
            hash.to_hex(),
            "2db7d27130b6fe46b95541a70bc69ac51d9ea02825f7a7ab41ec4c414989421e"
        );

        let db = Database::<V1>::create(4);
        let hash = db.root_hash();
        assert_eq!(
            hash.to_hex(),
            "2db7d27130b6fe46b95541a70bc69ac51d9ea02825f7a7ab41ec4c414989421e"
        );
    }
}
