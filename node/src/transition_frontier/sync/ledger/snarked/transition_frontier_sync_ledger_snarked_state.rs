use std::collections::{BTreeMap, VecDeque};

use mina_p2p_messages::v2::LedgerHash;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::ledger::{tree_height_for_num_accounts, LedgerAddress};
use crate::p2p::channels::rpc::P2pRpcId;
use crate::p2p::PeerId;
use crate::rpc::LedgerSyncProgress;
use crate::transition_frontier::sync::ledger::SyncLedgerTarget;

use super::{PeerLedgerQueryError, ACCOUNT_SUBTREE_HEIGHT};

static SYNC_PENDING_EMPTY: BTreeMap<LedgerAddress, LedgerAddressQueryPending> = BTreeMap::new();

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncLedgerSnarkedState {
    /// Doing BFS to sync snarked ledger tree.
    Pending {
        time: Timestamp,
        target: SyncLedgerTarget,
        /// Number of accounts in this ledger (as claimed by the Num_accounts query result)
        num_accounts: u64,
        /// Number of accounts received and accepted so far
        num_accounts_accepted: u64,
        /// Number of hashes received and accepted so far
        num_hashes_accepted: u64,
        /// Queue of addresses to query and the expected contents hash
        queue: VecDeque<LedgerQueryQueued>,
        /// Pending ongoing address queries and their attempts
        #[serde_as(as = "Vec<(_, _)>")]
        pending_addresses: BTreeMap<LedgerAddress, LedgerAddressQueryPending>,
        /// Pending num account query attempts
        pending_num_accounts: Option<LedgerNumAccountsQueryPending>,
    },
    Success {
        time: Timestamp,
        target: SyncLedgerTarget,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LedgerQueryQueued {
    NumAccounts,
    Address {
        address: LedgerAddress,
        expected_hash: LedgerHash,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LedgerAddressQueryPending {
    pub time: Timestamp,
    pub expected_hash: LedgerHash,
    pub attempts: BTreeMap<PeerId, PeerRpcState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LedgerNumAccountsQueryPending {
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

    pub fn rpc_id(&self) -> Option<P2pRpcId> {
        match self {
            Self::Init { .. } => None,
            Self::Pending { rpc_id, .. } => Some(*rpc_id),
            Self::Error { rpc_id, .. } => Some(*rpc_id),
            Self::Success { rpc_id, .. } => Some(*rpc_id),
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
            num_accounts: 0,
            num_accounts_accepted: 0,
            num_hashes_accepted: 0,
            queue: vec![LedgerQueryQueued::NumAccounts].into(),
            pending_addresses: Default::default(),
            pending_num_accounts: Default::default(),
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

    pub fn is_num_accounts_query_next(&self) -> bool {
        match self {
            Self::Pending { queue, .. } => queue
                .front()
                .map_or(false, |q| matches!(q, LedgerQueryQueued::NumAccounts)),
            _ => false,
        }
    }

    pub fn num_accounts_pending(&self) -> Option<&LedgerNumAccountsQueryPending> {
        match self {
            Self::Pending {
                pending_num_accounts,
                ..
            } => pending_num_accounts.as_ref(),
            _ => None,
        }
    }

    pub fn fetch_pending(&self) -> Option<&BTreeMap<LedgerAddress, LedgerAddressQueryPending>> {
        match self {
            Self::Pending {
                pending_addresses, ..
            } => Some(pending_addresses),
            _ => None,
        }
    }

    pub fn sync_address_retry_iter(&self) -> impl '_ + Iterator<Item = LedgerAddress> {
        let pending = match self {
            Self::Pending {
                pending_addresses, ..
            } => pending_addresses,
            _ => &SYNC_PENDING_EMPTY,
        };
        pending
            .iter()
            .filter(|(_, s)| s.attempts.values().all(|s| s.is_error()))
            .map(|(addr, _)| addr.clone())
    }

    pub fn sync_address_next(&self) -> Option<(LedgerAddress, LedgerHash)> {
        match self {
            Self::Pending { queue, .. } => match queue.front().map(|a| a.clone()) {
                Some(LedgerQueryQueued::Address {
                    address,
                    expected_hash,
                }) => Some((address, expected_hash)),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn estimation(&self) -> Option<LedgerSyncProgress> {
        match self {
            TransitionFrontierSyncLedgerSnarkedState::Pending {
                num_accounts,
                num_accounts_accepted,
                num_hashes_accepted,
                ..
            } if *num_accounts > 0 => {
                // TODO(tizoc): this approximation is very rough, could be improved.
                // Also we count elements to be fetched and not request to be made which
                // would be more accurate (accounts are feched in groups of 64, hashes of 2).
                let tree_height = tree_height_for_num_accounts(*num_accounts);
                let fill_ratio = (*num_accounts as f64) / 2f64.powf(tree_height as f64);
                let num_hashes_estimate = 2u64.pow((tree_height - ACCOUNT_SUBTREE_HEIGHT) as u32);
                let num_hashes_estimate = (num_hashes_estimate as f64 * fill_ratio).ceil() as u64;
                let fetched = *num_accounts_accepted + num_hashes_accepted;
                let estimation = fetched.max(*num_accounts + num_hashes_estimate);

                Some(LedgerSyncProgress {
                    fetched,
                    estimation,
                })
            }
            TransitionFrontierSyncLedgerSnarkedState::Success { .. } => {
                return Some(LedgerSyncProgress {
                    fetched: 1,
                    estimation: 1,
                })
            }
            _ => None,
        }
    }

    pub fn peer_num_account_query_get(
        &self,
        peer_id: &PeerId,
        rpc_id: P2pRpcId,
    ) -> Option<&LedgerNumAccountsQueryPending> {
        match self {
            Self::Pending {
                pending_num_accounts: Some(pending),
                ..
            } => {
                let expected_rpc_id = rpc_id;
                pending.attempts.get(peer_id).and_then(|s| {
                    if s.rpc_id()? == expected_rpc_id {
                        Some(pending)
                    } else {
                        None
                    }
                })
            }
            _ => None,
        }
    }

    pub fn peer_num_account_query_state_get_mut(
        &mut self,
        peer_id: &PeerId,
        rpc_id: P2pRpcId,
    ) -> Option<&mut PeerRpcState> {
        match self {
            Self::Pending {
                pending_num_accounts,
                ..
            } => {
                let expected_rpc_id = rpc_id;
                pending_num_accounts
                    .as_mut()?
                    .attempts
                    .get_mut(peer_id)
                    .filter(|s| match s {
                        PeerRpcState::Pending { rpc_id, .. } => *rpc_id == expected_rpc_id,
                        PeerRpcState::Error { rpc_id, .. } => *rpc_id == expected_rpc_id,
                        PeerRpcState::Success { rpc_id, .. } => *rpc_id == expected_rpc_id,
                        _ => false,
                    })
            }
            _ => None,
        }
    }

    pub fn peer_address_query_get(
        &self,
        peer_id: &PeerId,
        rpc_id: P2pRpcId,
    ) -> Option<(&LedgerAddress, &LedgerAddressQueryPending)> {
        match self {
            Self::Pending {
                pending_addresses, ..
            } => {
                let expected_rpc_id = rpc_id;
                pending_addresses.iter().find(|(_, s)| {
                    s.attempts
                        .get(peer_id)
                        .map_or(false, |s| s.rpc_id() == Some(expected_rpc_id))
                })
            }
            _ => None,
        }
    }

    pub fn peer_address_query_state_get_mut(
        &mut self,
        peer_id: &PeerId,
        rpc_id: P2pRpcId,
    ) -> Option<&mut PeerRpcState> {
        match self {
            Self::Pending {
                pending_addresses: pending,
                ..
            } => {
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

    pub fn peer_address_query_pending_rpc_ids<'a>(
        &'a self,
        peer_id: &'a PeerId,
    ) -> impl 'a + Iterator<Item = P2pRpcId> {
        let pending = match self {
            Self::Pending {
                pending_addresses, ..
            } => pending_addresses,
            _ => &SYNC_PENDING_EMPTY,
        };
        pending.values().filter_map(move |s| {
            s.attempts
                .iter()
                .find(|(id, _)| *id == peer_id)
                .and_then(|(_, s)| s.pending_rpc_id())
        })
    }

    pub fn peer_num_accounts_rpc_id(&self, peer_id: &PeerId) -> Option<P2pRpcId> {
        let pending = match self {
            Self::Pending {
                pending_num_accounts,
                ..
            } => pending_num_accounts.as_ref(),
            _ => None,
        };

        pending?
            .attempts
            .iter()
            .find(|(id, _)| *id == peer_id)
            .and_then(|(_, s)| s.pending_rpc_id())
    }
}
