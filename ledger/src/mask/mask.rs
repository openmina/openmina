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
    tree_version::V2,
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

    pub fn unregister_mask(mask: Mask, behavior: UnregisterBehavior) {
        use UnregisterBehavior::*;

        assert!(mask.is_attached());
        let parent = mask.get_parent().unwrap();

        let trigger_detach_signal = matches!(behavior, Check | Recursive);

        match behavior {
            Check => {
                assert!(
                    !mask.children().is_empty(),
                    "mask has children that must be unregistered first"
                );
            }
            IPromiseIAmReparentingThisMask => (),
            Recursive => {
                for child in mask.children() {
                    Self::unregister_mask(child, Recursive);
                }
            }
        }

        let removed = parent.remove_child(&mask);
        assert!(removed.is_some(), "Mask not a child of the parent");

        mask.unset_parent(trigger_detach_signal);
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

    fn location_of_account(&self, account_id: AccountId) -> Option<Address> {
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
                let addr = self.location_of_account(account_id.clone());
                (account_id.clone(), addr)
            })
            .collect()
    }

    fn get_or_create_account(
        &mut self,
        account_id: AccountId,
        account: Account,
    ) -> Result<GetOrCreated, DatabaseError> {
        let accounts: HashMap<AccountId, u64> = self
            .to_list()
            .into_iter()
            .enumerate()
            .map(|(index, account)| (account.id(), index as u64))
            .collect();

        if let Some(index) = accounts.get(&account_id) {
            let depth = self.depth();
            return Ok(GetOrCreated::Existed(Address::from_index(
                AccountIndex(*index),
                depth as usize,
            )));
        };

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
        let parent = match self.get_parent() {
            None => return self.with(|this| this.inner.get(addr)),
            Some(parent) => parent,
        };

        let account_index = addr.to_index();
        if let Some(account) = self.with(|this| this.owning_account.get(&account_index).cloned()) {
            return Some(account);
        }

        parent.get(addr)
    }

    fn get_batch(&self, addr: &[Address]) -> Vec<(Address, Option<Account>)> {
        addr.iter()
            .map(|addr| (addr.clone(), self.get(addr.clone())))
            .collect()
    }

    fn set(&mut self, addr: Address, account: Account) {
        let existing = self.get(addr.clone()).is_some();

        self.with(|this| {
            let account_index: AccountIndex = addr.to_index();
            let account_id = account.id();
            let token_id = account.token_id.clone();

            this.owning_account.insert(account_index, account);
            this.id_to_addr.insert(account_id.clone(), addr.clone());
            this.token_to_account.insert(token_id, account_id);

            if !existing {
                this.naccounts += 1;
                this.last_location = Some(addr);
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
        let parent = match self.get_parent() {
            Some(parent) => parent,
            None => return self.with(|this| this.inner.index_of_account(account_id)),
        };

        if let Some(addr) = self.with(|this| this.id_to_addr.get(&account_id).cloned()) {
            return Some(addr.to_index());
        };

        parent.index_of_account(account_id)
    }

    fn merkle_root(&self) -> Fp {
        todo!()
    }

    fn merkle_path(&self, addr: Address) -> AddressIterator {
        addr.into_iter()
    }

    fn merkle_path_at_index(&self, index: AccountIndex) -> Option<AddressIterator> {
        let addr = Address::from_index(index, self.depth() as usize);
        Some(addr.into_iter())
    }

    fn remove_accounts(&mut self, ids: &[AccountId]) {
        let parent = match self.get_parent() {
            Some(parent) => parent,
            None => return self.with(|this| this.inner.remove_accounts(ids)),
        };

        let (mask_keys, parent_keys): (Vec<_>, Vec<_>) = self.with(|this| {
            ids.iter()
                .cloned()
                .partition(|id| this.id_to_addr.contains_key(id))
        });

        parent.with(|parent| {
            parent.remove_accounts(&parent_keys);
        });

        // for parent_key in parent_keys {

        // }

        todo!()
    }

    fn detached_signal(&mut self) {
        todo!()
    }

    fn depth(&self) -> u8 {
        self.depth()
    }

    fn num_accounts(&self) -> usize {
        todo!()
    }

    fn merkle_path_at_addr(&self, addr: Address) -> Option<AddressIterator> {
        todo!()
    }

    fn get_inner_hash_at_addr(&self, addr: Address) -> Result<Fp, ()> {
        todo!()
    }

    fn set_inner_hash_at_addr(&mut self, addr: Address, hash: Fp) -> Result<(), ()> {
        todo!()
    }

    fn set_all_accounts_rooted_at(
        &mut self,
        addr: Address,
        accounts: &[Account],
    ) -> Result<(), ()> {
        todo!()
    }

    fn get_all_accounts_rooted_at(&self, addr: Address) -> Option<Vec<(Address, Account)>> {
        todo!()
    }

    fn make_space_for(&mut self, space: usize) {
        todo!()
    }
}
