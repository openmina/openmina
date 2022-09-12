use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    collections::HashSet,
    hash::Hash,
    rc::Rc,
    str::FromStr,
    sync::Mutex,
};

use mina_hasher::Fp;
use ocaml_interop::{
    impl_to_ocaml_polymorphic_variant, impl_to_ocaml_variant, ocaml_export, DynBox, OCaml,
    OCamlBytes, OCamlInt, OCamlList, OCamlRef, OCamlRuntime, RawOCaml, ToOCaml, BoxRoot,
};
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::{
    account::{Account, AccountId, NonZeroCurvePointUncompressedStableV1},
    address::Address,
    base::{AccountIndex, BaseLedger},
    tree::Database,
    tree_version::V2,
};

static DATABASE: Lazy<Mutex<Database<V2>>> = Lazy::new(|| Mutex::new(Database::create(30)));

// #[derive(Clone)]
struct DatabaseFFI(Rc<RefCell<Database<V2>>>);

fn with_db<F, R>(rt: &mut &mut OCamlRuntime, db: OCamlRef<DynBox<DatabaseFFI>>, fun: F) -> R
where
    F: FnOnce(&mut Database<V2>) -> R,
{
    let db = rt.get(db);
    let db: &DatabaseFFI = db.borrow();
    let mut db = db.0.borrow_mut();

    fun(&mut db)
}

struct MyOCamlClosure(*const RawOCaml);

// external database_get_or_create_account : database -> account_id -> account -> (([ `Added | `Existed ] * addr), rust_dberror) result = "rust_database_get_or_create_account"

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

#[derive(Debug, PartialEq, Eq)]
pub enum DatabaseErrorFFI {
    OutOfLeaves,
}

impl_to_ocaml_variant! {
    DatabaseErrorFFI {
        DatabaseErrorFFI::OutOfLeaves,
    }
}

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

fn get_cloned_db(
    rt: &mut &mut OCamlRuntime,
    db: OCamlRef<DynBox<DatabaseFFI>>,
) -> Rc<RefCell<Database<V2>>> {
    let db = rt.get(db);
    let db: &DatabaseFFI = db.borrow();
    Rc::clone(&db.0)
}

