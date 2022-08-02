use std::{borrow::Cow, fmt::Debug};

use crate::{
    account::{get_legacy_hash_of, Account, AccountLegacy},
    address::{Address, AddressIterator, Direction},
};
use mina_hasher::Fp;

pub trait TreeVersion {
    type Account: Debug + Clone;

    fn hash_node(depth: usize, left: Fp, right: Fp) -> Fp;
    fn hash_leaf(leaf: &Self::Account) -> Fp;
    fn empty_hash_at_depth(depth: usize) -> Fp;
}

struct V1;
struct V2;

impl TreeVersion for V2 {
    type Account = Account;

    fn hash_node(depth: usize, left: Fp, right: Fp) -> Fp {
        let param = format!("CodaMklTree{:03}", depth);

        crate::hash::hash_with_kimchi(Cow::Owned(param), &[left, right])
    }

    fn hash_leaf(leaf: &Self::Account) -> Fp {
        leaf.hash()
    }

    fn empty_hash_at_depth(depth: usize) -> Fp {
        (0..depth).fold(Account::empty().hash(), |prev_hash, depth| {
            Self::hash_node(depth, prev_hash, prev_hash)
        })
    }
}

impl TreeVersion for V1 {
    type Account = AccountLegacy;

    fn hash_node(depth: usize, left: Fp, right: Fp) -> Fp {
        use mina_hasher::{create_legacy, Hashable, Hasher, ROInput};

        #[derive(Clone)]
        struct TwoHashes(Fp, Fp);

        impl Hashable for TwoHashes {
            type D = u32; // depth

            fn to_roinput(&self) -> ROInput {
                let mut roi = ROInput::new();
                roi.append_field(self.0);
                roi.append_field(self.1);
                roi
            }

            fn domain_string(depth: Self::D) -> Option<String> {
                Some(format!("CodaMklTree{:03}", depth))
            }
        }

        let mut hasher = create_legacy::<TwoHashes>(depth as u32);
        hasher.update(&TwoHashes(left, right));
        hasher.digest()
    }

    fn hash_leaf(leaf: &Self::Account) -> Fp {
        use mina_hasher::{create_legacy, Hasher};

        let mut hasher = create_legacy::<AccountLegacy>(());
        hasher.update(leaf);
        hasher.digest()
    }

    fn empty_hash_at_depth(depth: usize) -> Fp {
        (0..depth).fold(account_empty_hash(), |prev_hash, depth| {
            Self::hash_node(depth, prev_hash, prev_hash)
        })
    }
}

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
struct Database<T: TreeVersion> {
    root: Option<NodeOrLeaf<T>>,
    depth: u8,
    last_location: Option<Address>,
}

impl<T: TreeVersion> NodeOrLeaf<T> {
    fn add_to_path(node_or_leaf: &mut Self, account: T::Account, path: AddressIterator) {
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

    fn hash(&self, depth: usize) -> Fp {
        let node = match self {
            NodeOrLeaf::Node(node) => node,
            NodeOrLeaf::Leaf(leaf) => {
                return T::hash_leaf(&*leaf.account);
                // return get_hash_of((), &*leaf.account);
            }
        };

        let left_hash = match &node.left {
            Some(left) => left.hash(depth - 1),
            None => T::empty_hash_at_depth(depth),
        };

        let right_hash = match &node.right {
            Some(right) => right.hash(depth - 1),
            None => T::empty_hash_at_depth(depth),
        };

        let hash = T::hash_node(depth, left_hash, right_hash);

        println!("depth={:?} HASH={:?}", depth, hash.to_string(),);

        // println!("depth={:?} HASH={:?} left={:?} right={:?}",
        //          depth,
        //          hash.to_string(),
        //          left_hash.to_string(),
        //          right_hash.to_string(),
        // );

        hash
    }
}

#[derive(Debug, PartialEq, Eq)]
enum DatabaseError {
    OutOfLeaves,
}

fn account_empty_hash() -> Fp {
    get_legacy_hash_of((), &AccountLegacy::empty())
}

impl<T: TreeVersion> Database<T> {
    fn create(depth: u8) -> Self {
        assert!((1..0xfe).contains(&depth));

        Self {
            depth,
            root: None,
            last_location: None,
        }
    }

    fn create_account(
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
        NodeOrLeaf::add_to_path(root, account, path_iter);

        self.last_location = Some(location.clone());

        Ok(location)
    }

    fn root_hash(&self) -> Fp {
        match self.root.as_ref() {
            Some(root) => root.hash(self.depth as usize - 1),
            None => T::empty_hash_at_depth(self.depth as usize),
        }
    }

    fn naccounts(&self) -> usize {
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

#[cfg(test)]
mod tests {
    use o1_utils::FieldHelpers;

    use crate::account::AccountLegacy;

    use super::*;

    #[test]
    fn test_db() {
        let two: usize = 2;

        for depth in 2..17 {
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
    fn test_hash_empty() {
        let account_empty_hash = account_empty_hash();
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

    /// An empty tree produces the same hash than a tree full of empty accounts
    #[test]
    fn test_root_hash() {
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
