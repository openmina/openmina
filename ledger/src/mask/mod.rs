#![allow(clippy::module_inception)]

mod mask;
mod mask_impl;

pub use mask::*;

use once_cell::sync::Lazy;
use std::{collections::HashSet, sync::Mutex};

use crate::Uuid;

// block masks(k = 290) + staking/next epoch masks (2) + 2 root masks = 294.
static MASKS_ALIVE: Lazy<Mutex<HashSet<Uuid>>> =
    Lazy::new(|| Mutex::new(HashSet::with_capacity(294)));

fn exec<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashSet<Uuid>) -> R,
{
    f(&mut MASKS_ALIVE.lock().unwrap())
}

pub(super) fn alive_add(uuid: &Uuid) {
    exec(|list| {
        list.insert(uuid.to_owned());
    });
}

pub(super) fn alive_remove(uuid: &Uuid) {
    exec(|list| {
        list.remove(uuid);
    });
}

pub fn is_alive(uuid: &Uuid) -> bool {
    exec(|list| list.contains(uuid))
}

pub fn alive_len() -> usize {
    exec(|list| list.len())
}

pub fn alive_collect<B>() -> B
where
    B: FromIterator<Uuid>,
{
    exec(|list| list.iter().cloned().collect())
}
