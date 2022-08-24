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

    pub fn get_parent(&self) -> Mask {
        self.with(|this| this.parent.clone().unwrap())
    }

    pub fn unregister_mask(mask: Mask, behavior: UnregisterBehavior) {
        use UnregisterBehavior::*;

        assert!(mask.is_attached());
        let parent = mask.get_parent();

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
}

impl BaseLedger for Mask {
    fn to_list(&self) -> Vec<Account> {
        todo!()
    }

    fn iter<F>(&self, fun: F)
    where
        F: FnMut(&Account),
    {
        todo!()
    }

    fn fold<B, F>(&self, init: B, fun: F) -> B
    where
        F: FnMut(B, &Account) -> B,
    {
        todo!()
    }

    fn fold_with_ignored_accounts<B, F>(&self, ignoreds: HashSet<AccountId>, init: B, fun: F) -> B
    where
        F: FnMut(B, &Account) -> B,
    {
        todo!()
    }

    fn fold_until<B, F>(&self, init: B, fun: F) -> B
    where
        F: FnMut(B, &Account) -> std::ops::ControlFlow<B, B>,
    {
        todo!()
    }

    fn accounts(&self) -> HashSet<AccountId> {
        todo!()
    }

    fn token_owner(&self, token_id: TokenId) -> Option<AccountId> {
        todo!()
    }

    fn token_owners(&self) -> HashSet<AccountId> {
        todo!()
    }

    fn tokens(&self, public_key: CompressedPubKey) -> HashSet<TokenId> {
        todo!()
    }

    fn location_of_account(&self, account_id: AccountId) -> Option<Address> {
        todo!()
    }

    fn location_of_account_batch(
        &self,
        account_ids: &[AccountId],
    ) -> Vec<(AccountId, Option<Address>)> {
        todo!()
    }

    fn get_or_create_account(
        &mut self,
        account_id: AccountId,
        account: Account,
    ) -> Result<GetOrCreated, DatabaseError> {
        todo!()
    }

    fn close(self) {
        todo!()
    }

    fn last_filled(&self) -> Option<Address> {
        todo!()
    }

    fn get_uuid(&self) -> Uuid {
        todo!()
    }

    fn get_directory(&self) -> Option<PathBuf> {
        todo!()
    }

    fn get(&self, addr: Address) -> Option<Account> {
        todo!()
    }

    fn get_batch(&self, addr: &[Address]) -> Vec<(Address, Option<Account>)> {
        todo!()
    }

    fn set(&mut self, addr: Address, account: Account) {
        todo!()
    }

    fn set_batch(&mut self, list: &[(Address, Account)]) {
        todo!()
    }

    fn get_at_index(&self, index: AccountIndex) -> Option<Account> {
        todo!()
    }

    fn set_at_index(&mut self, index: AccountIndex, account: Account) -> Result<(), ()> {
        todo!()
    }

    fn index_of_account(&self, account_id: AccountId) -> Option<AccountIndex> {
        todo!()
    }

    fn merkle_root(&self) -> Fp {
        todo!()
    }

    fn merkle_path(&self, addr: Address) -> AddressIterator {
        todo!()
    }

    fn merkle_path_at_index(&self, index: AccountIndex) -> Option<AddressIterator> {
        todo!()
    }

    fn remove_accounts(&mut self, ids: &[AccountId]) {
        todo!()
    }

    fn detached_signal(&mut self) {
        todo!()
    }

    fn depth(&self) -> u8 {
        todo!()
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
