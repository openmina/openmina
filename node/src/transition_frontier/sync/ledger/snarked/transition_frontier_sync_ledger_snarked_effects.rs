use mina_p2p_messages::v2::MinaLedgerSyncLedgerQueryStableV1;
use p2p::channels::rpc::{P2pChannelsRpcAction, P2pRpcRequest};
use p2p::PeerId;
use redux::ActionMeta;

use crate::ledger::{LedgerAddress, LEDGER_DEPTH};
use crate::Store;

use super::{
    PeerLedgerQueryResponse, TransitionFrontierSyncLedgerSnarkedAction,
    TransitionFrontierSyncLedgerSnarkedService,
};

fn query_peer_init<S: redux::Service>(
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

    let query = if address.length() >= LEDGER_DEPTH - 1 {
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
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryPending {
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

                let mut retry_addresses = store
                    .state()
                    .transition_frontier
                    .sync
                    .ledger()
                    .and_then(|s| s.snarked())
                    .map_or(vec![], |s| s.sync_retry_iter().collect());
                retry_addresses.reverse();

                for (peer_id, _) in peer_ids {
                    if let Some(address) = retry_addresses.last() {
                        if store.dispatch(
                            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryRetry {
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
                        .and_then(|s| s.sync_next());
                    match address {
                        Some(address) => {
                            store.dispatch(
                                TransitionFrontierSyncLedgerSnarkedAction::PeerQueryInit {
                                    peer_id,
                                    address,
                                },
                            );
                        }
                        None if retry_addresses.is_empty() => break,
                        None => {}
                    }
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryInit { peer_id, address } => {
                query_peer_init(store, *peer_id, address.clone());
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryRetry { peer_id, address } => {
                query_peer_init(store, *peer_id, address.clone());
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryError { .. } => {
                store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQuerySuccess {
                peer_id,
                rpc_id,
                response,
            } => {
                let ledger = store.state().transition_frontier.sync.ledger();
                let Some(address) = ledger
                    .and_then(|s| s.snarked()?.peer_query_get(peer_id, *rpc_id))
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
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesReceived {
                address,
                hashes,
                ..
            } => {
                let Some(snarked_ledger_hash) = None.or_else(|| {
                    let ledger = store.state().transition_frontier.sync.ledger()?;
                    Some(ledger.snarked()?.ledger_hash().clone())
                }) else {
                    return;
                };
                store
                    .service
                    .hashes_set(snarked_ledger_hash, address, hashes.clone())
                    .unwrap();

                if !store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery) {
                    store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::Success);
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsReceived {
                address,
                accounts,
                ..
            } => {
                let Some(snarked_ledger_hash) = None.or_else(|| {
                    let ledger = store.state().transition_frontier.sync.ledger()?;
                    Some(ledger.snarked()?.ledger_hash().clone())
                }) else {
                    return;
                };
                store
                    .service
                    .accounts_set(snarked_ledger_hash, address, accounts.clone())
                    .unwrap();

                if !store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery) {
                    store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::Success);
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryPending { .. } => {}
            TransitionFrontierSyncLedgerSnarkedAction::Success => {}
        }
    }
}
