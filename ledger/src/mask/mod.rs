#![allow(clippy::module_inception)]

mod mask;
mod mask_impl;

pub use mask::*;

/// Used for tests, to make sure we don't leak masks
#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;
    use std::{collections::HashSet, sync::Mutex};

    use crate::Uuid;

    static MASK_ALIVE: Lazy<Mutex<HashSet<Uuid>>> =
        Lazy::new(|| Mutex::new(HashSet::with_capacity(256)));

    pub fn add_mask(uuid: &Uuid) {
        MASK_ALIVE.lock().unwrap().insert(uuid.to_string());
    }

    pub fn remove_mask(uuid: &Uuid) {
        MASK_ALIVE.lock().unwrap().remove(uuid);
    }

    pub fn is_mask_alive(uuid: &Uuid) -> bool {
        MASK_ALIVE.lock().unwrap().contains(uuid)
    }
}

#[cfg(not(test))]
mod tests {
    use crate::Uuid;

    pub fn add_mask(_: &Uuid) {}

    pub fn remove_mask(_: &Uuid) {}
}
