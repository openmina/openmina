use std::{borrow::Borrow, cell::RefCell, collections::HashSet, hash::Hash, rc::Rc, str::FromStr};

use mina_hasher::Fp;
use ocaml_interop::{
    impl_to_ocaml_polymorphic_variant, impl_to_ocaml_variant, ocaml_export, DynBox, OCaml,
    OCamlBytes, OCamlInt, OCamlList, OCamlRef, OCamlRuntime, ToOCaml,
};
use serde::Deserialize;

use crate::{
    account::{Account, AccountId, BigInt, NonZeroCurvePointUncompressedStableV1},
    address::Address,
    base::{AccountIndex, BaseLedger, MerklePath},
    ffi::DatabaseFFI,
    Mask, UnregisterBehavior,
};

// #[derive(Clone)]
struct MaskFFI(Rc<RefCell<Option<Mask>>>);

impl Drop for MaskFFI {
    fn drop(&mut self) {
        let mask_id = RefCell::borrow(&self.0).as_ref().map(|mask| mask.short());
        eprintln!("rust_mask_drop {:?}", mask_id);
    }
}

fn with_mask<F, R>(rt: &mut &mut OCamlRuntime, mask: OCamlRef<DynBox<MaskFFI>>, fun: F) -> R
where
    F: FnOnce(&mut Mask) -> R,
{
    // println!("111");
    let mask = rt.get(mask);
    // println!("222");
    let mask: &MaskFFI = mask.borrow();
    // println!("333");
    let mut mask = mask.0.borrow_mut();

    // println!(
    //     "with_mask {:p}",
    //     Arc::as_ptr(&mask.borrow().as_ref().unwrap().inner)
    // );

    // let mut db = db.0.borrow_mut();

    fun(mask.as_mut().unwrap())
}

pub enum PolymorphicGetOrAdded {
    Added,
    Existed,
}

impl_to_ocaml_polymorphic_variant! {
    PolymorphicGetOrAdded {
        PolymorphicGetOrAdded::Added,
        PolymorphicGetOrAdded::Existed,
    }
}

pub enum PolymorphicPath {
    Left(Vec<u8>),
    Right(Vec<u8>),
}

impl_to_ocaml_polymorphic_variant! {
    PolymorphicPath {
        PolymorphicPath::Left(hash: OCamlBytes),
        PolymorphicPath::Right(hash: OCamlBytes),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DatabaseErrorFFI {
    OutOfLeaves,
}

impl_to_ocaml_variant! {
    DatabaseErrorFFI {
        DatabaseErrorFFI::OutOfLeaves,
    }
}

// type rust_grandchildren = [ `Check | `Recursive | `I_promise_I_am_reparenting_this_mask ]
pub enum PolymorphicGrandchildren {
    Check,
    Recursive,
    I_promise_I_am_reparenting_this_mask,
}

// static DB_CLOSED: Lazy<Mutex<HashMap<PathBuf, Database<V2>>>> =
//     Lazy::new(|| Mutex::new(HashMap::with_capacity(16)));

// static DB_CLOSED: Arc<Mutex<Option<HashMap<PathBuf, Database<V2>>>>> = Arc::new(Mutex::new(None));

fn get_list_of<'a, T>(
    rt: &'a mut &mut OCamlRuntime,
    list: OCamlRef<OCamlList<OCamlBytes>>,
) -> Vec<T>
where
    T: Deserialize<'a>,
{
    let mut list_ref = rt.get(list);
    let mut list = Vec::with_capacity(2048);

    while let Some((head, tail)) = list_ref.uncons() {
        let object: T = serde_binprot::from_slice(head.as_bytes()).unwrap();
        list.push(object);
        list_ref = tail;
    }

    list
}

fn get_set_of<'a, T>(
    rt: &'a mut &mut OCamlRuntime,
    list: OCamlRef<OCamlList<OCamlBytes>>,
) -> HashSet<T>
where
    T: Deserialize<'a> + Hash + Eq,
{
    let mut list_ref = rt.get(list);
    let mut set = HashSet::with_capacity(2048);

    while let Some((head, tail)) = list_ref.uncons() {
        let object: T = serde_binprot::from_slice(head.as_bytes()).unwrap();
        set.insert(object);
        list_ref = tail;
    }

    set
}

