use mina_curves::pasta::Fp;
use std::collections::HashMap;

use super::{AccountId, ToVerifiableCache, ToVerifiableStrategy, VerificationKeyWire};

pub struct Cache {
    cache: HashMap<AccountId, VerificationKeyWire>,
}

impl Cache {
    pub fn new(cache: HashMap<AccountId, VerificationKeyWire>) -> Self {
        Self { cache }
    }
}

impl ToVerifiableCache for Cache {
    fn find(&self, account_id: &AccountId, vk_hash: &Fp) -> Option<&VerificationKeyWire> {
        self.cache
            .get(account_id)
            .filter(|vk| &vk.hash() == vk_hash)
    }

    fn add(&mut self, account_id: AccountId, vk: VerificationKeyWire) {
        self.cache.insert(account_id, vk);
    }
}

pub struct FromAppliedSequence;

impl ToVerifiableStrategy for FromAppliedSequence {
    type Cache = Cache;
}
