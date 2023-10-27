/// Cache generic field(s) (or any data actually)
macro_rules! cache {
    ($F:tt, $compute:expr) => {{
        // We want to support the generic field F, so we use std::any.
        // Initializer is `const`.
        // Note that the macro `thread_local!` uses internally the
        // unstable `#[thread_local]`.
        // We use `ManuallyDrop` to avoid dealing with destructor state
        // so it's fast.
        // The destructor won't be run, but we don't care.
        //
        // See
        // https://github.com/rust-lang/rust/blob/635c4a5e612b0ee8af6615635599702d3dce9906/library/std/src/sys/common/thread_local/fast_local.rs#

        use std::mem::ManuallyDrop;
        use std::cell::RefCell;
        use std::collections::BTreeMap;
        use std::any::{Any, TypeId};

        // Note: Another container than `BTreeMap` could be used here
        thread_local! {
            static CACHE: ManuallyDrop<RefCell<BTreeMap<TypeId, Box<dyn Any>>>> =
                const { ManuallyDrop::new(RefCell::new(BTreeMap::new())) };
        }

        CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            let type_id = std::any::TypeId::of::<$F>();
            if let Some(cached) = cache.get(&type_id).and_then(|c| c.downcast_ref::<$F>()) {
                return *cached;
            }
            let data = $compute;
            cache.insert(type_id, Box::new(data));
            data
        })
    }};
}

/// Cache one field (or any data actually)
macro_rules! cache_one {
    ($F:tt, $compute:expr) => {{
        // See comments in `cache` above
        // Here we don't support generic

        use std::cell::RefCell;
        use std::mem::ManuallyDrop;

        thread_local! {
            static CACHE: ManuallyDrop<RefCell<Option<Box<$F>>>> =
                const { ManuallyDrop::new(RefCell::new(Option::None)) };
        }

        CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            if let Some(cached) = cache.as_ref() {
                return (**cached).clone();
            }
            let data = $compute;
            let _ = cache.insert(Box::new(data.clone()));
            data
        })
    }};
}

#[cfg(test)]
mod tests {
    use crate::proofs::witness::FieldWitness;
    use ark_ec::short_weierstrass_jacobian::GroupAffine;
    use poly_commitment::srs::endos;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering::Relaxed;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    #[test]
    fn test_cache() {
        use mina_curves::pasta::Fq;
        use mina_hasher::Fp;

        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        fn my_test<F: FieldWitness>() -> (F, F::Scalar) {
            cache!((F, F::Scalar), {
                COUNTER.fetch_add(1, Relaxed);
                endos::<GroupAffine<F::Parameters>>()
            })
        }

        let counter = || COUNTER.load(Relaxed);

        assert_eq!(counter(), 0);

        dbg!(my_test::<Fp>());
        assert_eq!(counter(), 1);
        dbg!(my_test::<Fp>());
        dbg!(my_test::<Fp>());
        dbg!(my_test::<Fp>());
        assert_eq!(counter(), 1);

        dbg!(my_test::<Fq>());
        assert_eq!(counter(), 2);
        dbg!(my_test::<Fq>());
        dbg!(my_test::<Fq>());
        dbg!(my_test::<Fq>());
        assert_eq!(counter(), 2);

        dbg!(my_test::<Fp>());
        dbg!(my_test::<Fq>());
        dbg!(my_test::<Fp>());
        dbg!(my_test::<Fq>());
        assert_eq!(counter(), 2);
    }
}
