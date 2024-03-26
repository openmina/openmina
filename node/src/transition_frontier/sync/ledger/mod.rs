pub mod snarked;
pub mod staged;

mod transition_frontier_sync_ledger_state;
pub use transition_frontier_sync_ledger_state::*;

mod transition_frontier_sync_ledger_actions;
pub use transition_frontier_sync_ledger_actions::*;

mod transition_frontier_sync_ledger_reducer;

mod transition_frontier_sync_ledger_effects;
pub use transition_frontier_sync_ledger_effects::*;

use mina_p2p_messages::v2::{LedgerHash, MinaBaseStagedLedgerHashStableV1, StateHash};
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum SyncLedgerTargetKind {
    StakingEpoch,
    NextEpoch,
    Root,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncLedgerTarget {
    pub kind: SyncLedgerTargetKind,
    pub snarked_ledger_hash: LedgerHash,
    pub staged: Option<SyncStagedLedgerTarget>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncLedgerTargetWithStaged {
    pub kind: SyncLedgerTargetKind,
    pub snarked_ledger_hash: LedgerHash,
    pub staged: SyncStagedLedgerTarget,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncStagedLedgerTarget {
    pub block_hash: StateHash,
    pub hashes: MinaBaseStagedLedgerHashStableV1,
}

impl SyncLedgerTarget {
    /// Set synchronization target to ledger at the root of transition frontier.
    pub fn root(root_block: &ArcBlockWithHash) -> Self {
        Self {
            kind: SyncLedgerTargetKind::Root,
            snarked_ledger_hash: root_block.snarked_ledger_hash().clone(),
            staged: Some(SyncStagedLedgerTarget {
                block_hash: root_block.hash().clone(),
                hashes: root_block.staged_ledger_hashes().clone(),
            }),
        }
    }

    /// Set synchronization target to current best tip's staking epoch ledger.
    pub fn staking_epoch(best_tip: &ArcBlockWithHash) -> Self {
        // TODO(tizoc): should this return None when it matches the genesis ledger?
        Self {
            kind: SyncLedgerTargetKind::StakingEpoch,
            snarked_ledger_hash: best_tip.staking_epoch_ledger_hash().clone(),
            staged: None,
        }
    }

    /// Set synchronization target to current best tip's staking epoch ledger.
    ///
    /// Will return `None` if we shouldn't synchronize it, in case when
    /// current next_epoch_ledger isn't finalized (reached root) or it
    /// is equal to the genesis ledger.
    ///
    /// In such case, we will reconstruct next_epoch_ledger anyways,
    /// once transition frontier's root will be first slot in the bew epoch.
    pub fn next_epoch(best_tip: &ArcBlockWithHash, root_block: &ArcBlockWithHash) -> Option<Self> {
        if best_tip.next_epoch_ledger_hash() != root_block.next_epoch_ledger_hash()
            || best_tip.next_epoch_ledger_hash() == best_tip.genesis_ledger_hash()
        {
            return None;
        }
        Some(Self {
            kind: SyncLedgerTargetKind::NextEpoch,
            snarked_ledger_hash: best_tip.next_epoch_ledger_hash().clone(),
            staged: None,
        })
    }

    pub fn with_staged(self) -> Option<SyncLedgerTargetWithStaged> {
        Some(SyncLedgerTargetWithStaged {
            kind: self.kind,
            snarked_ledger_hash: self.snarked_ledger_hash,
            staged: self.staged?,
        })
    }
}

impl From<SyncLedgerTargetWithStaged> for SyncLedgerTarget {
    fn from(value: SyncLedgerTargetWithStaged) -> Self {
        Self {
            kind: value.kind,
            snarked_ledger_hash: value.snarked_ledger_hash,
            staged: Some(value.staged),
        }
    }
}
