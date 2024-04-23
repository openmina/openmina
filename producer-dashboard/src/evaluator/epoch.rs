use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use vrf::{VrfEvaluationOutput, VrfWonSlot};

use crate::{
    archive::{Block, ChainStatus},
    node::epoch_ledgers::Balances,
    storage::locked_btreemap::LockedBTreeMap,
};

pub type EpochStorage = LockedBTreeMap<u32, EpochSummary>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochSlots {
    // epoch_number: u32,
    inner: Vec<SlotData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedSummary {
    epoch_number: u32,
    summary: Option<EpochSummary>,
    sub_windows: Vec<EpochSummary>,
    #[serde(flatten)]
    balances: Balances,
}

impl EpochSlots {
    pub fn new(inner: Vec<SlotData>) -> Self {
        Self {
            // epoch_number,
            inner,
        }
    }

    pub fn merged_summary(&self, epoch_number: u32, balances: Balances) -> MergedSummary {
        if self.inner.is_empty() {
            MergedSummary {
                epoch_number,
                summary: None,
                sub_windows: vec![],
                balances: Balances::default(),
            }
        } else {
            let summary = self.summary();
            MergedSummary {
                epoch_number,
                summary: Some(summary),
                sub_windows: self.sub_windows(),
                balances,
            }
        }
    }

    pub fn sub_windows(&self) -> Vec<EpochSummary> {
        let chunk_size = self.inner.len() / 15;

        self.inner
            .chunks_exact(chunk_size)
            .map(|window| EpochSlots::new(window.to_vec()).summary())
            .collect()
    }

    pub fn summary(&self) -> EpochSummary {
        let mut won_slots = 0;
        let mut canonical_blocks = 0;
        let mut orphaned_blocks = 0;
        let mut missed_blocks = 0;
        let mut future_rights = 0;

        let mut earned_rewards = 0;

        for slot in self.inner.iter() {
            match slot.block_status {
                SlotStatus::Canonical | SlotStatus::CanonicalPending => {
                    won_slots += 1;
                    canonical_blocks += 1;
                    earned_rewards += 720;
                }
                SlotStatus::Missed => {
                    won_slots += 1;
                    missed_blocks += 1;
                }
                SlotStatus::Orphaned | SlotStatus::OrphanedPending => {
                    won_slots += 1;
                    orphaned_blocks += 1;
                }
                SlotStatus::ToBeProduced => {
                    won_slots += 1;
                    future_rights += 1;
                }
                SlotStatus::Pending
                | SlotStatus::Lost
                | SlotStatus::Empty
                | SlotStatus::Foreign
                | SlotStatus::ForeignToBeProduced => {
                    // Handle other statuses if necessary
                }
            }
        }

        let slot_start = self.inner.first().unwrap().global_slot.to_u32();
        let slot_end = self.inner.last().unwrap().global_slot.to_u32();
        let expected_rewards = (won_slots as u32) * 720;

        EpochSummary {
            // epoch_number: self.epoch_number,
            max: won_slots,
            won_slots,
            canonical: canonical_blocks,
            orphaned: orphaned_blocks,
            missed: missed_blocks,
            expected_rewards,
            earned_rewards,
            future_rights,
            slot_start,
            slot_end,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochSummary {
    // epoch_number: u32,
    max: usize,
    won_slots: usize,
    canonical: usize,
    orphaned: usize,
    missed: usize,
    future_rights: usize,
    slot_start: u32,
    slot_end: u32,
    expected_rewards: u32,
    earned_rewards: u32,
}

// impl EpochSummary {
//     pub fn new(epoch_number: u32) -> Self {
//         Self {
//             // epoch_number,
//             max: 0,
//             won_slots: 0,
//             canonical: 0,
//             orphaned: 0,
//             missed: 0,
//             expected_rewards: 0,
//             earned_rewards: 0,
//             future_rights: 0,
//             slot_start: 0,
//             slot_end: 0,
//         }
//     }
//     pub fn partition(&self) {
//         todo!()
//     }
// }

pub struct EpochSlice {
    part: usize,
    epoch_data: EpochSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlotStatus {
    Canonical,
    CanonicalPending,
    Missed,
    Orphaned,
    OrphanedPending,
    Pending,
    ToBeProduced,
    Lost,
    Empty,
    Foreign,
    ForeignToBeProduced,
}

impl From<ChainStatus> for SlotStatus {
    fn from(value: ChainStatus) -> Self {
        match value {
            ChainStatus::Canonical => SlotStatus::Canonical,
            ChainStatus::Orphaned => SlotStatus::Orphaned,
            ChainStatus::Pending => SlotStatus::Pending,
        }
    }
}

impl SlotStatus {
    pub fn in_transition_frontier(&self) -> bool {
        matches!(
            self,
            SlotStatus::CanonicalPending | SlotStatus::OrphanedPending
        )
    }
}

// TODO: better naming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawSlot(u32);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct RawGlobalSlot(u32);

impl From<RawGlobalSlot> for RawSlot {
    fn from(value: RawGlobalSlot) -> Self {
        RawSlot(value.0 % 7140)
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

impl RawGlobalSlot {
    pub fn to_u32(&self) -> u32 {
        self.0
    }

    pub fn epoch(&self) -> u32 {
        self.0 / 7140
    }
}

// TODO(adonagy): move to its own module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotData {
    slot: RawSlot,
    global_slot: RawGlobalSlot,
    block_status: SlotStatus,
    timestamp: i64,
    state_hash: Option<String>,
    height: Option<u32>,
    is_current_slot: bool,
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
    block_status: SlotStatus,
}

impl SlotBlockUpdate {
    pub fn new(height: u32, state_hash: String, block_status: SlotStatus) -> Self {
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
            .map_or(SlotStatus::ToBeProduced, |block| block.block_status);
        let state_hash = block.clone().map(|block| block.state_hash);
        let height = block.map(|block| block.height);
        let global_slot: RawGlobalSlot = global_slot.into();

        Self {
            slot: global_slot.clone().into(),
            global_slot,
            block_status,
            state_hash,
            height,
            timestamp,
            is_current_slot: false,
        }
    }

    pub fn global_slot(&self) -> RawGlobalSlot {
        self.global_slot.clone()
    }

    pub fn has_block(&self) -> bool {
        self.state_hash.is_some()
    }

    pub fn block_status(&self) -> SlotStatus {
        self.block_status.clone()
    }

    pub fn new_lost(global_slot: u32, timestamp: i64) -> Self {
        let global_slot: RawGlobalSlot = global_slot.into();
        Self {
            slot: global_slot.clone().into(),
            global_slot,
            block_status: SlotStatus::Empty,
            timestamp,
            state_hash: None,
            height: None,
            is_current_slot: false,
        }
    }

    pub fn add_block(&mut self, block: SlotBlockUpdate) {
        self.state_hash = Some(block.state_hash);
        self.height = Some(block.height);
        self.block_status = block.block_status;
    }

    pub fn update_block_status(&mut self, block_status: SlotStatus) {
        self.block_status = block_status;
    }

    pub fn set_as_current(&mut self) {
        self.is_current_slot = true;
    }

    pub fn unset_as_current(&mut self) {
        self.is_current_slot = false;
    }
}
