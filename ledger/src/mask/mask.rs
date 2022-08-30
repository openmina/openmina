use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::{
    account::{Account, AccountId, TokenId},
    address::{Address, AddressIterator},
    base::{AccountIndex, BaseLedger, GetOrCreated, Uuid},
    tree::{Database, DatabaseError},
    tree_version::{TreeVersion, V2},
};

struct MaskInner {
    parent: Option<Mask>,
    inner: Database<V2>,
    owning_account: HashMap<AccountIndex, Account>,
    token_to_account: HashMap<TokenId, AccountId>,
    id_to_addr: HashMap<AccountId, Address>,
    last_location: Option<Address>,
    first_location_in_mask: Option<Address>,
    depth: u8,
    naccounts: usize,
    /// All childs of this mask
    childs: HashMap<Uuid, Mask>,
}

impl std::fmt::Debug for MaskInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaskInner")
            .field("uuid", &self.inner.get_uuid())
            .field("parent", &self.parent.as_ref().map(|p| p.uuid()))
            .field("owning_account", &self.owning_account.len())
            .field("token_to_account", &self.token_to_account.len())
            .field("id_to_addr", &self.id_to_addr.len())
            .field("last_location", &self.last_location)
            .field("first_location_in_mask", &self.first_location_in_mask)
            .field("depth", &self.depth)
            .field("naccounts", &self.naccounts)
            .field("childs", &self.childs.len())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct Mask {
    // Using a mutex for now but this can be replaced with a RefCell
    inner: Arc<Mutex<MaskInner>>,
}

#[derive(Debug)]
pub enum UnregisterBehavior {
    Check,
    Recursive,
    IPromiseIAmReparentingThisMask,
}

impl Mask {
    fn with<F, R>(&self, fun: F) -> R
    where
        F: FnOnce(&mut MaskInner) -> R,
    {
        let mut inner = self.inner.lock().expect("lock failed");
        fun(&mut inner)
    }

    pub fn new_root(db: Database<V2>) -> Self {
        let depth = db.depth();

        Self {
            inner: Arc::new(Mutex::new(MaskInner {
                parent: None,
                inner: db,
                owning_account: Default::default(),
                token_to_account: Default::default(),
                id_to_addr: Default::default(),
                last_location: None,
                depth,
                naccounts: 0,
                childs: HashMap::with_capacity(2),
                first_location_in_mask: None,
            })),
        }
    }

