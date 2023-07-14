mod stats_actions;
pub mod actions {
    pub use super::stats_actions::*;
}
use actions::{ActionStats, ActionStatsForBlock, ActionStatsSnapshot};

mod stats_sync;
pub mod sync {
    pub use super::stats_sync::*;
}
use sync::{SyncStats, SyncStatsSnapshot, SyncingLedger};

use std::collections::VecDeque;

use redux::{ActionMeta, ActionWithMeta, Timestamp};
use shared::block::{ArcBlockWithHash, Block, BlockWithHash};

use crate::transition_frontier::sync::TransitionFrontierSyncBlockState;
use crate::ActionKind;

pub type ActionKindWithMeta = ActionWithMeta<ActionKind>;

pub struct Stats {
    last_action: ActionKindWithMeta,
    action_stats: ActionStats,
    sync_stats: SyncStats,
}

impl Stats {
    pub fn new() -> Self {
        let mut action_stats_per_block = VecDeque::new();
        action_stats_per_block.push_back(ActionStatsForBlock {
            id: 0,
            time: Timestamp::ZERO,
            block_level: 1,
            // TODO(binier): use configured genesis hash.
            block_hash: "3NKeMoncuHab5ScarV5ViyF16cJPT4taWNSaTLS64Dp67wuXigPZ"
                .parse()
                .unwrap(),
            cpu_idle: 0,
            cpu_busy: 0,
            stats: Default::default(),
        });
        Self {
            last_action: ActionMeta::ZERO.with_action(ActionKind::None),
            action_stats: ActionStats {
                since_start: Default::default(),
                per_block: action_stats_per_block,
            },
            sync_stats: Default::default(),
        }
    }

    pub fn new_sync_target(&mut self, time: Timestamp, best_tip: &ArcBlockWithHash) -> &mut Self {
        self.sync_stats.new_target(time, best_tip);
        self
    }

    pub fn syncing_ledger(&mut self, update: SyncingLedger) -> &mut Self {
        self.sync_stats.ledger(update);
        self
    }

    pub fn syncing_blocks_init(
        &mut self,
        states: &[TransitionFrontierSyncBlockState],
    ) -> &mut Self {
        self.sync_stats.blocks_init(states);
        self
    }

    pub fn syncing_block_update(&mut self, state: &TransitionFrontierSyncBlockState) -> &mut Self {
        self.sync_stats.block_update(state);
        self
    }

    pub fn new_best_tip<T: AsRef<Block>>(
        &mut self,
        time: Timestamp,
        block: &BlockWithHash<T>,
    ) -> &mut Self {
        self.action_stats
            .new_best_tip(time, block.height(), block.hash.clone());
        self.sync_stats.synced(time);
        self
    }

    pub fn new_action(&mut self, kind: ActionKind, meta: ActionMeta) -> &mut Self {
        let action = meta.with_action(kind);
        self.action_stats.add(&action, &self.last_action);
        self.last_action = action;
        self
    }

    pub fn collect_action_stats_since_start(&self) -> ActionStatsSnapshot {
        self.action_stats.since_start.clone()
    }

    pub fn collect_action_stats_for_block_with_id(
        &self,
        id: Option<u64>,
    ) -> Option<ActionStatsForBlock> {
        self.action_stats.collect_stats_for_block_with_id(id)
    }

    pub fn collect_sync_stats(&self, limit: Option<usize>) -> Vec<SyncStatsSnapshot> {
        self.sync_stats.collect_stats(limit)
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}
