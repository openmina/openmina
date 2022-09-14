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
    base::{next_uuid, AccountIndex, BaseLedger, GetOrCreated, MerklePath, Uuid},
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
    token_to_account: HashMap<T::TokenId, AccountId>,
    depth: u8,
    last_location: Option<Address>,
    naccounts: usize,
    uuid: Uuid,
}

impl NodeOrLeaf<V2> {
    fn go_to(&self, path: AddressIterator) -> Option<&Self> {
        let mut node_or_leaf = self;

        for direction in path {
            let node = node_or_leaf.node()?;

            let child = match direction {
                Direction::Left => node.left.as_ref()?,
                Direction::Right => node.right.as_ref()?,
            };

            node_or_leaf = &*child;
        }

        Some(node_or_leaf)
    }

    fn go_to_mut(&mut self, path: AddressIterator) -> Option<&mut Self> {
        let mut node_or_leaf = self;

        for direction in path {
            let node = node_or_leaf.node_mut()?;

            let child = match direction {
                Direction::Left => node.left.as_mut()?,
                Direction::Right => node.right.as_mut()?,
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
    fn node(&self) -> Option<&Node<T>> {
        match self {
            NodeOrLeaf::Node(node) => Some(node),
            NodeOrLeaf::Leaf(_) => None,
        }
    }

    fn node_mut(&mut self) -> Option<&mut Node<T>> {
        match self {
            NodeOrLeaf::Node(node) => Some(node),
            NodeOrLeaf::Leaf(_) => None,
        }
    }

    fn add_account_on_path(&mut self, account: T::Account, path: AddressIterator) {
        let mut node_or_leaf = self;

        for direction in path {
            let node = node_or_leaf.node_mut().expect("Expected node");

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
                    None => T::empty_hash_at_depth(0), // Empty account
                };
            }
        };

        let depth = match depth {
            Some(depth) => depth,
            None => panic!("invalid depth"),
        };

        let left_hash = match node.left.as_ref() {
            Some(left) => left.hash(depth.checked_sub(1)),
            None => T::empty_hash_at_depth(depth),
        };

        let right_hash = match node.right.as_ref() {
            Some(right) => right.hash(depth.checked_sub(1)),
            None => T::empty_hash_at_depth(depth),
        };

        T::hash_node(depth, left_hash, right_hash)
    }

    fn hash_at_path(
        &self,
        depth: Option<usize>,
        path: &mut AddressIterator,
        merkle_path: &mut Vec<MerklePath>,
    ) -> Fp {
        let next_direction = path.next();

        let node = match self {
            NodeOrLeaf::Node(node) => node,
            NodeOrLeaf::Leaf(leaf) => {
                return match leaf.account.as_ref() {
                    Some(account) => T::hash_leaf(account),
                    None => T::empty_hash_at_depth(0), // Empty account
                };
            }
        };

        let depth = match depth {
            Some(depth) => depth,
            None => panic!("invalid depth"),
        };

        let left_hash = match node.left.as_ref() {
            Some(left) => left.hash(depth.checked_sub(1)),
            None => T::empty_hash_at_depth(depth),
        };

        let right_hash = match node.right.as_ref() {
            Some(right) => right.hash(depth.checked_sub(1)),
            None => T::empty_hash_at_depth(depth),
        };

        if let Some(direction) = next_direction {
            let hash = match direction {
                Direction::Left => MerklePath::Left(left_hash),
                Direction::Right => MerklePath::Right(right_hash),
            };
            merkle_path.push(hash)
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

    fn iter_recursive_with_addr<F>(
        &self,
        index: &mut u64,
        depth: u8,
        fun: &mut F,
    ) -> ControlFlow<()>
    where
        F: FnMut(Address, &T::Account) -> ControlFlow<()>,
    {
        match self {
            NodeOrLeaf::Leaf(leaf) => {
                let addr = Address::from_index(AccountIndex(*index), depth as usize);
                *index += 1;
                match leaf.account.as_ref() {
                    Some(account) => fun(addr, account),
                    None => ControlFlow::Continue(()),
                }
            }
            NodeOrLeaf::Node(node) => {
                if let Some(left) = node.left.as_ref() {
                    left.iter_recursive_with_addr(index, depth, fun)?;
                };
                if let Some(right) = node.right.as_ref() {
                    right.iter_recursive_with_addr(index, depth, fun)?;
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
    fn create_account(
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

        let token_id = account.token_id.clone();
        let location = match self.last_location.as_ref() {
            Some(last) => last.next().ok_or(DatabaseError::OutOfLeaves)?,
            None => Address::first(self.depth as usize),
        };

        let root = self.root.as_mut().unwrap();
        root.add_account_on_path(account, location.iter());

        self.last_location = Some(location.clone());
        self.naccounts += 1;

        self.token_to_account.insert(token_id, account_id.clone());
        self.id_to_addr.insert(account_id, location.clone());

        Ok(GetOrCreated::Added(location))
    }

    pub fn iter_with_addr<F>(&self, mut fun: F)
    where
        F: FnMut(Address, &Account),
    {
        let root = match self.root.as_ref() {
            Some(root) => root,
            None => return,
        };

        let mut index = 0;
        let depth = self.depth;

        root.iter_recursive_with_addr(&mut index, depth, &mut |addr, account| {
            fun(addr, account);
            ControlFlow::Continue(())
        });
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
            id_to_addr: HashMap::with_capacity(max_naccounts as usize / 2),
            token_to_account: HashMap::with_capacity(max_naccounts as usize / 2),
            uuid: next_uuid(),
        }
    }

    pub fn root_hash(&self) -> Fp {
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

        root.iter_recursive(&mut |account| {
            accounts.push(account.clone());
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

        root.iter_recursive(&mut |account| {
            fun(account);
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
        root.iter_recursive(&mut |account| {
            let res = fun(accum.take().unwrap(), account);
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
        self.fold(init, |accum, account| {
            let account_id = account.id();

            if !ignoreds.contains(&account_id) {
                fun(accum, account)
            } else {
                accum
            }
        })
    }

    fn fold_until<B, F>(&self, init: B, mut fun: F) -> B
    where
        F: FnMut(B, &Account) -> ControlFlow<B, B>,
    {
        let root = match self.root.as_ref() {
            Some(root) => root,
            None => return init,
        };

        let mut accum = Some(init);
        root.iter_recursive(&mut |account| match fun(accum.take().unwrap(), account) {
            ControlFlow::Continue(account) => {
                accum = Some(account);
                ControlFlow::Continue(())
            }
            ControlFlow::Break(account) => {
                accum = Some(account);
                ControlFlow::Break(())
            }
        });

        accum.unwrap()
    }

    fn accounts(&self) -> HashSet<AccountId> {
        self.id_to_addr.keys().cloned().collect()
    }

    fn token_owner(&self, token_id: TokenId) -> Option<AccountId> {
        self.token_to_account.get(&token_id).cloned()
    }

    fn token_owners(&self) -> HashSet<AccountId> {
        self.token_to_account.values().cloned().collect()
    }

    fn tokens(&self, public_key: CompressedPubKey) -> HashSet<TokenId> {
        let root = match self.root.as_ref() {
            Some(root) => root,
            None => return HashSet::default(),
        };

        let mut set = HashSet::with_capacity(self.naccounts);

        root.iter_recursive(&mut |account| {
            if account.public_key == public_key {
                set.insert(account.token_id.clone());
            }

            ControlFlow::Continue(())
        });

        set
    }

    fn location_of_account(&self, account_id: &AccountId) -> Option<Address> {
        self.id_to_addr.get(account_id).cloned()
    }

    fn location_of_account_batch(
        &self,
        account_ids: &[AccountId],
    ) -> Vec<(AccountId, Option<Address>)> {
        account_ids
            .iter()
            .map(|account_id| {
                let addr = self.id_to_addr.get(account_id).cloned();
                (account_id.clone(), addr)
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

    fn close(&self) {
        // Drop
    }

    fn last_filled(&self) -> Option<Address> {
        self.last_location.clone()
    }

    fn get_uuid(&self) -> crate::base::Uuid {
        self.uuid
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
            self.token_to_account.remove(&id.token_id);
        } else {
            self.naccounts += 1;
        }

        self.token_to_account
            .insert(account.token_id.clone(), id.clone());
        self.id_to_addr.insert(id, addr.clone());
        root.add_account_on_path(account, addr.iter());

        if self
            .last_location
            .as_ref()
            .map(|l| l.to_index() < addr.to_index())
            .unwrap_or(true)
        {
            self.last_location = Some(addr);
        }
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

    fn merkle_path(&self, addr: Address) -> Vec<MerklePath> {
        let root = match self.root.as_ref() {
            Some(root) => root,
            None => return Vec::new(),
        };

        let mut merkle_path = Vec::with_capacity(addr.length());

        let mut path = addr.into_iter();
        root.hash_at_path(Some(self.depth as usize - 1), &mut path, &mut merkle_path);

        merkle_path
    }

    fn merkle_path_at_index(&self, index: AccountIndex) -> Vec<MerklePath> {
        let addr = Address::from_index(index, self.depth as usize);
        self.merkle_path(addr)
    }

    fn remove_accounts(&mut self, ids: &[AccountId]) {
        let root = match self.root.as_mut() {
            Some(root) => root,
            None => return,
        };

        let mut addrs = ids
            .iter()
            .map(|accound_id| self.id_to_addr.remove(accound_id).unwrap())
            .collect::<Vec<_>>();
        addrs.sort_by_key(|a| a.to_index());

        for addr in addrs.iter().rev() {
            let leaf = match root.get_mut_leaf_on_path(addr.iter()) {
                Some(leaf) => leaf,
                None => continue,
            };

            let account = match leaf.account.take() {
                Some(account) => account,
                None => continue,
            };

            let id = account.id();
            self.id_to_addr.remove(&id);
            self.token_to_account.remove(&id.token_id);

            self.naccounts = self
                .naccounts
                .checked_sub(1)
                .expect("invalid naccounts counter");

            if self
                .last_location
                .as_ref()
                .map(|last| last == addr)
                .unwrap_or(false)
            {
                self.last_location = addr.prev();
            }
        }
    }

    fn detached_signal(&mut self) {
        todo!()
    }

    fn depth(&self) -> u8 {
        self.depth
    }

    fn num_accounts(&self) -> usize {
        self.naccounts
    }

    fn merkle_path_at_addr(&self, addr: Address) -> Vec<MerklePath> {
        self.merkle_path(addr)
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

    fn set_inner_hash_at_addr(&mut self, _addr: Address, _hash: Fp) -> Result<(), ()> {
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

        let children = addr.iter_children(self.depth as usize);
        let mut accounts = Vec::with_capacity(children.len());

        for child_addr in children {
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

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

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
            "27e0bcd12f2d03ef88f7b733c0edcee1b82c81cadb15ed70b0faf6fd701d2a15"
        );

        for (depth, s) in [
            (
                0,
                "27e0bcd12f2d03ef88f7b733c0edcee1b82c81cadb15ed70b0faf6fd701d2a15",
            ),
            (
                5,
                "634f07966c6de25a6b5153d28f24cf5f45bb2f90cc6d54bd601aa4ba7f319f22",
            ),
            (
                10,
                "29dc9c35041d25a70c9ef90fab4d05257f9fb73bf90202e0b15c7b2e13124d17",
            ),
            (
                11,
                "573cb5bb84a9cc1b5403b31e48e9bce9cccfe63be94ea9c66671ab5b2852162a",
            ),
            (
                100,
                "97539c708e2f269ac25c9ce8e5622bd0400798385f9416c3f78099f7f13bd70d",
            ),
            (
                2000,
                "cf990d33fb9266d7151e0c83b244a50e232fc1d9d3ae5180733a6f474f258f0e",
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

#[cfg(test)]
mod tests_ocaml {
    use rand::Rng;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use super::*;

    // "add and retrieve an account"
    #[test]
    fn test_add_retrieve_account() {
        let mut db = Database::<V2>::create(4);

        let account = Account::rand();
        let location = db.create_account(account.id(), account.clone()).unwrap();
        let get_account = db.get(location.clone()).unwrap();

        assert_eq!(account, get_account);
    }

    // "accounts are atomic"
    #[test]
    fn test_accounts_are_atomic() {
        let mut db = Database::<V2>::create(4);

        let account = Account::rand();
        let location: Address = db
            .create_account(account.id(), account.clone())
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
        account1.balance = 100;

        let location1 = db
            .get_or_create_account(account1.id(), account1.clone())
            .unwrap();

        let mut account2 = account1;
        account2.balance = 200;

        let location2 = db
            .get_or_create_account(account2.id(), account2.clone())
            .unwrap();

        let addr1: Address = location1.clone();
        let addr2: Address = location2.clone();

        assert_eq!(addr1, addr2);
        assert!(matches!(location2, GetOrCreated::Existed(_)));
        assert_ne!(db.get(location1.addr()).unwrap(), account2);
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
                .map(|addr| (addr, Account::rand()))
                .collect::<Vec<_>>();

            let merkle_root1 = db.merkle_root();
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
            accounts.push((next_addr.clone(), Account::rand()));
            addr = next_addr;
        }

        db.set_batch_accounts(&accounts);

        for (addr, account) in &accounts {
            let account_id = account.id();
            let location = db.location_of_account(&account_id).unwrap();
            let queried_account = db.get(location.clone()).unwrap();

            assert_eq!(*addr, location);
            assert_eq!(*account, queried_account);
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
            let accounts = children.map(|_| Account::rand()).collect::<Vec<_>>();

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
            assert_eq!(account, indexed_account);
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
            let account = Account::rand();
            let index = rand::thread_rng().gen_range(0..NACCOUNTS);
            let index = AccountIndex(index as u64);

            db.set_at_index(index.clone(), account.clone()).unwrap();
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
            let account = Account::rand();
            accounts.push(account.clone());
            db.get_or_create_account(account.id(), account).unwrap();
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
            total_balance += account.balance as u128;
            db.get_or_create_account(account.id(), account).unwrap();
        }

        let retrieved = db.fold(0u128, |acc, account| acc + account.balance as u128);
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
                total_balance += account.balance as u128;
                last_id = account.id();
            }
            db.get_or_create_account(account.id(), account).unwrap();
        }

        let retrieved = db.fold_until(0u128, |mut acc, account| {
            acc += account.balance as u128;

            if account.id() != last_id {
                ControlFlow::Continue(acc)
            } else {
                ControlFlow::Break(acc)
            }
        });

        assert_eq!(total_balance, retrieved);
    }
}