    pub fn new_child(depth: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(MaskInner {
                parent: None,
                inner: Database::<V2>::create(depth as u8), // TODO: Make it None
                owning_account: Default::default(),
                token_to_account: Default::default(),
                id_to_addr: Default::default(),
                last_location: None,
                depth: depth as u8,
                naccounts: 0,
                childs: HashMap::with_capacity(2),
                first_location_in_mask: None,
            })),
        }
    }

    pub fn is_attached(&self) -> bool {
        self.with(|this| this.parent.is_some())
    }

    pub fn set_parent(&self, parent: &Mask) {
        self.with(|this| {
            assert!(this.parent.is_none(), "mask is already attached");

            this.parent = Some(parent.clone());
        })
    }

    fn uuid(&self) -> Uuid {
        self.with(|this| this.inner.get_uuid())
    }

    /// Make `mask` a child of `self`
    pub fn register_mask(&self, mask: Mask) -> Mask {
        self.with(|this| {
            let old = this.childs.insert(mask.uuid(), mask.clone());
            assert!(old.is_none(), "mask is already registered");

            mask.set_parent(self);
            mask
        })
    }

    pub fn get_parent(&self) -> Option<Mask> {
        self.with(|this| this.parent.clone())
    }

    /// Detach this mask from its parent
    pub fn unregister_mask(&self, behavior: UnregisterBehavior) {
        use UnregisterBehavior::*;

        assert!(self.is_attached());
        let parent = self.get_parent().unwrap();

        let trigger_detach_signal = matches!(behavior, Check | Recursive);

        match behavior {
            Check => {
                assert!(
                    !self.children().is_empty(),
                    "mask has children that must be unregistered first"
                );
            }
            IPromiseIAmReparentingThisMask => (),
            Recursive => {
                for child in self.children() {
                    child.unregister_mask(Recursive);
                }
            }
        }

        let removed = parent.remove_child(&self);
        assert!(removed.is_some(), "Mask not a child of the parent");

        self.unset_parent(trigger_detach_signal);
    }

    pub fn unset_parent(&self, trigger_detach_signal: bool) {
        let parent = self.remove_parent();

        assert!(
            parent.is_some(),
            "unset_parent called on a non-attached mask"
        );

        if trigger_detach_signal {
            // TODO: Async.Ivar.fill_if_empty t.detached_parent_signal () ;
        }
    }

    ///              o
    ///             /
    ///            /
    ///   o --- o -
    ///   ^     ^  \
    ///  parent |   \
    ///        mask  o
    ///            children
    ///
    /// Removes the attached mask from its parent and attaches the children to the
    /// parent instead. Raises an exception if the merkle roots of the mask and the
    /// parent are not the same.
    pub fn remove_and_reparent(&self) {
        let parent = self.get_parent().expect("Mask doesn't have parent");
        parent
            .remove_child(self)
            .expect("Parent doesn't have this mask as child");

        // we can only reparent if merkle roots are the same
        assert_eq!(parent.merkle_root(), self.merkle_root());

        let children = self.children();

        for child in &children {
            child.unregister_mask(UnregisterBehavior::IPromiseIAmReparentingThisMask);
        }

        self.remove_parent();
        // self.unregister_mask(UnregisterBehavior::IPromiseIAmReparentingThisMask);

        for child in children {
            parent.register_mask(child);
        }

        // TODO: Self should be removed/unallocated
    }

    /// get hash from mask, if present, else from its parent
    pub fn get_hash(&self, addr: Address) -> Option<Fp> {
        self.get_inner_hash_at_addr(addr).ok()
    }

    /// commit all state to the parent, flush state locally
    pub fn commit(&self) {
        let mut parent = self.get_parent().expect("Mask doesn't have parent");
        assert_ne!(parent.uuid(), self.uuid());

        let old_root_hash = self.merkle_root();
        let depth = self.depth() as usize;

        let accounts = self.with(|this| {
            this.token_to_account.clear();
            this.id_to_addr.clear();
            std::mem::take(&mut this.owning_account)
        });

        for (index, account) in accounts {
            let addr = Address::from_index(index.clone(), depth);
            parent.set(addr, account);
        }

        // Parent merkle root after committing should be the same as the \
        // old one in the mask
        assert_eq!(old_root_hash, parent.merkle_root());
    }

    /// called when parent sets an account; update local state
    ///
    /// if the mask's parent sets an account, we can prune an entry in the mask
    /// if the account in the parent is the same in the mask *)
    pub fn parent_set_notify(&self, account: &Account) {
        assert!(self.is_attached());

        let account_id = account.id();

        let own_account = match self.with(|this| {
            this.id_to_addr
                .get(&account_id)
                .and_then(|addr| this.owning_account.get(&addr.to_index()))
                .cloned()
        }) {
            Some(own) => own,
            None => return,
        };

        if own_account != *account {
            // Do not delete our account if it is different than the parent one
            return;
        }

        self.remove_own_account(&[account_id]);
    }

    pub fn children(&self) -> Vec<Mask> {
        self.with(|this| this.childs.values().cloned().collect())
    }

    pub fn remove_parent(&self) -> Option<Mask> {
        self.with(|this| this.parent.take())
    }

    pub fn remove_child(&self, child: &Mask) -> Option<Mask> {
        let uuid = child.uuid();

        self.with(|this| this.childs.remove(&uuid))
    }

    pub fn depth(&self) -> u8 {
        self.with(|this| this.depth)
    }

    fn emulate_tree_to_get_hash(&self) -> Fp {
        let tree_depth = self.depth() as usize;
        let naccounts = self.num_accounts();
        let mut account_index = 0;

        self.emulate_recursive(0, tree_depth, &mut account_index, naccounts as u64)
    }

    fn emulate_tree_to_get_hash_at(&self, addr: Address) -> Fp {
        let tree_depth = self.depth() as usize;

        let current_depth = addr.length();

        let mut children = addr.iter_children(tree_depth);
        let naccounts = children.len();

        // First child
        let mut account_index = children.next().unwrap().to_index().0 as u64;

        self.emulate_recursive(
            current_depth,
            tree_depth,
            &mut account_index,
            naccounts as u64,
        )
    }

    fn emulate_recursive(
        &self,
        current_depth: usize,
        tree_depth: usize,
        account_index: &mut u64,
        naccounts: u64,
    ) -> Fp {
        if current_depth == tree_depth {
            let account_addr = Address::from_index(AccountIndex(*account_index), tree_depth);
            let account = match self.get(account_addr) {
                Some(account) => account,
                None => return V2::empty_hash_at_depth(0),
            };

            *account_index += 1;
            return account.hash();
        }

        let left_hash =
            self.emulate_recursive(current_depth + 1, tree_depth, account_index, naccounts);
        let right_hash = if *account_index < naccounts {
            self.emulate_recursive(current_depth + 1, tree_depth, account_index, naccounts)
        } else {
            V2::empty_hash_at_depth(tree_depth - current_depth)
        };

        V2::hash_node(tree_depth - current_depth, left_hash, right_hash)
    }

    fn remove_own_account(&self, ids: &[AccountId]) {
        self.with(|this| {
            let mut addrs = ids
                .iter()
                .map(|account_id| this.id_to_addr.remove(&account_id).unwrap())
                .collect::<Vec<_>>();
            addrs.sort_by(|a, b| a.to_index().cmp(&b.to_index()));

            for addr in addrs.iter().rev() {
                let account = this.owning_account.remove(&addr.to_index()).unwrap();
                this.token_to_account.remove(&account.token_id).unwrap();

                if this
                    .last_location
                    .as_ref()
                    .map(|last| last == addr)
                    .unwrap_or(false)
                {
                    this.last_location = addr.prev();
                }

                if this
                    .first_location_in_mask
                    .as_ref()
                    .map(|first| first == addr)
                    .unwrap_or(false)
                {
                    this.last_location = None;
                    this.first_location_in_mask = None;
                }

                this.naccounts -= 1;
            }
        });
    }

    /// For tests only, check if the address is in the mask, without checking parent
    #[cfg(test)]
    fn test_is_in_mask(&self, addr: &Address) -> bool {
        let index = addr.to_index();
        self.with(|this| match this.parent {
            Some(_) => this.owning_account.contains_key(&index),
            None => this.inner.get(addr.clone()).is_some(),
        })
    }
}

