use std::{
    collections::{HashMap, HashSet},
    ops::ControlFlow,
    path::PathBuf,
};

use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::{
    next_uuid, Account, AccountId, AccountIndex, AccountLegacy, Address, AddressIterator,
    BaseLedger, Direction, GetOrCreated, HashesMatrix, MerklePath, TokenId, TreeVersion, Uuid, V1,
    V2,
};

use super::DatabaseError;

#[derive(Clone)]
pub struct DatabaseImpl<T: TreeVersion> {
    accounts: Vec<Option<T::Account>>,
    pub hashes_matrix: HashesMatrix,
    id_to_addr: HashMap<AccountId, Address>,
    token_to_account: HashMap<T::TokenId, AccountId>,
    depth: u8,
    last_location: Option<Address>,
    naccounts: usize,
    uuid: Uuid,
    directory: PathBuf,
}

impl<T: TreeVersion> std::fmt::Debug for DatabaseImpl<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database")
            // .field("accounts", &self.accounts)
            .field("hashes_matrix", &self.hashes_matrix)
            // .field("id_to_addr", &self.id_to_addr)
            // .field("token_to_account", &self.token_to_account)
            // .field("depth", &self.depth)
            // .field("last_location", &self.last_location)
            .field("naccounts", &self.naccounts)
            .field("uuid", &self.uuid)
            .field("directory", &self.directory)
            .finish()
    }
}

// #[derive(Debug, PartialEq, Eq)]
// pub enum DatabaseError {
//     OutOfLeaves,
// }

impl DatabaseImpl<V2> {
    pub fn clone_db(&self, new_directory: PathBuf) -> Self {
        Self {
            // root: self.root.clone(),
            accounts: self.accounts.clone(),
            id_to_addr: self.id_to_addr.clone(),
            token_to_account: self.token_to_account.clone(),
            depth: self.depth,
            last_location: self.last_location.clone(),
            naccounts: self.naccounts,
            uuid: next_uuid(),
            directory: new_directory,
            hashes_matrix: HashesMatrix::new(self.depth as usize),
            // root_hash: RefCell::new(*self.root_hash.borrow()),
        }
    }

    fn remove(&mut self, addr: Address) -> Option<Account> {
        let index = addr.to_index();
        let index: usize = index.0 as usize;

        if let Some(account) = self.accounts.get_mut(index) {
            return account.take();
        }

        None
    }

    fn create_account(
        &mut self,
        account_id: AccountId,
        account: Account,
    ) -> Result<GetOrCreated, DatabaseError> {
        // if self.root.is_none() {
        //     self.root = Some(NodeOrLeaf::Node(Node::default()));
        // }

        if let Some(addr) = self.id_to_addr.get(&account_id).cloned() {
            return Ok(GetOrCreated::Existed(addr));
        }

        let token_id = account.token_id.clone();
        let location = match self.last_location.as_ref() {
            Some(last) => last.next().ok_or(DatabaseError::OutOfLeaves)?,
            None => Address::first(self.depth as usize),
        };

        assert_eq!(location.to_index(), self.accounts.len());
        self.accounts.push(Some(account));

        // let root = self.root.as_mut().unwrap();
        // root.add_account_on_path(account, location.iter());

        self.last_location = Some(location.clone());
        self.naccounts += 1;

        self.token_to_account.insert(token_id, account_id.clone());
        self.id_to_addr.insert(account_id, location.clone());

        // self.root_hash.borrow_mut().take();

        Ok(GetOrCreated::Added(location))
    }

    pub fn iter_with_addr<F>(&self, mut fun: F)
    where
        F: FnMut(Address, &Account),
    {
        let depth = self.depth as usize;

        for (index, account) in self.accounts.iter().enumerate() {
            let account = match account {
                Some(account) => account,
                None => continue,
            };

            let addr = Address::from_index(index.into(), depth);
            fun(addr, account);
        }
    }

