use std::{collections::HashMap, marker::PhantomData};

use ark_ff::Zero;
use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::{hash_noinputs, hash_with_kimchi, Inputs};

use super::{
    currency::{Amount, Magnitude},
    transaction_logic::Coinbase,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StackId(u64);

impl StackId {
    pub fn incr_by_one(&self) -> Result<Self, String> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or_else(|| "Stack_id overflow".to_string())
    }

    pub fn zero() -> Self {
        Self(0)
    }
}

struct CoinbaseData(CompressedPubKey, Amount);

impl CoinbaseData {
    pub fn empty() -> Self {
        Self(CompressedPubKey::empty(), Amount::zero())
    }

    pub fn of_coinbase(cb: Coinbase) -> Self {
        Self(cb.receiver, cb.amount)
    }

    pub fn genesis() -> Self {
        Self::empty()
    }
}

#[derive(Debug)]
struct CoinbaseStack(Fp);

impl CoinbaseStack {
    pub fn push(&self, cb: Coinbase) -> Self {
        let coinbase = CoinbaseData::of_coinbase(cb);

        let mut inputs = Inputs::new();

        // cb.public_key
        inputs.append_field(coinbase.0.x);
        inputs.append_bool(coinbase.0.is_odd);

        // cb.amount
        inputs.append_u64(coinbase.1.as_u64());

        // self
        inputs.append_field(self.0);

        let hash = hash_with_kimchi("CoinbaseStack", &inputs.to_fields());
        Self(hash)
    }

    pub fn empty() -> Self {
        Self(hash_noinputs("CoinbaseStack"))
    }
}

type StackHash = Fp;

#[derive(Debug)]
struct StateStack {
    init: StackHash,
    curr: StackHash,
}

impl StateStack {
    fn push(&self, state_body_hash: Fp) -> Self {
        let mut inputs = Inputs::new();

        inputs.append_field(self.curr);
        inputs.append_field(state_body_hash);

        let hash = hash_with_kimchi("MinaProtoState", &inputs.to_fields());

        Self {
            init: self.init,
            curr: hash,
        }
    }

    fn empty() -> Self {
        Self {
            init: Fp::zero(),
            curr: Fp::zero(),
        }
    }

    fn create(init: StackHash) -> Self {
        Self { init, curr: init }
    }
}

pub mod update {
    use crate::scan_state::currency::{Amount, Magnitude};

    #[derive(Debug)]
    pub enum Action {
        None,
        One,
        TwoCoinbaseInFirst,
        TwoCoinbaseInSecond,
    }

    #[derive(Debug)]
    pub struct Update {
        action: Action,
        coinbase_amount: Amount,
    }

    impl Update {
        fn genesis() -> Self {
            Self {
                action: Action::None,
                coinbase_amount: Amount::zero(),
            }
        }
    }
}

#[derive(Debug)]
struct Stack {
    data: CoinbaseStack,
    state: StateStack,
}

impl Stack {
    fn empty() -> Self {
        Self {
            data: CoinbaseStack::empty(),
            state: StateStack::empty(),
        }
    }
}

struct PendingCoinbase {
    tree: merkle_tree::MiniMerkleTree<StackId, Stack, StackHasher>,
    pos_list: Vec<StackId>,
    new_pos: StackId,
}

struct StackHasher;

impl merkle_tree::TreeHasher<Stack> for StackHasher {
    fn hash_value(value: &Stack) -> Fp {
        let mut inputs = Inputs::new();

        inputs.append_field(value.data.0);

        inputs.append_field(value.state.init);
        inputs.append_field(value.state.curr);

        hash_with_kimchi("CoinbaseStack", &inputs.to_fields())
    }

    fn merge_hash(depth: usize, left: Fp, right: Fp) -> Fp {
        let param = format!("MinaCbMklTree{:03}", depth);

        crate::hash::hash_with_kimchi(param.as_str(), &[left, right])
    }

    fn empty_value() -> Stack {
        Stack::empty()
    }
}

/// Keep it a bit generic, in case we need a merkle tree somewhere else
pub mod merkle_tree {
    use crate::{Address, AddressIterator, Direction, HashesMatrix, MerklePath};

    use super::*;

    pub trait TreeHasher<V> {
        fn hash_value(value: &V) -> Fp;
        fn empty_value() -> V;
        fn merge_hash(depth: usize, left: Fp, right: Fp) -> Fp;
    }

    pub struct MiniMerkleTree<K, V, H> {
        values: Vec<V>,
        indexes: HashMap<K, Address>,
        hashes_matrix: HashesMatrix,
        depth: usize,
        _hasher: PhantomData<H>,
    }

    impl<K, V, H> MiniMerkleTree<K, V, H>
    where
        K: Eq + std::hash::Hash,
        H: TreeHasher<V>,
    {
        pub fn create(depth: usize) -> Self {
            assert!(depth < u64::BITS as usize); // Less than `Address` nbits

            let max_values = 2u64.pow(depth as u32) as usize;

            Self {
                values: Vec::with_capacity(max_values),
                indexes: HashMap::new(),
                depth,
                hashes_matrix: HashesMatrix::new(depth),
                _hasher: PhantomData,
            }
        }

        fn get(&self, addr: Address) -> Option<&V> {
            assert_eq!(addr.length(), self.depth);

            let index = addr.to_index().0 as usize;
            self.values.get(index)
        }

        pub fn get_exn(&self, addr: Address) -> &V {
            self.get(addr).unwrap()
        }

        pub fn set_exn(&mut self, addr: Address, value: V) {
            use std::cmp::Ordering::*;

            assert_eq!(addr.length(), self.depth);
            let index = addr.to_index().0 as usize;

            match index.cmp(&self.values.len()) {
                Less => self.values[index] = value,
                Equal => self.values.push(value),
                Greater => panic!("wrong use of `set_exn`"),
            }

            self.hashes_matrix.invalidate_hashes(addr.to_index());
        }

        pub fn find_index_exn(&self, key: K) -> Address {
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
                Some(value) => H::hash_value(value),
                None => H::hash_value(&H::empty_value()),
            };

            self.hashes_matrix.set(&addr, hash);

            hash
        }

        fn get_node_hash(&mut self, addr: &Address, left: Fp, right: Fp) -> Fp {
            if let Some(hash) = self.hashes_matrix.get(addr) {
                return *hash;
            };

            let depth_in_tree = self.depth - addr.length();

            let hash = H::merge_hash(depth_in_tree - 1, left, right);
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

    //   [%%define_locally
    //   M.
    //     ( of_hash
    //     , get_exn
    //     , path_exn
    //     , set_exn
    //     , find_index_exn
    //     , add_path
    //     , merkle_root )]
    // end
}

#[cfg(test)]
mod tests {
    use crate::FpExt;

    use super::{merkle_tree::MiniMerkleTree, *};

    #[test]
    fn test_merkle_tree() {
        {
            const DEPTH: usize = 3;
            let mut tree = MiniMerkleTree::<StackId, Stack, StackHasher>::create(DEPTH);
            let merkle_root = tree.merkle_root();
            assert_eq!(
                merkle_root.to_decimal(),
                "9939061863620980199451530646711695641079091335264396436068661296746064363179"
            );
        }

        {
            const DEPTH: usize = 5;
            let mut tree = MiniMerkleTree::<StackId, Stack, StackHasher>::create(DEPTH);
            let merkle_root = tree.merkle_root();
            assert_eq!(
                merkle_root.to_decimal(),
                "25504365445533103805898245102289650498571312278321176071043666991586378788150"
            );
        }
    }
}
