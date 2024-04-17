use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use vrf::{VrfEvaluationOutput, VrfWonSlot};

use crate::{
    archive::{Block, ChainStatus},
    storage::locked_btreemap::LockedBTreeMap,
};

pub type EpochStorage = LockedBTreeMap<u32, EpochSummary>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochSlots {
    epoch_number: u32,
    inner: Vec<SlotData>
}

impl EpochSlots {
    pub fn new(epoch_number: u32, inner: Vec<SlotData>) -> Self {
        Self { epoch_number, inner }
    }

    // pub fn partition(&self) -> Vec<EpochSummary> {
    //     let chunk_size = (self.inner.len() + 4) / 5;
        
    //     self.inner.chun
    // }

    pub fn summary(&self) -> EpochSummary{
        let mut won_slots = 0;
        let mut canonical_blocks = 0;
        let mut orphaned_blocks = 0;
        let mut missed_blocks = 0;

        // You might want to define how rewards are calculated; placeholder values here:
        let mut earned_rewards = 0;

        for slot in self.inner.iter() {
            match slot.block_status {
                BlockStatus::Canonical => {
                    won_slots += 1;
                    canonical_blocks += 1;
                    earned_rewards += 720;
                },
                BlockStatus::CanonicalPending => {
                    won_slots += 1;
                },
                BlockStatus::Missed => {
                    won_slots += 1;
                    missed_blocks += 1;
                },
                BlockStatus::Orphaned | BlockStatus::OrphanedPending => {
                    won_slots += 1;
                    orphaned_blocks += 1;
                },
                BlockStatus::Future => {
                    won_slots += 1;
                }
                BlockStatus::Pending | BlockStatus::Lost => {
                    // Handle other statuses if necessary
                },
            }
        }

        let expected_rewards = (won_slots as u32) * 720;

        EpochSummary {
            epoch_number: self.epoch_number,
            won_slots,
            canonical_blocks,
            orphaned_blocks,
            missed_blocks,
            expected_rewards,
            earned_rewards,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochSummary {
    epoch_number: u32,
    won_slots: usize,
    canonical_blocks: usize,
    orphaned_blocks: usize,
    missed_blocks: usize,
    expected_rewards: u32,
    earned_rewards: u32,
}

impl EpochSummary {
    pub fn new(epoch_number: u32) -> Self {
        Self {
            epoch_number,
            won_slots: 0,
            canonical_blocks: 0,
            orphaned_blocks: 0,
            missed_blocks: 0,
            expected_rewards: 0,
            earned_rewards: 0,
        }
    }
    pub fn partition(&self) {
        todo!()
    }
}

pub struct EpochSlice {
    part: usize,
    epoch_data: EpochSummary,
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
    Lost,
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
        matches!(
            self,
            BlockStatus::CanonicalPending | BlockStatus::OrphanedPending
        )
    }
}

// TODO: better naming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawSlot(u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawGlobalSlot(u32);

impl From<RawGlobalSlot> for RawSlot {
    fn from(value: RawGlobalSlot) -> Self {
        // Calculate the zero-based index in the current epoch
        let zero_based_slot = (value.0 - 1) % 7140;
        // Convert it back to one-based by adding 1
        RawSlot(zero_based_slot + 1)
    }
}

impl From<u32> for RawSlot {
    fn from(value: u32) -> Self {
        RawSlot(value)
    }
}

impl From<u32> for RawGlobalSlot {
    fn from(value: u32) -> Self {
        RawGlobalSlot(value)
    }
}

// TODO(adonagy): move to its own module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotData {
    slot: RawSlot,
    global_slot: RawGlobalSlot,
    block_status: BlockStatus,
    timestamp: i64,
    state_hash: Option<String>,
    height: Option<u32>,
}

// impl From<VrfWonSlot> for SlotData {
//     fn from(value: VrfWonSlot) -> Self {
//         Self::new(value.global_slot, None)
//     }
// }

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
            block_status: value.chain_status.into(),
        }
    }
}

impl From<&Block> for SlotBlockUpdate {
    fn from(value: &Block) -> Self {
        value.clone().into()
    }
}

impl SlotData {
    pub fn new(global_slot: u32, timestamp: i64, block: Option<SlotBlockUpdate>) -> Self {
        let block_status = block
            .clone()
            .map_or(BlockStatus::Future, |block| block.block_status);
        let state_hash = block.clone().map(|block| block.state_hash);
        let height = block.map(|block| block.height);
        let global_slot: RawGlobalSlot = global_slot.into();

        Self {
            slot: global_slot.clone().into(),
            global_slot,
            block_status,
            state_hash,
            height,
            timestamp
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