fn get_list_addr_account<'a>(
    rt: &'a mut &mut OCamlRuntime,
    list: OCamlRef<OCamlList<(String, OCamlBytes)>>,
) -> Vec<(Address, Account)> {
    let mut list_ref = rt.get(list);
    let mut list = Vec::with_capacity(2048);

    while let Some((head, tail)) = list_ref.uncons() {
        let addr = head.fst().as_str();
        let account = head.snd().as_bytes();

        let addr = Address::try_from(addr).unwrap();
        let object: Account = serde_binprot::from_slice(account).unwrap();
        list.push((addr, object));

        list_ref = tail;
    }

    list
}

fn get_addr(rt: &mut &mut OCamlRuntime, addr: OCamlRef<String>) -> Address {
    let addr_ref = rt.get(addr);
    Address::try_from(addr_ref.as_str()).unwrap()
}

fn get<'a, T>(rt: &'a mut &mut OCamlRuntime, object: OCamlRef<OCamlBytes>) -> T
where
    T: Deserialize<'a>,
{
    let object_ref = rt.get(object);
    serde_binprot::from_slice(object_ref.as_bytes()).unwrap()
}

fn get_index(rt: &mut &mut OCamlRuntime, index: OCamlRef<OCamlInt>) -> AccountIndex {
    let index: i64 = index.to_rust(rt);
    let index: u64 = index.try_into().unwrap();
    AccountIndex(index)
}

fn hash_to_ocaml(hash: Fp) -> Vec<u8> {
    let hash: BigInt = hash.into();
    serde_binprot::to_vec(&hash).unwrap()
}

fn get_cloned_mask(
    rt: &mut &mut OCamlRuntime,
    mask: OCamlRef<DynBox<MaskFFI>>,
) -> Rc<RefCell<Option<Mask>>> {
    let mask = rt.get(mask);
    let mask: &MaskFFI = mask.borrow();
    // let mut mask = mask.0.borrow_mut();
    Rc::clone(&mask.0)
}

