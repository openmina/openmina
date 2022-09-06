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
    base::{next_uuid, AccountIndex, BaseLedger, GetOrCreated, Uuid},
    tree::{Database, DatabaseError},
    tree_version::V2,
};

use super::mask_impl::MaskImpl;

#[derive(Clone, Debug)]
pub struct Mask {
    // Using a mutex for now but this can be replaced with a RefCell
    inner: Arc<Mutex<MaskImpl>>,
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
        F: FnOnce(&mut MaskImpl) -> R,
    {
        let mut inner = self.inner.lock().expect("lock failed");
        fun(&mut inner)
    }
}

impl Mask {
    pub fn new_root(db: Database<V2>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(MaskImpl::Root {
                database: db,
                childs: HashMap::with_capacity(2),
            })),
        }
    }

    pub fn new_unattached(depth: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(MaskImpl::Unattached {
                owning_account: Default::default(),
                token_to_account: Default::default(),
                id_to_addr: Default::default(),
                last_location: None,
                depth: depth as u8,
                childs: HashMap::with_capacity(2),
                uuid: next_uuid(),
            })),
        }
    }

    pub fn set_parent(&self, parent: &Mask) {
        self.with(|this| this.set_parent(parent))
    }

    /// Make `mask` a child of `self`
    pub fn register_mask(&self, mask: Mask) -> Mask {
        let self_mask = self.clone();
        self.with(|this| this.register_mask(self_mask, mask))
    }

    /// Detach this mask from its parent
    pub fn unregister_mask(&self, behavior: UnregisterBehavior) {
        self.with(|this| this.unregister_mask(behavior))
    }

    pub(super) fn remove_child_uuid(&self, uuid: Uuid) -> Option<Mask> {
        self.with(|this| this.remove_child_uuid(uuid))
    }

    pub fn is_attached(&self) -> bool {
        self.with(|this| this.is_attached())
    }

    fn uuid(&self) -> Uuid {
        self.with(|this| this.get_uuid())
    }

    pub fn get_parent(&self) -> Option<Mask> {
        self.with(|this| this.get_parent())
    }

    pub fn unset_parent(&self, trigger_detach_signal: bool) {
        self.with(|this| this.unset_parent(trigger_detach_signal))
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
        self.with(|this| this.remove_and_reparent())
    }

    /// get hash from mask, if present, else from its parent
    pub fn get_hash(&self, addr: Address) -> Option<Fp> {
        self.with(|this| this.get_hash(addr))
    }

    /// commit all state to the parent, flush state locally
    pub fn commit(&self) {
        self.with(|this| this.commit())
    }

    /// called when parent sets an account; update local state
    ///
    /// if the mask's parent sets an account, we can prune an entry in the mask
    /// if the account in the parent is the same in the mask *)
    pub fn parent_set_notify(&self, account: &Account) {
        self.with(|this| this.parent_set_notify(account))
    }

    pub fn remove_parent(&self) -> Option<Mask> {
        self.with(|this| this.remove_parent())
    }

    pub fn depth(&self) -> u8 {
        self.with(|this| this.depth())
    }

    pub(super) fn set_impl(&mut self, addr: Address, account: Account, ignore: Option<Uuid>) {
        self.with(|this| this.set_impl(addr, account, ignore))
    }

    /// For tests only, check if the address is in the mask, without checking parent
    #[cfg(test)]
    fn test_is_in_mask(&self, addr: &Address) -> bool {
        self.with(|this| this.test_is_in_mask(addr))
    }
}

impl BaseLedger for Mask {
    fn to_list(&self) -> Vec<Account> {
        self.with(|this| this.to_list())
    }

    fn iter<F>(&self, fun: F)
    where
        F: FnMut(&Account),
    {
        self.with(|this| this.iter(fun))
    }

    fn fold<B, F>(&self, init: B, fun: F) -> B
    where
        F: FnMut(B, &Account) -> B,
    {
        self.with(|this| this.fold(init, fun))
    }

    fn fold_with_ignored_accounts<B, F>(&self, ignoreds: HashSet<AccountId>, init: B, fun: F) -> B
    where
        F: FnMut(B, &Account) -> B,
    {
        self.with(|this| this.fold_with_ignored_accounts(ignoreds, init, fun))
    }

    fn fold_until<B, F>(&self, init: B, fun: F) -> B
    where
        F: FnMut(B, &Account) -> std::ops::ControlFlow<B, B>,
    {
        self.with(|this| this.fold_until(init, fun))
    }