    fn emulate_tree_to_get_hash_at(&mut self, addr: Address) -> Fp {
        if let Some(hash) = self.hashes_matrix.get(&addr) {
            return *hash;
        };

        // let tree_depth = self.depth() as usize;
        // let mut children = addr.iter_children(tree_depth);

        // // First child
        // let first_account_index = children.next().unwrap().to_index().0 as u64;
        // let mut nremaining = self
        //     .naccounts()
        //     .saturating_sub(first_account_index as usize);

        let last_account = self
            .last_filled()
            .unwrap_or_else(|| Address::first(self.depth as usize));

        self.emulate_tree_recursive(addr, &last_account)
    }

    // fn emulate_recursive(&mut self, addr: Address, nremaining: &mut usize) -> Fp {
    fn emulate_tree_recursive(&mut self, addr: Address, last_account: &Address) -> Fp {
        let tree_depth = self.depth as usize;
        let current_depth = tree_depth - addr.length();

        if current_depth == 0 {
            return self
                .get_account_hash(addr.to_index())
                .unwrap_or_else(|| self.hashes_matrix.empty_hash_at_depth(0));
        }

        let mut get_child_hash = |addr: Address| {
            if let Some(hash) = self.hashes_matrix.get(&addr) {
                *hash
            } else if addr.is_before(last_account) {
                self.emulate_tree_recursive(addr, last_account)
            } else {
                self.hashes_matrix.empty_hash_at_depth(current_depth - 1)
            }
        };

        let left_hash = get_child_hash(addr.child_left());
        let right_hash = get_child_hash(addr.child_right());

        match self.hashes_matrix.get(&addr) {
            Some(hash) => *hash,
            None => {
                let hash = V2::hash_node(current_depth - 1, left_hash, right_hash);
                self.hashes_matrix.set(&addr, hash);
                hash
            }
        }
    }

