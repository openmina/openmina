use crate::{
    ledger::{ledger_empty_hash_at_depth, tree_height_for_num_accounts, LEDGER_DEPTH},
    transition_frontier::sync::ledger::snarked::{
        LedgerNumAccountsQueryPending, LedgerQueryQueued,
    },
};

use super::{
    LedgerAddressQueryPending, PeerRpcState, TransitionFrontierSyncLedgerSnarkedAction,
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
                if let Self::Pending {
                    queue,
                    pending_num_accounts,
                    ..
                } = self
                {
                    let next = queue.pop_front();
                    debug_assert!(matches!(next, Some(LedgerQueryQueued::NumAccounts)));

                    *pending_num_accounts = Some(LedgerNumAccountsQueryPending {
                        time: meta.time(),
                        attempts: std::iter::once((
                            *peer_id,
                            PeerRpcState::Init { time: meta.time() },
                        ))
                        .collect(),
                    });
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsPending {
                peer_id,
                rpc_id,
            } => {
                let Self::Pending {
                    pending_num_accounts: Some(pending),
                    ..
                } = self
                else {
                    return;
                };

                let Some(rpc_state) = pending.attempts.get_mut(peer_id) else {
                    return;
                };

                *rpc_state = PeerRpcState::Pending {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                };
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsRetry { peer_id } => {
                if let Self::Pending {
                    pending_num_accounts: Some(pending),
                    ..
                } = self
                {
                    pending
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
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsAccepted {
                num_accounts: accepted_num_accounts,
                contents_hash,
                ..
            } => {
                let Self::Pending {
                    pending_num_accounts,
                    num_accounts,
                    num_accounts_accepted,
                    num_hashes_accepted,
                    queue,
                    ..
                } = self
                else {
                    return;
                };

                *num_accounts = *accepted_num_accounts;
                *num_accounts_accepted = 0;
                *num_hashes_accepted = 0;
                *pending_num_accounts = None;

                // We know at which node to begin querying, so we skip all the intermediary depths
                queue.push_back(LedgerQueryQueued::Address {
                    address: ledger::Address::first(
                        LEDGER_DEPTH - tree_height_for_num_accounts(*num_accounts),
                    ),
                    expected_hash: contents_hash.clone(),
                });
            }
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsRejected { .. } => {
                // TODO(tizoc): should this be reflected in the state somehow?
            }

            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressInit {
                address,
                expected_hash,
                peer_id,
            } => {
                if let Self::Pending {
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
                if let Self::Pending {
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
                let Self::Pending {
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
                let Self::Pending {
                    queue,
                    pending_addresses: pending,
                    num_hashes_accepted,
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
                    queue.push_back(LedgerQueryQueued::Address {
                        address: address.child_left(),
                        expected_hash: left.clone(),
                    });
                }
                if right != previous_right {
                    queue.push_back(LedgerQueryQueued::Address {
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
                let Self::Pending {
                    pending_addresses: pending,
                    num_accounts_accepted,
                    ..
                } = self
                else {
                    return;
                };

                *num_accounts_accepted += count;
                pending.remove(address);
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsRejected { .. } => {
                // TODO(tizoc): should this be reflected in the state somehow?
            }
            TransitionFrontierSyncLedgerSnarkedAction::Success => {
                let Self::Pending { target, .. } = self else {
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
