use p2p::channels::rpc::P2pChannelsRpcRequestSendAction;

use crate::p2p::channels::rpc::P2pRpcRequest;
use crate::Store;

use super::sync::ledger::{
    TransitionFrontierSyncLedgerAction, TransitionFrontierSyncLedgerInitAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction,
    TransitionFrontierSyncLedgerStagedLedgerReconstructPendingAction,
};
use super::{
    TransitionFrontierAction, TransitionFrontierActionWithMeta,
    TransitionFrontierRootLedgerSyncPendingAction, TransitionFrontierRootLedgerSyncSuccessAction,
    TransitionFrontierSyncBlockApplyPendingAction, TransitionFrontierSyncBlockApplySuccessAction,
    TransitionFrontierSyncBlockFetchSuccessAction, TransitionFrontierSyncBlockNextApplyInitAction,
    TransitionFrontierSyncBlocksFetchAndApplyPeerQueryInitAction,
    TransitionFrontierSyncBlocksFetchAndApplyPeerQueryPendingAction,
    TransitionFrontierSyncBlocksFetchAndApplyPeerQueryRetryAction,
    TransitionFrontierSyncBlocksFetchAndApplyPeersQueryAction,
    TransitionFrontierSyncBlocksFetchAndApplyPendingAction,
    TransitionFrontierSyncBlocksFetchAndApplySuccessAction, TransitionFrontierSyncState,
    TransitionFrontierSyncedAction,
};