impl BaseLedger for Mask {
    fn to_list(&self) -> Vec<Account> {
        let num_accounts = self.num_accounts();
        let mut accounts = Vec::with_capacity(num_accounts);
        let depth = self.depth() as usize;

        for index in 0..num_accounts {
            let index = AccountIndex(index as u64);
            let addr = Address::from_index(index, depth);
            accounts.push(self.get(addr).unwrap_or_else(|| Account::empty()));
        }

        accounts
    }

    fn iter<F>(&self, fun: F)
    where
        F: FnMut(&Account),
    {
        let accounts = self.to_list();
        accounts.iter().for_each(fun)
    }

    fn fold<B, F>(&self, init: B, fun: F) -> B
    where
        F: FnMut(B, &Account) -> B,
    {
        let accounts = self.to_list();
        accounts.iter().fold(init, fun)
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
        let accounts = self.to_list();
        accounts.iter().fold(init, |accum, account| {
            if !ignoreds.contains(&account.id()) {
                fun(accum, account)
            } else {
                accum
            }
        })
    }

    fn fold_until<B, F>(&self, init: B, mut fun: F) -> B
    where
        F: FnMut(B, &Account) -> std::ops::ControlFlow<B, B>,
    {
        use std::ops::ControlFlow::*;

        let accounts = self.to_list();
        let mut accum = init;

        for account in &accounts {
            match fun(accum, account) {
                Continue(acc) => accum = acc,
                Break(acc) => {
                    accum = acc;
                    break;
                }
            }
        }

        accum
    }

    fn accounts(&self) -> HashSet<AccountId> {
        self.to_list()
            .into_iter()
            .map(|account| account.id())
            .collect()
    }

    fn token_owner(&self, token_id: TokenId) -> Option<AccountId> {
        if let Some(account_id) = self.with(|this| this.token_to_account.get(&token_id).cloned()) {
            return Some(account_id);
        };

        match self.get_parent() {
            Some(parent) => parent.token_owner(token_id),
            None => self.with(|this| this.inner.token_owner(token_id)),
        }
    }

    fn token_owners(&self) -> HashSet<AccountId> {
        // TODO: Not sure if it's the correct impl
        self.to_list()
            .into_iter()
            .map(|account| account.id())
            .collect()
    }

    fn tokens(&self, public_key: CompressedPubKey) -> HashSet<TokenId> {
        let mut set = HashSet::with_capacity(1024);

        for account in self.to_list() {
            if account.public_key == public_key {
                set.insert(account.token_id);
            }
        }

        set
    }

    fn location_of_account(&self, account_id: &AccountId) -> Option<Address> {
        if let Some(addr) = self.with(|this| this.id_to_addr.get(&account_id).cloned()) {
            return Some(addr);
        }

        match self.get_parent() {
            Some(parent) => parent.location_of_account(account_id),
            None => self.with(|this| this.inner.location_of_account(account_id)),
        }
    }

    fn location_of_account_batch(
        &self,
        account_ids: &[AccountId],
    ) -> Vec<(AccountId, Option<Address>)> {
        account_ids
            .iter()
            .map(|account_id| {
                let addr = self.location_of_account(account_id);
                (account_id.clone(), addr)
            })
            .collect()
    }

