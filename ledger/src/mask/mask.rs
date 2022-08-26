use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
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
    depth: u8,
    naccounts: usize,
    /// All childs of this mask
    childs: HashMap<Uuid, Mask>,
}

impl Deref for MaskInner {
    type Target = Database<V2>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for MaskInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Clone)]
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
        self.with(|this| this.get_uuid())
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

        self.with(|this| {
            let addr = match this.id_to_addr.remove(&account_id) {
                Some(addr) => addr,
                None => return,
            };
            let account = this.owning_account.remove(&addr.to_index()).unwrap();
            this.token_to_account.remove(&account.token_id).unwrap();
        })
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
        match self.get_parent() {
            None => self.with(|this| this.inner.to_list()),
            Some(parent) => {
                let mut accounts = parent.to_list();

                self.with(|this| {
                    for (index, account) in this.owning_account.iter() {
                        let index = index.0 as usize;
                        accounts[index] = account.clone(); // TODO: Handle out of bound (extend the vec)
                    }
                });

                accounts
            }
        }
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

        self.with(|this| {
            let location = match this.last_location.as_ref() {
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

            Ok(GetOrCreated::Added(location))
        })
    }

    fn close(self) {
        // Drop
    }

    fn last_filled(&self) -> Option<Address> {
        match self.get_parent() {
            Some(parent) => {
                let last_filled_parent = parent.last_filled().unwrap();
                let last_filled = self.with(|this| this.last_location.clone()).unwrap();

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
                        this.last_location = Some(addr);
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
        let (mask_keys, parent_keys): (Vec<_>, Vec<_>) = self.with(|this| {
            ids.iter()
                .cloned()
                .partition(|id| this.id_to_addr.contains_key(id))
        });

        let parent = match self.get_parent() {
            Some(parent) => parent,
            None => return self.with(|this| this.inner.remove_accounts(ids)),
        };

        parent.with(|parent| {
            parent.remove_accounts(&parent_keys);
        });

        self.with(|this| {
            for parent_key in mask_keys {
                let addr = this.id_to_addr.remove(&parent_key).unwrap();
                let account = this.owning_account.remove(&addr.to_index()).unwrap();
                this.token_to_account.remove(&account.token_id).unwrap();
                this.naccounts -= 1;
                // TODO: Update Self::last_location
            }
        });
    }

    fn detached_signal(&mut self) {
        todo!()
    }

    fn depth(&self) -> u8 {
        self.depth()
    }

    fn num_accounts(&self) -> usize {
        self.with(|this| this.naccounts)
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
}

// let%test_unit "get_all_accounts should preserve the ordering of accounts by \
//                location with noncontiguous updates of accounts on the mask" =
//   (* see similar test in test_database *)
//   if Test.depth <= 8 then
//     Test.with_chain (fun _ ~mask:mask1 ~mask_as_base:_ ~mask2 ->
//         let num_accounts = 1 lsl Test.depth in
//         let gen_values gen list_length =
//           Quickcheck.random_value
//             (Quickcheck.Generator.list_with_length list_length gen)
//         in
//         let account_ids = Account_id.gen_accounts num_accounts in
//         let balances = gen_values Balance.gen num_accounts in
//         let base_accounts =
//           List.map2_exn account_ids balances ~f:(fun public_key balance ->
//               Account.create public_key balance )
//         in
//         List.iter base_accounts ~f:(fun account ->
//             ignore @@ create_new_account_exn mask1 account ) ;
//         let num_subset =
//           Quickcheck.random_value (Int.gen_incl 3 num_accounts)
//         in
//         let subset_indices, subset_accounts =
//           List.permute
//             (List.mapi base_accounts ~f:(fun index account ->
//                  (index, account) ) )
//           |> (Fn.flip List.take) num_subset
//           |> List.unzip
//         in
//         let subset_balances = gen_values Balance.gen num_subset in
//         let subset_updated_accounts =
//           List.map2_exn subset_accounts subset_balances
//             ~f:(fun account balance ->
//               let updated_account = { account with balance } in
//               ignore
//                 ( create_existing_account_exn mask2 updated_account
//                   : Test.Location.t ) ;
//               updated_account )
//         in
//         let updated_accounts_map =
//           Int.Map.of_alist_exn
//             (List.zip_exn subset_indices subset_updated_accounts)
//         in
//         let expected_accounts =
//           List.mapi base_accounts ~f:(fun index base_account ->
//               Option.value
//                 (Map.find updated_accounts_map index)
//                 ~default:base_account )
//         in
//         let retrieved_accounts =
//           List.map ~f:snd
//           @@ Mask.Attached.get_all_accounts_rooted_at_exn mask2
//                (Mask.Addr.root ())
//         in
//         assert (
//           Int.equal
//             (List.length base_accounts)
//             (List.length retrieved_accounts) ) ;
//         assert (List.equal Account.equal expected_accounts retrieved_accounts) )
