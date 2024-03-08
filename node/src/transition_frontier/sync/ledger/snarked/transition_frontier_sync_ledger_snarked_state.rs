use std::collections::BTreeMap;

use mina_p2p_messages::v2::LedgerHash;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::ledger::LedgerAddress;
use crate::p2p::channels::rpc::P2pRpcId;
use crate::p2p::PeerId;
use crate::rpc::LedgerSyncProgress;
use crate::transition_frontier::sync::ledger::SyncLedgerTarget;

use super::PeerLedgerQueryError;

static SYNC_PENDING_EMPTY: BTreeMap<LedgerAddress, LedgerQueryPending> = BTreeMap::new();

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncLedgerSnarkedState {
    /// Doing BFS to sync snarked ledger tree.
    Pending {
        time: Timestamp,
        target: SyncLedgerTarget,
        pending: BTreeMap<LedgerAddress, LedgerQueryPending>,
        /// `None` means we are done.
        next_addr: Option<LedgerAddress>,
        end_addr: LedgerAddress,
    },
    Success {
        time: Timestamp,
        target: SyncLedgerTarget,
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
    pub fn pending(time: Timestamp, target: SyncLedgerTarget) -> Self {
        Self::Pending {
            time,
            target,
            pending: Default::default(),
            next_addr: Some(LedgerAddress::root()),
            end_addr: LedgerAddress::root(),
        }
    }

    pub fn target(&self) -> &SyncLedgerTarget {
        match self {
            Self::Pending { target, .. } => target,
            Self::Success { target, .. } => target,
        }
    }

    pub fn ledger_hash(&self) -> &LedgerHash {
        &self.target().snarked_ledger_hash
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

    pub fn estimation(&self) -> Option<LedgerSyncProgress> {
        const BITS: usize = 35;

        let Self::Pending {
            next_addr,
            end_addr,
            ..
        } = self
        else {
            return None;
        };

        let next_addr = next_addr.as_ref()?;

        let current_length = next_addr.length();

        // The ledger is a binary tree, it synchronizes layer by layer, the next layer is at most
        // twice as big as this layer, but can be smaller (by one). For simplicity, let's call
        // a branch or a leaf of the tree a tree element (or an element) and make no distinction.
        // This doesn't matter for the estimation. Total 35 layers (0, 1, 2, ..., 34). On the last
        // layer there could be 2 ^ 34 items. Of course it is much less than that. So the first
        // few layers contain only 1 element.

        // When the sync algorithm asks childs on the item, it gets two values, left and right.
        // The algorithm asks childs only on the existing item, so the left child must exist.
        // But the right child can be missing. In this case it is marked by the special constant.
        // If the sync algorithm encounters a non-existent right child, it sets `end_addr`
        // to the address of the left sibling (the last existing element of this layer).

        // The `end_addr` is initialized with layer is zero and position is zero (root).

        // Let it be the first non-existent (right child).
        // In extereme case it will be right sibling of root, so layer is zero and position is one.
        // Therefore, further `estimated_this_layer` cannot be zero.
        let estimated_end_addr = end_addr.next().unwrap_or(end_addr.clone());

        // The chance of `end_addr` being updated during fetching the layer is 50%, so its length
        // (number of layers) may be less than the current layer. Let's calculate end address
        // at the current layer.
        let estimated_this_layer =
            estimated_end_addr.to_index().0 << (current_length - estimated_end_addr.length());

        // The number of items on the previous layer is twice less than the number of items
        // on this layer, but cannot be 0.
        let prev_layers = (0..current_length)
            .map(|layer| (estimated_this_layer >> (current_length - layer)).max(1))
            .sum::<u64>();

        // Number of layers pending.
        let further_layers_number = BITS - 1 - current_length;
        // Assume the next layer contains twice as many, but it could be twice as many minus one.
        // So the estimate may become smaller.
        let estimated_next_layer = estimated_this_layer * 2;
        // Sum of powers of 2 is power of 2 minus 1
        let estimated_next_layers = ((1 << further_layers_number) - 1) * estimated_next_layer;

        // We have this many elements on this layer. Add one, because address indexes start at 0.
        let this_layer = next_addr.to_index().0 + 1;

        Some(LedgerSyncProgress {
            fetched: prev_layers + this_layer,
            estimation: prev_layers + estimated_this_layer + estimated_next_layers,
        })
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