    fn get_or_create_account(
        &mut self,
        account_id: AccountId,
        account: Account,
    ) -> Result<GetOrCreated, DatabaseError> {
        if let Some(addr) = self.location_of_account(&account_id) {
            return Ok(GetOrCreated::Existed(addr));
        }

        let last_location = self.last_filled();

        self.with(|this| match this.parent {
            Some(_) => {
                let location = match last_location {
                    Some(last) => last.next().ok_or(DatabaseError::OutOfLeaves).unwrap(),
                    None => Address::first(this.depth as usize),
                };

                let account_index: AccountIndex = location.to_index();
                let token_id = account.token_id.clone();

                this.id_to_addr.insert(account_id.clone(), location.clone());
                this.last_location = Some(location.clone());
                this.token_to_account.insert(token_id, account_id);
                this.owning_account.insert(account_index, account);
                this.naccounts += 1;

                if this.first_location_in_mask.is_none() {
                    this.first_location_in_mask = Some(location.clone());
                }

                Ok(GetOrCreated::Added(location))
            }
            None => this.inner.get_or_create_account(account_id, account),
        })
    }

    fn close(self) {
        // Drop
    }

    fn last_filled(&self) -> Option<Address> {
        match self.get_parent() {
            Some(parent) => {
                let last_filled_parent = match parent.last_filled() {
                    Some(last) => last,
                    None => return self.with(|this| this.last_location.clone()),
                };

                let last_filled = match self.with(|this| this.last_location.clone()) {
                    Some(last) => last,
                    None => return Some(last_filled_parent),
                };

                let last_filled_parent_index = last_filled_parent.to_index();
                let last_filled_index = last_filled.to_index();

                if last_filled_index > last_filled_parent_index {
                    Some(last_filled)
                } else {
                    Some(last_filled_parent)
                }
            }
            None => self.with(|this| this.inner.last_filled()),
        }
    }

    fn get_uuid(&self) -> Uuid {
        // TODO
        todo!()
    }

    fn get_directory(&self) -> Option<PathBuf> {
        None
    }

    fn get(&self, addr: Address) -> Option<Account> {
        let account_index = addr.to_index();
        if let Some(account) = self.with(|this| this.owning_account.get(&account_index).cloned()) {
            return Some(account);
        }

        match self.get_parent() {
            Some(parent) => parent.get(addr),
            None => return self.with(|this| this.inner.get(addr)),
        }
    }

    fn get_batch(&self, addr: &[Address]) -> Vec<(Address, Option<Account>)> {
        addr.iter()
            .map(|addr| (addr.clone(), self.get(addr.clone())))
            .collect()
    }

    fn set(&mut self, addr: Address, account: Account) {
        let existing = self.get(addr.clone()).is_some();

        self.with(|this| {
            for child in this.childs.values() {
                child.parent_set_notify(&account)
            }

            match this.parent {
                Some(_) => {
                    let account_index: AccountIndex = addr.to_index();
                    let account_id = account.id();
                    let token_id = account.token_id.clone();

                    this.owning_account.insert(account_index, account);
                    this.id_to_addr.insert(account_id.clone(), addr.clone());
                    this.token_to_account.insert(token_id, account_id);

                    if !existing {
                        this.naccounts += 1;
                    }

                    if this
                        .last_location
                        .as_ref()
                        .map(|l| l.to_index() < addr.to_index())
                        .unwrap_or(true)
                    {
                        this.last_location = Some(addr.clone());
                    }

                    if this
                        .first_location_in_mask
                        .as_ref()
                        .map(|l| l.to_index() > addr.to_index())
                        .unwrap_or(true)
                    {
                        this.first_location_in_mask = Some(addr);
                    }
                }
                None => {
                    this.inner.set(addr, account);
                }
            }
        })
    }

    fn set_batch(&mut self, list: &[(Address, Account)]) {
        for (addr, account) in list {
            self.set(addr.clone(), account.clone())
        }
    }

    fn get_at_index(&self, index: AccountIndex) -> Option<Account> {
        let addr = Address::from_index(index, self.depth() as usize);
        self.get(addr)
    }

    fn set_at_index(&mut self, index: AccountIndex, account: Account) -> Result<(), ()> {
        let addr = Address::from_index(index, self.depth() as usize);
        self.set(addr, account);
        Ok(())
    }

    fn index_of_account(&self, account_id: AccountId) -> Option<AccountIndex> {
        if let Some(addr) = self.with(|this| this.id_to_addr.get(&account_id).cloned()) {
            return Some(addr.to_index());
        };

        match self.get_parent() {
            Some(parent) => parent.index_of_account(account_id),
            None => return self.with(|this| this.inner.index_of_account(account_id)),
        }
    }

    fn merkle_root(&self) -> Fp {
        self.emulate_tree_to_get_hash()
    }

    fn merkle_path(&self, addr: Address) -> AddressIterator {
        addr.into_iter()
    }

    fn merkle_path_at_index(&self, index: AccountIndex) -> Option<AddressIterator> {
        let addr = Address::from_index(index, self.depth() as usize);
        Some(addr.into_iter())
    }

