use std::collections::BTreeMap;
use std::sync::Arc;

use mina_p2p_messages::v2::LedgerHash;
use redux::Timestamp;
use serde::{Deserialize, Serialize};
use shared::block::ArcBlockWithHash;

use crate::ledger::LedgerAddress;
use crate::p2p::channels::rpc::{P2pRpcId, StagedLedgerAuxAndPendingCoinbases};
use crate::p2p::PeerId;

use super::PeerLedgerQueryError;

static SNARKED_LEDGER_SYNC_PENDING_EMPTY: BTreeMap<LedgerAddress, LedgerQueryPending> =
    BTreeMap::new();

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncLedgerState {
    Init {
        time: Timestamp,
        block: ArcBlockWithHash,
    },
    /// Doing BFS to sync snarked ledger tree.
    SnarkedLedgerSyncPending {
        time: Timestamp,
        block: ArcBlockWithHash,
        pending: BTreeMap<LedgerAddress, LedgerQueryPending>,
        /// `None` means we are done.
        next_addr: Option<LedgerAddress>,
        end_addr: LedgerAddress,
    },
    SnarkedLedgerSyncSuccess {
        time: Timestamp,
        block: ArcBlockWithHash,
    },
    StagedLedgerReconstructPending {
        time: Timestamp,
        block: ArcBlockWithHash,
        attempts: BTreeMap<PeerId, PeerStagedLedgerReconstructState>,
    },
    StagedLedgerReconstructSuccess {
        time: Timestamp,
        block: ArcBlockWithHash,
    },
    Success {
        time: Timestamp,
        block: ArcBlockWithHash,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LedgerQueryPending {
    pub time: Timestamp,
    pub attempts: BTreeMap<PeerId, PeerRpcState>,
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
        error: PeerLedgerQueryError,
    },
    Success {
        time: Timestamp,
        rpc_id: P2pRpcId,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerStagedLedgerReconstructState {
    /// Fetching pieces required to reconstruct staged ledger from
    /// snarked ledger.
    PartsFetchPending {
        time: Timestamp,
        rpc_id: P2pRpcId,
    },
    PartsFetchError {
        time: Timestamp,
        rpc_id: P2pRpcId,
        error: PeerLedgerQueryError,
    },
    /// Fetched pieces required to reconstruct staged ledger from
    /// snarked ledger.
    PartsFetchSuccess {
        time: Timestamp,
        parts: Arc<StagedLedgerAuxAndPendingCoinbases>,
    },
    PartsApplySuccess {
        time: Timestamp,
    },
}

impl PeerStagedLedgerReconstructState {
    pub fn is_fetch_pending(&self) -> bool {
        matches!(self, Self::PartsFetchPending { .. })
    }

    pub fn is_fetch_success(&self) -> bool {
        matches!(self, Self::PartsFetchSuccess { .. })
    }

    pub fn is_apply_success(&self) -> bool {
        matches!(self, Self::PartsApplySuccess { .. })
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::PartsFetchError { .. })
    }

    pub fn fetch_pending_rpc_id(&self) -> Option<P2pRpcId> {
        match self {
            Self::PartsFetchPending { rpc_id, .. } => Some(*rpc_id),
            _ => None,
        }
    }
}

impl PeerRpcState {
    pub fn pending_rpc_id(&self) -> Option<P2pRpcId> {
        match self {
            Self::Pending { rpc_id, .. } => Some(*rpc_id),
            _ => None,
        }
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }
}

impl TransitionFrontierSyncLedgerState {
    pub fn block(&self) -> &ArcBlockWithHash {
        match self {
            Self::Init { block, .. } => block,
            Self::SnarkedLedgerSyncPending { block, .. } => block,
            Self::SnarkedLedgerSyncSuccess { block, .. } => block,
            Self::StagedLedgerReconstructPending { block, .. } => block,
            Self::StagedLedgerReconstructSuccess { block, .. } => block,
            Self::Success { block, .. } => block,
        }
    }

    pub fn snarked_ledger_hash(&self) -> LedgerHash {
        self.block().snarked_ledger_hash()
    }

    pub fn snarked_ledger_sync_retry_iter(&self) -> impl '_ + Iterator<Item = LedgerAddress> {
        let pending = match self {
            Self::SnarkedLedgerSyncPending { pending, .. } => pending,
            _ => &SNARKED_LEDGER_SYNC_PENDING_EMPTY,
        };
        pending
            .iter()
            .filter(|(_, s)| s.attempts.values().all(|s| s.is_error()))
            .map(|(addr, _)| addr.clone())
    }

    pub fn snarked_ledger_sync_next(&self) -> Option<LedgerAddress> {
        match self {
            Self::SnarkedLedgerSyncPending { next_addr, .. } => next_addr.clone(),
            _ => None,
        }
    }

    pub fn snarked_ledger_peer_query_get(
        &self,
        peer_id: &PeerId,
        rpc_id: P2pRpcId,
    ) -> Option<(&LedgerAddress, &LedgerQueryPending)> {
        match self {
            Self::SnarkedLedgerSyncPending { pending, .. } => {
                let expected_rpc_id = rpc_id;
                pending.iter().find(|(_, s)| {
                    s.attempts.get(peer_id).map_or(false, |s| match s {
                        PeerRpcState::Pending { rpc_id, .. } => *rpc_id == expected_rpc_id,
                        PeerRpcState::Error { rpc_id, .. } => *rpc_id == expected_rpc_id,
                        PeerRpcState::Success { rpc_id, .. } => *rpc_id == expected_rpc_id,
                        _ => false,
                    })
                })
            }
            _ => None,
        }
    }

    pub fn snarked_ledger_peer_query_get_mut(
        &mut self,
        peer_id: &PeerId,
        rpc_id: P2pRpcId,
    ) -> Option<&mut PeerRpcState> {
        match self {
            Self::SnarkedLedgerSyncPending { pending, .. } => {
                let expected_rpc_id = rpc_id;
                pending.iter_mut().find_map(|(_, s)| {
                    s.attempts.get_mut(peer_id).filter(|s| match s {
                        PeerRpcState::Pending { rpc_id, .. } => *rpc_id == expected_rpc_id,
                        PeerRpcState::Error { rpc_id, .. } => *rpc_id == expected_rpc_id,
                        PeerRpcState::Success { rpc_id, .. } => *rpc_id == expected_rpc_id,
                        _ => false,
                    })
                })
            }
            _ => None,
        }
    }

    pub fn snarked_ledger_peer_query_pending_rpc_ids<'a>(
        &'a self,
        peer_id: &'a PeerId,
    ) -> impl 'a + Iterator<Item = P2pRpcId> {
        let pending = match self {
            Self::SnarkedLedgerSyncPending { pending, .. } => pending,
            _ => &SNARKED_LEDGER_SYNC_PENDING_EMPTY,
        };
        pending.values().filter_map(move |s| {
            s.attempts
                .iter()
                .find(|(id, _)| *id == peer_id)
                .and_then(|(_, s)| s.pending_rpc_id())
        })
    }

    pub fn staged_ledger_reconstruct_filter_available_peers<'a>(
        &'a self,
        iter: impl 'a + Iterator<Item = (PeerId, P2pRpcId)>,
    ) -> impl 'a + Iterator<Item = (PeerId, P2pRpcId)> {
        iter.filter(move |(peer_id, _)| match self {
            Self::SnarkedLedgerSyncSuccess { .. } => true,
            Self::StagedLedgerReconstructPending { attempts, .. } => {
                !attempts.contains_key(&peer_id)
                    && (attempts.is_empty() || attempts.iter().all(|(_, s)| s.is_error()))
            }
            _ => false,
        })
    }

    pub fn staged_ledger_parts_fetch_rpc_id(&self, peer_id: &PeerId) -> Option<P2pRpcId> {
        match self {
            Self::StagedLedgerReconstructPending { attempts, .. } => {
                attempts.get(peer_id).and_then(|p| p.fetch_pending_rpc_id())
            }
            _ => None,
        }
    }
}
