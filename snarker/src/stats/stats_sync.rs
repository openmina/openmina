use std::collections::VecDeque;

use mina_p2p_messages::v2::StateHash;
use redux::Timestamp;
use serde::{Deserialize, Serialize};
use shared::block::ArcBlockWithHash;

use crate::transition_frontier::sync::TransitionFrontierSyncBlockState;

const MAX_SNAPSHOTS_LEN: usize = 256;

#[derive(Default)]
pub struct SyncStats {
    snapshots: VecDeque<SyncStatsSnapshot>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncStatsSnapshot {
    pub kind: SyncKind,
    pub best_tip_received: Timestamp,
    pub synced: Option<Timestamp>,
    pub ledgers: SyncLedgers,
    pub blocks: Vec<SyncBlock>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncKind {
    Bootstrap,
    Catchup,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SyncLedgers {
    pub root: Option<SyncLedger>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SyncLedger {
    pub snarked: SyncSnarkedLedger,
    pub staged: SyncStagedLedger,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SyncSnarkedLedger {
    pub fetch_hashes_start: Option<Timestamp>,
    pub fetch_hashes_end: Option<Timestamp>,
    pub fetch_accounts_start: Option<Timestamp>,
    pub fetch_accounts_end: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SyncStagedLedger {
    pub fetch_parts_start: Option<Timestamp>,
    pub fetch_parts_end: Option<Timestamp>,
    pub reconstruct_start: Option<Timestamp>,
    pub reconstruct_end: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncBlock {
    pub global_slot: Option<u32>,
    pub height: u32,
    pub hash: StateHash,
    pub pred_hash: StateHash,
    pub status: SyncBlockStatus,
    pub fetch_start: Option<Timestamp>,
    pub fetch_end: Option<Timestamp>,
    pub apply_start: Option<Timestamp>,
    pub apply_end: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncBlockStatus {
    Missing,
    Fetching,
    Fetched,
    Applying,
    Applied,
}

pub enum SyncingLedger {
    FetchHashes {
        start: Timestamp,
        end: Timestamp,
    },
    FetchAccounts {
        start: Timestamp,
        end: Timestamp,
    },
    FetchParts {
        start: Timestamp,
        end: Option<Timestamp>,
    },
    ApplyParts {
        start: Timestamp,
        end: Option<Timestamp>,
    },
}

impl SyncStats {
    pub fn new_target(&mut self, time: Timestamp, best_tip: &ArcBlockWithHash) -> &mut Self {
        let kind = match self.snapshots.back().map_or(true, |s| {
            matches!(s.kind, SyncKind::Bootstrap) && s.synced.is_none()
        }) {
            true => SyncKind::Bootstrap,
            false => SyncKind::Catchup,
        };
        let best_tip_block_state = SyncBlock {
            global_slot: Some(best_tip.global_slot()),
            height: best_tip.height(),
            hash: best_tip.hash().clone(),
            pred_hash: best_tip.pred_hash().clone(),
            status: SyncBlockStatus::Fetched,
            fetch_start: None,
            fetch_end: None,
            apply_start: None,
            apply_end: None,
        };

        if self.snapshots.len() >= MAX_SNAPSHOTS_LEN {
            self.snapshots.pop_front();
        }

        self.snapshots.push_back(SyncStatsSnapshot {
            kind,
            best_tip_received: time,
            synced: None,
            ledgers: Default::default(),
            blocks: vec![best_tip_block_state],
        });

        self
    }

    pub fn ledger(&mut self, update: SyncingLedger) -> &mut Self {
        let Some(snapshot) = self.snapshots.back_mut() else { return self };
        let ledger = snapshot.ledgers.root.get_or_insert_with(Default::default);

        match update {
            SyncingLedger::FetchHashes { start, end } => {
                ledger.snarked.fetch_hashes_start.get_or_insert(start);
                let cur_end = ledger.snarked.fetch_hashes_end.get_or_insert(end);
                *cur_end = end.max(*cur_end);
            }
            SyncingLedger::FetchAccounts { start, end } => {
                ledger.snarked.fetch_accounts_start.get_or_insert(start);
                let cur_end = ledger.snarked.fetch_accounts_end.get_or_insert(end);
                *cur_end = end.max(*cur_end);
            }
            SyncingLedger::FetchParts { start, end } => {
                ledger.staged.fetch_parts_start.get_or_insert(start);
                if let Some(end) = end {
                    let cur_end = ledger.staged.fetch_parts_end.get_or_insert(end);
                    *cur_end = end.max(*cur_end);
                }
            }
            SyncingLedger::ApplyParts { start, end } => {
                ledger.staged.reconstruct_start.get_or_insert(start);
                if let Some(end) = end {
                    let cur_end = ledger.staged.reconstruct_end.get_or_insert(end);
                    *cur_end = end.max(*cur_end);
                }
            }
        }

        self
    }

    pub fn blocks_init(&mut self, states: &[TransitionFrontierSyncBlockState]) -> &mut Self {
        let Some(snapshot) = self.snapshots.back_mut() else { return self };
        let Some(best_tip_height) = states
            .last()
            .and_then(|s| s.block())
            .map(|b| b.height()) else { return self };

        snapshot.blocks = states
            .into_iter()
            .rev()
            .take_while(|s| !s.is_apply_success())
            .enumerate()
            .map(|(i, s)| {
                let height = best_tip_height - i as u32;
                let hash = s.block_hash().clone();
                let pred_hash = s
                    .block()
                    .map(|b| b.pred_hash())
                    .unwrap_or_else(|| states[states.len() - i - 2].block_hash())
                    .clone();
                let mut stats = SyncBlock::new(height, hash, pred_hash);
                stats.update_with_block_state(s);
                stats
            })
            .collect();

        self
    }

    pub fn block_update(&mut self, state: &TransitionFrontierSyncBlockState) -> &mut Self {
        let Some(snapshot) = self.snapshots.back_mut() else { return self };
        let block_hash = state.block_hash();
        let Some(stats) = snapshot.blocks.iter_mut().find(|b| &b.hash == block_hash) else { return self };
        stats.update_with_block_state(state);
        self
    }

    pub fn synced(&mut self, time: Timestamp) -> &mut Self {
        let Some(snapshot) = self.snapshots.back_mut() else { return self };
        snapshot.synced = Some(time);
        self
    }

    pub fn collect_stats(&self, limit: Option<usize>) -> Vec<SyncStatsSnapshot> {
        let limit = limit.unwrap_or(usize::MAX);
        self.snapshots.iter().rev().take(limit).cloned().collect()
    }
}

impl SyncBlock {
    pub fn new(height: u32, hash: StateHash, pred_hash: StateHash) -> Self {
        Self {
            global_slot: None,
            height,
            hash,
            pred_hash,
            status: SyncBlockStatus::Missing,
            fetch_start: None,
            fetch_end: None,
            apply_start: None,
            apply_end: None,
        }
    }

    pub fn update_with_block_state(&mut self, state: &TransitionFrontierSyncBlockState) {
        match state {
            TransitionFrontierSyncBlockState::FetchPending { attempts, .. } => {
                if let Some(time) = attempts
                    .iter()
                    .filter_map(|(_, v)| v.fetch_pending_since())
                    .min()
                {
                    self.status = SyncBlockStatus::Missing;
                    self.fetch_start.get_or_insert(time);
                } else {
                    self.status = SyncBlockStatus::Fetching;
                }
            }
            TransitionFrontierSyncBlockState::FetchSuccess { time, block, .. } => {
                self.global_slot.get_or_insert_with(|| block.global_slot());
                self.status = SyncBlockStatus::Fetched;
                self.fetch_end = Some(*time);
            }
            TransitionFrontierSyncBlockState::ApplyPending { time, block, .. } => {
                self.global_slot.get_or_insert_with(|| block.global_slot());
                self.status = SyncBlockStatus::Applying;
                self.apply_start = Some(*time);
            }
            TransitionFrontierSyncBlockState::ApplySuccess { time, block, .. } => {
                self.global_slot.get_or_insert_with(|| block.global_slot());
                self.status = SyncBlockStatus::Applied;
                self.apply_end = Some(*time);
            }
        }
    }
}
