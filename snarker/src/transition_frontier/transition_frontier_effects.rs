use p2p::channels::rpc::P2pChannelsRpcRequestSendAction;
use redux::Timestamp;

use crate::ledger::LEDGER_DEPTH;
use crate::p2p::channels::rpc::P2pRpcRequest;
use crate::stats::sync::SyncingLedger;
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
        TransitionFrontierAction::SyncInit(a) => {
            if let Some(stats) = store.service.stats() {
                stats.new_sync_target(meta.time(), &a.best_tip);
                if let TransitionFrontierSyncState::BlocksFetchAndApplyPending { chain, .. } =
                    &store.state.get().transition_frontier.sync
                {
                    stats.syncing_blocks_init(chain);
                }
            }
            store.dispatch(TransitionFrontierRootLedgerSyncPendingAction {});
        }
        TransitionFrontierAction::SyncBestTipUpdate(a) => {
            if let Some(stats) = store.service.stats() {
                stats.new_sync_target(meta.time(), &a.best_tip);
                if let TransitionFrontierSyncState::BlocksFetchAndApplyPending { chain, .. } =
                    &store.state.get().transition_frontier.sync
                {
                    stats.syncing_blocks_init(chain);
                }
            }
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
            if let Some(stats) = store.service.stats() {
                if let TransitionFrontierSyncState::BlocksFetchAndApplyPending { chain, .. } =
                    &store.state.get().transition_frontier.sync
                {
                    stats.syncing_blocks_init(chain);
                }
            }
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
        TransitionFrontierAction::SyncBlocksFetchAndApplyPeerQueryPending(a) => {
            if let Some(stats) = store.service.stats() {
                if let Some(state) = store
                    .state
                    .get()
                    .transition_frontier
                    .sync
                    .block_state(&a.hash)
                {
                    stats.syncing_block_update(state);
                }
            }
        }
        TransitionFrontierAction::SyncBlocksFetchAndApplyPeerQueryError(_) => {
            store.dispatch(TransitionFrontierSyncBlocksFetchAndApplyPeersQueryAction {});
        }
        TransitionFrontierAction::SyncBlocksFetchAndApplyPeerQuerySuccess(a) => {
            store.dispatch(TransitionFrontierSyncBlocksFetchAndApplyPeersQueryAction {});
            store.dispatch(TransitionFrontierSyncBlockFetchSuccessAction {
                hash: a.response.hash,
            });
        }
        TransitionFrontierAction::SyncBlockFetchSuccess(a) => {
            if let Some(stats) = store.service.stats() {
                if let Some(state) = store
                    .state
                    .get()
                    .transition_frontier
                    .sync
                    .block_state(&a.hash)
                {
                    stats.syncing_block_update(state);
                }
            }
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
        TransitionFrontierAction::SyncBlockApplyPending(a) => {
            if let Some(stats) = store.service.stats() {
                if let Some(state) = store
                    .state
                    .get()
                    .transition_frontier
                    .sync
                    .block_state(&a.hash)
                {
                    stats.syncing_block_update(state);
                }
            }
        }
        TransitionFrontierAction::SyncBlockApplySuccess(a) => {
            if let Some(stats) = store.service.stats() {
                if let Some(state) = store
                    .state
                    .get()
                    .transition_frontier
                    .sync
                    .block_state(&a.hash)
                {
                    stats.syncing_block_update(state);
                }
            }

            // TODO(binier): uncomment once ledger communication is async.
            // if !store.dispatch(TransitionFrontierSyncBlockNextApplyInitAction {}) {
            store.dispatch(TransitionFrontierSyncBlocksFetchAndApplySuccessAction {});
            // }
        }
        TransitionFrontierAction::SyncBlocksFetchAndApplySuccess(_) => {
            let sync = &store.state.get().transition_frontier.sync;
            let TransitionFrontierSyncState::BlocksFetchAndApplySuccess { chain, .. } = sync else { return };
            let Some(root_block) = chain.first() else { return };
            let ledgers_to_keep = chain
                .iter()
                .flat_map(|b| [b.snarked_ledger_hash(), b.staged_ledger_hash()])
                .cloned()
                .collect();
            store.service.commit(ledgers_to_keep, root_block);
            store.dispatch(TransitionFrontierSyncedAction {});
        }
        TransitionFrontierAction::Synced(_) => {
            let Some(best_tip) = store.state.get().transition_frontier.best_tip() else { return };
            if let Some(stats) = store.service.stats() {
                stats.new_best_tip(meta.time(), best_tip);
            }
            // TODO(binier): publish new best tip
        }
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
                if let Some(stats) = store.service().stats() {
                    let (start, end) = (meta.time(), meta.time());
                    if action.address.length() < LEDGER_DEPTH - 1 {
                        stats.syncing_ledger(SyncingLedger::FetchHashes { start, end });
                    } else {
                        stats.syncing_ledger(SyncingLedger::FetchAccounts { start, end });
                    }
                }
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
                if let Some(stats) = store.service.stats() {
                    if let Some((start, end)) = store
                        .state
                        .get()
                        .transition_frontier
                        .sync
                        .root_ledger()
                        .and_then(|s| {
                            s.snarked_ledger_peer_query_get(&action.peer_id, action.rpc_id)
                        })
                        .map(|(_, s)| (s.time, meta.time()))
                    {
                        if action.response.is_child_hashes() {
                            stats.syncing_ledger(SyncingLedger::FetchHashes { start, end });
                        } else if action.response.is_child_accounts() {
                            stats.syncing_ledger(SyncingLedger::FetchAccounts { start, end });
                        }
                    }
                }
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
                if let Some(stats) = store.service().stats() {
                    let (start, end) = (meta.time(), None);
                    stats.syncing_ledger(SyncingLedger::FetchParts { start, end });
                }
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsFetchPending(_) => {}
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsFetchError(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsFetchSuccess(action) => {
                if let Some(stats) = store.service().stats() {
                    let (start, end) = (Timestamp::ZERO, Some(meta.time()));
                    stats.syncing_ledger(SyncingLedger::FetchParts { start, end });
                }
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsApplyInit(action) => {
                if let Some(stats) = store.service().stats() {
                    let (start, end) = (meta.time(), None);
                    stats.syncing_ledger(SyncingLedger::ApplyParts { start, end });
                }
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsApplySuccess(action) => {
                if let Some(stats) = store.service().stats() {
                    let (start, end) = (Timestamp::ZERO, Some(meta.time()));
                    stats.syncing_ledger(SyncingLedger::ApplyParts { start, end });
                }
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
