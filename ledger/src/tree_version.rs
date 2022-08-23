use std::{fmt::Debug, hash::Hash};

use mina_hasher::Fp;

use crate::account::{get_legacy_hash_of, Account, AccountLegacy, TokenId, TokenIdLegacy};

pub trait TreeVersion {
    type Account: Debug + Clone;
    type TokenId: Debug + Clone + Hash + PartialEq;

    fn hash_node(depth: usize, left: Fp, right: Fp) -> Fp;
    fn hash_leaf(leaf: &Self::Account) -> Fp;
    fn empty_hash_at_depth(depth: usize) -> Fp;
}

#[derive(Clone)]
pub struct V1;

#[derive(Clone)]
pub struct V2;

impl TreeVersion for V2 {
    type Account = Account;
    type TokenId = TokenId;

    fn hash_node(depth: usize, left: Fp, right: Fp) -> Fp {
        let param = format!("CodaMklTree{:03}", depth);

        crate::hash::hash_with_kimchi(param.as_str(), &[left, right])
    }

    fn hash_leaf(leaf: &Self::Account) -> Fp {
        leaf.hash()
    }

    fn empty_hash_at_depth(depth: usize) -> Fp {
        // let now = std::time::Instant::now();

        let res = (0..depth).fold(Account::empty().hash(), |prev_hash, depth| {
            Self::hash_node(depth, prev_hash, prev_hash)
        });
        // println!("empty_hash_at_depth={:?} {:?}", depth, now.elapsed());

        res
    }
}

impl TreeVersion for V1 {
    type Account = AccountLegacy;
    type TokenId = TokenIdLegacy;

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
        (0..depth).fold(account_empty_legacy_hash(), |prev_hash, depth| {
            Self::hash_node(depth, prev_hash, prev_hash)
        })
    }
}

pub fn account_empty_legacy_hash() -> Fp {
    get_legacy_hash_of((), &AccountLegacy::empty())
}
