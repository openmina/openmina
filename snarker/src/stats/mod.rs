mod stats_actions;
pub use stats_actions::{ActionStats, ActionStatsForBlock, ActionStatsSnapshot};

use std::collections::VecDeque;

use redux::{ActionMeta, ActionWithMeta, Timestamp};
use shared::block::{Block, BlockWithHash};

use crate::ActionKind;

pub type ActionKindWithMeta = ActionWithMeta<ActionKind>;

pub struct Stats {
    last_action: ActionKindWithMeta,
    action_stats: ActionStats,
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
        }
    }

    pub fn new_best_tip<T: AsRef<Block>>(
        &mut self,
        time: Timestamp,
        block: &BlockWithHash<T>,
    ) -> &mut Self {
        self.action_stats
            .new_best_tip(time, block.height(), block.hash.clone());
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
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}
