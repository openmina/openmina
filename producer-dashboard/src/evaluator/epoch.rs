use std::ops::AddAssign;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    archive::{Block, ChainStatus},
    node::epoch_ledgers::{Balances, NanoMina},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochSlots {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllTimeSummary {
    #[serde(flatten)]
    slot_summary: SlotSummary,
}

impl EpochSlots {
    pub fn new(inner: Vec<SlotData>) -> Self {
        Self { inner }
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

    pub fn slot_summary(&self) -> (SlotSummary, bool) {
        let mut slot_summary = SlotSummary::default();
        let mut is_current = false;
        for slot in self.inner.iter() {
            if slot.is_current_slot {
                is_current = true;
            }

            match slot.block_status {
                SlotStatus::Canonical | SlotStatus::CanonicalPending => {
                    slot_summary.won_slots += 1;
                    slot_summary.canonical += 1;
                    slot_summary.earned_rewards += NanoMina::new(720.into());
                }
                SlotStatus::Missed => {
                    slot_summary.won_slots += 1;
                    slot_summary.missed += 1;
                }
                SlotStatus::Orphaned | SlotStatus::OrphanedPending => {
                    slot_summary.won_slots += 1;
                    slot_summary.orphaned += 1;
                }
                SlotStatus::ToBeProduced => {
                    slot_summary.won_slots += 1;
                    slot_summary.future_rights += 1;
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
        slot_summary.expected_rewards = NanoMina::new((slot_summary.won_slots * 720).into());
        (slot_summary, is_current)
    }

    fn summary(&self) -> EpochSummary {
        let (slot_summary, is_current) = self.slot_summary();

        let slot_start = self.inner.first().unwrap().global_slot.to_u32();
        let slot_end = self.inner.last().unwrap().global_slot.to_u32();

        EpochSummary {
            max: slot_summary.won_slots,
            slot_summary,
            slot_start,
            slot_end,
            is_current,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochSummary {
    max: usize,
    #[serde(flatten)]
    slot_summary: SlotSummary,
    slot_start: u32,
    slot_end: u32,
    is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SlotSummary {
    won_slots: usize,
    canonical: usize,
    orphaned: usize,
    missed: usize,
    future_rights: usize,
    expected_rewards: NanoMina,
    earned_rewards: NanoMina,
}

impl AddAssign for SlotSummary {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            won_slots: self.won_slots + rhs.won_slots,
            canonical: self.canonical + rhs.canonical,
            orphaned: self.orphaned + rhs.orphaned,
            missed: self.missed + rhs.missed,
            future_rights: self.future_rights + rhs.future_rights,
            expected_rewards: self.expected_rewards.clone() + rhs.expected_rewards,
            earned_rewards: self.earned_rewards.clone() + rhs.earned_rewards,
        }
    }
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
    pub fn new_won(global_slot: u32, timestamp: i64) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let block_status = if timestamp >= now {
            // Future
            SlotStatus::ToBeProduced
        } else {
            // Default to missed for past slots, if there were not missed the archive watcdog will ammend the status immediately a block has been seen
            SlotStatus::Missed
        };
        let global_slot: RawGlobalSlot = global_slot.into();

        Self {
            slot: global_slot.clone().into(),
            global_slot,
            block_status,
            state_hash: None,
            height: None,
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
