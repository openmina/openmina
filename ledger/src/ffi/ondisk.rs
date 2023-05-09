use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use ocaml_interop::{
    ocaml_export, DynBox, FromOCaml, OCaml, OCamlBytes, OCamlInt, OCamlList, OCamlRef,
    OCamlRuntime, ToOCaml,
};

use crate::ondisk::{batch::Batch, Database};

pub struct DatabaseFFI(pub Rc<RefCell<Option<Database>>>);
pub struct BatchFFI(pub Rc<RefCell<Batch>>);

impl Drop for DatabaseFFI {
    fn drop(&mut self) {
        let mask_id = RefCell::borrow(&self.0)
            .as_ref()
            .map(|mask| mask.get_uuid().clone());
        elog!("rust_ondisk_database_drop {:?}", mask_id);
    }
}

fn with_db<F, R>(rt: &mut &mut OCamlRuntime, db: OCamlRef<DynBox<DatabaseFFI>>, fun: F) -> R
where
    F: FnOnce(&mut Database) -> R,
{
    let db = rt.get(db);
    let db: &DatabaseFFI = db.borrow();
    let mut db = db.0.borrow_mut();

    fun(db.as_mut().unwrap())
}

fn with_batch<F, R>(rt: &mut &mut OCamlRuntime, db: OCamlRef<DynBox<BatchFFI>>, fun: F) -> R
where
    F: FnOnce(&mut Batch) -> R,
{
    let db = rt.get(db);
    let db: &BatchFFI = db.borrow();
    let mut db = db.0.borrow_mut();

    fun(&mut db)
}

fn get<V, T: FromOCaml<V>>(rt: &mut &mut OCamlRuntime, value: OCamlRef<V>) -> T {
    let value = rt.get(value);
    value.to_rust::<T>()
}

fn get_list_of<V, T, F>(
    rt: &mut &mut OCamlRuntime,
    values: OCamlRef<OCamlList<V>>,
    fun: F,
) -> Vec<T>
where
    F: Fn(OCaml<V>) -> T,
{
    let mut values_ref = rt.get(values);

    let mut values = Vec::with_capacity(2048);
    while let Some((head, tail)) = values_ref.uncons() {
        let key: T = fun(head);

        values.push(key);
        values_ref = tail;
    }

    values
}

