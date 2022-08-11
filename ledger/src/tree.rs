use std::fmt::Debug;

use crate::{
    address::{Address, AddressIterator, Direction},
    tree_version::TreeVersion,
};
use mina_hasher::Fp;

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
    account: Box<T::Account>,
}

#[derive(Debug)]
pub struct Database<T: TreeVersion> {
    root: Option<NodeOrLeaf<T>>,
    depth: u8,
    last_location: Option<Address>,
    naccounts: usize,
}

impl<T: TreeVersion> NodeOrLeaf<T> {
    fn add_account_on_path(node_or_leaf: &mut Self, account: T::Account, path: AddressIterator) {
        let mut node_or_leaf = node_or_leaf;

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
            account: Box::new(account),
        });
    }

    fn hash(&self, depth: Option<usize>) -> Fp {
        let node = match self {
            NodeOrLeaf::Node(node) => node,
            NodeOrLeaf::Leaf(leaf) => {
                return T::hash_leaf(&*leaf.account);
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
}

#[derive(Debug, PartialEq, Eq)]
pub enum DatabaseError {
    OutOfLeaves,
}

impl<T: TreeVersion> Database<T> {
    pub fn create(depth: u8) -> Self {
        assert!((1..0xfe).contains(&depth));

        Self {
            depth,
            root: None,
            last_location: None,
            naccounts: 0,
        }
    }

    pub fn create_account(
        &mut self,
        _account_id: (),
        account: T::Account,
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
        NodeOrLeaf::add_account_on_path(root, account, path_iter);

        self.last_location = Some(location.clone());
        self.naccounts += 1;

        Ok(location)
    }

    pub fn root_hash(&self) -> Fp {
        println!("naccounts={:?}", self.naccounts);
        match self.root.as_ref() {
            Some(root) => root.hash(Some(self.depth as usize - 1)),
            None => T::empty_hash_at_depth(self.depth as usize),
        }
    }

    #[cfg(test)]
    fn naccounts(&self) -> usize {
        let mut naccounts = 0;

        if let Some(root) = self.root.as_ref() {
            self.naccounts_recursive(root, &mut naccounts)
        };

        naccounts
    }

    #[cfg(test)]
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
                db.create_account((), Account::create()).unwrap();
            }

            let naccounts = db.naccounts();
            assert_eq!(naccounts, two.pow(depth as u32));

            assert_eq!(
                db.create_account((), Account::create()).unwrap_err(),
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

    /// An empty tree produces the same hash than a tree full of empty accounts
    #[test]
    fn test_root_hash_v2() {
        let mut db = Database::<V2>::create(4);
        for _ in 0..16 {
            db.create_account((), Account::empty()).unwrap();
        }
        assert_eq!(
            db.create_account((), Account::empty()).unwrap_err(),
            DatabaseError::OutOfLeaves
        );
        let hash = db.root_hash();
        println!("ROOT_HASH={:?}", hash.to_string());
        assert_eq!(
            hash.to_hex(),
            "169bada2f4bb2ea2b8189f47cf2b665e3e0fb135233242ae1b52794eb3fe7924"
        );

        let mut db = Database::<V2>::create(4);
        for _ in 0..1 {
            db.create_account((), Account::empty()).unwrap();
        }
        let hash = db.root_hash();
        assert_eq!(
            hash.to_hex(),
            "169bada2f4bb2ea2b8189f47cf2b665e3e0fb135233242ae1b52794eb3fe7924"
        );

        let db = Database::<V2>::create(4);
        let hash = db.root_hash();
        assert_eq!(
            hash.to_hex(),
            "169bada2f4bb2ea2b8189f47cf2b665e3e0fb135233242ae1b52794eb3fe7924"
        );
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
