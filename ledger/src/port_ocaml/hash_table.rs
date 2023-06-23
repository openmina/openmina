use std::collections::BTreeMap;

use super::hash::OCamlHash;

/// https://github.com/janestreet/base/blob/v0.14/src/hashtbl.ml
pub struct HashTable<K, V> {
    /// Note: OCaml uses AVL trees, but we just need an ordered map
    table: Vec<BTreeMap<K, V>>,
    length: usize,
}

impl<K: std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for HashTable<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let table: Vec<_> = self.table.iter().map(BTreeMap::len).collect();

        f.debug_struct("HashTable")
            .field("table", &table)
            .field("length", &self.length)
            .finish()
    }
}

impl<K: Ord + OCamlHash + Clone, V: Clone> HashTable<K, V> {
    pub fn create() -> Self {
        let size = 1u64.next_power_of_two();

        let table = (0..size).map(|_| BTreeMap::new()).collect();

        Self { table, length: 0 }
    }

    fn slot(&self, key: &K) -> usize {
        let hash = key.ocaml_hash();
        let hash = hash as usize;
        hash & (self.table.len() - 1)
    }

    fn find(&self, key: &K) -> Option<&V> {
        let slot = self.slot(key);
        let tree = &self.table[slot];

        tree.get(key)
    }

    fn set_impl(&mut self, key: K, data: V) {
        let slot = self.slot(&key);
        let tree = &mut self.table[slot];

        let old = tree.insert(key, data);
        if old.is_none() {
            self.length += 1;
        }
    }

    fn set(&mut self, key: K, data: V) {
        self.set_impl(key, data);
        self.maybe_resize_table();
    }

    fn maybe_resize_table(&mut self) {
        let len = self.table.len();
        let should_grow = self.length > len;

        if !should_grow {
            return;
        }

        let new_array_length = len * 2;
        if new_array_length <= len {
            return;
        }

        let new_table = (0..new_array_length).map(|_| BTreeMap::new()).collect();
        let old_table = std::mem::replace(&mut self.table, new_table);
        self.length = 0;

        for (key, value) in old_table.into_iter().flat_map(BTreeMap::into_iter) {
            self.set_impl(key, value);
        }
    }

    /// https://github.com/janestreet/base/blob/v0.14/src/hashtbl.ml#L450
    pub fn update<F>(&mut self, key: K, fun: F)
    where
        F: FnOnce(Option<&V>) -> V,
    {
        let value = fun(self.find(&key));
        self.set(key, value)
    }
}

impl<K: Clone, V: Clone> HashTable<K, V> {
    /// https://github.com/janestreet/base/blob/v0.14/src/hashtbl.ml#L259-L281
    pub fn to_alist(&self) -> Vec<(K, V)> {
        self.table
            .iter()
            .flat_map(BTreeMap::iter)
            .map(|(key, value)| (key.clone(), value.clone()))
            .rev() // rev because OCaml builds the list in reverse: `(key, data) :: list`
            .collect()
    }
}