ocaml_export! {
    fn rust_ondisk_database_create(
        rt,
        dir_name: OCamlRef<String>
    ) -> OCaml<DynBox<DatabaseFFI>> {
        let dir_name: String = get(rt, dir_name);

        elog!("rust_ondisk_database_create={:?}", dir_name);

        let db = Database::create(dir_name).unwrap();
        let db = DatabaseFFI(Rc::new(RefCell::new(Some(db))));

        OCaml::box_value(rt, db)
    }

    fn rust_ondisk_database_get_uuid(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>
    ) -> OCaml<String> {
        let uuid = with_db(rt, db, |db| {
            db.get_uuid().clone()
        });

        uuid.to_ocaml(rt)
    }

    fn rust_ondisk_database_close(
        rt,
        _db: OCamlRef<DynBox<DatabaseFFI>>
    ) {
        // TODO

        OCaml::unit()
    }

    fn rust_ondisk_database_get(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        key: OCamlRef<OCamlBytes>,
    ) -> OCaml<Option<OCamlBytes>> {
        let key: Vec<u8> = get(rt, key);

        let value = with_db(rt, db, |db| {
            db.get(&key).unwrap()
        });

        value.to_ocaml(rt)
    }

    fn rust_ondisk_database_set(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        key: OCamlRef<OCamlBytes>,
        value: OCamlRef<OCamlBytes>,
    ) {
        let key: Box<[u8]> = get(rt, key);
        let value: Vec<u8> = get(rt, value);

        with_db(rt, db, |db| {
            db.set(key, value).unwrap()
        });

        OCaml::unit()
    }

    fn rust_ondisk_database_get_batch(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        keys: OCamlRef<OCamlList<OCamlBytes>>,
    ) -> OCaml<OCamlList<Option<OCamlBytes>>> {
        let keys: Vec<Box<[u8]>> = get_list_of(rt, keys, |v| v.as_bytes().into());

        let values: Vec<Option<Vec<u8>>> = with_db(rt, db, |db| {
            db.get_batch(keys).unwrap()
        });

        values.to_ocaml(rt)
    }

    fn rust_ondisk_database_set_batch(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        remove_keys: OCamlRef<OCamlList<OCamlBytes>>,
        key_data_pairs: OCamlRef<OCamlList<(OCamlBytes, OCamlBytes)>>,
    ) {
        let remove_keys: Vec<Box<[u8]>> = get_list_of(rt, remove_keys, |v| {
            v.as_bytes().into()
        });

        let key_data_pairs: Vec<(Box<[u8]>, Vec<u8>)> = get_list_of(rt, key_data_pairs, |v| {
            let (key, value) = v.to_tuple();

            let key = key.as_bytes().into();
            let value = value.as_bytes().to_vec();

            (key, value)
        });

        with_db(rt, db, |db| {
            db.set_batch(key_data_pairs, remove_keys).unwrap()
        });

        OCaml::unit()
    }

    fn rust_ondisk_database_make_checkpoint(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        directory: OCamlRef<String>
    ) {
        let directory: String = get(rt, directory);

        with_db(rt, db, |db| {
            db.make_checkpoint(directory.as_str()).unwrap()
        });

        OCaml::unit()
    }

    fn rust_ondisk_database_create_checkpoint(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        directory: OCamlRef<String>
    ) -> OCaml<DynBox<DatabaseFFI>> {
        let directory: String = get(rt, directory);

        let checkpoint = with_db(rt, db, |db| {
            db.create_checkpoint(directory.as_str()).unwrap()
        });
        let checkpoint = DatabaseFFI(Rc::new(RefCell::new(Some(checkpoint))));

        OCaml::box_value(rt, checkpoint)
    }

    fn rust_ondisk_database_remove(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        key: OCamlRef<OCamlBytes>,
    ) {
        let key: Box<[u8]> = get(rt, key);

        with_db(rt, db, |db| {
            db.remove(key).unwrap()
        });

        OCaml::unit()
    }

    fn rust_ondisk_database_to_alist(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
    ) -> OCaml<OCamlList<(OCamlBytes, OCamlBytes)>> {
        let alist = with_db(rt, db, |db| {
            db.to_alist().unwrap()
        });

        alist.to_ocaml(rt)
    }

    fn rust_ondisk_database_batch_create(
        rt,
        _index: OCamlRef<OCamlInt>,
    ) -> OCaml<DynBox<BatchFFI>> {
        let batch: Batch = Batch::new();
        let batch: BatchFFI = BatchFFI(Rc::new(RefCell::new(batch)));
        OCaml::box_value(rt, batch)
    }

    fn rust_ondisk_database_batch_set(
        rt,
        batch: OCamlRef<DynBox<BatchFFI>>,
        key: OCamlRef<OCamlBytes>,
        value: OCamlRef<OCamlBytes>,
    ) {
        let key: Box<[u8]> = get(rt, key);
        let value: Vec<u8> = get(rt, value);

        with_batch(rt, batch, |batch| {
            batch.set(key, value)
        });

        OCaml::unit()
    }

    fn rust_ondisk_database_batch_remove(
        rt,
        batch: OCamlRef<DynBox<BatchFFI>>,
        key: OCamlRef<OCamlBytes>,
    ) {
        let key: Box<[u8]> = get(rt, key);

        with_batch(rt, batch, |batch| {
            batch.remove(key)
        });

        OCaml::unit()
    }

    fn rust_ondisk_database_batch_run(
        rt,
        db: OCamlRef<DynBox<DatabaseFFI>>,
        batch: OCamlRef<DynBox<BatchFFI>>,
    ) {
        let db = rt.get(db);
        let db: &DatabaseFFI = db.borrow();
        let mut db = db.0.borrow_mut();
        let db = db.as_mut().unwrap();

        let batch = rt.get(batch);
        let batch: &BatchFFI = batch.borrow();
        let mut batch = batch.0.borrow_mut();

        db.run_batch(&mut batch);

        OCaml::unit()
    }
}