pub fn transition_frontier_effects<S: crate::Service>(
    store: &mut Store<S>,
    action: TransitionFrontierActionWithMeta,
) {
    let (action, meta) = action.split();

    match action {
        TransitionFrontierAction::SyncInit(_) => {
            store.dispatch(TransitionFrontierRootLedgerSyncPendingAction {});
        }
        TransitionFrontierAction::SyncBestTipUpdate(_) => {
            // if root snarked ledger changed.
            store.dispatch(TransitionFrontierSyncLedgerInitAction {});
            // if root snarked ledger stayed same but root block changed
            // while reconstructing staged ledger.
            store.dispatch(TransitionFrontierSyncLedgerStagedLedgerReconstructPendingAction {});
            store.dispatch(TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction {});
            // if we don't need to sync root staged ledger.
            store.dispatch(TransitionFrontierSyncBlocksFetchAndApplyPeersQueryAction {});
            // if we already have a block ready to be applied.
            store.dispatch(TransitionFrontierSyncBlockNextApplyInitAction {});

            // TODO(binier): cleanup ledgers
        }
        TransitionFrontierAction::RootLedgerSyncPending(_) => {
            store.dispatch(TransitionFrontierSyncLedgerInitAction {});
        }
        TransitionFrontierAction::RootLedgerSyncSuccess(_) => {
            store.dispatch(TransitionFrontierSyncBlocksFetchAndApplyPendingAction {});
        }
        TransitionFrontierAction::SyncBlocksFetchAndApplyPending(_) => {
            store.dispatch(TransitionFrontierSyncBlocksFetchAndApplyPeersQueryAction {});
        }
        TransitionFrontierAction::SyncBlocksFetchAndApplyPeersQuery(_) => {
            // TODO(binier): make sure they have the ledger we want to query.
            let mut peer_ids = store
                .state()
                .p2p
                .ready_peers_iter()
                .filter(|(_, p)| p.channels.rpc.can_send_request())
                .map(|(id, p)| (*id, p.connected_since))
                .collect::<Vec<_>>();
            peer_ids.sort_by(|(_, t1), (_, t2)| t2.cmp(t1));

            let mut retry_hashes = store
                .state()
                .transition_frontier
                .sync
                .blocks_fetch_retry_iter()
                .collect::<Vec<_>>();
            retry_hashes.reverse();

            for (peer_id, _) in peer_ids {
                if let Some(hash) = retry_hashes.last() {
                    if store.dispatch(
                        TransitionFrontierSyncBlocksFetchAndApplyPeerQueryRetryAction {
                            peer_id,
                            hash: hash.clone(),
                        },
                    ) {
                        retry_hashes.pop();
                        continue;
                    }
                }

                match store.state().transition_frontier.sync.blocks_fetch_next() {
                    Some(hash) => {
                        store.dispatch(
                            TransitionFrontierSyncBlocksFetchAndApplyPeerQueryInitAction {
                                peer_id,
                                hash,
                            },
                        );
                    }
                    None if retry_hashes.is_empty() => break,
                    None => {}
                }
            }
        }
        TransitionFrontierAction::SyncBlocksFetchAndApplyPeerQueryInit(a) => {
            let Some(rpc_id) = store.state().p2p.get_ready_peer(&a.peer_id)
                .map(|v| v.channels.rpc.next_local_rpc_id()) else { return };

            if store.dispatch(P2pChannelsRpcRequestSendAction {
                peer_id: a.peer_id,
                id: rpc_id,
                request: P2pRpcRequest::Block(a.hash.clone()),
            }) {
                store.dispatch(
                    TransitionFrontierSyncBlocksFetchAndApplyPeerQueryPendingAction {
                        hash: a.hash,
                        peer_id: a.peer_id,
                        rpc_id,
                    },
                );
            }
        }
        TransitionFrontierAction::SyncBlocksFetchAndApplyPeerQueryRetry(a) => {
            let Some(rpc_id) = store.state().p2p.get_ready_peer(&a.peer_id)
                .map(|v| v.channels.rpc.next_local_rpc_id()) else { return };

            if store.dispatch(P2pChannelsRpcRequestSendAction {
                peer_id: a.peer_id,
                id: rpc_id,
                request: P2pRpcRequest::Block(a.hash.clone()),
            }) {
                store.dispatch(
                    TransitionFrontierSyncBlocksFetchAndApplyPeerQueryPendingAction {
                        hash: a.hash,
                        peer_id: a.peer_id,
                        rpc_id,
                    },
                );
            }
        }
        TransitionFrontierAction::SyncBlocksFetchAndApplyPeerQueryPending(_) => {}
        TransitionFrontierAction::SyncBlocksFetchAndApplyPeerQueryError(_) => {
            store.dispatch(TransitionFrontierSyncBlocksFetchAndApplyPeersQueryAction {});
        }
        TransitionFrontierAction::SyncBlocksFetchAndApplyPeerQuerySuccess(a) => {
            store.dispatch(TransitionFrontierSyncBlocksFetchAndApplyPeersQueryAction {});
            store.dispatch(TransitionFrontierSyncBlockFetchSuccessAction {
                hash: a.response.hash,
            });
        }
        TransitionFrontierAction::SyncBlockFetchSuccess(_) => {
            store.dispatch(TransitionFrontierSyncBlockNextApplyInitAction {});
        }
        TransitionFrontierAction::SyncBlockNextApplyInit(_) => {
            // TODO(binier): remove loop once ledger communication is async.
            loop {
                let Some((block, pred_block)) = store
                    .state()
                    .transition_frontier
                    .sync
                    .blocks_apply_next()
                    .map(|v| (v.0.clone(), v.1.clone()))
                    else { return };
                let hash = block.hash.clone();

                store
                    .dispatch(TransitionFrontierSyncBlockApplyPendingAction { hash: hash.clone() });
                store.service.block_apply(block, pred_block).unwrap();

                store.dispatch(TransitionFrontierSyncBlockApplySuccessAction { hash });
            }
        }
        TransitionFrontierAction::SyncBlockApplyPending(_) => {}
        TransitionFrontierAction::SyncBlockApplySuccess(_) => {
            // TODO(binier): uncomment once ledger communication is async.
            // if !store.dispatch(TransitionFrontierSyncBlockNextApplyInitAction {}) {
            store.dispatch(TransitionFrontierSyncBlocksFetchAndApplySuccessAction {});
            // }
        }
        TransitionFrontierAction::SyncBlocksFetchAndApplySuccess(_) => {
            let sync = &store.state().transition_frontier.sync;
            let TransitionFrontierSyncState::BlocksFetchAndApplySuccess { chain, .. } = sync else { return };
            let ledgers_to_keep = chain
                .iter()
                .flat_map(|b| [b.snarked_ledger_hash(), b.staged_ledger_hash()])
                .cloned()
                .collect();
            store.service.commit(ledgers_to_keep);
            store.dispatch(TransitionFrontierSyncedAction {});
        }
        TransitionFrontierAction::Synced(_) => {}
        TransitionFrontierAction::SyncLedger(action) => match action {
            TransitionFrontierSyncLedgerAction::Init(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPending(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeersQuery(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQueryInit(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQueryRetry(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQueryPending(_) => {}
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQueryError(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQuerySuccess(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncChildHashesReceived(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncChildAccountsReceived(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncSuccess(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerReconstructPending(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsFetchInit(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsFetchPending(_) => {}
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsFetchError(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsFetchSuccess(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsApplyInit(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsApplySuccess(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerReconstructSuccess(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::Success(_) => {
                store.dispatch(TransitionFrontierRootLedgerSyncSuccessAction {});
            }
        },
    }
}
