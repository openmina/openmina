use mina_curves::pasta::Fp;
use std::collections::HashMap;

use super::{AccountId, ToVerifiableCache, ToVerifiableStrategy, VerificationKeyWire};

pub struct Cache {
    cache: HashMap<AccountId, HashMap<Fp, VerificationKeyWire>>,
}

impl Cache {
    pub fn new(cache: HashMap<AccountId, HashMap<Fp, VerificationKeyWire>>) -> Self {
        Self { cache }
    }
}

impl ToVerifiableCache for Cache {
    fn find(&self, account_id: &AccountId, vk_hash: &Fp) -> Option<&VerificationKeyWire> {
        let vks = self.cache.get(account_id)?;
        vks.get(vk_hash)
    }

    fn add(&mut self, account_id: AccountId, vk: VerificationKeyWire) {
        let vks = self.cache.entry(account_id).or_default();
        vks.insert(vk.hash(), vk);
    }
}

pub struct FromUnappliedSequence;

impl ToVerifiableStrategy for FromUnappliedSequence {
    type Cache = Cache;
}
