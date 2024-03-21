use mina_p2p_messages::v2::MinaLedgerSyncLedgerQueryStableV1;
use p2p::channels::rpc::{P2pChannelsRpcAction, P2pRpcRequest};
use p2p::PeerId;
use redux::ActionMeta;

use crate::ledger::{hash_node_at_depth, LedgerAddress, LEDGER_DEPTH};
use crate::Store;

use super::{
    PeerLedgerQueryResponse, TransitionFrontierSyncLedgerSnarkedAction,
    TransitionFrontierSyncLedgerSnarkedService, ACCOUNT_SUBTREE_HEIGHT,
};

fn peer_query_num_accounts_init<S: redux::Service>(store: &mut Store<S>, peer_id: PeerId) {
    let Some((ledger_hash, rpc_id)) = None.or_else(|| {
        let state = store.state();
        let ledger = state.transition_frontier.sync.ledger()?;
        let ledger_hash = ledger.snarked()?.ledger_hash();

        let p = state.p2p.get_ready_peer(&peer_id)?;
        let rpc_id = p.channels.rpc.next_local_rpc_id();

        Some((ledger_hash.clone(), rpc_id))
    }) else {
        return;
    };

    if store.dispatch(P2pChannelsRpcAction::RequestSend {
        peer_id,
        id: rpc_id,
        request: P2pRpcRequest::LedgerQuery(
            ledger_hash,
            MinaLedgerSyncLedgerQueryStableV1::NumAccounts,
        ),
    }) {
        store.dispatch(
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsPending {
                peer_id,
                rpc_id,
            },
        );
    }
}

fn peer_query_address_init<S: redux::Service>(
    store: &mut Store<S>,
    peer_id: PeerId,
    address: LedgerAddress,
) {
    let Some((ledger_hash, rpc_id)) = None.or_else(|| {
        let state = store.state();
        let ledger = state.transition_frontier.sync.ledger()?;
        let ledger_hash = ledger.snarked()?.ledger_hash();

        let p = store.state().p2p.get_ready_peer(&peer_id)?;
        let rpc_id = p.channels.rpc.next_local_rpc_id();

        Some((ledger_hash.clone(), rpc_id))
    }) else {
        return;
    };

    let query = if address.length() >= LEDGER_DEPTH - ACCOUNT_SUBTREE_HEIGHT {
        MinaLedgerSyncLedgerQueryStableV1::WhatContents(address.clone().into())
    } else {
        MinaLedgerSyncLedgerQueryStableV1::WhatChildHashes(address.clone().into())
    };

    if store.dispatch(P2pChannelsRpcAction::RequestSend {
        peer_id,
        id: rpc_id,
        request: P2pRpcRequest::LedgerQuery(ledger_hash, query),
    }) {
        store.dispatch(
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressPending {
                address,
                peer_id,
                rpc_id,
            },
        );
    }
}

