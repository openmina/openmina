use std::iter;

use mina_p2p_messages::v2::MinaLedgerSyncLedgerQueryStableV1;
use p2p::{
    channels::rpc::{P2pChannelsRpcAction, P2pRpcId, P2pRpcRequest},
    disconnection::{P2pDisconnectionAction, P2pDisconnectionReason},
    PeerId,
};

use crate::{
    ledger::{
        ledger_empty_hash_at_depth, tree_height_for_num_accounts, LedgerAddress, LEDGER_DEPTH,
    },
    Action, State,
};

use super::{
    LedgerAddressQueryPending, PeerLedgerQueryResponse, PeerRpcState,
    TransitionFrontierSyncLedgerSnarkedAction,
    TransitionFrontierSyncLedgerSnarkedActionWithMetaRef, TransitionFrontierSyncLedgerSnarkedState,
    ACCOUNT_SUBTREE_HEIGHT,
};

impl TransitionFrontierSyncLedgerSnarkedState {
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: TransitionFrontierSyncLedgerSnarkedActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else {
            // TODO: log or propagate
            return;
        };
        let (action, meta) = action.split();

        match action {
            TransitionFrontierSyncLedgerSnarkedAction::Pending => {
                // handled in parent reducer. TODO(refactor): should have a callback instead?

                // Dispatch
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeersQuery => {
                let mut retry_addresses: Vec<_> = state.sync_address_retry_iter().collect();
                let mut addresses: Vec<_> = state.sync_address_query_iter().collect();

                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();

                // TODO(binier): make sure they have the ledger we want to query.
                let mut peer_ids = global_state
                    .p2p
                    .ready_peers_iter()
                    .filter(|(_, p)| p.channels.rpc.can_send_request())
                    .map(|(id, p)| (*id, p.connected_since))
                    .collect::<Vec<_>>();
                peer_ids.sort_by(|(_, t1), (_, t2)| t2.cmp(t1));

                // If this dispatches, we can avoid even trying the following steps because we will
                // not query address unless we have completed the Num_accounts request first.
                if let Some((peer_id, _)) = peer_ids.first() {
                    if dispatcher.push_if_enabled(
                        TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsInit {
                            peer_id: *peer_id,
                        },
                        global_state,
                        meta.time(),
                    ) || dispatcher.push_if_enabled(
                        TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsRetry {
                            peer_id: *peer_id,
                        },
                        global_state,
                        meta.time(),
                    ) {
                        return;
                    }
                }

                for (peer_id, _) in peer_ids {
                    if let Some(address) = retry_addresses.last() {
                        if dispatcher.push_if_enabled(
                            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressRetry {
                                peer_id,
                                address: address.clone(),
                            },
                            global_state,
                            meta.time(),
                        ) {
                            retry_addresses.pop();
                            continue;
                        }
                    }

                    match addresses.pop() {
                        Some((address, expected_hash)) => {
                            dispatcher.push(
                                TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressInit {
                                    peer_id,
                                    expected_hash,
                                    address,
                                },
                            );
                        }
                        None if retry_addresses.is_empty() => break,
                        None => {}
                    }
                }
            }

            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsInit { peer_id } => {
                if let Self::NumAccountsPending {
                    pending_num_accounts,
                    ..
                } = state
                {
                    pending_num_accounts
                        .attempts
                        .insert(*peer_id, PeerRpcState::Init { time: meta.time() });
                }

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                peer_query_num_accounts_init(dispatcher, global_state, *peer_id)
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsPending {
                peer_id,
                rpc_id,
            } => {
                let Self::NumAccountsPending {
                    pending_num_accounts,
                    ..
                } = state
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
                } = state
                {
                    pending_num_accounts
                        .attempts
                        .insert(*peer_id, PeerRpcState::Init { time: meta.time() });
                }

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                peer_query_num_accounts_init(dispatcher, global_state, *peer_id)
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsError {
                peer_id,
                rpc_id,
                error,
            } => {
                let Some(rpc_state) = state.peer_num_account_query_state_get_mut(peer_id, *rpc_id)
                else {
                    return;
                };

                *rpc_state = PeerRpcState::Error {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                    error: error.clone(),
                };

                // Dispatch
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsSuccess {
                peer_id,
                rpc_id,
                response,
            } => {
                let Some(rpc_state) = state.peer_num_account_query_state_get_mut(peer_id, *rpc_id)
                else {
                    return;
                };
                *rpc_state = PeerRpcState::Success {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                };

                // Dispatch
                let dispatcher = state_context.into_dispatcher();

                match response {
                    PeerLedgerQueryResponse::NumAccounts(count, contents_hash) => {
                        dispatcher.push(
                            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsReceived {
                                num_accounts: *count,
                                contents_hash: contents_hash.clone(),
                                sender: *peer_id,
                            },
                        );
                    }
                    // TODO(tizoc): These shouldn't happen, log some warning or something
                    PeerLedgerQueryResponse::ChildHashes(_, _) => {}
                    PeerLedgerQueryResponse::ChildAccounts(_) => {}
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsReceived {
                num_accounts,
                contents_hash,
                sender,
            } => {
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();

                let Some(snarked_ledger_hash) = None.or_else(|| {
                    let snarked_ledger =
                        global_state.transition_frontier.sync.ledger()?.snarked()?;
                    Some(snarked_ledger.ledger_hash().clone())
                }) else {
                    return;
                };

                // Given the claimed number of accounts we can figure out the height of the subtree,
                // and compute the root hash assuming all other nodes contain empty hashes.
                // The result must match the snarked ledger hash for this response to be considered
                // valid.
                // NOTE: incorrect account numbers may be accepted (if they fall in the same range)
                // because what is actually being validated is the content hash and tree height,
                // not the actual number of accounts.
                let Ok(actual_hash) = crate::ledger::complete_num_accounts_tree_with_empties(
                    contents_hash,
                    *num_accounts,
                ) else {
                    dispatcher.push(
                        TransitionFrontierSyncLedgerSnarkedAction::NumAccountsRejected {
                            num_accounts: *num_accounts,
                            sender: *sender,
                        },
                    );
                    return;
                };

                if snarked_ledger_hash == actual_hash {
                    dispatcher.push(
                        TransitionFrontierSyncLedgerSnarkedAction::NumAccountsAccepted {
                            num_accounts: *num_accounts,
                            contents_hash: contents_hash.clone(),
                            sender: *sender,
                        },
                    );
                } else {
                    dispatcher.push(
                        TransitionFrontierSyncLedgerSnarkedAction::NumAccountsRejected {
                            num_accounts: *num_accounts,
                            sender: *sender,
                        },
                    );
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsAccepted {
                num_accounts,
                contents_hash,
                ..
            } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(
                    TransitionFrontierSyncLedgerSnarkedAction::NumAccountsSuccess {
                        num_accounts: *num_accounts,
                        contents_hash: contents_hash.clone(),
                    },
                );
            }
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsRejected { sender, .. } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(
                    P2pDisconnectionAction::Init { peer_id: *sender, reason: P2pDisconnectionReason::TransitionFrontierSyncLedgerSnarkedNumAccountsRejected }
                );
                dispatcher.push(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
            }
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsSuccess {
                num_accounts,
                contents_hash,
            } => {
                let Self::NumAccountsPending { target, .. } = state else {
                    return;
                };

                let target = target.clone();

                *state = Self::NumAccountsSuccess {
                    time: meta.time(),
                    target,
                    num_accounts: *num_accounts,
                    contents_hash: contents_hash.clone(),
                };

                // Dispatch
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransitionFrontierSyncLedgerSnarkedAction::MerkleTreeSyncPending);
            }

            TransitionFrontierSyncLedgerSnarkedAction::MerkleTreeSyncPending => {
                let Self::NumAccountsSuccess {
                    target,
                    num_accounts,
                    contents_hash,
                    ..
                } = state
                else {
                    return;
                };

                // We know at which node to begin querying, so we skip all the intermediary depths
                let first_node_address = ledger::Address::first(
                    LEDGER_DEPTH - tree_height_for_num_accounts(*num_accounts),
                );
                let expected_hash = contents_hash.clone();
                let first_query = (first_node_address, expected_hash);

                *state = Self::MerkleTreeSyncPending {
                    time: meta.time(),
                    target: target.clone(),
                    total_accounts_expected: *num_accounts,
                    synced_accounts_count: 0,
                    synced_hashes_count: 0,
                    queue: iter::once(first_query).collect(),
                    pending_addresses: Default::default(),
                };

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                if !dispatcher.push_if_enabled(
                    TransitionFrontierSyncLedgerSnarkedAction::PeersQuery,
                    global_state,
                    meta.time(),
                ) {
                    dispatcher
                        .push(TransitionFrontierSyncLedgerSnarkedAction::MerkleTreeSyncSuccess);
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::MerkleTreeSyncSuccess => {
                let Self::MerkleTreeSyncPending { target, .. } = state else {
                    return;
                };
                *state = Self::MerkleTreeSyncSuccess {
                    time: meta.time(),
                    target: target.clone(),
                };

                // Dispatch
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransitionFrontierSyncLedgerSnarkedAction::Success);
            }

            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressInit {
                address,
                expected_hash,
                peer_id,
            } => {
                if let Self::MerkleTreeSyncPending {
                    queue,
                    pending_addresses,
                    ..
                } = state
                {
                    let removed = queue.remove(address);
                    debug_assert!(removed.is_some());

                    pending_addresses.insert(
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

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                peer_query_address_init(dispatcher, global_state, *peer_id, address.clone());
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressRetry {
                address,
                peer_id,
            } => {
                if let Self::MerkleTreeSyncPending {
                    pending_addresses: pending,
                    ..
                } = state
                {
                    if let Some(pending) = pending.get_mut(address) {
                        pending
                            .attempts
                            .insert(*peer_id, PeerRpcState::Init { time: meta.time() });
                    }
                }

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                peer_query_address_init(dispatcher, global_state, *peer_id, address.clone());
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressPending {
                address,
                peer_id,
                rpc_id,
            } => {
                let Self::MerkleTreeSyncPending {
                    pending_addresses: pending,
                    ..
                } = state
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
                let Some(rpc_state) = state.peer_address_query_state_get_mut(peer_id, *rpc_id)
                else {
                    return;
                };

                *rpc_state = PeerRpcState::Error {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                    error: error.clone(),
                };

                // Dispatch
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressSuccess {
                peer_id,
                rpc_id,
                response,
            } => {
                let Some(rpc_state) = state.peer_address_query_state_get_mut(peer_id, *rpc_id)
                else {
                    return;
                };
                *rpc_state = PeerRpcState::Success {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                };

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let ledger = global_state.transition_frontier.sync.ledger();
                let Some(address) = ledger
                    .and_then(|s| s.snarked()?.peer_address_query_get(peer_id, *rpc_id))
                    .map(|(addr, _)| addr.clone())
                else {
                    return;
                };

                match response {
                    PeerLedgerQueryResponse::ChildHashes(left, right) => {
                        dispatcher.push(
                            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesReceived {
                                address,
                                hashes: (left.clone(), right.clone()),
                                sender: *peer_id,
                            },
                        );
                    }
                    PeerLedgerQueryResponse::ChildAccounts(accounts) => {
                        dispatcher.push(
                            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsReceived {
                                address,
                                accounts: accounts.clone(),
                                sender: *peer_id,
                            },
                        );
                    }
                    // TODO(tizoc): This shouldn't happen, log some warning or something
                    PeerLedgerQueryResponse::NumAccounts(_, _) => {}
                }
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
                } = state
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
                    let previous = queue.insert(address.child_left(), left.clone());
                    debug_assert!(previous.is_none());
                }
                if right != previous_right {
                    let previous = queue.insert(address.child_right(), right.clone());
                    debug_assert!(previous.is_none());
                }

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                if !dispatcher.push_if_enabled(
                    TransitionFrontierSyncLedgerSnarkedAction::PeersQuery,
                    global_state,
                    meta.time(),
                ) {
                    dispatcher
                        .push(TransitionFrontierSyncLedgerSnarkedAction::MerkleTreeSyncSuccess);
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesRejected { .. } => {
                // TODO(tizoc): should this be reflected in the state somehow?
                // TODO(tizoc): we do nothing here, but the peer must be punished somehow
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
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
                } = state
                else {
                    return;
                };

                *synced_accounts_count += count;
                pending.remove(address);

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                if !dispatcher.push_if_enabled(
                    TransitionFrontierSyncLedgerSnarkedAction::PeersQuery,
                    global_state,
                    meta.time(),
                ) {
                    dispatcher
                        .push(TransitionFrontierSyncLedgerSnarkedAction::MerkleTreeSyncSuccess);
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsRejected { .. } => {
                // TODO(tizoc): should this be reflected in the state somehow?
                // TODO(tizoc): we do nothing here, but the peer must be punished somehow
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
            }
            TransitionFrontierSyncLedgerSnarkedAction::Success => {
                let Self::MerkleTreeSyncSuccess { target, .. } = state else {
                    return;
                };
                *state = Self::Success {
                    time: meta.time(),
                    target: target.clone(),
                };
            }
        }
    }
}

fn peer_query_num_accounts_init(
    dispatcher: &mut redux::Dispatcher<Action, State>,
    state: &State,
    peer_id: PeerId,
) {
    let Some((ledger_hash, rpc_id)) = None.or_else(|| {
        let ledger = state.transition_frontier.sync.ledger()?;
        let ledger_hash = ledger.snarked()?.ledger_hash();

        let p = state.p2p.get_ready_peer(&peer_id)?;
        let rpc_id = p.channels.next_local_rpc_id();

        Some((ledger_hash.clone(), rpc_id))
    }) else {
        return;
    };

    dispatcher.push(P2pChannelsRpcAction::RequestSend {
        peer_id,
        id: rpc_id,
        request: Box::new(P2pRpcRequest::LedgerQuery(
            ledger_hash,
            MinaLedgerSyncLedgerQueryStableV1::NumAccounts,
        )),
        on_init: Some(redux::callback!(
            on_send_p2p_num_accounts_rpc_request(
                (peer_id: PeerId, rpc_id: P2pRpcId, _request: P2pRpcRequest)
            ) -> crate::Action {
                TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsPending {
                    peer_id,
                    rpc_id,
                }
            }
        )),
    });
}

fn peer_query_address_init(
    dispatcher: &mut redux::Dispatcher<Action, State>,
    state: &State,
    peer_id: PeerId,
    address: LedgerAddress,
) {
    let Some((ledger_hash, rpc_id)) = None.or_else(|| {
        let ledger = state.transition_frontier.sync.ledger()?;
        let ledger_hash = ledger.snarked()?.ledger_hash();

        let p = state.p2p.get_ready_peer(&peer_id)?;
        let rpc_id = p.channels.next_local_rpc_id();

        Some((ledger_hash.clone(), rpc_id))
    }) else {
        return;
    };

    let query = if address.length() >= LEDGER_DEPTH - ACCOUNT_SUBTREE_HEIGHT {
        MinaLedgerSyncLedgerQueryStableV1::WhatContents(address.clone().into())
    } else {
        MinaLedgerSyncLedgerQueryStableV1::WhatChildHashes(address.clone().into())
    };

    dispatcher.push(P2pChannelsRpcAction::RequestSend {
        peer_id,
        id: rpc_id,
        request: Box::new(P2pRpcRequest::LedgerQuery(ledger_hash, query)),
        on_init: Some(redux::callback!(
            on_send_p2p_query_address_rpc_request(
                (peer_id: PeerId, rpc_id: P2pRpcId, request: P2pRpcRequest)
            ) -> crate::Action {
                let P2pRpcRequest::LedgerQuery(_, query) = request else {
                    unreachable!()
                };
                let address = match query {
                    MinaLedgerSyncLedgerQueryStableV1::WhatChildHashes(address) => address.into(),
                    MinaLedgerSyncLedgerQueryStableV1::WhatContents(address) => address.into(),
                    MinaLedgerSyncLedgerQueryStableV1::NumAccounts => unreachable!(),
                };

                TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressPending {
                    peer_id,
                    rpc_id,
                    address,
                }
            }
        )),
    });
}
