use sled::{Db, Tree};
use std::path::PathBuf;

use crate::{
    evaluator::epoch::{BlockStatus, SlotBlockUpdate, SlotData},
    node::epoch_ledgers::Ledger,
};

#[derive(Clone)]
pub struct Database {
    db: Db,
    current_epoch: Tree,
    historical_epochs: Tree,
    seeds: Tree,
    epoch_ledgers: Tree,
    produced_blocks: Tree,
}

impl Database {
    pub fn open(path: PathBuf) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        let seeds = db.open_tree("seeds")?;
        let current_epoch = db.open_tree("current_epoch")?;
        let historical_epochs = db.open_tree("historical_epochs")?;
        let epoch_ledgers = db.open_tree("epoch_ledgers")?;
        let produced_blocks = db.open_tree("produced_blocks")?;

        Ok(Self {
            db,
            current_epoch,
            seeds,
            historical_epochs,
            epoch_ledgers,
            produced_blocks,
        })
    }

    fn store<T: serde::Serialize, K: AsRef<[u8]>>(
        &self,
        tree: &Tree,
        key: K,
        item: &T,
    ) -> Result<(), sled::Error> {
        let serialized = serialize_bincode(item)?;
        tree.insert(key.as_ref(), serialized)?;
        Ok(())
    }

    fn retrieve<T: serde::de::DeserializeOwned, K: AsRef<[u8]>>(
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

    pub fn update<T, K, F>(&self, tree: &Tree, key: K, mut func: F) -> Result<(), sled::Error>
    where
        T: serde::de::DeserializeOwned + serde::Serialize,
        K: AsRef<[u8]>,
        F: FnMut(T) -> T,
    {
        let existing_bytes = tree.get(key.as_ref())?.ok_or_else(|| {
            sled::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Key not found",
            ))
        })?;

        // Deserialize the current value
        let current_item: T = deserialize_bincode(&existing_bytes)?;

        // Apply the update function to get the new item
        let updated_item = func(current_item);

        // Serialize and store the updated item
        let serialized = serialize_bincode(&updated_item)?;
        tree.insert(key.as_ref(), serialized)?;

        Ok(())
    }

    // Specific methods using the generic helpers
    pub fn store_ledger(&self, epoch: u32, ledger: &Ledger) -> Result<(), sled::Error> {
        self.store(&self.epoch_ledgers, epoch.to_be_bytes(), ledger)
    }

    pub fn get_ledger(&self, epoch: u32) -> Result<Option<Ledger>, sled::Error> {
        self.retrieve(&self.epoch_ledgers, epoch.to_be_bytes())
    }

    pub fn store_seed(&self, epoch: u32, seed: String) -> Result<(), sled::Error> {
        self.store(&self.seeds, epoch.to_be_bytes(), &seed)
    }

    pub fn get_seed(&self, epoch: u32) -> Result<Option<String>, sled::Error> {
        self.retrieve(&self.seeds, epoch.to_be_bytes())
    }

    pub fn store_slot(&self, slot: u32, slot_data: &SlotData) -> Result<(), sled::Error> {
        self.store(&self.current_epoch, slot.to_be_bytes(), slot_data)
    }

    pub fn update_slot_status(
        &self,
        slot: u32,
        block_status: BlockStatus,
    ) -> Result<(), sled::Error> {
        self.update(
            &self.current_epoch,
            slot.to_be_bytes(),
            |mut slot_entry: SlotData| {
                slot_entry.update_block_status(block_status.clone());
                slot_entry
            },
        )
    }

    pub fn update_slot_block(&self, slot: u32, block: SlotBlockUpdate) -> Result<(), sled::Error> {
        self.update(
            &self.current_epoch,
            slot.to_be_bytes(),
            |mut slot_entry: SlotData| {
                slot_entry.add_block(block.clone());
                slot_entry
            },
        )
    }

    pub fn has_slot(&self, slot: u32) -> Result<bool, sled::Error> {
        self.current_epoch.contains_key(slot.to_be_bytes())
    }

    pub fn store_block(&self, state_hash: String, slot: u32) -> Result<(), sled::Error> {
        self.store(&self.produced_blocks, state_hash.as_bytes(), &slot)
    }

    // pub fn get_block(&self, state_hash: String) -> Result<Option<String>, sled::Error> {
    //     self.retrieve(&self.produced_blocks, state_hash.as_bytes())
    // }

    pub fn seen_block(&self, state_hash: String) -> Result<bool, sled::Error> {
        self.produced_blocks.contains_key(state_hash.as_bytes())
    }
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
