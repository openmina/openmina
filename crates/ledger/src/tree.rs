use crate::{
    address::Address,
    base::AccountIndex,
    tree_version::{TreeVersion, V2},
};
use mina_curves::pasta::Fp;
use once_cell::sync::Lazy;
use std::{collections::BTreeMap, fmt::Debug, sync::Mutex};

#[derive(Clone, Debug)]
struct Leaf<T: TreeVersion> {
    account: Option<Box<T::Account>>,
}

#[derive(PartialEq)]
pub struct HashesMatrix {
    /// 2 dimensions matrix
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
        f.debug_struct("HashesMatrix")
            .field("matrix_len", &self.matrix.len())
            .field("nhashes", &self.nhashes)
            .finish()
    }
}

impl HashesMatrix {
    pub fn new(ledger_depth: usize) -> Self {
        Self {
            matrix: BTreeMap::new(),
            ledger_depth,
            empty_hashes: vec![None; ledger_depth],
            nhashes: 0,
        }
    }

    pub fn get(&self, addr: &Address) -> Option<&Fp> {
        let linear: u64 = addr.to_linear_index();
        self.matrix.get(&linear)
    }

    pub fn set(&mut self, addr: &Address, hash: Fp) {
        let linear: u64 = addr.to_linear_index();
        let old = self.matrix.insert(linear, hash);
        assert!(old.is_none());
        self.nhashes += 1;
    }

    /// Do not use directly. Used to forcefully reconstructing the hashes
    /// matrix from raw data.
    pub fn set_raw_index(&mut self, idx: u64, hash: Fp) {
        let old = self.matrix.insert(idx, hash);
        assert!(old.is_none());
        self.nhashes += 1;
    }

    pub fn remove(&mut self, addr: &Address) {
        let linear: u64 = addr.to_linear_index();
        self.remove_at_index(linear);
    }

    fn remove_at_index(&mut self, linear: u64) {
        let old = self.matrix.remove(&linear);
        if old.is_some() {
            self.nhashes -= 1;
        }
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
        *self = Self {
            matrix: BTreeMap::new(),
            ledger_depth,
            empty_hashes: vec![None; ledger_depth],
            nhashes: 0,
        }
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

    pub fn get_raw_inner_hashes(&self) -> Vec<(u64, Fp)> {
        self.matrix.clone().into_iter().collect()
    }

    pub fn set_raw_inner_hashes(&mut self, hashes: Vec<(u64, Fp)>) {
        for (idx, hash) in hashes {
            self.set_raw_index(idx, hash);
        }
    }
}

static HASH_EMPTIES: Lazy<Mutex<Vec<Fp>>> = Lazy::new(|| {
    /// This value needs to be changed when the tree's height change
    const RANGE_HEIGHT: std::ops::Range<usize> = 0..36;

    Mutex::new((RANGE_HEIGHT).map(V2::empty_hash_at_height).collect())
});