    fn remove_accounts(&mut self, ids: &[AccountId]) {
        let mut parent = match self.get_parent() {
            Some(parent) => parent,
            None => return self.with(|this| this.inner.remove_accounts(ids)),
        };

        let (mask_keys, parent_keys): (Vec<_>, Vec<_>) = self.with(|this| {
            ids.iter()
                .cloned()
                .partition(|id| this.id_to_addr.contains_key(id))
        });

        if !parent_keys.is_empty() {
            parent.remove_accounts(&parent_keys);
        }

        self.remove_own_account(&mask_keys);
    }

    fn detached_signal(&mut self) {
        todo!()
    }

    fn depth(&self) -> u8 {
        self.depth()
    }

    fn num_accounts(&self) -> usize {
        self.last_filled()
            .map(|addr| addr.to_index().0 as usize + 1)
            .unwrap_or(0)
        // self.with(|this| this.naccounts)
    }

    fn merkle_path_at_addr(&self, addr: Address) -> Option<AddressIterator> {
        Some(addr.into_iter())
    }

    fn get_inner_hash_at_addr(&self, addr: Address) -> Result<Fp, ()> {
        let self_depth = self.depth() as usize;

        if addr.length() > self_depth {
            return Err(());
        }

        Ok(self.emulate_tree_to_get_hash_at(addr))
    }

    fn set_inner_hash_at_addr(&mut self, _addr: Address, _hash: Fp) -> Result<(), ()> {
        todo!()
    }

    fn set_all_accounts_rooted_at(
        &mut self,
        addr: Address,
        accounts: &[Account],
    ) -> Result<(), ()> {
        let self_depth = self.depth() as usize;

        if addr.length() > self_depth {
            return Err(());
        }

        for (child_addr, account) in addr.iter_children(self_depth).zip(accounts) {
            self.set(child_addr, account.clone());
        }

        Ok(())
    }