ocaml_export! {
    fn rust_database_create(
        rt,
        depth: OCamlRef<OCamlInt>
    ) -> OCaml<DynBox<DatabaseFFI>> {
        let depth: i64 = depth.to_rust(rt);
        let depth: u8 = depth.try_into().unwrap();

        let db = Database::<V2>::create(depth);
        let db = DatabaseFFI(Rc::new(RefCell::new(db)));

        OCaml::box_value(rt, db)
    }

    fn rust_database_get_uuid(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>
    ) -> OCaml<OCamlInt> {
        let uuid = with_db(rt, db, |db| {
            db.get_uuid() as i64
        });

        // let db = rt.get(db);
        // let db: &DatabaseFFI = db.borrow();

        // // TODO: Make it a 36 characters string
        // let uuid = db.0.borrow().get_uuid() as i64;

        uuid.to_ocaml(rt)
    }

    fn rust_database_depth(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>
    ) -> OCaml<OCamlInt> {
        let depth = with_db(rt, db, |db| {
            db.depth() as i64
        });

        depth.to_ocaml(rt)
    }

    fn rust_database_create_checkpoint(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>
    ) -> OCaml<OCamlInt> {
        let db = rt.get(db);
        let _db: &DatabaseFFI = db.borrow();
        todo!()
    }

    fn rust_database_make_checkpoint(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>
    ) -> OCaml<OCamlInt> {
        let db = rt.get(db);
        let _db: &DatabaseFFI = db.borrow();
        todo!()
    }

    fn rust_database_close(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>
    ) {
        with_db(rt, db, |db| {
            db.close();
        });

        OCaml::unit()
    }

    fn rust_add_account(
        rt,
        account: OCamlRef<OCamlBytes>,
    ) {
        println!("RUST BEGIN");
        let account_ref = rt.get(account);
        let account = account_ref.as_bytes();

        let account: Account = serde_binprot::from_slice(account).unwrap();

        println!("account={:?}", account);
        println!("account_hash={:?}", account.hash().to_string());

        println!("RUST END 1");
        OCaml::unit()
    }

    fn rust_add_account_with_hash(
        rt,
        account: OCamlRef<OCamlBytes>,
        hash: OCamlRef<String>,
    ) {
        // println!("RUST BEGIN");
        let account_ref = rt.get(account);
        let account = account_ref.as_bytes();
        let account_bytes = account;
        let _account_len = account.len();
        let hash: String = hash.to_rust(rt);
        let hash = Fp::from_str(&hash).unwrap();

        let account: Account = serde_binprot::from_slice(account).unwrap();
        let account_hash = account.hash();

        if hash != account_hash {
            println!("different hash ! bytes={:?}", account_bytes);
            println!("ocaml_hash={:?}", hash.to_string());
            println!("rust_hash ={:?}", account_hash.to_string());
            assert_eq!(hash, account_hash);
        }

        // println!("hash={:?}", hash.to_string());
        // println!("provided={:?}", hash.to_string());
        // println!("computed={:?}", account_hash.to_string());

        let ser = serde_binprot::to_vec(&account).unwrap();

        // println!("from_ocaml={:?}", account_bytes);
        // println!("rust_ocaml={:?}", ser);

        // assert_eq!(account_len, ser.len());

        let account2: Account = serde_binprot::from_slice(&ser).unwrap();
        let account_hash2 = account2.hash();
        assert_eq!(account_hash, account_hash2);

        // println!("account={:?}", account);
        // println!("account_hash={:?}", account.hash().to_string());

        let mut db = DATABASE.lock().unwrap();
        let id = account.id();
        db.get_or_create_account(id, account).unwrap();

        // println!("RUST END");
        OCaml::unit()
    }

    fn rust_root_hash(rt, ocaml_hash: OCamlRef<String>) {
        let db = DATABASE.lock().unwrap();
        let hash = db.root_hash();

        let ocaml_hash: String = ocaml_hash.to_rust(rt);
        let ocaml_hash = Fp::from_str(&ocaml_hash).unwrap();

        println!("naccounts ={:?}", db.naccounts());
        println!("rust_root_hash ={:?}", hash.to_string());
        println!("ocaml_root_hash={:?}", ocaml_hash.to_string());

        assert_eq!(hash, ocaml_hash);

        OCaml::unit()
    }

    fn rust_random_account(rt, _unused: OCamlRef<String>) -> OCaml<OCamlBytes> {
        let res = impl_rust_random_account();
        // println!("rust_random_account begin");

        // // let account = Account::rand();
        // // let ser = serde_binprot::to_vec(&account).unwrap();

        // let ser: Vec<u8> = vec![178, 29, 73, 50, 85, 80, 131, 166, 53, 11, 48, 224, 103, 89, 161, 207, 149, 31, 170, 21, 165, 181, 94, 18, 149, 177, 54, 71, 185, 77, 109, 49, 1, 144, 247, 164, 171, 110, 24, 3, 12, 25, 163, 63, 125, 83, 66, 174, 2, 160, 62, 45, 137, 185, 47, 16, 129, 145, 190, 203, 124, 35, 119, 251, 26, 1, 1, 6, 49, 50, 56, 54, 56, 56, 252, 29, 154, 218, 214, 79, 98, 177, 181, 253, 181, 152, 127, 0, 145, 177, 91, 155, 59, 239, 161, 174, 217, 42, 201, 30, 46, 11, 187, 88, 49, 5, 111, 254, 222, 87, 42, 45, 90, 1, 236, 173, 205, 215, 241, 20, 0, 77, 12, 197, 234, 69, 202, 22, 55, 50, 183, 255, 238, 8, 29, 79, 199, 92, 12, 146, 223, 105, 45, 135, 77, 89, 73, 141, 11, 137, 28, 54, 21, 0, 1, 4, 4, 1, 0, 4, 3, 4, 3, 2, 3, 0, 6, 49, 49, 56, 54, 54, 51];

        // let mut account2: Account = serde_binprot::from_slice(&ser).unwrap();

        // let account_hash2 = account2.hash();

        // println!("HASH2={:?}", account_hash2.to_string());

        // let ser = serde_binprot::to_vec(&account2).unwrap();


        // println!("rust_random_account end");

        res.to_ocaml(rt)
    }

    fn rust_database_get(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        addr: OCamlRef<String>,
    ) -> OCaml<Option<OCamlBytes>> {
        let addr = get_addr(rt, addr);

        let account = with_db(rt, db, |db| {
            db.get(addr)
        }).map(|account| {
            serde_binprot::to_vec(&account).unwrap()
        });

        account.to_ocaml(rt)
    }

    fn rust_database_get_batch(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        addrs: OCamlRef<OCamlList<OCamlBytes>>,
    ) -> OCaml<OCamlList<(String, Option<OCamlBytes>)>> {
        let mut addrs_ref = rt.get(addrs);

        let mut addrs = Vec::with_capacity(2048);
        while let Some((head, tail)) = addrs_ref.uncons() {
            let addr = Address::try_from(head.as_str()).unwrap();
            addrs.push(addr);
            addrs_ref = tail;
        }

        let accounts: Vec<(String, Option<Vec<u8>>)> = with_db(rt, db, |db| {
            db.get_batch(&addrs)
        }).into_iter()
          .map(|(addr, opt_account)| {
              let addr = addr.to_string();
              let opt_account = opt_account.map(|acc| serde_binprot::to_vec(&acc).unwrap());
              (addr, opt_account)
          })
          .collect();

        accounts.to_ocaml(rt)
    }

    fn rust_database_get_list(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
    ) -> OCaml<OCamlList<OCamlBytes>> {

        let accounts: Vec<Vec<u8>> = with_db(rt, db, |db| {
            db.to_list()
        }).into_iter()
          .map(|account| {
              serde_binprot::to_vec(&account).unwrap()
          })
          .collect();

        accounts.to_ocaml(rt)
    }

    fn rust_database_accounts(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
    ) -> OCaml<OCamlList<OCamlBytes>> {

        let accounts: Vec<Vec<u8>> = with_db(rt, db, |db| {
            db.accounts()
        }).into_iter()
          .map(|account_id| {
              serde_binprot::to_vec(&account_id).unwrap()
          })
          .collect();

        accounts.to_ocaml(rt)
    }

    fn rust_database_get_inner_hash_at_addr(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        addr: OCamlRef<String>,
    ) -> OCaml<String> {
        let addr = get_addr(rt, addr);

        let hash = with_db(rt, db, |db| {
            db.get_inner_hash_at_addr(addr)
        }).map(|hash| {
              hash.to_string()
          })
          .unwrap();

        hash.to_ocaml(rt)
    }

    fn rust_database_set_inner_hash_at_addr(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        addr: OCamlRef<String>,
        hash: OCamlRef<String>,
    ) {
        let addr = get_addr(rt, addr);

        let hash: String = hash.to_rust(rt);
        let hash = Fp::from_str(&hash).unwrap();

        with_db(rt, db, |db| {
            db.set_inner_hash_at_addr(addr, hash).unwrap()
        });

        OCaml::unit()
    }

    fn rust_database_get_at_index(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        index: OCamlRef<OCamlInt>
    ) -> OCaml<OCamlBytes> {
        let index = get_index(rt, index);

        let account = with_db(rt, db, |db| {
            db.get_at_index(index).unwrap()
        });
        let account = serde_binprot::to_vec(&account).unwrap();

        account.to_ocaml(rt)
    }

    fn rust_database_location_of_account(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        account_id: OCamlRef<OCamlBytes>
    ) -> OCaml<Option<String>> {
        let account_id = get(rt, account_id);

        let addr = with_db(rt, db, |db| {
            db.location_of_account(&account_id)
        }).map(|addr| {
            addr.to_string()
        });

        addr.to_ocaml(rt)
    }

    fn rust_database_location_of_account_batch(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        account_ids: OCamlRef<OCamlList<OCamlBytes>>
    ) -> OCaml<OCamlList<(OCamlBytes, Option<String>)>> {
        let account_ids = get_list_of::<AccountId>(rt, account_ids);

        let addrs = with_db(rt, db, |db| {
            db.location_of_account_batch(&account_ids)
        }).into_iter()
          .map(|(account_id, opt_addr)| {
              let account_id = serde_binprot::to_vec(&account_id).unwrap();
              let addr = opt_addr.map(|addr| addr.to_string());
              (account_id, addr)
        })
          .collect::<Vec<_>>();

        addrs.to_ocaml(rt)
    }

    fn rust_database_last_filled(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
    ) -> OCaml<Option<String>> {
        let addr = with_db(rt, db, |db| {
            db.last_filled()
        }).map(|addr| {
            addr.to_string()
        });

        addr.to_ocaml(rt)
    }

    fn rust_database_token_owners(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
    ) -> OCaml<OCamlList<OCamlBytes>> {
        let owners = with_db(rt, db, |db| {
            db.token_owners()
        }).iter()
          .map(|account_id| {
              serde_binprot::to_vec(account_id).unwrap()
        })
          .collect::<Vec<_>>();

        owners.to_ocaml(rt)
    }

    fn rust_database_token_owner(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        token_id: OCamlRef<OCamlBytes>,
    ) -> OCaml<Option<OCamlBytes>> {
        let token_id = get(rt, token_id);

        let owner = with_db(rt, db, |db| {
            db.token_owner(token_id)
        }).map(|account_id| {
            serde_binprot::to_vec(&account_id).unwrap()
        });

        owner.to_ocaml(rt)
    }

    fn rust_database_tokens(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        pubkey: OCamlRef<OCamlBytes>,
    ) -> OCaml<OCamlList<OCamlBytes>> {
        let pubkey: NonZeroCurvePointUncompressedStableV1 = get(rt, pubkey);

        let tokens = with_db(rt, db, |db| {
            db.tokens(pubkey.into())
        }).iter()
          .map(|token_id| {
            serde_binprot::to_vec(token_id).unwrap()
        })
          .collect::<Vec<_>>();

        tokens.to_ocaml(rt)
    }

    fn rust_database_set(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        addr: OCamlRef<String>,
        account: OCamlRef<OCamlBytes>,
    ) {
        let addr = get_addr(rt, addr);
        let account = get(rt, account);

        with_db(rt, db, |db| {
            db.set(addr, account)
        });

        OCaml::unit()
    }

    fn rust_database_index_of_account(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        account_id: OCamlRef<OCamlBytes>
    ) -> OCaml<OCamlInt> {
        let account_id = get(rt, account_id);

        let index = with_db(rt, db, |db| {
            db.index_of_account(account_id)
        }).map(|index| {
            index.0 as i64
        })
          .unwrap();

        index.to_ocaml(rt)
    }

    fn rust_database_set_at_index(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        index: OCamlRef<OCamlInt>,
        account: OCamlRef<OCamlBytes>,
    ) {
        let index = get_index(rt, index);
        let account = get(rt, account);

        with_db(rt, db, |db| {
            db.set_at_index(index, account)
        }).unwrap();

        OCaml::unit()
    }

    fn rust_database_get_or_create_account(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        account_id: OCamlRef<OCamlBytes>,
        account: OCamlRef<OCamlBytes>,
    ) -> OCaml<Result<(PolymorphicGetOrAdded, String), DatabaseErrorFFI>> {
        let account_id = get(rt, account_id);
        let account = get(rt, account);

        let result = with_db(rt, db, |db| {
            db.get_or_create_account(account_id, account)
        });

        use crate::base::GetOrCreated::*;
        use crate::tree::DatabaseError::*;

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

    fn rust_database_num_accounts(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>
    ) -> OCaml<OCamlInt> {
        let num_accounts = with_db(rt, db, |db| {
            db.num_accounts() as i64
        });

        num_accounts.to_ocaml(rt)
    }

    fn rust_database_iter(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        ocaml_method: OCamlRef<fn(OCamlBytes)>,
    ) {
        let db = get_cloned_db(rt, db);
        let db: Ref<Database<V2>> = (*db).borrow();
        let ocaml_method = ocaml_method.to_boxroot(rt);

        db.iter(|account| {
            let account: Vec<u8> = serde_binprot::to_vec(&account).unwrap();

            let _: Result<OCaml<()>, _> = ocaml_method.try_call(rt, &account);
        });

        OCaml::unit()
    }

    fn rust_database_foldi(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        ocaml_method: OCamlRef<fn(String, OCamlBytes)>,
    ) {
        let db = get_cloned_db(rt, db);
        let db: Ref<Database<V2>> = (*db).borrow();
        let ocaml_method = ocaml_method.to_boxroot(rt);

        db.iter_with_addr(|addr, account| {
            let account: Vec<u8> = serde_binprot::to_vec(&account).unwrap();
            let addr = addr.to_string();

            let _: Result<OCaml<()>, _> = ocaml_method.try_call(rt, &addr, &account);
        });

        OCaml::unit()
    }

    fn rust_database_foldi_with_ignored_accounts(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        ignored_accounts: OCamlRef<OCamlList<OCamlBytes>>,
        ocaml_method: OCamlRef<fn(String, OCamlBytes)>,
    ) {
        let db = get_cloned_db(rt, db);
        let db: Ref<Database<V2>> = (*db).borrow();
        let ocaml_method = ocaml_method.to_boxroot(rt);

        let ignored_accounts = get_set_of::<AccountId>(rt, ignored_accounts);

        db.iter_with_addr(|addr, account| {
            if ignored_accounts.contains(&account.id()) {
                return;
            }

            let account: Vec<u8> = serde_binprot::to_vec(&account).unwrap();
            let addr = addr.to_string();

            let _: Result<OCaml<()>, _> = ocaml_method.try_call(rt, &addr, &account);
        });

        OCaml::unit()
    }

    fn rust_database_merkle_root(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
    ) -> OCaml<String> {
        let hash = with_db(rt, db, |db| {
            db.merkle_root()
        }).to_string();

        hash.to_ocaml(rt)
    }

    fn rust_database_remove_accounts(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        account_ids: OCamlRef<OCamlList<OCamlBytes>>,
    ) {
        let account_ids = get_list_of(rt, account_ids);

        with_db(rt, db, |db| {
            db.remove_accounts(&account_ids)
        });

        OCaml::unit()
    }

    fn rust_database_set_all_accounts_rooted_at(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        addr: OCamlRef<String>,
        accounts: OCamlRef<OCamlList<OCamlBytes>>,
    ) {
        let addr = get_addr(rt, addr);
        let accounts = get_list_of(rt, accounts);

        with_db(rt, db, |db| {
            db.set_all_accounts_rooted_at(addr, &accounts).unwrap()
        });

        OCaml::unit()
    }

    fn rust_database_set_batch_accounts(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        accounts: OCamlRef<OCamlList<(String, OCamlBytes)>>,
    ) {
        let accounts = get_list_addr_account(rt, accounts);

        with_db(rt, db, |db| {
            db.set_batch_accounts(&accounts)
        });

        OCaml::unit()
    }

    fn rust_database_get_all_accounts_rooted_at(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        addr: OCamlRef<String>,
    ) -> OCaml<OCamlList<(String, OCamlBytes)>> {
        let addr = get_addr(rt, addr);

        let accounts = with_db(rt, db, |db| {
            db.get_all_accounts_rooted_at(addr)
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

  // external database_merkle_path : database -> addr -> bytes list = "rust_database_merkle_path"
  // external database_merkle_path_at_addr : database -> bytes -> bytes list = "rust_database_merkle_path_at_addr"
  // external database_merkle_path_at_index : database -> int -> bytes list = "rust_database_merkle_path_at_index"

}

// database_create : int -> database = "rust_database_create"
// database_get_uuid : database -> string = "rust_database_get_uuid"
// database_depth : database -> int = "rust_database_depth"
// database_create_checkpoint : database -> database = "rust_database_create_checkpoint"
// database_make_checkpoint : database -> unit = "rust_database_make_checkpoint"
// database_close : database -> unit = "rust_database_close"
// database_get : database -> addr -> account option = "rust_database_get"
// database_get_batch : database -> addr list -> (addr * (account option)) list = "rust_database_get_batch"
// database_get_list : database -> bytes list = "rust_database_get_list"
// database_accounts : database -> bytes list = "rust_database_accounts"
// database_get_inner_hash_at_addr : database -> addr -> bytes = "rust_database_get_inner_hash_at_addr"
// database_set_inner_hash_at_addr : database -> addr -> bytes -> unit = "rust_database_set_inner_hash_at_addr"
// database_get_at_index : database -> int -> account = "rust_database_get_at_index"
// database_iter : database -> (int -> bytes -> unit) -> unit = "rust_database_iter"
// database_location_of_account : database -> account_id -> addr option = "rust_database_location_of_account"
// database_location_of_account_batch : database -> account_id list -> (account_id * (addr option)) list = "rust_database_location_of_account_batch"

// database_last_filled : database -> addr option = "rust_database_last_filled"
// database_token_owners : database -> bytes list = "rust_database_token_owners"
// database_token_owner : database -> token_id -> account_id option = "rust_database_token_owner"
// database_tokens : database -> pubkey -> token_id list = "rust_database_tokens"
// database_set : database -> addr -> account -> unit = "rust_database_set"
// database_index_of_account : database -> account_id -> int = "rust_database_index_of_account"
// database_set_at_index : database -> int -> account -> unit = "rust_database_set_at_index"
// database_get_or_create_account : database -> account_id -> account -> (([ `Added | `Existed ] * addr), rust_dberror) result = "rust_database_get_or_create_account"
// database_num_accounts : database -> int = "rust_database_num_accounts"
// database_fold_with_account_ids : database -> bytes list -> bytes -> (bytes -> unit) -> bytes = "rust_database_fold_with_ignored_accounts"
// database_fold : database -> bytes -> (bytes -> unit) -> bytes = "rust_database_fold"
// database_fold_until : database -> bytes -> (bytes -> bool) -> bytes = "rust_database_fold_until"
// database_merkle_root : database -> bytes = "rust_database_merkle_root"
// database_remove_accounts : database -> account_id list -> unit = "rust_database_remove_accounts"
// database_merkle_path : database -> addr -> bytes list = "rust_database_merkle_path"
// database_merkle_path_at_addr : database -> bytes -> bytes list = "rust_database_merkle_path_at_addr"
// database_merkle_path_at_index : database -> int -> bytes list = "rust_database_merkle_path_at_index"
// database_set_all_accounts_rooted_at : database -> addr -> bytes list -> unit = "rust_database_set_all_accounts_rooted_at"
// database_set_batch_accounts : database -> (addr * account) list -> unit = "rust_database_set_batch_accounts"
// database_get_all_accounts_rooted_at : database -> addr -> (addr * account) list = "rust_database_get_all_accounts_rooted_at"

// (* TODO: Make those method *)
// database_foldi : database -> (addr -> bytes -> unit) -> unit = "rust_database_foldi"
// database_foldi_with_ignored_accounts : database -> account list -> (addr -> bytes -> unit) -> unit = "rust_database_foldi_with_ignored_accounts"

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
