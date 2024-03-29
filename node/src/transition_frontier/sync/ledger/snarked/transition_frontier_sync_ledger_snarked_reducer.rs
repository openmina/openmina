use std::iter;

use crate::ledger::{ledger_empty_hash_at_depth, tree_height_for_num_accounts, LEDGER_DEPTH};

use super::{
    LedgerAddressQuery, LedgerAddressQueryPending, PeerRpcState,
    TransitionFrontierSyncLedgerSnarkedAction,
    TransitionFrontierSyncLedgerSnarkedActionWithMetaRef, TransitionFrontierSyncLedgerSnarkedState,
};

impl TransitionFrontierSyncLedgerSnarkedState {
    pub fn reducer(&mut self, action: TransitionFrontierSyncLedgerSnarkedActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            TransitionFrontierSyncLedgerSnarkedAction::Pending => {
                // handled in parent reducer.
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeersQuery => {}

            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsInit { peer_id } => {
                if let Self::NumAccountsPending {
                    pending_num_accounts,
                    ..
                } = self
                {
                    pending_num_accounts
                        .attempts
                        .insert(*peer_id, PeerRpcState::Init { time: meta.time() });
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsPending {
                peer_id,
                rpc_id,
            } => {
                let Self::NumAccountsPending {
                    pending_num_accounts,
                    ..
                } = self
                else {
                    return;
                };

                let Some(rpc_state) = pending_num_accounts.attempts.get_mut(peer_id) else {
                    return;
                };

                *rpc_state = PeerRpcState::Pending {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                };
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsRetry { peer_id } => {
                if let Self::NumAccountsPending {
                    pending_num_accounts,
                    ..
                } = self
                {
                    pending_num_accounts
                        .attempts
                        .insert(*peer_id, PeerRpcState::Init { time: meta.time() });
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsError {
                peer_id,
                rpc_id,
                error,
            } => {
                let Some(rpc_state) = self.peer_num_account_query_state_get_mut(peer_id, *rpc_id)
                else {
                    return;
                };

                *rpc_state = PeerRpcState::Error {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                    error: error.clone(),
                };
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsSuccess {
                peer_id,
                rpc_id,
                ..
            } => {
                let Some(rpc_state) = self.peer_num_account_query_state_get_mut(peer_id, *rpc_id)
                else {
                    return;
                };
                *rpc_state = PeerRpcState::Success {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                };
            }
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsReceived { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsAccepted { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsRejected { .. } => {
                // TODO(tizoc): should this be reflected in the state somehow?
            }
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsSuccess {
                num_accounts,
                contents_hash,
            } => {
                let Self::NumAccountsPending { target, .. } = self else {
                    return;
                };

                let target = target.clone();

                *self = Self::NumAccountsSuccess {
                    time: meta.time(),
                    target,
                    num_accounts: *num_accounts,
                    contents_hash: contents_hash.clone(),
                };
            }

            TransitionFrontierSyncLedgerSnarkedAction::MerkleTreeSyncPending => {
                let Self::NumAccountsSuccess {
                    target,
                    num_accounts,
                    contents_hash,
                    ..
                } = self
                else {
                    return;
                };

                // We know at which node to begin querying, so we skip all the intermediary depths
                let first_query = LedgerAddressQuery {
                    address: ledger::Address::first(
                        LEDGER_DEPTH - tree_height_for_num_accounts(*num_accounts),
                    ),
                    expected_hash: contents_hash.clone(),
                };

                *self = Self::MerkleTreeSyncPending {
                    time: meta.time(),
                    target: target.clone(),
                    total_accounts_expected: *num_accounts,
                    synced_accounts_count: 0,
                    synced_hashes_count: 0,
                    queue: iter::once(first_query).collect(),
                    pending_addresses: Default::default(),
                };
            }
            TransitionFrontierSyncLedgerSnarkedAction::MerkleTreeSyncSuccess => {
                let Self::MerkleTreeSyncPending { target, .. } = self else {
                    return;
                };
                *self = Self::MerkleTreeSyncSuccess {
                    time: meta.time(),
                    target: target.clone(),
                };
            }

            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressInit {
                address,
                expected_hash,
                peer_id,
            } => {
                if let Self::MerkleTreeSyncPending {
                    queue,
                    pending_addresses: pending,
                    ..
                } = self
                {
                    let _next = queue.pop_front();
                    //debug_assert_eq!(next.as_ref().map(|p| &p.0), Some(address));

                    pending.insert(
                        address.clone(),
                        LedgerAddressQueryPending {
                            time: meta.time(),
                            expected_hash: expected_hash.clone(),
                            attempts: std::iter::once((
                                *peer_id,
                                PeerRpcState::Init { time: meta.time() },
                            ))
                            .collect(),
                        },
                    );
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressRetry {
                address,
                peer_id,
            } => {
                if let Self::MerkleTreeSyncPending {
                    pending_addresses: pending,
                    ..
                } = self
                {
                    if let Some(pending) = pending.get_mut(address) {
                        pending
                            .attempts
                            .insert(*peer_id, PeerRpcState::Init { time: meta.time() });
                    }
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressPending {
                address,
                peer_id,
                rpc_id,
            } => {
                let Self::MerkleTreeSyncPending {
                    pending_addresses: pending,
                    ..
                } = self
                else {
                    return;
                };
                let Some(rpc_state) = pending
                    .get_mut(address)
                    .and_then(|s| s.attempts.get_mut(peer_id))
                else {
                    return;
                };

                *rpc_state = PeerRpcState::Pending {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                };
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressError {
                peer_id,
                rpc_id,
                error,
            } => {
                let Some(rpc_state) = self.peer_address_query_state_get_mut(peer_id, *rpc_id)
                else {
                    return;
                };

                *rpc_state = PeerRpcState::Error {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                    error: error.clone(),
                };
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressSuccess {
                peer_id,
                rpc_id,
                ..
            } => {
                let Some(rpc_state) = self.peer_address_query_state_get_mut(peer_id, *rpc_id)
                else {
                    return;
                };
                *rpc_state = PeerRpcState::Success {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                };
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesReceived { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesAccepted {
                address,
                hashes,
                previous_hashes,
                ..
            } => {
                let Self::MerkleTreeSyncPending {
                    queue,
                    pending_addresses: pending,
                    synced_hashes_count: num_hashes_accepted,
                    ..
                } = self
                else {
                    return;
                };

                // Once hashes are accepted, we can consider this query fulfilled
                pending.remove(address);

                let (left, right) = hashes;
                let (previous_left, previous_right) = previous_hashes;

                // TODO(tizoc): for non-stale hashes, we can consider the full subtree
                // as accepted. Given the value of `num_accounts` and the position
                // in the tree we could estimate how many accounts and hashes
                // from that subtree will be skipped and add them to the count.

                // Empty node hashes are not counted in the stats.
                let empty = ledger_empty_hash_at_depth(address.length() + 1);
                *num_hashes_accepted += (*left != empty) as u64 + (*right != empty) as u64;

                if left != previous_left {
                    queue.push_back(LedgerAddressQuery {
                        address: address.child_left(),
                        expected_hash: left.clone(),
                    });
                }
                if right != previous_right {
                    queue.push_back(LedgerAddressQuery {
                        address: address.child_right(),
                        expected_hash: right.clone(),
                    });
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesRejected { .. } => {
                // TODO(tizoc): should this be reflected in the state somehow?
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsReceived { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsAccepted {
                address,
                count,
                ..
            } => {
                let Self::MerkleTreeSyncPending {
                    pending_addresses: pending,
                    synced_accounts_count,
                    ..
                } = self
                else {
                    return;
                };

                *synced_accounts_count += count;
                pending.remove(address);
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsRejected { .. } => {
                // TODO(tizoc): should this be reflected in the state somehow?
            }
            TransitionFrontierSyncLedgerSnarkedAction::Success => {
                let Self::MerkleTreeSyncPending { target, .. } = self else {
                    return;
                };
                *self = Self::Success {
                    time: meta.time(),
                    target: target.clone(),
                };
            }
        }
    }
}