ocaml_export! {
    fn rust_mask_create(
        rt,
        depth: OCamlRef<OCamlInt>,
    ) -> OCaml<DynBox<MaskFFI>> {
        let depth: i64 = depth.to_rust(rt);
        let depth: usize = depth.try_into().unwrap();

        let mask = Mask::new_unattached(depth);

        let mask = MaskFFI(Rc::new(RefCell::new(Some(mask))));

        OCaml::box_value(rt, mask)
    }

    fn rust_cast_database_to_mask(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>
    ) -> OCaml<DynBox<MaskFFI>> {
        // let bt = backtrace::Backtrace::new();
        // eprintln!("rust_cast_database_to_mask bt={:#?}", bt);

        let db = {
            let db = rt.get(db);
            let db: &DatabaseFFI = db.borrow();

            // eprintln!("CAST_DATABASE_TO_MASK PTR={:p}", Rc::as_ptr(&db.0));

            let db = db.0.borrow_mut();
            let db = db.as_ref().unwrap().clone();

            db
        };

        // eprintln!("AAA");
        let mask = Mask::new_root(db);
        let mask = MaskFFI(Rc::new(RefCell::new(Some(mask))));
        // eprintln!("BBB");

        OCaml::box_value(rt, mask)
    }

    fn rust_cast(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>
    ) -> OCaml<DynBox<MaskFFI>> {
        // let bt = backtrace::Backtrace::new();
        // eprintln!("rust_cast bt={:#?}", bt);

        let mask = rt.get(mask);

        // let mask = with_mask(rt, mask, |mask| {
        //     mask.clone()
        // });
        // let mask = MaskFFI(Rc::new(RefCell::new(Some(mask))));

        mask
        // OCaml::box_value(rt, mask)
    }

    fn rust_mask_copy(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>
    ) -> OCaml<DynBox<MaskFFI>> {
        let mask = with_mask(rt, mask, |mask| {
            mask.clone()
        });
        let mask = MaskFFI(Rc::new(RefCell::new(Some(mask))));

        OCaml::box_value(rt, mask)
    }

    fn rust_mask_set_parent(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        parent: OCamlRef<DynBox<MaskFFI>>
    ) -> OCaml<DynBox<MaskFFI>> {
        let parent = {
            let parent = rt.get(parent);
            let parent: &MaskFFI = parent.borrow();
            let parent = parent.0.borrow_mut();
            (*parent).as_ref().unwrap().clone()
        };

        let mask = with_mask(rt, mask, |mask| {
            mask.set_parent(&parent)
        });
        let mask = MaskFFI(Rc::new(RefCell::new(Some(mask))));

        OCaml::box_value(rt, mask)
    }

    fn rust_mask_register_mask(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        mask2: OCamlRef<DynBox<MaskFFI>>
    ) -> OCaml<DynBox<MaskFFI>> {
        // let bt = backtrace::Backtrace::new();

        // println!("AAA bt={:#?}", bt);
        let mask2 = {
            let mask2 = rt.get(mask2);
            let mask2: &MaskFFI = mask2.borrow();
            let mask2 = mask2.0.borrow_mut();
            // println!("BBB {:p}", Arc::as_ptr(&mask2.borrow().as_ref().unwrap().inner));
            (*mask2).as_ref().unwrap().clone()
        };
        // println!("CCC");

        let mask = with_mask(rt, mask, |mask| {
            mask.register_mask(mask2)
        });
        // println!("DDD");

        let mask = MaskFFI(Rc::new(RefCell::new(Some(mask))));

        OCaml::box_value(rt, mask)
    }

    fn rust_mask_unregister_mask(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        behavior: OCamlRef<PolymorphicGrandchildren>
    ) -> OCaml<DynBox<MaskFFI>> {
        let behavior = rt.get(behavior);

        let behavior = ocaml_interop::ocaml_unpack_variant! {
            behavior => {
                Check => UnregisterBehavior::Check,
                Recursive => UnregisterBehavior::Recursive,
                I_promise_I_am_reparenting_this_mask => UnregisterBehavior::IPromiseIAmReparentingThisMask,
            }
        }.unwrap_or(UnregisterBehavior::Check);

        let mask = with_mask(rt, mask, |mask| {
            mask.unregister_mask(behavior)
        });

        let mask = MaskFFI(Rc::new(RefCell::new(Some(mask))));

        OCaml::box_value(rt, mask)
    }

    fn rust_mask_remove_and_reparent(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>
    ) {
        with_mask(rt, mask, |mask| {
            mask.remove_and_reparent()
        });

        OCaml::unit()
    }

    fn rust_mask_commit(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>
    ) {
        with_mask(rt, mask, |mask| {
            mask.commit()
        });

        OCaml::unit()
    }

    fn rust_mask_get_uuid(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>
    ) -> OCaml<String> {
        let uuid = with_mask(rt, mask, |mask| {
            mask.get_uuid()
        });

        uuid.to_ocaml(rt)
    }

    fn rust_mask_get_directory(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>
    ) -> OCaml<Option<String>> {
        let dir = with_mask(rt, mask, |mask| {
            mask.get_directory().map(|d| d.into_os_string().into_string().unwrap())
        });

        dir.to_ocaml(rt)
    }

    fn rust_mask_depth(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>
    ) -> OCaml<OCamlInt> {
        let depth = with_mask(rt, mask, |mask| {
            mask.depth() as i64
        });

        depth.to_ocaml(rt)
    }

    fn rust_mask_close(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>
    ) {
        // let mask = rt.get(mask);
        // let mask: &MaskFFI = mask.borrow();

        with_mask(rt, mask, |mask| {
            mask.close();
        });

        OCaml::unit()
    }

    fn rust_mask_get(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        addr: OCamlRef<String>,
    ) -> OCaml<Option<OCamlBytes>> {
        let addr = get_addr(rt, addr);

        let account = with_mask(rt, mask, |mask| {
            mask.get(addr)
        }).map(|account| {
            serde_binprot::to_vec(&account).unwrap()
        });

        account.to_ocaml(rt)
    }

    fn rust_mask_get_batch(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        addrs: OCamlRef<OCamlList<OCamlBytes>>,
    ) -> OCaml<OCamlList<(String, Option<OCamlBytes>)>> {
        let mut addrs_ref = rt.get(addrs);

        let mut addrs = Vec::with_capacity(2048);
        while let Some((head, tail)) = addrs_ref.uncons() {
            let addr = Address::try_from(head.as_str()).unwrap();
            addrs.push(addr);
            addrs_ref = tail;
        }

        let accounts: Vec<(String, Option<Vec<u8>>)> = with_mask(rt, mask, |mask| {
            mask.get_batch(&addrs)
        }).into_iter()
          .map(|(addr, opt_account)| {
              let addr = addr.to_string();
              let opt_account = opt_account.map(|acc| serde_binprot::to_vec(&acc).unwrap());
              (addr, opt_account)
          })
          .collect();

        accounts.to_ocaml(rt)
    }

    fn rust_mask_get_list(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
    ) -> OCaml<OCamlList<OCamlBytes>> {

        let accounts: Vec<Vec<u8>> = with_mask(rt, mask, |mask| {
            mask.to_list()
        }).into_iter()
          .map(|account| {
              serde_binprot::to_vec(&account).unwrap()
          })
          .collect();

        accounts.to_ocaml(rt)
    }

    fn rust_mask_accounts(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
    ) -> OCaml<OCamlList<OCamlBytes>> {

        let accounts: Vec<Vec<u8>> = with_mask(rt, mask, |mask| {
            mask.accounts()
        }).into_iter()
          .map(|account_id| {
              serde_binprot::to_vec(&account_id).unwrap()
          })
          .collect();

        accounts.to_ocaml(rt)
    }

    fn rust_mask_get_inner_hash_at_addr(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        addr: OCamlRef<String>,
    ) -> OCaml<OCamlBytes> {
        let addr = get_addr(rt, addr);

        let hash = with_mask(rt, mask, |mask| {
            mask.get_inner_hash_at_addr(addr)
        }).map(|hash| {
              hash_to_ocaml(hash)
          })
          .unwrap();

        hash.to_ocaml(rt)
    }

    fn rust_mask_set_inner_hash_at_addr(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        addr: OCamlRef<String>,
        hash: OCamlRef<String>,
    ) {
        let addr = get_addr(rt, addr);

        let hash: String = hash.to_rust(rt);
        let hash = Fp::from_str(&hash).unwrap();

        with_mask(rt, mask, |mask| {
            mask.set_inner_hash_at_addr(addr, hash).unwrap()
        });

        OCaml::unit()
    }

    fn rust_mask_get_at_index(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        index: OCamlRef<OCamlInt>
    ) -> OCaml<OCamlBytes> {
        let index = get_index(rt, index);

        let account = with_mask(rt, mask, |mask| {
            mask.get_at_index(index).unwrap()
        });
        let account = serde_binprot::to_vec(&account).unwrap();

        account.to_ocaml(rt)
    }

    fn rust_mask_location_of_account(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        account_id: OCamlRef<OCamlBytes>
    ) -> OCaml<Option<String>> {
        let account_id = get(rt, account_id);

        let addr = with_mask(rt, mask, |mask| {
            mask.location_of_account(&account_id)
        }).map(|addr| {
            addr.to_string()
        });

        addr.to_ocaml(rt)
    }

    fn rust_mask_location_of_account_batch(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        account_ids: OCamlRef<OCamlList<OCamlBytes>>
    ) -> OCaml<OCamlList<(OCamlBytes, Option<String>)>> {
        let account_ids = get_list_of::<AccountId>(rt, account_ids);

        let addrs = with_mask(rt, mask, |mask| {
            mask.location_of_account_batch(&account_ids)
        }).into_iter()
          .map(|(account_id, opt_addr)| {
              let account_id = serde_binprot::to_vec(&account_id).unwrap();
              let addr = opt_addr.map(|addr| addr.to_string());
              (account_id, addr)
        })
          .collect::<Vec<_>>();

        addrs.to_ocaml(rt)
    }

    fn rust_mask_last_filled(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
    ) -> OCaml<Option<String>> {
        let addr = with_mask(rt, mask, |mask| {
            mask.last_filled()
        }).map(|addr| {
            addr.to_string()
        });

        addr.to_ocaml(rt)
    }

    fn rust_mask_token_owners(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
    ) -> OCaml<OCamlList<OCamlBytes>> {
        let owners = with_mask(rt, mask, |mask| {
            mask.token_owners()
        }).iter()
          .map(|account_id| {
              serde_binprot::to_vec(account_id).unwrap()
        })
          .collect::<Vec<_>>();

        owners.to_ocaml(rt)
    }

    fn rust_mask_token_owner(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        token_id: OCamlRef<OCamlBytes>,
    ) -> OCaml<Option<OCamlBytes>> {
        let token_id = get(rt, token_id);

        let owner = with_mask(rt, mask, |mask| {
            mask.token_owner(token_id)
        }).map(|account_id| {
            serde_binprot::to_vec(&account_id).unwrap()
        });

        owner.to_ocaml(rt)
    }

    fn rust_mask_tokens(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        pubkey: OCamlRef<OCamlBytes>,
    ) -> OCaml<OCamlList<OCamlBytes>> {
        let pubkey: NonZeroCurvePointUncompressedStableV1 = get(rt, pubkey);

        let tokens = with_mask(rt, mask, |mask| {
            mask.tokens(pubkey.into())
        }).iter()
          .map(|token_id| {
            serde_binprot::to_vec(token_id).unwrap()
        })
          .collect::<Vec<_>>();

        tokens.to_ocaml(rt)
    }

    fn rust_mask_set(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        addr: OCamlRef<String>,
        account: OCamlRef<OCamlBytes>,
    ) {
        let addr = get_addr(rt, addr);
        let account = get(rt, account);

        with_mask(rt, mask, |mask| {
            mask.set(addr, account)
        });

        OCaml::unit()
    }

    fn rust_mask_index_of_account(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        account_id: OCamlRef<OCamlBytes>
    ) -> OCaml<OCamlInt> {
        let account_id = get(rt, account_id);

        let index = with_mask(rt, mask, |mask| {
            mask.index_of_account(account_id)
        }).map(|index| {
            index.0 as i64
        })
          .unwrap();

        index.to_ocaml(rt)
    }

    fn rust_mask_set_at_index(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        index: OCamlRef<OCamlInt>,
        account: OCamlRef<OCamlBytes>,
    ) {
        let index = get_index(rt, index);
        let account = get(rt, account);

        with_mask(rt, mask, |mask| {
            mask.set_at_index(index, account)
        }).unwrap();

        OCaml::unit()
    }

    fn rust_mask_get_or_create_account(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        account_id: OCamlRef<OCamlBytes>,
        account: OCamlRef<OCamlBytes>,
    ) -> OCaml<Result<(PolymorphicGetOrAdded, String), DatabaseErrorFFI>> {
        let account_id = get(rt, account_id);
        let account = get(rt, account);

        let result = with_mask(rt, mask, |mask| {
            mask.get_or_create_account(account_id, account)
        });

        use crate::base::GetOrCreated::*;
        use crate::database::DatabaseError::*;

        let result = match result {
            Ok(value) => {
                let get_or_added = match value {
                    Added(_) => PolymorphicGetOrAdded::Added,
                    Existed(_) => PolymorphicGetOrAdded::Existed,
                };
                let addr = value.addr();
                Ok((get_or_added, addr.to_string()))
            },
            Err(e) => match e {
                OutOfLeaves => Err(DatabaseErrorFFI::OutOfLeaves),
            },
        };

        result.to_ocaml(rt)
    }

    fn rust_mask_num_accounts(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>
    ) -> OCaml<OCamlInt> {
        let num_accounts = with_mask(rt, mask, |mask| {
            mask.num_accounts() as i64
        });

        num_accounts.to_ocaml(rt)
    }

    fn rust_mask_iter(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        ocaml_method: OCamlRef<fn(OCamlBytes)>,
    ) {
        let (num_accounts, depth) = with_mask(rt, mask, |mask| {
            (mask.num_accounts(), mask.depth())
        });

        let ocaml_method = ocaml_method.to_boxroot(rt);

        for index in 0..num_accounts {
            let index = AccountIndex(index as u64);
            let addr = Address::from_index(index, depth as usize);

            let account = with_mask(rt, mask, |mask| {
                mask.get(addr)
            });

            let account = match account {
                Some(account) => account,
                None => continue,
            };

            let account = serde_binprot::to_vec(&account).unwrap();

            let _: Result<OCaml<()>, _> = ocaml_method.try_call(rt, &account);
        }

        OCaml::unit()
    }

    fn rust_mask_foldi(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        ocaml_method: OCamlRef<fn(String, OCamlBytes)>,
    ) {
        let (num_accounts, depth) = with_mask(rt, mask, |mask| {
            (mask.num_accounts(), mask.depth())
        });

        let ocaml_method = ocaml_method.to_boxroot(rt);

        for index in 0..num_accounts {
            let index = AccountIndex(index as u64);
            let addr = Address::from_index(index, depth as usize);

            let account = with_mask(rt, mask, |mask| {
                mask.get(addr.clone())
            });

            let account = match account {
                Some(account) => account,
                None => continue,
            };

            let account = serde_binprot::to_vec(&account).unwrap();
            let addr = addr.to_string();

            let _: Result<OCaml<()>, _> = ocaml_method.try_call(rt, &addr, &account);
        }

        OCaml::unit()
    }

    fn rust_mask_foldi_with_ignored_accounts(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        ignored_accounts: OCamlRef<OCamlList<OCamlBytes>>,
        ocaml_method: OCamlRef<fn(String, OCamlBytes)>,
    ) {
        let (num_accounts, depth) = with_mask(rt, mask, |mask| {
            (mask.num_accounts(), mask.depth())
        });

        let ignored_accounts = get_set_of::<AccountId>(rt, ignored_accounts);
        let ocaml_method = ocaml_method.to_boxroot(rt);

        for index in 0..num_accounts {
            let index = AccountIndex(index as u64);
            let addr = Address::from_index(index, depth as usize);

            let account = with_mask(rt, mask, |mask| {
                mask.get(addr.clone())
            });

            let account = match account {
                Some(account) => account,
                None => continue,
            };

            if ignored_accounts.contains(&account.id()) {
                continue;
            }

            let account = serde_binprot::to_vec(&account).unwrap();
            let addr = addr.to_string();

            let _: Result<OCaml<()>, _> = ocaml_method.try_call(rt, &addr, &account);
        }

        OCaml::unit()
    }

    fn rust_mask_merkle_root(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
    ) -> OCaml<OCamlBytes> {
        let hash = with_mask(rt, mask, |mask| {
            mask.merkle_root()
        });

        let hash = hash_to_ocaml(hash);

        hash.to_ocaml(rt)
    }

    fn rust_mask_remove_accounts(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        account_ids: OCamlRef<OCamlList<OCamlBytes>>,
    ) {
        let account_ids = get_list_of(rt, account_ids);

        with_mask(rt, mask, |mask| {
            mask.remove_accounts(&account_ids)
        });

        OCaml::unit()
    }

    fn rust_mask_set_all_accounts_rooted_at(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        addr: OCamlRef<String>,
        accounts: OCamlRef<OCamlList<OCamlBytes>>,
    ) {
        let addr = get_addr(rt, addr);
        let accounts = get_list_of(rt, accounts);

        with_mask(rt, mask, |mask| {
            mask.set_all_accounts_rooted_at(addr, &accounts).unwrap()
        });

        OCaml::unit()
    }

    fn rust_mask_set_batch_accounts(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        accounts: OCamlRef<OCamlList<(String, OCamlBytes)>>,
    ) {
        let accounts = get_list_addr_account(rt, accounts);

        with_mask(rt, mask, |mask| {
            mask.set_batch_accounts(&accounts)
        });

        OCaml::unit()
    }

    fn rust_mask_get_all_accounts_rooted_at(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        addr: OCamlRef<String>,
    ) -> OCaml<OCamlList<(String, OCamlBytes)>> {
        let addr = get_addr(rt, addr);

        let accounts = with_mask(rt, mask, |mask| {
            mask.get_all_accounts_rooted_at(addr)
        }).unwrap_or_default()
          .iter()
            .map(|(addr, account)| {
              let addr = addr.to_string();
              let account = serde_binprot::to_vec(account).unwrap();
              (addr, account)
            })
            .collect::<Vec<_>>();

        accounts.to_ocaml(rt)
    }

    fn rust_mask_merkle_path(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        addr: OCamlRef<String>,
    ) -> OCaml<OCamlList<PolymorphicPath>> {
        let addr = get_addr(rt, addr);

        let path = with_mask(rt, mask, |mask| {
            mask.merkle_path(addr)
        }).into_iter()
          .map(|path| {
              match path {
                  MerklePath::Left(hash) => PolymorphicPath::Left(hash_to_ocaml(hash)),
                  MerklePath::Right(hash) => PolymorphicPath::Right(hash_to_ocaml(hash)),
              }
          })
          .collect::<Vec<_>>();

        path.to_ocaml(rt)
    }

    fn rust_mask_merkle_path_at_addr(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        addr: OCamlRef<String>,
    ) -> OCaml<OCamlList<PolymorphicPath>> {
        let addr = get_addr(rt, addr);

        let path = with_mask(rt, mask, |mask| {
            mask.merkle_path(addr)
        }).into_iter()
          .map(|path| {
              match path {
                  MerklePath::Left(hash) => PolymorphicPath::Left(hash_to_ocaml(hash)),
                  MerklePath::Right(hash) => PolymorphicPath::Right(hash_to_ocaml(hash)),
              }
          })
          .collect::<Vec<_>>();

        path.to_ocaml(rt)
    }

    fn rust_mask_merkle_path_at_index(
        rt,
        mask: OCamlRef<DynBox<MaskFFI>>,
        index: OCamlRef<OCamlInt>,
    ) -> OCaml<OCamlList<PolymorphicPath>> {
        let index = get_index(rt, index);

        let path = with_mask(rt, mask, |mask| {
            let depth = mask.depth();
            let addr = Address::from_index(index, depth as usize);
            mask.merkle_path(addr)
        }).into_iter()
          .map(|path| {
              match path {
                  MerklePath::Left(hash) => PolymorphicPath::Left(hash_to_ocaml(hash)),
                  MerklePath::Right(hash) => PolymorphicPath::Right(hash_to_ocaml(hash)),
                  // MerklePath::Left(hash) => PolymorphicPath::Left(hash.to_string()),
                  // MerklePath::Right(hash) => PolymorphicPath::Right(hash.to_string()),
              }
          })
          .collect::<Vec<_>>();

        path.to_ocaml(rt)
    }
}

