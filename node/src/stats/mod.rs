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

mod stats_block_producer;
pub mod block_producer {
    pub use super::stats_block_producer::*;
}
use block_producer::BlockProducerStats;

use openmina_core::block::{ArcBlockWithHash, Block, BlockWithHash};
use redux::{ActionMeta, ActionWithMeta, Timestamp};

use crate::transition_frontier::sync::ledger::staged::PeerStagedLedgerPartsFetchError;
use crate::transition_frontier::sync::ledger::SyncLedgerTargetKind;
use crate::transition_frontier::sync::TransitionFrontierSyncBlockState;
use crate::ActionKind;

pub type ActionKindWithMeta = ActionWithMeta<ActionKind>;

pub struct Stats {
    last_action: ActionKindWithMeta,
    action_stats: ActionStats,
    sync_stats: SyncStats,
    block_producer_stats: BlockProducerStats,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            last_action: ActionMeta::ZERO.with_action(ActionKind::None),
            action_stats: Default::default(),
            sync_stats: Default::default(),
            block_producer_stats: Default::default(),
        }
    }

    pub fn block_producer(&mut self) -> &mut BlockProducerStats {
        &mut self.block_producer_stats
    }

    pub fn new_sync_target(
        &mut self,
        time: Timestamp,
        best_tip: &ArcBlockWithHash,
        root_block: &ArcBlockWithHash,
    ) -> &mut Self {
        self.sync_stats.new_target(time, best_tip, root_block);
        self
    }

    pub fn syncing_ledger(
        &mut self,
        kind: SyncLedgerTargetKind,
        update: SyncingLedger,
    ) -> &mut Self {
        self.sync_stats.ledger(kind, update);
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

    pub fn new_best_chain<T: AsRef<Block>>(
        &mut self,
        time: Timestamp,
        chain: &[BlockWithHash<T>],
    ) -> &mut Self {
        let best_tip = chain.last().unwrap();
        self.action_stats
            .new_best_tip(time, best_tip.height(), best_tip.hash().clone());
        self.sync_stats.synced(time);
        self.block_producer_stats.new_best_chain(time, chain);
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

    pub fn get_sync_time(&self) -> Option<Timestamp> {
        self.sync_stats
            .collect_stats(Some(1))
            .first()
            .and_then(|stats| stats.synced)
    }

    pub fn staging_ledger_fetch_failure(
        &mut self,
        error: &PeerStagedLedgerPartsFetchError,
        time: Timestamp,
    ) {
        self.sync_stats
            .staging_ledger_fetch_failure(format!("{error:?}"), time)
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}
