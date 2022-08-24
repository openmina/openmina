use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex},
};

use crate::{
    base::{BaseLedger, Uuid},
    tree::Database,
    tree_version::V2,
};

use super::Masking;

struct MaskableInner {
    inner: Database<V2>,
    /// All childs of this mask
    registered_masks: HashMap<Uuid, Masking>,
}

impl Deref for MaskableInner {
    type Target = Database<V2>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Clone)]
pub struct Maskable {
    inner: Arc<Mutex<MaskableInner>>,
}

#[derive(Debug)]
pub enum UnregisterBehavior {
    Check,
    Recursive,
    IPromiseIAmReparentingThisMask,
}

impl Maskable {
    fn with_self<F, R>(&self, fun: F) -> R
    where
        F: FnOnce(&mut MaskableInner) -> R,
    {
        let mut inner = self.inner.lock().expect("lock failed");
        fun(&mut inner)
    }

    fn uuid(&self) -> Uuid {
        self.with_self(|this| this.get_uuid())
    }

    pub fn register_mask(&self, mask: Masking) -> Masking {
        self.with_self(|this| {
            let old = this.registered_masks.insert(mask.uuid(), mask.clone());
            assert!(old.is_none(), "mask is already registered");

            mask.set_parent(self);
            mask
        })
    }

    pub fn unregister_mask(&self, mask: Masking, behavior: UnregisterBehavior) {
        use UnregisterBehavior::*;

        self.with_self(|this| {
            let mask_uuid = mask.uuid();

            assert_eq!(mask_uuid, this.get_uuid());

            this.registered_masks.remove(&mask_uuid);

            match behavior {
                Check => {
                    // mask.i
                    // assert!(mask.chi)
                }
                Recursive => todo!(),
                IPromiseIAmReparentingThisMask => todo!(),
            }
        })
    }
}
