use std::collections::BTreeMap;

use mina_p2p_messages::v2::{self, LedgerHash, MinaStateProtocolStateValueStableV2, StateHash};
use openmina_core::block::{AppliedBlock, ArcBlockWithHash};
use redux::Timestamp;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::p2p::channels::rpc::P2pRpcId;
use crate::p2p::PeerId;

use super::ledger::{SyncLedgerTarget, SyncLedgerTargetKind, TransitionFrontierSyncLedgerState};
use super::PeerBlockFetchError;

#[derive(Serialize, Deserialize, Display, Debug, Clone)]
pub enum TransitionFrontierSyncState {
    Idle,
    Init {
        time: Timestamp,
        best_tip: ArcBlockWithHash,
        root_block: ArcBlockWithHash,
        blocks_inbetween: Vec<StateHash>,
    },
    StakingLedgerPending(TransitionFrontierSyncLedgerPending),
    StakingLedgerSuccess {
        time: Timestamp,
        best_tip: ArcBlockWithHash,
        root_block: ArcBlockWithHash,
        blocks_inbetween: Vec<StateHash>,
        needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    },
    NextEpochLedgerPending(TransitionFrontierSyncLedgerPending),
    NextEpochLedgerSuccess {
        time: Timestamp,
        best_tip: ArcBlockWithHash,
        root_block: ArcBlockWithHash,
        blocks_inbetween: Vec<StateHash>,
        needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    },
    RootLedgerPending(TransitionFrontierSyncLedgerPending),
    RootLedgerSuccess {
        time: Timestamp,
        best_tip: ArcBlockWithHash,
        root_block: ArcBlockWithHash,
        blocks_inbetween: Vec<StateHash>,
        root_block_updates: Vec<ArcBlockWithHash>,
        needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    },
    BlocksPending {
        time: Timestamp,
        chain: Vec<TransitionFrontierSyncBlockState>,
        /// Snarked ledger updates/transitions that happened while we
        /// were synchronizing blocks. If those updates do happen, we
        /// need to create those snarked ledgers from the closest snarked
        /// ledger that we have in the service already synchronized.
        ///
        /// Contains a map where the `key` is the new snarked ledger and
        /// the `value` is more info required to construct that ledger.
        root_snarked_ledger_updates: TransitionFrontierRootSnarkedLedgerUpdates,
        needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    },
    BlocksSuccess {
        time: Timestamp,
        chain: Vec<AppliedBlock>,
        root_snarked_ledger_updates: TransitionFrontierRootSnarkedLedgerUpdates,
        needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    },
    CommitPending {
        time: Timestamp,
        chain: Vec<AppliedBlock>,
        root_snarked_ledger_updates: TransitionFrontierRootSnarkedLedgerUpdates,
        needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    },
    CommitSuccess {
        time: Timestamp,
        chain: Vec<AppliedBlock>,
        root_snarked_ledger_updates: TransitionFrontierRootSnarkedLedgerUpdates,
        needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    },
    Synced {
        time: Timestamp,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerPending {
    pub time: Timestamp,
    pub best_tip: ArcBlockWithHash,
    pub root_block: ArcBlockWithHash,
    pub blocks_inbetween: Vec<StateHash>,
    pub root_block_updates: Vec<ArcBlockWithHash>,
    pub ledger: TransitionFrontierSyncLedgerState,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TransitionFrontierRootSnarkedLedgerUpdates(
    BTreeMap<LedgerHash, TransitionFrontierRootSnarkedLedgerUpdate>,
);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierRootSnarkedLedgerUpdate {
    pub parent: LedgerHash,
    /// Staged ledger hash of the applied block, that had the same snarked
    /// ledger as the target. From that staged ledger we can fetch
    /// transactions that we need to apply on top of `parent` in order
    /// to construct target snarked ledger.
    pub staged_ledger_hash: v2::MinaBaseStagedLedgerHashStableV1,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncBlockState {
    FetchPending {
        time: Timestamp,
        block_hash: StateHash,
        attempts: BTreeMap<PeerId, PeerRpcState>,
    },
    FetchSuccess {
        time: Timestamp,
        block: ArcBlockWithHash,
    },
    ApplyPending {
        time: Timestamp,
        block: ArcBlockWithHash,
    },
    ApplyError {
        time: Timestamp,
        block: ArcBlockWithHash,
        error: String,
    },
    ApplySuccess {
        time: Timestamp,
        block: AppliedBlock,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerRpcState {
    Init {
        time: Timestamp,
    },
    Pending {
        time: Timestamp,
        rpc_id: P2pRpcId,
    },
    Error {
        time: Timestamp,
        rpc_id: P2pRpcId,
        error: PeerBlockFetchError,
    },
    Success {
        time: Timestamp,
        block: ArcBlockWithHash,
    },
}

#[derive(Serialize, Deserialize, Display, Debug, Clone)]
pub enum SyncPhase {
    Bootstrap,
    Catchup,
    Synced,
}

impl TransitionFrontierSyncState {
    /// If the synchronization process has started but is not yet complete
    pub fn is_pending(&self) -> bool {
        !matches!(self, Self::Idle | Self::Synced { .. })
    }

    pub fn is_commit_pending(&self) -> bool {
        matches!(self, Self::CommitPending { .. })
    }

    /// If the synchronization process is complete
    pub fn is_synced(&self) -> bool {
        matches!(self, Self::Synced { .. })
    }

    pub fn time(&self) -> Option<redux::Timestamp> {
        match self {
            Self::Idle => None,
            Self::Init { time, .. } => Some(*time),
            Self::StakingLedgerPending(s) => Some(s.time),
            Self::StakingLedgerSuccess { time, .. } => Some(*time),
            Self::NextEpochLedgerPending(s) => Some(s.time),
            Self::NextEpochLedgerSuccess { time, .. } => Some(*time),
            Self::RootLedgerPending(s) => Some(s.time),
            Self::RootLedgerSuccess { time, .. } => Some(*time),
            Self::BlocksPending { time, .. } => Some(*time),
            Self::BlocksSuccess { time, .. } => Some(*time),
            Self::CommitPending { time, .. } => Some(*time),
            Self::CommitSuccess { time, .. } => Some(*time),
            Self::Synced { time, .. } => Some(*time),
        }
    }

    pub fn root_block(&self) -> Option<&ArcBlockWithHash> {
        match self {
            Self::Idle => None,
            Self::Init { root_block, .. } => Some(root_block),
            Self::StakingLedgerPending(s) => Some(&s.root_block),
            Self::StakingLedgerSuccess { root_block, .. } => Some(root_block),
            Self::NextEpochLedgerPending(s) => Some(&s.root_block),
            Self::NextEpochLedgerSuccess { root_block, .. } => Some(root_block),
            Self::RootLedgerPending(s) => Some(&s.root_block),
            Self::RootLedgerSuccess { root_block, .. } => Some(root_block),
            Self::BlocksPending { chain, .. } => chain.first().and_then(|b| b.block()),
            Self::BlocksSuccess { chain, .. } => chain.first().map(AppliedBlock::block_with_hash),
            Self::CommitPending { chain, .. } => chain.first().map(AppliedBlock::block_with_hash),
            Self::CommitSuccess { chain, .. } => chain.first().map(AppliedBlock::block_with_hash),
            Self::Synced { .. } => None,
        }
    }

    pub fn best_tip(&self) -> Option<&ArcBlockWithHash> {
        match self {
            Self::Idle => None,
            Self::Init { best_tip, .. } => Some(best_tip),
            Self::StakingLedgerPending(s) => Some(&s.best_tip),
            Self::StakingLedgerSuccess { best_tip, .. } => Some(best_tip),
            Self::NextEpochLedgerPending(s) => Some(&s.best_tip),
            Self::NextEpochLedgerSuccess { best_tip, .. } => Some(best_tip),
            Self::RootLedgerPending(s) => Some(&s.best_tip),
            Self::RootLedgerSuccess { best_tip, .. } => Some(best_tip),
            Self::BlocksPending { chain, .. } => chain.last().and_then(|b| b.block()),
            Self::BlocksSuccess { chain, .. } => chain.last().map(AppliedBlock::block_with_hash),
            Self::CommitPending { chain, .. } => chain.last().map(AppliedBlock::block_with_hash),
            Self::CommitSuccess { chain, .. } => chain.last().map(AppliedBlock::block_with_hash),
            Self::Synced { .. } => None,
        }
    }

    pub fn ledger(&self) -> Option<&TransitionFrontierSyncLedgerState> {
        match self {
            Self::StakingLedgerPending(s) => Some(&s.ledger),
            Self::NextEpochLedgerPending(s) => Some(&s.ledger),
            Self::RootLedgerPending(s) => Some(&s.ledger),
            _ => None,
        }
    }

    pub fn ledger_mut(&mut self) -> Option<&mut TransitionFrontierSyncLedgerState> {
        match self {
            Self::StakingLedgerPending(s) => Some(&mut s.ledger),
            Self::NextEpochLedgerPending(s) => Some(&mut s.ledger),
            Self::RootLedgerPending(s) => Some(&mut s.ledger),
            _ => None,
        }
    }

    pub fn ledger_target(&self) -> Option<SyncLedgerTarget> {
        self.ledger().map(|s| s.target())
    }

    pub fn ledger_target_kind(&self) -> Option<SyncLedgerTargetKind> {
        self.ledger().map(|s| s.target_kind())
    }

    /// True if the synchronization of the target ledger is complete.
    ///
    /// Epoch ledgers only require the snarked ledger to be synchronized,
    /// but the ledger at the root of the transition frontier also requires
    /// the staging ledger to be synchronized.
    pub fn is_ledger_sync_complete(&self) -> bool {
        match self {
            Self::StakingLedgerPending(s) => s.ledger.is_snarked_ledger_synced(),
            Self::NextEpochLedgerPending(s) => s.ledger.is_snarked_ledger_synced(),
            Self::RootLedgerPending(s) => s.ledger.staged().map_or(false, |s| s.is_success()),
            _ => false,
        }
    }

    pub fn blocks_iter(&self) -> impl Iterator<Item = &TransitionFrontierSyncBlockState> {
        static EMPTY: Vec<TransitionFrontierSyncBlockState> = Vec::new();
        match self {
            Self::BlocksPending { chain, .. } => chain.iter(),
            _ => EMPTY.iter(),
        }
    }

    pub fn pending_count(&self) -> usize {
        self.blocks_iter()
            .filter(|b| !matches!(b, TransitionFrontierSyncBlockState::ApplySuccess { .. }))
            .count()
    }

    pub fn blocks_fetch_retry_iter(&self) -> impl '_ + Iterator<Item = StateHash> {
        self.blocks_iter().filter_map(|s| s.retry_hash()).cloned()
    }

    pub fn blocks_fetch_next(&self) -> Option<StateHash> {
        self.blocks_iter().find_map(|s| match s {
            TransitionFrontierSyncBlockState::FetchPending {
                block_hash,
                attempts,
                ..
            } => Some(block_hash).filter(|_| attempts.is_empty()).cloned(),
            _ => None,
        })
    }

    pub fn block_state(&self, hash: &StateHash) -> Option<&TransitionFrontierSyncBlockState> {
        self.blocks_iter().find(|s| s.block_hash() == hash)
    }

    pub fn block_state_mut(
        &mut self,
        hash: &StateHash,
    ) -> Option<&mut TransitionFrontierSyncBlockState> {
        match self {
            Self::BlocksPending { chain, .. } => chain.iter_mut().find(|s| s.block_hash() == hash),
            _ => None,
        }
    }

    pub fn is_fetch_pending_from_peer(
        &self,
        hash: &StateHash,
        peer_id: &PeerId,
        rpc_id: P2pRpcId,
    ) -> bool {
        self.block_state(hash)
            .map_or(false, |s| s.is_fetch_pending_from_peer(peer_id, rpc_id))
    }

    pub fn blocks_fetch_from_peer_pending_rpc_ids<'a>(
        &'a self,
        peer_id: &'a PeerId,
    ) -> impl 'a + Iterator<Item = P2pRpcId> {
        self.blocks_iter()
            .filter_map(|b| b.fetch_pending_from_peer_rpc_id(peer_id))
    }

    pub fn blocks_apply_pending(&self) -> Option<&ArcBlockWithHash> {
        self.blocks_iter()
            .find(|s| s.is_apply_pending())
            .and_then(|s| s.block())
    }

    pub fn blocks_apply_next(&self) -> Option<(&ArcBlockWithHash, &AppliedBlock)> {
        let mut last_applied = None;
        for s in self.blocks_iter() {
            if s.is_apply_success() {
                last_applied = s.applied_block();
            } else if s.is_fetch_success() {
                return Some((s.block()?, last_applied?));
            } else {
                return None;
            }
        }
        None
    }

    pub fn sync_phase(&self) -> SyncPhase {
        match self {
            TransitionFrontierSyncState::Idle
            | TransitionFrontierSyncState::Init { .. }
            | TransitionFrontierSyncState::StakingLedgerPending(_)
            | TransitionFrontierSyncState::StakingLedgerSuccess { .. }
            | TransitionFrontierSyncState::NextEpochLedgerPending(_)
            | TransitionFrontierSyncState::NextEpochLedgerSuccess { .. }
            | TransitionFrontierSyncState::RootLedgerPending(_)
            | TransitionFrontierSyncState::RootLedgerSuccess { .. } => SyncPhase::Bootstrap,
            TransitionFrontierSyncState::BlocksPending { .. }
            | TransitionFrontierSyncState::BlocksSuccess { .. }
            | TransitionFrontierSyncState::CommitPending { .. }
            | TransitionFrontierSyncState::CommitSuccess { .. } => SyncPhase::Catchup,
            TransitionFrontierSyncState::Synced { .. } => SyncPhase::Synced,
        }
    }
}

impl TransitionFrontierSyncBlockState {
    pub fn is_fetch_success(&self) -> bool {
        matches!(self, Self::FetchSuccess { .. })
    }

    pub fn is_apply_pending(&self) -> bool {
        matches!(self, Self::ApplyPending { .. })
    }

    pub fn is_apply_error(&self) -> bool {
        matches!(self, Self::ApplyError { .. })
    }

    pub fn is_apply_success(&self) -> bool {
        matches!(self, Self::ApplySuccess { .. })
    }

    pub fn block_hash(&self) -> &StateHash {
        match self {
            Self::FetchPending { block_hash, .. } => block_hash,
            Self::FetchSuccess { block, .. }
            | Self::ApplyPending { block, .. }
            | Self::ApplyError { block, .. } => &block.hash,
            Self::ApplySuccess { block, .. } => block.hash(),
        }
    }

    pub fn block(&self) -> Option<&ArcBlockWithHash> {
        match self {
            Self::FetchPending { .. } => None,
            Self::FetchSuccess { block, .. }
            | Self::ApplyPending { block, .. }
            | Self::ApplyError { block, .. } => Some(block),
            Self::ApplySuccess { block, .. } => Some(block.block_with_hash()),
        }
    }

    pub fn applied_block(&self) -> Option<&AppliedBlock> {
        match self {
            Self::FetchPending { .. }
            | Self::FetchSuccess { .. }
            | Self::ApplyPending { .. }
            | Self::ApplyError { .. } => None,
            Self::ApplySuccess { block, .. } => Some(block),
        }
    }

    pub fn take_block(self) -> Option<ArcBlockWithHash> {
        match self {
            Self::FetchPending { .. } => None,
            Self::FetchSuccess { block, .. }
            | Self::ApplyPending { block, .. }
            | Self::ApplyError { block, .. } => Some(block),
            Self::ApplySuccess { block, .. } => Some(block.block),
        }
    }

    pub fn take_applied_block(self) -> Option<AppliedBlock> {
        match self {
            Self::FetchPending { .. }
            | Self::FetchSuccess { .. }
            | Self::ApplyPending { .. }
            | Self::ApplyError { .. } => None,
            Self::ApplySuccess { block, .. } => Some(block),
        }
    }

    pub fn fetch_pending_hash(&self) -> Option<&StateHash> {
        match self {
            Self::FetchPending { block_hash, .. } => Some(block_hash),
            _ => None,
        }
    }

    pub fn retry_hash(&self) -> Option<&StateHash> {
        let Self::FetchPending {
            block_hash,
            attempts,
            ..
        } = self
        else {
            return None;
        };
        Some(block_hash)
            .filter(|_| !attempts.is_empty() && attempts.iter().all(|(_, s)| s.is_error()))
    }

    pub fn fetch_pending_from_peer_rpc_id(&self, peer_id: &PeerId) -> Option<P2pRpcId> {
        let Self::FetchPending { attempts, .. } = self else {
            return None;
        };
        attempts.get(peer_id).and_then(|v| v.fetch_pending_rpc_id())
    }

    pub fn is_fetch_init_from_peer(&self, peer_id: &PeerId) -> bool {
        let Self::FetchPending { attempts, .. } = self else {
            return false;
        };
        attempts.get(peer_id).map_or(false, |s| s.is_fetch_init())
    }

    pub fn is_fetch_pending_from_peer(&self, peer_id: &PeerId, rpc_id: P2pRpcId) -> bool {
        let Self::FetchPending { attempts, .. } = self else {
            return false;
        };
        attempts
            .get(peer_id)
            .and_then(|s| s.fetch_pending_rpc_id())
            .map_or(false, |expected| expected == rpc_id)
    }

    pub fn fetch_pending_attempts_mut(&mut self) -> Option<&mut BTreeMap<PeerId, PeerRpcState>> {
        match self {
            Self::FetchPending { attempts, .. } => Some(attempts),
            _ => None,
        }
    }

    pub fn fetch_pending_from_peer_mut(&mut self, peer_id: &PeerId) -> Option<&mut PeerRpcState> {
        let Self::FetchPending { attempts, .. } = self else {
            return None;
        };
        attempts.get_mut(peer_id)
    }

    pub fn fetch_pending_fetched_block(&self) -> Option<&ArcBlockWithHash> {
        let Self::FetchPending { attempts, .. } = self else {
            return None;
        };
        attempts.iter().find_map(|(_, s)| s.success_block())
    }
}

impl PeerRpcState {
    pub fn is_fetch_init(&self) -> bool {
        matches!(self, Self::Init { .. })
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    pub fn fetch_pending_rpc_id(&self) -> Option<P2pRpcId> {
        match self {
            Self::Pending { rpc_id, .. } => Some(*rpc_id),
            _ => None,
        }
    }

    pub fn fetch_pending_since(&self) -> Option<Timestamp> {
        match self {
            Self::Pending { time, .. } => Some(*time),
            _ => None,
        }
    }

    pub fn success_block(&self) -> Option<&ArcBlockWithHash> {
        match self {
            Self::Success { block, .. } => Some(block),
            _ => None,
        }
    }
}

impl TransitionFrontierRootSnarkedLedgerUpdates {
    pub fn get(
        &self,
        ledger_hash: &LedgerHash,
    ) -> Option<&TransitionFrontierRootSnarkedLedgerUpdate> {
        self.0.get(ledger_hash)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Caller must make sure `new_root` is part of `old_chain`.
    pub fn extend_with_needed<'a>(
        &mut self,
        new_root: &ArcBlockWithHash,
        old_chain: impl 'a + IntoIterator<Item = &'a ArcBlockWithHash>,
    ) {
        let mut old_chain = old_chain.into_iter().peekable();
        let Some(old_root) = old_chain.peek() else {
            return;
        };

        let Some(diff_len) = new_root.height().checked_sub(old_root.height()) else {
            return;
        };

        if new_root.snarked_ledger_hash() == old_root.snarked_ledger_hash() {
            return;
        }

        self.0.extend(
            old_chain
                .take(diff_len as usize)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .scan(new_root, |last_block, b| {
                    if last_block.snarked_ledger_hash() == b.snarked_ledger_hash() {
                        *last_block = b;
                        return Some(None);
                    }
                    let last_block = std::mem::replace(last_block, b);
                    let update = TransitionFrontierRootSnarkedLedgerUpdate {
                        parent: b.snarked_ledger_hash().clone(),
                        staged_ledger_hash: last_block.staged_ledger_hashes().clone(),
                    };
                    let snarked_ledger_hash = last_block.snarked_ledger_hash().clone();

                    Some(Some((snarked_ledger_hash, update)))
                })
                .flatten(),
        );
    }
}