    fn accounts(&self) -> HashSet<AccountId> {
        self.with(|this| this.accounts())
    }

    fn token_owner(&self, token_id: TokenId) -> Option<AccountId> {
        self.with(|this| this.token_owner(token_id))
    }

    fn token_owners(&self) -> HashSet<AccountId> {
        self.with(|this| this.token_owners())
    }

    fn tokens(&self, public_key: CompressedPubKey) -> HashSet<TokenId> {
        self.with(|this| this.tokens(public_key))
    }

    fn location_of_account(&self, account_id: &AccountId) -> Option<Address> {
        self.with(|this| this.location_of_account(account_id))
    }

    fn location_of_account_batch(
        &self,
        account_ids: &[AccountId],
    ) -> Vec<(AccountId, Option<Address>)> {
        self.with(|this| this.location_of_account_batch(account_ids))
    }

    fn get_or_create_account(
        &mut self,
        account_id: AccountId,
        account: Account,
    ) -> Result<GetOrCreated, DatabaseError> {
        self.with(|this| this.get_or_create_account(account_id, account))
    }

    fn close(&self) {
        // Drop self
    }

    fn last_filled(&self) -> Option<Address> {
        self.with(|this| this.last_filled())
    }

    fn get_uuid(&self) -> u64 {
        self.with(|this| this.get_uuid())
    }

    fn get_directory(&self) -> Option<PathBuf> {
        self.with(|this| this.get_directory())
    }

    fn get(&self, addr: Address) -> Option<Account> {
        self.with(|this| this.get(addr))
    }

    fn get_batch(&self, addr: &[Address]) -> Vec<(Address, Option<Account>)> {
        self.with(|this| this.get_batch(addr))
    }

    fn set(&mut self, addr: Address, account: Account) {
        self.with(|this| this.set(addr, account))
    }

    fn set_batch(&mut self, list: &[(Address, Account)]) {
        self.with(|this| this.set_batch(list))
    }

    fn get_at_index(&self, index: AccountIndex) -> Option<Account> {
        self.with(|this| this.get_at_index(index))
    }

    fn set_at_index(&mut self, index: AccountIndex, account: Account) -> Result<(), ()> {
        self.with(|this| this.set_at_index(index, account))
    }

    fn index_of_account(&self, account_id: AccountId) -> Option<AccountIndex> {
        self.with(|this| this.index_of_account(account_id))
    }

    fn merkle_root(&self) -> Fp {
        self.with(|this| this.merkle_root())
    }

    fn merkle_path(&self, addr: Address) -> AddressIterator {
        self.with(|this| this.merkle_path(addr))
    }

    fn merkle_path_at_index(&self, index: AccountIndex) -> Option<AddressIterator> {
        self.with(|this| this.merkle_path_at_index(index))
    }

    fn remove_accounts(&mut self, ids: &[AccountId]) {
        self.with(|this| this.remove_accounts(ids))
    }

    fn detached_signal(&mut self) {
        self.with(|this| this.detached_signal())
    }

    fn depth(&self) -> u8 {
        self.with(|this| this.depth())
    }

    fn num_accounts(&self) -> usize {
        self.with(|this| this.num_accounts())
    }

    fn merkle_path_at_addr(&self, addr: Address) -> Option<AddressIterator> {
        self.with(|this| this.merkle_path_at_addr(addr))
    }

    fn get_inner_hash_at_addr(&self, addr: Address) -> Result<Fp, ()> {
        self.with(|this| this.get_inner_hash_at_addr(addr))
    }

    fn set_inner_hash_at_addr(&mut self, addr: Address, hash: Fp) -> Result<(), ()> {
        self.with(|this| this.set_inner_hash_at_addr(addr, hash))
    }

    fn set_all_accounts_rooted_at(
        &mut self,
        addr: Address,
        accounts: &[Account],
    ) -> Result<(), ()> {
        self.with(|this| this.set_all_accounts_rooted_at(addr, accounts))
    }

    fn get_all_accounts_rooted_at(&self, addr: Address) -> Option<Vec<(Address, Account)>> {
        self.with(|this| this.get_all_accounts_rooted_at(addr))
    }

    fn make_space_for(&mut self, space: usize) {
        self.with(|this| this.make_space_for(space))
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
        (Mask::new_root(db), Mask::new_unattached(depth))
    }

    fn new_chain(depth: usize) -> (Mask, Mask, Mask) {
        let db = Database::<V2>::create(depth as u8);
        let layer1 = Mask::new_unattached(depth);
        let layer2 = Mask::new_unattached(depth);

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
