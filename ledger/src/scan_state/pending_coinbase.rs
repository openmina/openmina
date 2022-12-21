use std::{collections::HashMap, marker::PhantomData};

use ark_ff::Zero;
use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::{hash_noinputs, hash_with_kimchi, Inputs};

use super::{
    currency::{Amount, Magnitude},
    transaction_logic::Coinbase,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

    pub enum Action {
        UpdateNone,
        UpdateOne,
        UpdateTwoCoinbaseInFirst,
        UpdateTwoCoinbaseInSecond,
    }

    pub struct Update {
        action: Action,
        coinbase_amount: Amount,
    }

    impl Update {
        fn genesis() -> Self {
            Self {
                action: Action::UpdateNone,
                coinbase_amount: Amount::zero(),
            }
        }
    }
}

struct Stack {
    data: CoinbaseStack,
    state: StateStack,
}

struct PendingCoinbase {
    tree: (), // MerkleTree
    pos_list: Vec<StackId>,
    new_pos: StackId,
}

pub mod merkle_tree {
    use super::*;

    #[derive(Clone, Debug)]
    struct Addr(u64);

    enum Path<H> {
        Left(H),
        Right(H),
    }

    struct MiniMerkleTree<K, V> {
        values: Vec<V>,
        indexes: HashMap<K, Addr>,
    }

    impl<K, V> MiniMerkleTree<K, V>
    where
        K: Eq + std::hash::Hash,
    {
        fn create(depth: usize) -> Self {
            assert!(depth < u64::BITS as usize); // Less than `Addr` nbits

            let max_values = 2u64.pow(depth as u32) as usize;

            Self {
                values: Vec::with_capacity(max_values),
                indexes: HashMap::new(),
            }
        }

        fn of_hash() -> Self {
            todo!()
        }

        fn get_exn(&self, addr: Addr) -> &V {
            self.values.get(addr.0 as usize).unwrap()
        }

        fn set_exn(&mut self, addr: Addr, value: V) {
            use std::cmp::Ordering::*;

            let index = addr.0 as usize;

            match index.cmp(&self.values.len()) {
                Less => {
                    self.values[addr.0 as usize] = value;
                }
                Equal => self.values.push(value),
                Greater => panic!("wrong use of `set_exn`"),
            }
        }

        fn find_index_exn(&self, key: K) -> Addr {
            self.indexes.get(&key).cloned().unwrap()
        }

        fn path_exn(&self, addr: Addr) -> Self {
            todo!()
        }

        // fn add_path() -> Self {
        //     todo!()
        // }

        fn merkle_root() -> Self {
            todo!()
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
