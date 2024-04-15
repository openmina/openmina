use sled::{Db, Tree};
use std::path::PathBuf;

use crate::node::epoch_ledgers::Ledger;

pub struct Database {
    db: Db,
    current_epoch: Tree,
    historical_epochs: Tree,
    seeds: Tree,
    epoch_ledgers: Tree,
}

impl Database {
    pub fn open(path: PathBuf) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        let seeds = db.open_tree("seeds")?;
        let current_epoch = db.open_tree("current_epoch")?;
        let historical_epochs = db.open_tree("historical_epochs")?;
        let epoch_ledgers = db.open_tree("epoch_ledgers")?;

        Ok(Self {
            db,
            current_epoch,
            seeds,
            historical_epochs,
            epoch_ledgers,
        })
    }

    fn store_bincode<T: serde::Serialize, K: AsRef<[u8]>>(
        &self,
        tree: &Tree,
        key: K,
        item: &T,
    ) -> Result<(), sled::Error> {
        let serialized = serialize_bincode(item)?;
        tree.insert(key.as_ref(), serialized)?;
        Ok(())
    }

    fn retrieve_bincode<T: serde::de::DeserializeOwned, K: AsRef<[u8]>>(
        &self,
        tree: &Tree,
        key: K,
    ) -> Result<Option<T>, sled::Error> {
        match tree.get(key.as_ref())? {
            Some(serialized_item) => {
                let item = deserialize_bincode(&serialized_item)?;
                Ok(Some(item))
            }
            None => Ok(None),
        }
    }

    // Specific methods using the generic helpers
    pub fn store_ledger(&self, epoch: u32, ledger: &Ledger) -> Result<(), sled::Error> {
        self.store_bincode(&self.epoch_ledgers, epoch.to_be_bytes(), ledger)
    }

    pub fn get_ledger(&self, epoch: u32) -> Result<Option<Ledger>, sled::Error> {
        self.retrieve_bincode(&self.epoch_ledgers, epoch.to_be_bytes())
    }

    pub fn store_seed(&self, epoch: u32, seed: String) -> Result<(), sled::Error> {
        self.store_bincode(&self.seeds, epoch.to_be_bytes(), &seed)
    }

    pub fn get_seed(&self, epoch: u32) -> Result<Option<String>, sled::Error> {
        self.retrieve_bincode(&self.seeds, epoch.to_be_bytes())
    }

    pub fn save_slot(&self) {}
}

fn serialize_bincode<T: serde::Serialize>(item: &T) -> Result<Vec<u8>, sled::Error> {
    bincode::serialize(item)
        .map_err(|e| sled::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
}

fn deserialize_bincode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, sled::Error> {
    bincode::deserialize(bytes)
        .map_err(|e| sled::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
}

#[cfg(test)]
mod test {
    use std::env;

    use super::*;

    #[test]
    fn test_ledger_store_and_get() {
        const EPOCH: u32 = 4;

        let db_dir = env::temp_dir();
        let db = Database::open(db_dir).expect("Failed to open DB");

        let ledger = Ledger::load_from_file("test/files/staking-epoch-ledger.json".into())
            .expect("Failed to load ledger file");
        db.store_ledger(EPOCH, &ledger.clone())
            .expect("Failed to store ledger into the DB");

        let retrieved = db
            .get_ledger(EPOCH)
            .expect("Failed to retrieve ledger from the DB");

        assert_eq!(Some(ledger), retrieved);
    }

    #[test]
    fn test_seed_store_and_get() {
        const EPOCH: u32 = 4;

        let db_dir = env::temp_dir();
        let db = Database::open(db_dir).expect("Failed to open DB");

        let seed = "2vawAhPq9RsPXhz8NvrxB5VXuge8U9vQPGCtjqLZ5idHTUtWHWF8".to_string();

        db.store_seed(EPOCH, seed.clone())
            .expect("Failed to store seed into the DB");

        let retrieved = db
            .get_seed(EPOCH)
            .expect("Failed to retrieve seed from the DB");

        assert_eq!(Some(seed), retrieved);
    }
}