    fn emulate_tree_to_get_path(
        &mut self,
        addr: Address,
        last_account: &Address,
        path: &mut AddressIterator,
        merkle_path: &mut Vec<MerklePath>,
    ) -> Fp {
        let tree_depth = self.depth as usize;

        if addr.length() == self.depth as usize {
            return self
                .get_account_hash(addr.to_index())
                .unwrap_or_else(|| self.hashes_matrix.empty_hash_at_depth(0));
        }

        let next_direction = path.next();

        // We go until the end of the path
        if let Some(direction) = next_direction.as_ref() {
            let child = match direction {
                Direction::Left => addr.child_left(),
                Direction::Right => addr.child_right(),
            };
            self.emulate_tree_to_get_path(child, last_account, path, merkle_path);
        };

        let depth_in_tree = tree_depth - addr.length();

        let mut get_child_hash = |addr: Address| match self.hashes_matrix.get(&addr) {
            Some(hash) => *hash,
            None => {
                if let Some(hash) = self.hashes_matrix.get(&addr) {
                    *hash
                } else if addr.is_before(last_account) {
                    self.emulate_tree_to_get_path(addr, last_account, path, merkle_path)
                } else {
                    self.hashes_matrix.empty_hash_at_depth(depth_in_tree - 1)
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

        match self.hashes_matrix.get(&addr) {
            Some(hash) => *hash,
            None => {
                let hash = V2::hash_node(depth_in_tree - 1, left, right);
                self.hashes_matrix.set(&addr, hash);
                hash
            }
        }
    }

    pub fn create_checkpoint(&self, directory_name: String) {
        println!("create_checkpoint {}", directory_name);
    }

    pub fn make_checkpoint(&self, directory_name: String) {
        println!("make_checkpoint {}", directory_name);
    }

    pub fn get_cached_hash(&self, addr: &Address) -> Option<Fp> {
        self.hashes_matrix.get(addr).copied()
    }

    pub fn set_cached_hash(&mut self, addr: &Address, hash: Fp) {
        self.hashes_matrix.set(addr, hash);
    }

    pub fn empty_hash_at_depth(&mut self, depth: usize) -> Fp {
        self.hashes_matrix.empty_hash_at_depth(depth)
    }

    pub fn invalidate_hashes(&mut self, account_index: AccountIndex) {
        self.hashes_matrix.invalidate_hashes(account_index)
    }
}

impl DatabaseImpl<V1> {
    pub fn create_account(
        &mut self,
        _account_id: (),
        account: AccountLegacy,
    ) -> Result<Address, DatabaseError> {
        // if self.root.is_none() {
        //     self.root = Some(NodeOrLeaf::Node(Node::default()));
        // }

        let location = match self.last_location.as_ref() {
            Some(last) => last.next().ok_or(DatabaseError::OutOfLeaves)?,
            None => Address::first(self.depth as usize),
        };

        assert_eq!(location.to_index(), self.accounts.len());
        self.accounts.push(Some(account));

        // let root = self.root.as_mut().unwrap();
        // let path_iter = location.clone().into_iter();
        // root.add_account_on_path(account, path_iter);

        self.last_location = Some(location.clone());
        self.naccounts += 1;

        Ok(location)
    }
}

impl DatabaseImpl<V2> {
    pub fn create_with_dir(depth: u8, dir_name: Option<PathBuf>) -> Self {
        assert!((1..0xfe).contains(&depth));

        let max_naccounts = 2u64.pow(depth.min(25) as u32);

        let uuid = next_uuid();

        let path = match dir_name {
            Some(dir_name) => dir_name,
            None => {
                let directory = "minadb-".to_owned() + &uuid;

                let mut path = PathBuf::from("/tmp");
                path.push(&directory);
                path
            }
        };

        // println!(
        //     "DB depth={:?} uuid={:?} pid={:?} path={:?}",
        //     depth,
        //     uuid,
        //     crate::util::pid(),
        //     path
        // );

        std::fs::create_dir_all(&path).ok();

        Self {
            depth,
            accounts: Vec::with_capacity(20_000),
            last_location: None,
            naccounts: 0,
            id_to_addr: HashMap::with_capacity(max_naccounts as usize / 2),
            token_to_account: HashMap::with_capacity(max_naccounts as usize / 2),
            uuid,
            directory: path,
            hashes_matrix: HashesMatrix::new(depth as usize),
            // root_hash: Default::default(),
        }
    }

    pub fn create(depth: u8) -> Self {
        Self::create_with_dir(depth, None)
    }

    pub fn root_hash(&mut self) -> Fp {
        self.emulate_tree_to_get_hash_at(Address::root())
    }

    // Do not use
    pub fn naccounts(&self) -> usize {
        self.accounts.iter().filter_map(Option::as_ref).count()
    }

    // fn naccounts_recursive(&self, elem: &NodeOrLeaf<T>, naccounts: &mut usize) {
    //     match elem {
    //         NodeOrLeaf::Leaf(_) => *naccounts += 1,
    //         NodeOrLeaf::Node(node) => {
    //             if let Some(left) = node.left.as_ref() {
    //                 self.naccounts_recursive(left, naccounts);
    //             };
    //             if let Some(right) = node.right.as_ref() {
    //                 self.naccounts_recursive(right, naccounts);
    //             };
    //         }
    //     }
    // }
}

impl BaseLedger for DatabaseImpl<V2> {
    fn to_list(&self) -> Vec<Account> {
        self.accounts
            .iter()
            .filter_map(Option::as_ref)
            .cloned()
            .collect()
        // let root = match self.root.as_ref() {
        //     Some(root) => root,
        //     None => return Vec::new(),
        // };

        // let mut accounts = Vec::with_capacity(100);

        // root.iter_recursive(&mut |account| {
        //     accounts.push(account.clone());
        //     ControlFlow::Continue(())
        // });

        // accounts
    }

    fn iter<F>(&self, fun: F)
    where
        F: FnMut(&Account),
    {
        self.accounts
            .iter()
            .filter_map(Option::as_ref)
            .for_each(fun);

        // let root = match self.root.as_ref() {
        //     Some(root) => root,
        //     None => return,
        // };

        // root.iter_recursive(&mut |account| {
        //     fun(account);
        //     ControlFlow::Continue(())
        // });
    }

    fn fold<B, F>(&self, init: B, mut fun: F) -> B
    where
        F: FnMut(B, &Account) -> B,
    {
        let mut accum = init;
        for account in self.accounts.iter().filter_map(Option::as_ref) {
            accum = fun(accum, account);
        }
        accum

        // let root = match self.root.as_ref() {
        //     Some(root) => root,
        //     None => return init,
        // };

        // let mut accum = Some(init);
        // root.iter_recursive(&mut |account| {
        //     let res = fun(accum.take().unwrap(), account);
        //     accum = Some(res);
        //     ControlFlow::Continue(())
        // });

        // accum.unwrap()
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
        let mut accum = init;
        for account in self.accounts.iter().filter_map(Option::as_ref) {
            let account_id = account.id();

            if !ignoreds.contains(&account_id) {
                accum = fun(accum, account);
            }
        }
        accum
        // self.fold(init, |accum, account| {
        //     let account_id = account.id();

        //     if !ignoreds.contains(&account_id) {
        //         fun(accum, account)
        //     } else {
        //         accum
        //     }
        // })
    }

    fn fold_until<B, F>(&self, init: B, mut fun: F) -> B
    where
        F: FnMut(B, &Account) -> ControlFlow<B, B>,
    {
        let mut accum = init;
        for account in self.accounts.iter().filter_map(Option::as_ref) {
            match fun(accum, account) {
                ControlFlow::Continue(v) => {
                    accum = v;
                }
                ControlFlow::Break(v) => {
                    accum = v;
                    break;
                }
            }
        }
        accum

        // let root = match self.root.as_ref() {
        //     Some(root) => root,
        //     None => return init,
        // };

        // let mut accum = Some(init);
        // root.iter_recursive(&mut |account| match fun(accum.take().unwrap(), account) {
        //     ControlFlow::Continue(account) => {
        //         accum = Some(account);
        //         ControlFlow::Continue(())
        //     }
        //     ControlFlow::Break(account) => {
        //         accum = Some(account);
        //         ControlFlow::Break(())
        //     }
        // });

        // accum.unwrap()
    }

    fn accounts(&self) -> HashSet<AccountId> {
        self.id_to_addr.keys().cloned().collect()
    }

    fn token_owner(&self, token_id: TokenId) -> Option<AccountId> {
        self.token_to_account.get(&token_id).cloned()
    }

    fn token_owners(&self) -> HashSet<AccountId> {
        self.token_to_account.values().cloned().collect()
    }

    fn tokens(&self, public_key: CompressedPubKey) -> HashSet<TokenId> {
        let mut set = HashSet::with_capacity(100);

        for account in self.accounts.iter().filter_map(Option::as_ref) {
            if account.public_key == public_key {
                set.insert(account.token_id.clone());
            }
        }

        // let root = match self.root.as_ref() {
        //     Some(root) => root,
        //     None => return HashSet::default(),
        // };

        // let mut set = HashSet::with_capacity(self.naccounts);

        // root.iter_recursive(&mut |account| {
        //     if account.public_key == public_key {
        //         set.insert(account.token_id.clone());
        //     }

        //     ControlFlow::Continue(())
        // });

        set
    }

    fn location_of_account(&self, account_id: &AccountId) -> Option<Address> {
        let res = self.id_to_addr.get(account_id).cloned();

        println!("location_of_account id={:?}\n{:?}", account_id, res);

        res
    }

    fn location_of_account_batch(
        &self,
        account_ids: &[AccountId],
    ) -> Vec<(AccountId, Option<Address>)> {
        let res: Vec<_> = account_ids
            .iter()
            .map(|account_id| {
                let addr = self.id_to_addr.get(account_id).cloned();
                (account_id.clone(), addr)
            })
            .collect();

        println!(
            "location_of_account_batch ids={:?}\nres={:?}={:?}",
            account_ids,
            res.len(),
            res
        );

        res
    }

    fn get_or_create_account(
        &mut self,
        account_id: AccountId,
        account: Account,
    ) -> Result<GetOrCreated, DatabaseError> {
        let result = self.create_account(account_id, account);

        if let Ok(GetOrCreated::Added(addr)) = result.as_ref() {
            let account_index = addr.to_index();
            self.hashes_matrix.invalidate_hashes(account_index);
        };

        result
    }

    fn close(&self) {
        println!(
            "close pid={:?} uuid={:?} path={:?}",
            crate::util::pid(),
            self.uuid,
            self.directory
        );
        // Drop
    }

    fn last_filled(&self) -> Option<Address> {
        self.last_location.clone()
    }

    fn get_uuid(&self) -> crate::base::Uuid {
        self.uuid.clone()
    }

    fn get_directory(&self) -> Option<PathBuf> {
        Some(self.directory.clone())
    }

    fn get_account_hash(&mut self, account_index: AccountIndex) -> Option<Fp> {
        let addr = Address::from_index(account_index, self.depth as usize);

        if let Some(hash) = self.hashes_matrix.get(&addr) {
            return Some(*hash);
        }

        let account = self.get(addr.clone())?;
        let hash = account.hash();

        self.hashes_matrix.set(&addr, hash);

        Some(hash)
    }

    fn get(&self, addr: Address) -> Option<Account> {
        let index = addr.to_index();
        let index: usize = index.0 as usize;

        self.accounts.get(index)?.clone()

        // let acc = self.root.as_ref()?.get_on_path(addr.into_iter()).cloned();

        // if let Some(account) = &acc {
        //     println!("ACCOUNT{:?}", account.hash().to_string());
        // };

        // acc
    }

    fn get_batch(&self, addr: &[Address]) -> Vec<(Address, Option<Account>)> {
        let res: Vec<_> = addr
            .iter()
            .map(|addr| (addr.clone(), self.get(addr.clone())))
            .collect();

        // let root = match self.root.as_ref() {
        //     Some(root) => Cow::Borrowed(root),
        //     None => Cow::Owned(NodeOrLeaf::Node(Node::default())),
        // };

        // let res: Vec<_> = addr
        //     .iter()
        //     .map(|addr| (addr.clone(), root.get_on_path(addr.iter()).cloned()))
        //     .collect();

        println!("get_batch addrs={:?}\nres={:?}={:?}", addr, res.len(), res);

        res
    }

    fn set(&mut self, addr: Address, account: Account) {
        let index = addr.to_index();

        self.hashes_matrix.invalidate_hashes(index.clone());

        let index: usize = index.0 as usize;

        if self.accounts.len() <= index {
            self.accounts.resize(index + 1, None);
        }

        // if self.root.is_none() {
        //     self.root = Some(NodeOrLeaf::Node(Node::default()));
        // }

        let id = account.id();
        // let root = self.root.as_mut().unwrap();

        // Remove account at the address and it's index
        if let Some(account) = self.get(addr.clone()) {
            let id = account.id();
            self.id_to_addr.remove(&id);
            self.token_to_account.remove(&id.token_id);
        } else {
            self.naccounts += 1;
        }

        self.token_to_account
            .insert(account.token_id.clone(), id.clone());
        self.id_to_addr.insert(id, addr.clone());
        self.accounts[index] = Some(account);
        // root.add_account_on_path(account, addr.iter());

        if self
            .last_location
            .as_ref()
            .map(|l| l.to_index() < addr.to_index())
            .unwrap_or(true)
        {
            self.last_location = Some(addr);
        }

        // self.root_hash.borrow_mut().take();
    }

    fn set_batch(&mut self, list: &[(Address, Account)]) {
        println!("SET_BATCH {:?}", list.len());
        // println!("SET_BATCH {:?} {:?}", list.len(), list);
        for (addr, account) in list {
            assert_eq!(addr.length(), self.depth as usize, "addr={:?}", addr);
            self.set(addr.clone(), account.clone());
        }
    }

    fn get_at_index(&self, index: AccountIndex) -> Option<Account> {
        let addr = Address::from_index(index, self.depth as usize);
        self.get(addr)
    }

    fn set_at_index(&mut self, index: AccountIndex, account: Account) -> Result<(), ()> {
        let addr = Address::from_index(index, self.depth as usize);
        self.set(addr, account);

        // self.root_hash.borrow_mut().take();

        Ok(())
    }

    fn index_of_account(&self, account_id: AccountId) -> Option<AccountIndex> {
        self.id_to_addr.get(&account_id).map(Address::to_index)
    }

    fn merkle_root(&mut self) -> Fp {
        // let now = crate::util::Instant::now();

        self.root_hash()

        // let root = match *self.root_hash.borrow() {
        //     Some(root) => root,
        //     None => self.root_hash(),
        // };

        // println!(
        //     "uuid={:?} ROOT={} num_account={:?} elapsed={:?}",
        //     self.get_uuid(),
        //     root,
        //     self.num_accounts(),
        //     now.elapsed(),
        // );

        // self.root_hash.borrow_mut().replace(root);

        // println!("PATH={:#?}", self.merkle_path(Address::first(self.depth as usize)));

        // self.merkle_path(Address::first(self.depth as usize));

        // root
    }

    fn merkle_path(&mut self, addr: Address) -> Vec<MerklePath> {
        println!("merkle_path called depth={:?} addr={:?}", self.depth, addr);

        let mut merkle_path = Vec::with_capacity(addr.length());
        let mut path = addr.into_iter();
        let addr = Address::root();

        let last_account = self
            .last_filled()
            .unwrap_or_else(|| Address::first(self.depth as usize));

        // let tree_index = TreeIndex::root(self.depth() as usize);

        self.emulate_tree_to_get_path(addr, &last_account, &mut path, &mut merkle_path);

        merkle_path
    }

    fn merkle_path_at_index(&mut self, index: AccountIndex) -> Vec<MerklePath> {
        let addr = Address::from_index(index, self.depth as usize);
        self.merkle_path(addr)
    }

    fn remove_accounts(&mut self, ids: &[AccountId]) {
        // let root = match self.root.as_mut() {
        //     Some(root) => root,
        //     None => return,
        // };

        let mut addrs = ids
            .iter()
            .map(|accound_id| self.id_to_addr.remove(accound_id).unwrap())
            .collect::<Vec<_>>();
        addrs.sort_by_key(Address::to_index);

        for addr in addrs.iter().rev() {
            // let leaf = match root.get_mut_leaf_on_path(addr.iter()) {
            //     Some(leaf) => leaf,
            //     None => continue,
            // };

            // let account = match leaf.account.take() {
            //     Some(account) => account,
            //     None => continue,
            // };

            let account_index = addr.to_index();
            self.hashes_matrix.invalidate_hashes(account_index);

            let account = match self.remove(addr.clone()) {
                Some(account) => account,
                None => continue,
            };

            // let index = addr.to_index();
            // let account = std::mem::take()

            let id = account.id();
            self.id_to_addr.remove(&id);
            self.token_to_account.remove(&id.token_id);

            self.naccounts = self
                .naccounts
                .checked_sub(1)
                .expect("invalid naccounts counter");

            if self
                .last_location
                .as_ref()
                .map(|last| last == addr)
                .unwrap_or(false)
            {
                self.last_location = addr.prev();
            }
        }

        // self.root_hash.borrow_mut().take();
    }

    fn detached_signal(&mut self) {
        todo!()
    }

    fn depth(&self) -> u8 {
        self.depth
    }

    fn num_accounts(&self) -> usize {
        self.naccounts
    }

    fn merkle_path_at_addr(&mut self, addr: Address) -> Vec<MerklePath> {
        self.merkle_path(addr)
    }

    fn get_inner_hash_at_addr(&mut self, addr: Address) -> Result<Fp, ()> {
        let res = self.emulate_tree_to_get_hash_at(addr.clone());

        println!("get_inner_hash_at_addr addr={:?} hash={}", addr, res);

        Ok(res)
    }

    fn set_inner_hash_at_addr(&mut self, _addr: Address, _hash: Fp) -> Result<(), ()> {
        // No-op for now, because we don't store the hashes anywhere
        Ok(())
    }

    fn set_all_accounts_rooted_at(
        &mut self,
        addr: Address,
        accounts: &[Account],
    ) -> Result<(), ()> {
        if addr.length() > self.depth as usize {
            return Err(());
        }

        for (child_addr, account) in addr.iter_children(self.depth as usize).zip(accounts) {
            self.set(child_addr, account.clone());
        }

        Ok(())
    }

    fn get_all_accounts_rooted_at(&self, addr: Address) -> Option<Vec<(Address, Account)>> {
        if addr.length() > self.depth as usize {
            return None;
        }

        // let root = match self.root.as_ref() {
        //     Some(root) => root,
        //     None => return None,
        // };

        let children = addr.iter_children(self.depth as usize);
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
