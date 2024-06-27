use std::{
    collections::{BTreeMap, BTreeSet},
    io, sync::Arc,
};

use ledger::scan_state::currency::Fee;
use openmina_core::snark::SnarkJobId;


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(SnarkJobId);

#[derive(Debug, Clone)]
struct Snark(Arc<openmina_core::snark::Snark>);

impl PartialEq for Snark {
    fn eq(&self, other: &Self) -> bool {
        self.0.job_id() == other.0.job_id()
    }
}

impl Eq for Snark {}

impl PartialOrd for Snark {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.job_id().partial_cmp(&other.0.job_id())
    }
}

impl Ord for Snark {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.job_id().cmp(&other.0.job_id())
    }
}

impl Snark {
    fn address(&self) -> Address {
        Address(self.0.job_id())
    }
}

pub struct SnarkStorage {
    snarks: BTreeMap<Address, Snark>,
    snarks_by_fee: BTreeMap<Fee, BTreeSet<Snark>>,
}

pub enum StorageError {}

pub type Result<T> = std::result::Result<T, StorageError>;

fn map_set_insert(map: &mut BTreeMap<Fee, BTreeSet<Snark>>, snark: Snark) {
    let fee = (&snark.0.fee).into();
    match map.get_mut(&fee) {
        Some(set) => {
            set.replace(snark);
        }
        None => {
            let mut set = BTreeSet::new();
            set.replace(snark);
            map.insert(fee, set);
        }
    }
}

impl SnarkStorage {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            snarks: BTreeMap::new(),
            snarks_by_fee: BTreeMap::new(),
        })
    }

    pub fn insert(&mut self, snark: openmina_core::snark::Snark) -> Result<Address> {
        let item = Snark(Arc::new(snark));
        let addr = item.address();
        // ignore discarded duplicates; the SNARK pool always chooses the best one,
        // so if we get a duplicate here, we always want to replace the old one with
        // the new.
        self.snarks.insert(addr.clone(), item.clone());
        map_set_insert(&mut self.snarks_by_fee, item);
        Ok(addr)
    }

    pub fn get(&self, addr: &Address) -> Result<Option<Arc<openmina_core::snark::Snark>>> {
        Ok(self.snarks.get(addr).map(|snark| snark.0.clone()))
    }

    pub fn drop(&mut self, addr: &Address) -> Result<()>{
        if let Some(snark) = self.snarks.remove(addr) {
            let fee = (&snark.0.fee).into();
            if let Some(set) = self.snarks_by_fee.get_mut(&fee) {
                set.remove(&snark);
            } else {
                openmina_core::warn!(
                    openmina_core::log::system_time();
                    kind = "SNARK removal",
                    message = "SNARK not found in fee index",
                    fee = fee.as_u64(),
                    addr = addr.0.to_string(),
                )
            }
        } else {
                openmina_core::warn!(
                    openmina_core::log::system_time();
                    kind = "SNARK removal",
                    message = "SNARK not found",
                    addr = addr.0.to_string(),
                )
        }
        Ok(())
    }
}
