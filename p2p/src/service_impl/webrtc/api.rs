use std::{collections::BTreeMap, sync::Arc};

use openmina_core::channels::broadcast;
use tokio::sync::Mutex;

use crate::PeerId;

use super::{ApiInner, build_api};

#[derive(Clone)]
pub struct Api {
    inner: ApiInner,
    active_peers: Arc<Mutex<BTreeMap<PeerId, broadcast::Receiver<()>>>>
}

impl Api {
    pub fn new() -> Self {
        Self {
            inner: build_api(),
            active_peers: Mutex::new(BTreeMap::new()).into()
        }
    }

    pub(super) fn inner(&self) -> &ApiInner {
        &self.inner
    }
}