    fn get_all_accounts_rooted_at(&self, addr: Address) -> Option<Vec<(Address, Account)>> {
        let self_depth = self.depth() as usize;

        if addr.length() > self_depth {
            return None;
        }

        let children = addr.iter_children(self_depth);
        let mut accounts = Vec::with_capacity(children.len());

        for child_addr in children {
            let account = match self.get(child_addr.clone()) {
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
mod tests_mask_ocaml {
    use super::*;

    use rand::{thread_rng, Rng};
    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    const DEPTH: usize = 4;
    const FIRST_LOC: Address = Address::first(DEPTH);

    fn new_instances(depth: usize) -> (Mask, Mask) {
        let db = Database::<V2>::create(depth as u8);
        (Mask::new_root(db), Mask::new_child(depth))
    }

    fn new_chain(depth: usize) -> (Mask, Mask, Mask) {
        let db = Database::<V2>::create(depth as u8);
        let layer1 = Mask::new_child(depth);
        let layer2 = Mask::new_child(depth);

        let root = Mask::new_root(db);
        let layer1 = root.register_mask(layer1);
        let layer2 = layer1.register_mask(layer2);

        (root, layer1, layer2)
    }

    fn make_full_accounts(depth: usize) -> Vec<Account> {
        (0..2u64.pow(depth as u32))
            .map(|_| Account::rand())
            .collect()
    }

    // "parent, mask agree on set"
    #[test]
    fn test_parent_mask_agree_on_set() {
        let (mut root, mask) = new_instances(DEPTH);
        let mask = root.register_mask(mask);

        root.set(FIRST_LOC, Account::rand());

        let root_account = root.get(FIRST_LOC).unwrap();
        let mask_account = mask.get(FIRST_LOC).unwrap();

        assert_eq!(root_account, mask_account);
    }

    // "parent, mask agree on set"
    #[test]
    fn test_parent_mask_agree_on_set2() {
        let (mut root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        let account = Account::rand();
        root.set(FIRST_LOC, account.clone());
        mask.set(FIRST_LOC, account);

        let root_account = root.get(FIRST_LOC).unwrap();
        let mask_account = mask.get(FIRST_LOC).unwrap();

        assert_eq!(root_account, mask_account);
    }

    // "parent, mask agree on hashes; set in both mask and parent"
    #[test]
    fn test_parent_mask_agree_on_hashes() {
        let (mut root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        let account = Account::rand();
        root.set(FIRST_LOC, account.clone());
        mask.set(FIRST_LOC, account);

        assert_eq!(root.merkle_root(), mask.merkle_root());
    }

    // "parent, mask agree on hashes; set only in parent"
    #[test]
    fn test_parent_mask_agree_on_hashes_set_parent_only() {
        let (mut root, mask) = new_instances(DEPTH);
        let mask = root.register_mask(mask);

        let account = Account::rand();
        root.set(FIRST_LOC, account);

        assert_eq!(root.merkle_root(), mask.merkle_root());
    }

    // "mask delegates to parent"
    #[test]
    fn test_mask_delegate_to_parent() {
        let (mut root, mask) = new_instances(DEPTH);
        let mask = root.register_mask(mask);

        let account = Account::rand();
        root.set(FIRST_LOC, account.clone());

        let child_account = mask.get(FIRST_LOC).unwrap();

        assert_eq!(account, child_account);
    }

    // "mask prune after parent notification"
    #[test]
    fn test_mask_prune_after_parent_notif() {
        let (mut root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        // Set in mask
        let account = Account::rand();
        mask.set(FIRST_LOC, account.clone());

        assert!(mask.test_is_in_mask(&FIRST_LOC));

        root.set(FIRST_LOC, account);

        // The address is no more in the mask
        assert!(!mask.test_is_in_mask(&FIRST_LOC));
    }

    // "commit puts mask contents in parent, flushes mask"
    #[test]
    fn test_commit_puts_mask_in_parent_and_flush_mask() {
        let (root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        let account = Account::rand();
        mask.set(FIRST_LOC, account);

        assert!(mask.test_is_in_mask(&FIRST_LOC));

        mask.commit();

        // No more in mask
        assert!(!mask.test_is_in_mask(&FIRST_LOC));
        // The parent get the account
        assert!(root.get(FIRST_LOC).is_some());
    }

    // "commit at layer2, dumps to layer1, not in base"
    #[test]
    fn test_commit_layer2_dumps_to_layer1_not_in_base() {
        let (root, layer1, mut layer2) = new_chain(DEPTH);

        let account = Account::rand();

        layer2.set(FIRST_LOC, account);
        assert!(layer2.test_is_in_mask(&FIRST_LOC));
        assert!(!layer1.test_is_in_mask(&FIRST_LOC));

        layer2.commit();
        assert!(!layer2.test_is_in_mask(&FIRST_LOC));
        assert!(layer1.test_is_in_mask(&FIRST_LOC));
        assert!(!root.test_is_in_mask(&FIRST_LOC));
    }

    // "register and unregister mask"
    #[test]
    fn test_register_unregister_mask() {
        let (root, mask) = new_instances(DEPTH);
        let mask = root.register_mask(mask);
        mask.unregister_mask(UnregisterBehavior::Recursive);
    }

    // "mask and parent agree on Merkle root before set"
    #[test]
    fn test_agree_on_root_hash_before_set() {
        let (root, mask) = new_instances(DEPTH);
        let mask = root.register_mask(mask);

        assert_eq!(root.merkle_root(), mask.merkle_root());
    }

    // "mask and parent agree on Merkle root after set"
    #[test]
    fn test_agree_on_root_hash_after_set() {
        let (mut root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        let account = Account::rand();

        // the order of sets matters here; if we set in the mask first,
        // the set in the maskable notifies the mask, which then removes
        // the account, changing the Merkle root to what it was before the set

        root.set(FIRST_LOC, account.clone());
        mask.set(FIRST_LOC, account);

        assert!(root.test_is_in_mask(&FIRST_LOC));
        assert!(mask.test_is_in_mask(&FIRST_LOC));
        assert_eq!(root.merkle_root(), mask.merkle_root());
    }

    // "add and retrieve a block of accounts"
    #[test]
    fn test_add_retrieve_block_of_accounts() {
        let (root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        let accounts = make_full_accounts(DEPTH);

        for account in &accounts {
            let account_id = account.id();
            let res = mask
                .get_or_create_account(account_id, account.clone())
                .unwrap();
            assert!(matches!(res, GetOrCreated::Added(_)));
        }

        let retrieved_accounts = mask
            .get_all_accounts_rooted_at(Address::root())
            .unwrap()
            .into_iter()
            .map(|(_, acc)| acc)
            .collect::<Vec<_>>();

        assert_eq!(accounts, retrieved_accounts);
    }

    // "removing accounts from mask restores Merkle root"
    #[test]
    fn test_removing_accounts_from_mask_restore_root_hash() {
        let (root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        let accounts = (0..5).map(|_| Account::rand()).collect::<Vec<_>>();
        let accounts_ids = accounts.iter().map(Account::id).collect::<Vec<_>>();
        let root_hash0 = mask.merkle_root();

        for account in accounts {
            mask.get_or_create_account(account.id(), account).unwrap();
        }
        assert_ne!(root_hash0, mask.merkle_root());

        mask.remove_accounts(&accounts_ids);
        assert_eq!(root_hash0, mask.merkle_root());
    }

    // "removing accounts from parent restores Merkle root"
    #[test]
    fn test_removing_accounts_from_parent_restore_root_hash() {
        let (mut root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        let accounts = (0..5).map(|_| Account::rand()).collect::<Vec<_>>();
        let accounts_ids = accounts.iter().map(Account::id).collect::<Vec<_>>();
        let root_hash0 = mask.merkle_root();

        for account in accounts {
            root.get_or_create_account(account.id(), account).unwrap();
        }
        assert_ne!(root_hash0, mask.merkle_root());

        mask.remove_accounts(&accounts_ids);
        assert_eq!(root_hash0, mask.merkle_root());
    }

    // "removing accounts from parent and mask restores Merkle root"
    #[test]
    fn test_removing_accounts_from_parent_and_mask_restore_root_hash() {
        let (mut root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        let accounts = (0..10).map(|_| Account::rand()).collect::<Vec<_>>();
        let (accounts_parent, accounts_mask) = accounts.split_at(5);
        let accounts_ids = accounts.iter().map(Account::id).collect::<Vec<_>>();

        let root_hash0 = mask.merkle_root();

        for account in accounts_parent {
            root.get_or_create_account(account.id(), account.clone())
                .unwrap();
        }
        for account in accounts_mask {
            mask.get_or_create_account(account.id(), account.clone())
                .unwrap();
        }
        assert_ne!(root_hash0, mask.merkle_root());

        mask.remove_accounts(&accounts_ids);
        assert_eq!(root_hash0, mask.merkle_root());
    }

    // "fold of addition over account balances in parent and mask"
    #[test]
    fn test_fold_of_addition_over_account_balance_in_parent_and_mask() {
        let (mut root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        let accounts = (0..10).map(|_| Account::rand()).collect::<Vec<_>>();
        let balance = accounts
            .iter()
            .fold(0u128, |acc, account| acc + account.balance as u128);

        let (accounts_parent, accounts_mask) = accounts.split_at(5);

        for account in accounts_parent {
            root.get_or_create_account(account.id(), account.clone())
                .unwrap();
        }
        for account in accounts_mask {
            mask.get_or_create_account(account.id(), account.clone())
                .unwrap();
        }

        let retrieved_balance = mask.fold(0u128, |acc, account| acc + account.balance as u128);
        assert_eq!(balance, retrieved_balance);
    }

    fn create_existing_account(mask: &mut Mask, account: Account) {
        match mask
            .get_or_create_account(account.id(), account.clone())
            .unwrap()
        {
            GetOrCreated::Added(_) => panic!("Should add an existing account"),
            GetOrCreated::Existed(addr) => {
                mask.set(addr, account);
            }
        }
    }

    // "masking in to_list"
    #[test]
    fn test_masking_in_to_list() {
        let (mut root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        let mut accounts = (0..10).map(|_| Account::rand()).collect::<Vec<_>>();
        // Make balances non-zero
        accounts
            .iter_mut()
            .for_each(|account| account.balance = account.balance.checked_add(1).unwrap_or(1));

        for account in &accounts {
            root.get_or_create_account(account.id(), account.clone())
                .unwrap();
        }

        let parent_list = root.to_list();

        // Make balances to zero for those same account
        accounts.iter_mut().for_each(|account| account.balance = 0);

        for account in accounts {
            create_existing_account(&mut mask, account);
        }

        let mask_list = mask.to_list();

        assert_eq!(parent_list.len(), mask_list.len());
        // Same accounts and order
        assert_eq!(
            parent_list.iter().map(Account::id).collect::<Vec<_>>(),
            mask_list.iter().map(Account::id).collect::<Vec<_>>(),
        );
        // Balances of mask are zero
        assert_eq!(
            mask_list
                .iter()
                .fold(0u128, |acc, account| acc + account.balance as u128),
            0
        );
    }

    // "masking in foldi"
    #[test]
    fn test_masking_in_to_foldi() {
        let (mut root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        let mut accounts = (0..10).map(|_| Account::rand()).collect::<Vec<_>>();
        // Make balances non-zero
        accounts
            .iter_mut()
            .for_each(|account| account.balance = account.balance.checked_add(1).unwrap_or(1));

        for account in &accounts {
            root.get_or_create_account(account.id(), account.clone())
                .unwrap();
        }

        let parent_sum_balance = root.fold(0u128, |acc, account| acc + account.balance as u128);
        assert_ne!(parent_sum_balance, 0);

        // Make balances to zero for those same account
        accounts.iter_mut().for_each(|account| account.balance = 0);

        for account in accounts {
            create_existing_account(&mut mask, account);
        }

        let mask_sum_balance = mask.fold(0u128, |acc, account| acc + account.balance as u128);
        assert_eq!(mask_sum_balance, 0);
    }

    // "create_empty doesn't modify the hash"
    #[test]
    fn test_create_empty_doesnt_modify_the_hash() {
        let (root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        let start_hash = mask.merkle_root();

        let account = Account::empty();
        mask.get_or_create_account(account.id(), account).unwrap();

        assert_eq!(mask.num_accounts(), 1);
        assert_eq!(start_hash, mask.merkle_root());
    }

    // "reuse of locations for removed accounts"
    #[test]
    fn test_reuse_of_locations_for_removed_accounts() {
        let (root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        let accounts = (0..10).map(|_| Account::rand()).collect::<Vec<_>>();
        let accounts_ids = accounts.iter().map(Account::id).collect::<Vec<_>>();

        assert!(mask.last_filled().is_none());
        for account in accounts {
            mask.get_or_create_account(account.id(), account.clone())
                .unwrap();
        }
        assert!(mask.last_filled().is_some());

        mask.remove_accounts(&accounts_ids);
        assert!(mask.last_filled().is_none());
    }

    // "num_accounts for unique keys in mask and parent"
    #[test]
    fn test_num_accounts_for_unique_keys_in_mask_and_parent() {
        let (mut root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        let accounts = (0..10).map(|_| Account::rand()).collect::<Vec<_>>();

        for account in &accounts {
            mask.get_or_create_account(account.id(), account.clone())
                .unwrap();
        }

        let mask_num_accounts_before = mask.num_accounts();

        // Add same accounts to parent
        for account in &accounts {
            root.get_or_create_account(account.id(), account.clone())
                .unwrap();
        }

        let parent_num_accounts = root.num_accounts();
        let mask_num_accounts_after = mask.num_accounts();

        assert_eq!(accounts.len(), parent_num_accounts);
        assert_eq!(parent_num_accounts, mask_num_accounts_before);
        assert_eq!(parent_num_accounts, mask_num_accounts_after);
    }

    // "Mask reparenting works"
    #[test]
    fn test_mask_reparenting_works() {
        let (mut root, mut layer1, mut layer2) = new_chain(DEPTH);

        let acc1 = Account::rand();
        let acc2 = Account::rand();
        let acc3 = Account::rand();

        let loc1 = root.get_or_create_account(acc1.id(), acc1).unwrap().addr();
        let loc2 = layer1
            .get_or_create_account(acc2.id(), acc2)
            .unwrap()
            .addr();
        let loc3 = layer2
            .get_or_create_account(acc3.id(), acc3)
            .unwrap()
            .addr();

        // All accounts are accessible from layer2
        assert!(layer2.get(loc1.clone()).is_some());
        assert!(layer2.get(loc2.clone()).is_some());
        assert!(layer2.get(loc3.clone()).is_some());

        // acc1 is in root
        assert!(root.get(loc1.clone()).is_some());

        layer1.commit();

        // acc2 is in root
        assert!(root.get(loc2.clone()).is_some());

        layer1.remove_and_reparent();

        // acc1, acc2 are in root
        assert!(root.get(loc1.clone()).is_some());
        assert!(root.get(loc2.clone()).is_some());

        // acc3 not in root
        assert!(root.get(loc3.clone()).is_none());

        // All accounts are accessible from layer2
        assert!(layer2.get(loc1).is_some());
        assert!(layer2.get(loc2).is_some());
        assert!(layer2.get(loc3).is_some());
    }

    // "setting an account in the parent doesn't remove the masked
    // copy if the mask is still dirty for that account"
    #[test]
    fn test_set_account_in_parent_doesnt_remove_if_mask_is_dirty() {
        let (mut root, mask) = new_instances(DEPTH);
        let mut mask = root.register_mask(mask);

        let mut account = Account::rand();
        let mut account2 = account.clone();

        account.balance = 10;
        account2.balance = 5;

        let loc = mask
            .get_or_create_account(account.id(), account.clone())
            .unwrap()
            .addr();

        root.set(loc.clone(), account2);

        assert_eq!(mask.get(loc).unwrap(), account);
    }

    // "get_all_accounts should preserve the ordering of accounts by
    // location with noncontiguous updates of accounts on the mask"
    #[test]
    fn test_get_all_accounts_should_preserve_ordering() {
        let (_root, mut layer1, mut layer2) = new_chain(DEPTH);

        let accounts = make_full_accounts(DEPTH);

        for account in &accounts {
            layer1
                .get_or_create_account(account.id(), account.clone())
                .unwrap();
        }

        let mut updated_accounts = accounts.clone();
        let mut rng = thread_rng();
        let mut nmodified = 0;

        for account in updated_accounts.iter_mut() {
            if rng.gen::<u8>() >= 100 {
                continue;
            }
            account.balance = rng.gen();

            create_existing_account(&mut layer2, account.clone());
            nmodified += 1;
        }

        assert!(nmodified > 0);
        assert_eq!(
            updated_accounts,
            layer2
                .get_all_accounts_rooted_at(Address::root())
                .unwrap()
                .into_iter()
                .map(|(_, account)| account)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            accounts,
            layer1
                .get_all_accounts_rooted_at(Address::root())
                .unwrap()
                .into_iter()
                .map(|(_, account)| account)
                .collect::<Vec<_>>()
        );
    }
}
