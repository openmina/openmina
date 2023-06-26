use std::{collections::BTreeMap, fmt::Debug, sync::Mutex};

use crate::{
    address::Address,
    base::AccountIndex,
    tree_version::{TreeVersion, V2},
};
use mina_hasher::Fp;
use once_cell::sync::Lazy;

#[derive(Clone, Debug)]
struct Leaf<T: TreeVersion> {
    account: Option<Box<T::Account>>,
}

#[derive(PartialEq)]
pub struct HashesMatrix {
    /// 2 dimensions matrix
    // matrix: Vec<Option<Fp>>,
    matrix: BTreeMap<u64, Fp>,
    empty_hashes: Vec<Option<Fp>>,
    ledger_depth: usize,
    nhashes: usize,
}

impl Clone for HashesMatrix {
    fn clone(&self) -> Self {
        Self {
            matrix: self.matrix.clone(),
            empty_hashes: self.empty_hashes.clone(),
            ledger_depth: self.ledger_depth,
            nhashes: self.nhashes,
        }
    }
}

impl Debug for HashesMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // const SPACES: &[usize] = &[
        //     0, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192,
        // ];

        // let mut s = String::with_capacity(self.matrix.len() * 2);
        // let mut spaces = SPACES;
        // let mut current = 0;
        // let naccounts = 2u64.pow(self.ledger_depth as u32) as usize;

        // for h in self.matrix.iter() {
        //     let c = if h.is_some() { 'I' } else { '0' };
        //     s.push(c);

        //     if current == spaces[0] && current != naccounts {
        //         s.push(' ');
        //         current = 0;
        //         spaces = &spaces[1..];
        //     }

        //     current += 1;
        // }

        f.debug_struct("HashesMatrix")
            // .field("matrix", &s)
            .field("matrix_len", &self.matrix.len())
            // .field("real_matrix", &real)
            // .field("empty_hashes", &self.empty_hashes)
            // .field("ledger_depth", &self.ledger_depth)
            .field("nhashes", &self.nhashes)
            // .field("capacity", &self.matrix.capacity())
            .finish()
    }
}

impl HashesMatrix {
    pub fn new(ledger_depth: usize) -> Self {
        // let capacity = 2 * 2usize.pow(ledger_depth as u32) - 1;

        Self {
            // matrix: vec![None; capacity],
            matrix: BTreeMap::new(),
            ledger_depth,
            empty_hashes: vec![None; ledger_depth],
            nhashes: 0,
        }
    }

    pub fn get(&self, addr: &Address) -> Option<&Fp> {
        let linear = addr.to_linear_index();

        // self.matrix.get(linear)?.as_ref()
        let linear: u64 = linear.try_into().unwrap();
        self.matrix.get(&linear)
    }

    pub fn set(&mut self, addr: &Address, hash: Fp) {
        let linear = addr.to_linear_index();

        // if self.matrix.len() <= linear {
        //     self.matrix.resize(linear + 1, None);
        // }

        // assert!(self.matrix[linear].is_none());
        // self.matrix[linear] = Some(hash);
        let linear: u64 = linear.try_into().unwrap();
        let old = self.matrix.insert(linear, hash);
        assert!(old.is_none());
        self.nhashes += 1;
    }

    pub fn remove(&mut self, addr: &Address) {
        let linear = addr.to_linear_index();
        self.remove_at_index(linear);
    }

    fn remove_at_index(&mut self, index: usize) {
        let linear: u64 = index.try_into().unwrap();
        let old = self.matrix.remove(&linear);
        if old.is_some() {
            self.nhashes -= 1;
        }

        // let hash = match self.matrix.get_mut(index) {
        //     Some(hash) => hash,
        //     None => return,
        // };

        // if hash.is_some() {
        //     self.nhashes -= 1;
        //     *hash = None;
        // }
    }

    pub(super) fn transfert_hashes(&mut self, hashes: HashesMatrix) {
        for (index, hash) in hashes.matrix {
            let old = self.matrix.insert(index, hash);
            if old.is_none() {
                self.nhashes += 1;
            }
        }
    }

    pub fn invalidate_hashes(&mut self, account_index: AccountIndex) {
        let mut addr = Address::from_index(account_index, self.ledger_depth);

        loop {
            let index = addr.to_linear_index();
            self.remove_at_index(index);
            addr = match addr.parent() {
                Some(addr) => addr,
                None => break,
            }
        }
    }

    pub fn empty_hash_at_height(&mut self, height: usize) -> Fp {
        if let Some(Some(hash)) = self.empty_hashes.get(height) {
            return *hash;
        };

        // If `depth` is out of bound, see `HASH_EMPTIES`
        let hash = HASH_EMPTIES.lock().unwrap()[height];
        self.empty_hashes[height] = Some(hash);

        hash
    }

    pub fn clear(&mut self) {
        let ledger_depth = self.ledger_depth;
        // let capacity = 2 * 2usize.pow(ledger_depth as u32) - 1;

        *self = Self {
            // matrix: vec![None; capacity],
            matrix: BTreeMap::new(),
            ledger_depth,
            empty_hashes: vec![None; ledger_depth],
            nhashes: 0,
        }
        // self.matrix.clear();
        // self.empty_hashes.clear();
        // self.nhashes = 0;
    }

    pub fn take(&mut self) -> Self {
        let Self {
            matrix,
            empty_hashes,
            ledger_depth,
            nhashes,
        } = self;

        Self {
            matrix: std::mem::take(matrix),
            empty_hashes: std::mem::take(empty_hashes),
            ledger_depth: *ledger_depth,
            nhashes: *nhashes,
        }
    }
}

static HASH_EMPTIES: Lazy<Mutex<Vec<Fp>>> = Lazy::new(|| {
    /// This value needs to be changed when the tree's depth change
    const RANGE_DEPTH: std::ops::Range<usize> = 0..36;

    Mutex::new((RANGE_DEPTH).map(V2::empty_hash_at_height).collect())
});