impl TransitionFrontierSyncLedgerSnarkedAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>)
    where
        S: TransitionFrontierSyncLedgerSnarkedService,
    {
        match self {
            TransitionFrontierSyncLedgerSnarkedAction::Pending => {
                store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeersQuery => {
                // TODO(binier): make sure they have the ledger we want to query.
                let mut peer_ids = store
                    .state()
                    .p2p
                    .ready_peers_iter()
                    .filter(|(_, p)| p.channels.rpc.can_send_request())
                    .map(|(id, p)| (*id, p.connected_since))
                    .collect::<Vec<_>>();
                peer_ids.sort_by(|(_, t1), (_, t2)| t2.cmp(t1));

                // If this dispatches, we can avoid even trying the following steps because we will
                // not query address unless we have completed the Num_accounts request first.
                if let Some((peer_id, _)) = peer_ids.first() {
                    if store.dispatch(
                        TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsInit {
                            peer_id: *peer_id,
                        },
                    ) || store.dispatch(
                        TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsRetry {
                            peer_id: *peer_id,
                        },
                    ) {
                        return;
                    }
                }

                let mut retry_addresses = store
                    .state()
                    .transition_frontier
                    .sync
                    .ledger()
                    .and_then(|s| s.snarked())
                    .map_or(vec![], |s| s.sync_address_retry_iter().collect());
                retry_addresses.reverse();

                for (peer_id, _) in peer_ids {
                    if let Some(address) = retry_addresses.last() {
                        if store.dispatch(
                            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressRetry {
                                peer_id,
                                address: address.clone(),
                            },
                        ) {
                            retry_addresses.pop();
                            continue;
                        }
                    }

                    let address = store
                        .state()
                        .transition_frontier
                        .sync
                        .ledger()
                        .and_then(|s| s.snarked())
                        .and_then(|s| s.sync_address_next());
                    match address {
                        Some((address, expected_hash)) => {
                            // This dispatch here will pop from the queue and update sync_next
                            store.dispatch(
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
                peer_query_num_accounts_init(store, *peer_id)
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsPending { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsRetry { peer_id } => {
                peer_query_num_accounts_init(store, *peer_id)
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsError { .. } => {
                store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsSuccess {
                peer_id,
                response,
                ..
            } => {
                match response {
                    PeerLedgerQueryResponse::NumAccounts(count, contents_hash) => {
                        store.dispatch(
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
                let Some(snarked_ledger_hash) = None.or_else(|| {
                    let snarked_ledger =
                        store.state().transition_frontier.sync.ledger()?.snarked()?;
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
                let actual_hash = crate::ledger::complete_num_accounts_tree_with_empties(
                    contents_hash,
                    *num_accounts,
                );

                if snarked_ledger_hash == actual_hash {
                    store.dispatch(
                        TransitionFrontierSyncLedgerSnarkedAction::NumAccountsAccepted {
                            num_accounts: *num_accounts,
                            contents_hash: contents_hash.clone(),
                            sender: *sender,
                        },
                    );
                } else {
                    store.dispatch(
                        TransitionFrontierSyncLedgerSnarkedAction::NumAccountsRejected {
                            num_accounts: *num_accounts,
                            sender: *sender,
                        },
                    );
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsAccepted { .. } => {
                if !store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery) {
                    store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::Success);
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsRejected { .. } => {
                // TODO(tizoc): we do nothing here, but the peer must be punished somehow
                store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
            }

            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressInit {
                peer_id,
                address,
                expected_hash: _,
            } => {
                peer_query_address_init(store, *peer_id, address.clone());
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressRetry {
                peer_id,
                address,
            } => {
                peer_query_address_init(store, *peer_id, address.clone());
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressError { .. } => {
                store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressSuccess {
                peer_id,
                rpc_id,
                response,
            } => {
                let ledger = store.state().transition_frontier.sync.ledger();
                let Some(address) = ledger
                    .and_then(|s| s.snarked()?.peer_address_query_get(peer_id, *rpc_id))
                    .map(|(addr, _)| addr.clone())
                else {
                    return;
                };

                match response {
                    PeerLedgerQueryResponse::ChildHashes(left, right) => {
                        store.dispatch(
                            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesReceived {
                                address,
                                hashes: (left.clone(), right.clone()),
                                sender: *peer_id,
                            },
                        );
                    }
                    PeerLedgerQueryResponse::ChildAccounts(accounts) => {
                        store.dispatch(
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
            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesReceived {
                address,
                hashes: (left_hash, right_hash),
                sender,
                ..
            } => {
                let Some((snarked_ledger_hash, parent_hash)) = None.or_else(|| {
                    let snarked_ledger =
                        store.state().transition_frontier.sync.ledger()?.snarked()?;
                    let parent_hash = snarked_ledger
                        .fetch_pending()?
                        .get(address)?
                        .expected_hash
                        .clone();
                    Some((snarked_ledger.ledger_hash().clone(), parent_hash))
                }) else {
                    return;
                };

                let actual_hash = hash_node_at_depth(
                    address.length(),
                    left_hash.0.to_field(),
                    right_hash.0.to_field(),
                );
                if actual_hash != parent_hash.0.to_field() {
                    store.dispatch(
                        TransitionFrontierSyncLedgerSnarkedAction::ChildHashesRejected {
                            address: address.clone(),
                            hashes: (left_hash.clone(), right_hash.clone()),
                            sender: *sender,
                        },
                    );

                    return;
                }

                // TODO: for async ledger this needs an intermediary action
                let (previous_left_hash, previous_right_hash) = store
                    .service()
                    .child_hashes_get(snarked_ledger_hash, address)
                    .unwrap();

                store.dispatch(
                    TransitionFrontierSyncLedgerSnarkedAction::ChildHashesAccepted {
                        address: address.clone(),
                        hashes: (left_hash.clone(), right_hash.clone()),
                        previous_hashes: (previous_left_hash, previous_right_hash),
                    },
                );
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesAccepted { .. } => {
                if !store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery) {
                    store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::Success);
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesRejected { .. } => {
                // TODO(tizoc): we do nothing here, but the peer must be punished somehow
                store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsReceived {
                address,
                accounts,
                sender,
            } => {
                let Some((snarked_ledger_hash, parent_hash)) = None.or_else(|| {
                    let snarked_ledger =
                        store.state().transition_frontier.sync.ledger()?.snarked()?;
                    Some((
                        snarked_ledger.ledger_hash().clone(),
                        snarked_ledger
                            .fetch_pending()?
                            .get(address)?
                            .expected_hash
                            .clone(),
                    ))
                }) else {
                    return;
                };

                // After setting the accounts, we get the new computed hash.
                // It must be equal to the parent node hash, otherwise we got
                // bad data from the peer.
                let computed_hash = store
                    .service
                    .accounts_set(snarked_ledger_hash.clone(), address, accounts.clone())
                    .unwrap();

                if computed_hash != parent_hash {
                    store.dispatch(
                        TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsRejected {
                            address: address.clone(),
                            sender: *sender,
                        },
                    );
                    return;
                }

                // Setting accounts doesn't immediately compute the hashes in the merkle tree,
                // so we force that here before continuing.
                let compute_hashes_result = store
                    .service
                    .compute_snarked_ledger_hashes(&snarked_ledger_hash);

                if let Err(_) = compute_hashes_result {
                    // TODO(tizoc): log this error
                }

                store.dispatch(
                    TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsAccepted {
                        address: address.clone(),
                        count: accounts.len() as u64,
                        sender: *sender,
                    },
                );
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsAccepted { .. } => {
                if !store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery) {
                    store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::Success);
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsRejected { .. } => {
                // TODO(tizoc): we do nothing here, but the peer must be punished somehow
                store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressPending { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::Success => {}
        }
    }
}
