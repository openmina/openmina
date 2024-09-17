use sled::{Db, Tree};
use std::path::PathBuf;

use crate::{
    archive::Block,
    evaluator::epoch::{SlotBlockUpdate, SlotData, SlotStatus},
    node::epoch_ledgers::Ledger,
};

#[derive(Clone, Debug)]
pub struct Database {
    _db: Db,
    epoch_data: Tree,
    seeds: Tree,
    epoch_ledgers: Tree,
    blocks: Tree,
    produced_blocks_by_slot: Tree,
    _summaries: Tree,
}

impl Database {
    pub fn open(path: PathBuf) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        let seeds = db.open_tree("seeds")?;
        // TODO(adonagy): rename
        let current_epoch = db.open_tree("current_epoch")?;
        let epoch_ledgers = db.open_tree("epoch_ledgers")?;
        let blocks = db.open_tree("produced_blocks")?;
        let produced_blocks_by_slot = db.open_tree("produced_blocks_by_slot")?;
        let summaries = db.open_tree("sumaries")?;

        Ok(Self {
            _db: db,
            epoch_data: current_epoch,
            seeds,
            epoch_ledgers,
            blocks,
            produced_blocks_by_slot,
            _summaries: summaries,
        })
    }

    pub fn clear(&self) -> Result<(), sled::Error> {
        self._db.clear()?;
        self.epoch_data.clear()?;
        self.seeds.clear()?;
        self.epoch_ledgers.clear()?;
        self.blocks.clear()?;
        self.produced_blocks_by_slot.clear()?;
        self._summaries.clear()
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

    pub fn has_ledger(&self, epoch: &u32) -> Result<bool, sled::Error> {
        self.epoch_ledgers.contains_key(epoch.to_be_bytes())
    }

    pub fn store_seed(&self, epoch: u32, seed: String) -> Result<(), sled::Error> {
        self.store(&self.seeds, epoch.to_be_bytes(), &seed)
    }

    pub fn get_seed(&self, epoch: u32) -> Result<Option<String>, sled::Error> {
        self.retrieve(&self.seeds, epoch.to_be_bytes())
    }

    pub fn store_evaluated_slot(&self, slot: u32, slot_data: &SlotData) -> Result<(), sled::Error> {
        self.store(&self.epoch_data, slot.to_be_bytes(), slot_data)
    }

    pub fn update_slot_status(
        &self,
        slot: u32,
        block_status: SlotStatus,
        update_source: &str, // state_hash
    ) -> Result<(), sled::Error> {
        self.update(
            &self.epoch_data,
            slot.to_be_bytes(),
            |mut slot_entry: SlotData| {
                match slot_entry.state_hash() {
                    Some(stored_state_hash) if stored_state_hash == update_source => {
                        slot_entry.update_block_status(block_status.clone());
                    }
                    None => {
                        slot_entry.update_block_status(block_status.clone());
                    }
                    _ => {} // do nothing if state_hash doesn't match
                }
                slot_entry
            },
        )
    }

    pub fn update_slot_block(
        &self,
        slot: u32,
        block: SlotBlockUpdate,
        // TODO: simplify
        produced: bool,
        in_future: bool,
    ) -> Result<(), sled::Error> {
        self.update(
            &self.epoch_data,
            slot.to_be_bytes(),
            |mut slot_entry: SlotData| {
                slot_entry.add_block(block.clone());
                if !produced {
                    if in_future {
                        slot_entry.update_block_status(SlotStatus::ForeignToBeProduced);
                    } else {
                        slot_entry.update_block_status(SlotStatus::Foreign);
                    }
                }
                slot_entry
            },
        )
    }

    pub fn has_evaluated_slot(&self, slot: u32) -> Result<bool, sled::Error> {
        self.epoch_data.contains_key(slot.to_be_bytes())
    }

    pub fn has_canonical_block_on_slot(&self, slot: u32) -> Result<bool, sled::Error> {
        if let Some(slot_data) =
            self.retrieve::<SlotData, _>(&self.epoch_data, slot.to_be_bytes())?
        {
            Ok(slot_data.is_canonical())
        } else {
            Ok(false)
        }
    }

    pub fn store_block(&self, block: Block) -> Result<(), sled::Error> {
        self.store(&self.blocks, block.state_hash.as_bytes(), &block)?;
        self.store(
            &self.produced_blocks_by_slot,
            block.global_slot().to_be_bytes(),
            &block,
        )
    }

    pub fn get_blocks_in_range(
        &self,
        start_slot: u32,
        end_slot: u32,
    ) -> Result<Vec<Block>, sled::Error> {
        let start_key = start_slot.to_be_bytes();
        let end_key = end_slot.to_be_bytes();

        self.blocks
            .range(start_key..end_key)
            .map(|entry_result| {
                let (_key, value) = entry_result?;
                bincode::deserialize(&value)
                    .map_err(|e| sled::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
            })
            .collect()
    }

    pub fn get_slots_for_epoch(&self, epoch: u32) -> Result<Vec<SlotData>, sled::Error> {
        let start_slot = epoch * 7140;
        let end_slot = start_slot + 7140;

        // Convert slot numbers to byte arrays for the range query
        let start_key = start_slot.to_be_bytes();
        let end_key = end_slot.to_be_bytes();

        self.epoch_data
            .range(start_key..end_key)
            .map(|entry_result| {
                let (_key, value) = entry_result?;
                deserialize_bincode(&value)
            })
            .collect()
    }

    pub fn get_all_slots(&self) -> Result<Vec<SlotData>, sled::Error> {
        self.epoch_data
            .into_iter()
            .map(|entry_result| {
                let (_, value) = entry_result?;
                deserialize_bincode(&value)
            })
            .collect()
    }

    pub fn seen_block(&self, state_hash: String) -> Result<bool, sled::Error> {
        self.blocks.contains_key(state_hash.as_bytes())
    }

    pub fn seen_slot(&self, slot: u32) -> Result<bool, sled::Error> {
        self.produced_blocks_by_slot
            .contains_key(slot.to_be_bytes())
    }

    pub fn set_current_slot(&self, old: u32, new: u32) -> Result<(), sled::Error> {
        self.update(
            &self.epoch_data,
            old.to_be_bytes(),
            |mut slot_entry: SlotData| {
                slot_entry.unset_as_current();
                slot_entry
            },
        )?;

        self.update(
            &self.epoch_data,
            new.to_be_bytes(),
            |mut slot_entry: SlotData| {
                slot_entry.set_as_current();
                slot_entry
            },
        )
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
        const NEXT_EPOCH: u32 = 5;

        let db_dir = env::temp_dir();
        let db = Database::open(db_dir).expect("Failed to open DB");
        db.clear().unwrap();

        assert!(!db.has_ledger(&EPOCH).unwrap());

        let ledger = Ledger::load_from_file("test/files/staking-epoch-ledger.json".into())
            .expect("Failed to load ledger file");
        db.store_ledger(EPOCH, &ledger.clone())
            .expect("Failed to store ledger into the DB");

        assert!(db.has_ledger(&EPOCH).unwrap());

        let retrieved = db
            .get_ledger(EPOCH)
            .expect("Failed to retrieve ledger from the DB");

        assert_eq!(Some(ledger), retrieved);

        assert!(db.has_ledger(&EPOCH).unwrap());
        assert!(!db.has_ledger(&NEXT_EPOCH).unwrap());
    }

    #[test]
    fn test_seed_store_and_get() {
        const EPOCH: u32 = 4;

        let db_dir = env::temp_dir();
        let db = Database::open(db_dir).expect("Failed to open DB");
        db.clear().unwrap();

        let seed = "2vawAhPq9RsPXhz8NvrxB5VXuge8U9vQPGCtjqLZ5idHTUtWHWF8".to_string();

        db.store_seed(EPOCH, seed.clone())
            .expect("Failed to store seed into the DB");

        let retrieved = db
            .get_seed(EPOCH)
            .expect("Failed to retrieve seed from the DB");

        assert_eq!(Some(seed), retrieved);
    }
}
