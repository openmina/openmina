use std::collections::BTreeMap;

use mina_p2p_messages::v2::LedgerHash;
use redux::Timestamp;
use serde::{Deserialize, Serialize};
use shared::block::ArcBlockWithHash;

use crate::ledger::LedgerAddress;
use crate::p2p::channels::rpc::P2pRpcId;
use crate::p2p::PeerId;

use super::PeerLedgerQueryError;

static SYNC_PENDING_EMPTY: BTreeMap<LedgerAddress, LedgerQueryPending> = BTreeMap::new();

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncLedgerSnarkedState {
    /// Doing BFS to sync snarked ledger tree.
    Pending {
        time: Timestamp,
        block: ArcBlockWithHash,
        pending: BTreeMap<LedgerAddress, LedgerQueryPending>,
        /// `None` means we are done.
        next_addr: Option<LedgerAddress>,
        end_addr: LedgerAddress,
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

impl TransitionFrontierSyncLedgerSnarkedState {
    pub fn pending(time: Timestamp, block: ArcBlockWithHash) -> Self {
        Self::Pending {
            time,
            block,
            pending: Default::default(),
            next_addr: Some(LedgerAddress::root()),
            end_addr: LedgerAddress::root(),
        }
    }

    pub fn block(&self) -> &ArcBlockWithHash {
        match self {
            Self::Pending { block, .. } => block,
            Self::Success { block, .. } => block,
        }
    }

    pub fn ledger_hash(&self) -> &LedgerHash {
        self.block().snarked_ledger_hash()
    }

    pub fn fetch_pending(&self) -> Option<&BTreeMap<LedgerAddress, LedgerQueryPending>> {
        match self {
            Self::Pending { pending, .. } => Some(pending),
            _ => None,
        }
    }

    pub fn sync_retry_iter(&self) -> impl '_ + Iterator<Item = LedgerAddress> {
        let pending = match self {
            Self::Pending { pending, .. } => pending,
            _ => &SYNC_PENDING_EMPTY,
        };
        pending
            .iter()
            .filter(|(_, s)| s.attempts.values().all(|s| s.is_error()))
            .map(|(addr, _)| addr.clone())
    }

    pub fn sync_next(&self) -> Option<LedgerAddress> {
        match self {
            Self::Pending { next_addr, .. } => next_addr.clone(),
            _ => None,
        }
    }

    pub fn peer_query_get(
        &self,
        peer_id: &PeerId,
        rpc_id: P2pRpcId,
    ) -> Option<(&LedgerAddress, &LedgerQueryPending)> {
        match self {
            Self::Pending { pending, .. } => {
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

    pub fn peer_query_get_mut(
        &mut self,
        peer_id: &PeerId,
        rpc_id: P2pRpcId,
    ) -> Option<&mut PeerRpcState> {
        match self {
            Self::Pending { pending, .. } => {
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

    pub fn peer_query_pending_rpc_ids<'a>(
        &'a self,
        peer_id: &'a PeerId,
    ) -> impl 'a + Iterator<Item = P2pRpcId> {
        let pending = match self {
            Self::Pending { pending, .. } => pending,
            _ => &SYNC_PENDING_EMPTY,
        };
        pending.values().filter_map(move |s| {
            s.attempts
                .iter()
                .find(|(id, _)| *id == peer_id)
                .and_then(|(_, s)| s.pending_rpc_id())
        })
    }
}