// database_create : int -> database = "rust_mask_create"
// database_get_uuid : database -> string = "rust_mask_get_uuid"
// database_depth : database -> int = "rust_mask_depth"
// database_create_checkpoint : database -> database = "rust_mask_create_checkpoint"
// database_make_checkpoint : database -> unit = "rust_mask_make_checkpoint"
// database_close : database -> unit = "rust_mask_close"
// database_get : database -> addr -> account option = "rust_mask_get"
// database_get_batch : database -> addr list -> (addr * (account option)) list = "rust_mask_get_batch"
// database_get_list : database -> bytes list = "rust_mask_get_list"
// database_accounts : database -> bytes list = "rust_mask_accounts"
// database_get_inner_hash_at_addr : database -> addr -> bytes = "rust_mask_get_inner_hash_at_addr"
// database_set_inner_hash_at_addr : database -> addr -> bytes -> unit = "rust_mask_set_inner_hash_at_addr"
// database_get_at_index : database -> int -> account = "rust_mask_get_at_index"
// database_iter : database -> (int -> bytes -> unit) -> unit = "rust_mask_iter"
// database_location_of_account : database -> account_id -> addr option = "rust_mask_location_of_account"
// database_location_of_account_batch : database -> account_id list -> (account_id * (addr option)) list = "rust_mask_location_of_account_batch"

