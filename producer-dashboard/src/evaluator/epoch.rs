use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use vrf::VrfWonSlot;

use crate::{archive::{Block, ChainStatus}, storage::locked_btreemap::LockedBTreeMap};

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
    CanonicalPending,
    Missed,
    Orphaned,
    OrphanedPending,
    Pending,
    Future,
}

impl From<ChainStatus> for BlockStatus {
    fn from(value: ChainStatus) -> Self {
        match value {
            ChainStatus::Canonical => BlockStatus::Canonical,
            ChainStatus::Orphaned => BlockStatus::Orphaned,
            ChainStatus::Pending => BlockStatus::Pending,
        }
    }
}

impl BlockStatus {
    pub fn in_transition_frontier(&self) -> bool {
        matches!(self, BlockStatus::CanonicalPending | BlockStatus::OrphanedPending)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slot(u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSlot(u32);

impl From<GlobalSlot> for Slot {
    fn from(value: GlobalSlot) -> Self {
        // Calculate the zero-based index in the current epoch
        let zero_based_slot = (value.0 - 1) % 7140;
        // Convert it back to one-based by adding 1
        Slot(zero_based_slot + 1)
    }
}

impl From<u32> for Slot {
    fn from(value: u32) -> Self {
        Slot(value)
    }
}

impl From<u32> for GlobalSlot {
    fn from(value: u32) -> Self {
        GlobalSlot(value)
    }
}

// TODO(adonagy): move to its own module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotData {
    slot: Slot,
    global_slot: GlobalSlot,
    block_status: BlockStatus,
    state_hash: Option<String>,
    height: Option<u32>,
}

impl From<VrfWonSlot> for SlotData {
    fn from(value: VrfWonSlot) -> Self {
        Self::new(value.global_slot, None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotBlockUpdate {
    height: u32,
    state_hash: String,
    block_status: BlockStatus,
}

impl SlotBlockUpdate {
    pub fn new(height: u32, state_hash: String, block_status: BlockStatus) -> Self {
        Self {
            height,
            state_hash,
            block_status,
        }
    }
}

impl From<Block> for SlotBlockUpdate {
    fn from(value: Block) -> Self {
        Self {
            height: value.height as u32,
            state_hash: value.state_hash,
            block_status: value.chain_status.into()
        }
    }
}

impl From<&Block> for SlotBlockUpdate {
    fn from(value: &Block) -> Self {
        value.clone().into()
    }
}

impl SlotData {
    pub fn new(global_slot: u32, block: Option<SlotBlockUpdate>) -> Self {
        let block_status = block.clone().map_or(BlockStatus::Future, |block| block.block_status);
        let state_hash = block.clone().map(|block| block.state_hash);
        let height = block.map(|block| block.height);
        let global_slot: GlobalSlot = global_slot.into();

        Self {
            slot: global_slot.clone().into(),
            global_slot,
            block_status,
            state_hash,
            height
        }
    }

    pub fn add_block(&mut self, block: SlotBlockUpdate) {
        self.state_hash = Some(block.state_hash);
        self.height = Some(block.height);
        self.block_status = block.block_status;
    }

    pub fn update_block_status(&mut self, block_status: BlockStatus) {
        self.block_status = block_status;
    }
}