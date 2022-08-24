use std::sync::{Arc, Mutex};

use crate::{
    base::{BaseLedger, Uuid},
    tree::Database,
    tree_version::V2,
};

use super::Maskable;

struct MaskingInner {
    parent: Option<Maskable>,
    inner: Database<V2>,
}

#[derive(Clone)]
pub struct Masking {
    inner: Arc<Mutex<MaskingInner>>,
}

impl Masking {
    pub fn uuid(&self) -> Uuid {
        let inner = self.inner.lock().expect("lock failed");
        inner.inner.get_uuid()
    }

    pub fn is_attached(&self) -> bool {
        let inner = self.inner.lock().expect("lock failed");
        inner.parent.is_some()
    }

    pub fn set_parent(&self, parent: &Maskable) {
        let mut inner = self.inner.lock().expect("lock failed");
        assert!(inner.parent.is_none(), "mask is already attached");

        inner.parent = Some(parent.clone());
    }
}