// database_last_filled : database -> addr option = "rust_mask_last_filled"
// database_token_owners : database -> bytes list = "rust_mask_token_owners"
// database_token_owner : database -> token_id -> account_id option = "rust_mask_token_owner"
// database_tokens : database -> pubkey -> token_id list = "rust_mask_tokens"
// database_set : database -> addr -> account -> unit = "rust_mask_set"
// database_index_of_account : database -> account_id -> int = "rust_mask_index_of_account"
// database_set_at_index : database -> int -> account -> unit = "rust_mask_set_at_index"
// database_get_or_create_account : database -> account_id -> account -> (([ `Added | `Existed ] * addr), rust_maskerror) result = "rust_mask_get_or_create_account"
// database_num_accounts : database -> int = "rust_mask_num_accounts"
// database_fold_with_account_ids : database -> bytes list -> bytes -> (bytes -> unit) -> bytes = "rust_mask_fold_with_ignored_accounts"
// database_fold : database -> bytes -> (bytes -> unit) -> bytes = "rust_mask_fold"
// database_fold_until : database -> bytes -> (bytes -> bool) -> bytes = "rust_mask_fold_until"
// database_merkle_root : database -> bytes = "rust_mask_merkle_root"
// database_remove_accounts : database -> account_id list -> unit = "rust_mask_remove_accounts"
// database_merkle_path : database -> addr -> bytes list = "rust_mask_merkle_path"
// database_merkle_path_at_addr : database -> bytes -> bytes list = "rust_mask_merkle_path_at_addr"
// database_merkle_path_at_index : database -> int -> bytes list = "rust_mask_merkle_path_at_index"
// database_set_all_accounts_rooted_at : database -> addr -> bytes list -> unit = "rust_mask_set_all_accounts_rooted_at"
// database_set_batch_accounts : database -> (addr * account) list -> unit = "rust_mask_set_batch_accounts"
// database_get_all_accounts_rooted_at : database -> addr -> (addr * account) list = "rust_mask_get_all_accounts_rooted_at"

