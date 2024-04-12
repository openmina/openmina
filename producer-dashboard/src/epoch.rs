use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use vrf::VrfWonSlot;

use crate::{storage::locked_btreemap::LockedBTreeMap, archive::Block};

pub type EpochStorage = LockedBTreeMap<u32, EpochData>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochData {
    epoch_number: u32,
    won_slots: BTreeMap<u32, VrfWonSlot>,
    blocks: BTreeMap<u32, Block>,
    won_slots_count: usize,
    canonical_blocks: usize,
    orphaned_blocks: usize,
    missed_blocks: usize,
}

impl EpochData {
    pub fn new(epoch_number: u32, won_slots: BTreeMap<u32, VrfWonSlot>) -> Self {
        Self {
            epoch_number,
            won_slots_count: won_slots.len(),
            won_slots,
            blocks: BTreeMap::new(),
            canonical_blocks: 0,
            orphaned_blocks: 0,
            missed_blocks: 0,
        }
    }
    pub fn partition(&self) {
        todo!()
    }
}

pub struct EpochSlice {
    part: usize,
    epoch_data: EpochData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockStatus {
    Canonical,
    Pending,
    Missed,
    Orphaned,
}