// (* TODO: Make those method *)
// database_foldi : database -> (addr -> bytes -> unit) -> unit = "rust_mask_foldi"
// database_foldi_with_ignored_accounts : database -> account list -> (addr -> bytes -> unit) -> unit = "rust_mask_foldi_with_ignored_accounts"

#[allow(clippy::let_and_return)]
fn impl_rust_random_account() -> Vec<u8> {
    // println!("rust_random_account begin");

    let account = Account::rand();
    let ser = serde_binprot::to_vec(&account).unwrap();

    // let ser: Vec<u8> = vec![
    //     178, 29, 73, 50, 85, 80, 131, 166, 53, 11, 48, 224, 103, 89, 161, 207, 149, 31, 170, 21,
    //     165, 181, 94, 18, 149, 177, 54, 71, 185, 77, 109, 49, 1, 144, 247, 164, 171, 110, 24, 3,
    //     12, 25, 163, 63, 125, 83, 66, 174, 2, 160, 62, 45, 137, 185, 47, 16, 129, 145, 190, 203,
    //     124, 35, 119, 251, 26, 1, 1, 6, 49, 50, 56, 54, 56, 56, 252, 29, 154, 218, 214, 79, 98,
    //     177, 181, 253, 181, 152, 127, 0, 145, 177, 91, 155, 59, 239, 161, 174, 217, 42, 201, 30,
    //     46, 11, 187, 88, 49, 5, 111, 254, 222, 87, 42, 45, 90, 1, 236, 173, 205, 215, 241, 20, 0,
    //     77, 12, 197, 234, 69, 202, 22, 55, 50, 183, 255, 238, 8, 29, 79, 199, 92, 12, 146, 223,
    //     105, 45, 135, 77, 89, 73, 141, 11, 137, 28, 54, 21, 0, 1, 4, 4, 1, 0, 4, 3, 4, 3, 2, 3, 0,
    //     6, 49, 49, 56, 54, 54, 51,
    // ];

    // let account: Account = serde_binprot::from_slice(&ser).unwrap();

    // // account2.permissions = Permissions::user_default();

    // // let account_hash2 = account2.hash();

    // // println!("HASH2={:?}", account_hash2.to_string());

    // // let mut account2 = Account::empty();

    // // account2.public_key = account.public_key;
    // // account2.token_id = account.token_id;
    // // // account2.token_permissions = account.token_permissions;
    // // account2.token_permissions = TokenPermissions::TokenOwned { disable_new_accounts: false };

    // // println!("ACCOUNT={:#?}", account2);

    // let ser = serde_binprot::to_vec(&account).unwrap();

    // println!("rust_random_account end");

    ser
}

// pub struct Account {
//     pub public_key: CompressedPubKey,         // Public_key.Compressed.t
//     pub token_id: TokenId,                    // Token_id.t
//     pub token_permissions: TokenPermissions,  // Token_permissions.t
//     pub token_symbol: TokenSymbol,            // Token_symbol.t
//     pub balance: Balance,                     // Balance.t
//     pub nonce: Nonce,                         // Nonce.t
//     pub receipt_chain_hash: ReceiptChainHash, // Receipt.Chain_hash.t
//     pub delegate: Option<CompressedPubKey>,   // Public_key.Compressed.t option
//     pub voting_for: VotingFor,                // State_hash.t
//     pub timing: Timing,                       // Timing.t
//     pub permissions: Permissions<AuthRequired>, // Permissions.t
//     pub zkapp: Option<ZkAppAccount>,          // Zkapp_account.t
//     pub zkapp_uri: String,                    // string
// }
