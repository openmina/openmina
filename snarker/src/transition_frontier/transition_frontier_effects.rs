use p2p::channels::rpc::P2pChannelsRpcRequestSendAction;
use redux::Timestamp;

use crate::ledger::LEDGER_DEPTH;
use crate::p2p::channels::rpc::P2pRpcRequest;
use crate::stats::sync::SyncingLedger;
use crate::Store;

use super::sync::ledger::{
    TransitionFrontierSyncLedgerAction, TransitionFrontierSyncLedgerInitAction,
    TransitionFrontierSyncLedgerSnarkedPeersQueryAction,
    TransitionFrontierSyncLedgerStagedReconstructPendingAction,
};
use super::{
    TransitionFrontierAction, TransitionFrontierActionWithMeta,
    TransitionFrontierSyncBlocksApplyPendingAction, TransitionFrontierSyncBlocksApplySuccessAction,
    TransitionFrontierSyncBlocksFetchSuccessAction,
    TransitionFrontierSyncBlocksNextApplyInitAction,
    TransitionFrontierSyncBlocksPeerQueryInitAction,
    TransitionFrontierSyncBlocksPeerQueryPendingAction,
    TransitionFrontierSyncBlocksPeerQueryRetryAction, TransitionFrontierSyncBlocksPeersQueryAction,
    TransitionFrontierSyncBlocksPendingAction, TransitionFrontierSyncBlocksSuccessAction,
    TransitionFrontierSyncLedgerRootPendingAction, TransitionFrontierSyncLedgerRootSuccessAction,
    TransitionFrontierSyncState, TransitionFrontierSyncedAction,
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
                if let TransitionFrontierSyncState::BlocksPending { chain, .. } =
                    &store.state.get().transition_frontier.sync
                {
                    stats.syncing_blocks_init(chain);
                }
            }
            store.dispatch(TransitionFrontierSyncLedgerRootPendingAction {});
        }
        TransitionFrontierAction::SyncBestTipUpdate(a) => {
            if let Some(stats) = store.service.stats() {
                stats.new_sync_target(meta.time(), &a.best_tip);
                if let TransitionFrontierSyncState::BlocksPending { chain, .. } =
                    &store.state.get().transition_frontier.sync
                {
                    stats.syncing_blocks_init(chain);
                }
            }
            // if root snarked ledger changed.
            store.dispatch(TransitionFrontierSyncLedgerInitAction {});
            // if root snarked ledger stayed same but root block changed
            // while reconstructing staged ledger.
            store.dispatch(TransitionFrontierSyncLedgerStagedReconstructPendingAction {});
            store.dispatch(TransitionFrontierSyncLedgerSnarkedPeersQueryAction {});
            // if we don't need to sync root staged ledger.
            store.dispatch(TransitionFrontierSyncBlocksPeersQueryAction {});
            // if we already have a block ready to be applied.
            store.dispatch(TransitionFrontierSyncBlocksNextApplyInitAction {});

            // TODO(binier): cleanup ledgers
        }
        TransitionFrontierAction::SyncLedgerRootPending(_) => {
            store.dispatch(TransitionFrontierSyncLedgerInitAction {});
        }
        TransitionFrontierAction::SyncLedgerRootSuccess(_) => {
            store.dispatch(TransitionFrontierSyncBlocksPendingAction {});
        }
        TransitionFrontierAction::SyncBlocksPending(_) => {
            if let Some(stats) = store.service.stats() {
                if let TransitionFrontierSyncState::BlocksPending { chain, .. } =
                    &store.state.get().transition_frontier.sync
                {
                    stats.syncing_blocks_init(chain);
                }
            }
            store.dispatch(TransitionFrontierSyncBlocksPeersQueryAction {});
        }
        TransitionFrontierAction::SyncBlocksPeersQuery(_) => {
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
                    if store.dispatch(TransitionFrontierSyncBlocksPeerQueryRetryAction {
                        peer_id,
                        hash: hash.clone(),
                    }) {
                        retry_hashes.pop();
                        continue;
                    }
                }

                match store.state().transition_frontier.sync.blocks_fetch_next() {
                    Some(hash) => {
                        store.dispatch(TransitionFrontierSyncBlocksPeerQueryInitAction {
                            peer_id,
                            hash,
                        });
                    }
                    None if retry_hashes.is_empty() => break,
                    None => {}
                }
            }
        }
        TransitionFrontierAction::SyncBlocksPeerQueryInit(a) => {
            let Some(rpc_id) = store.state().p2p.get_ready_peer(&a.peer_id)
                .map(|v| v.channels.rpc.next_local_rpc_id()) else { return };

            if store.dispatch(P2pChannelsRpcRequestSendAction {
                peer_id: a.peer_id,
                id: rpc_id,
                request: P2pRpcRequest::Block(a.hash.clone()),
            }) {
                store.dispatch(TransitionFrontierSyncBlocksPeerQueryPendingAction {
                    hash: a.hash,
                    peer_id: a.peer_id,
                    rpc_id,
                });
            }
        }
        TransitionFrontierAction::SyncBlocksPeerQueryRetry(a) => {
            let Some(rpc_id) = store.state().p2p.get_ready_peer(&a.peer_id)
                .map(|v| v.channels.rpc.next_local_rpc_id()) else { return };

            if store.dispatch(P2pChannelsRpcRequestSendAction {
                peer_id: a.peer_id,
                id: rpc_id,
                request: P2pRpcRequest::Block(a.hash.clone()),
            }) {
                store.dispatch(TransitionFrontierSyncBlocksPeerQueryPendingAction {
                    hash: a.hash,
                    peer_id: a.peer_id,
                    rpc_id,
                });
            }
        }
        TransitionFrontierAction::SyncBlocksPeerQueryPending(a) => {
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
        TransitionFrontierAction::SyncBlocksPeerQueryError(_) => {
            store.dispatch(TransitionFrontierSyncBlocksPeersQueryAction {});
        }
        TransitionFrontierAction::SyncBlocksPeerQuerySuccess(a) => {
            store.dispatch(TransitionFrontierSyncBlocksPeersQueryAction {});
            store.dispatch(TransitionFrontierSyncBlocksFetchSuccessAction {
                hash: a.response.hash,
            });
        }
        TransitionFrontierAction::SyncBlocksFetchSuccess(a) => {
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
            store.dispatch(TransitionFrontierSyncBlocksNextApplyInitAction {});
        }
        TransitionFrontierAction::SyncBlocksNextApplyInit(_) => {
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

                store.dispatch(TransitionFrontierSyncBlocksApplyPendingAction {
                    hash: hash.clone(),
                });
                store.service.block_apply(block, pred_block).unwrap();

                store.dispatch(TransitionFrontierSyncBlocksApplySuccessAction { hash });
            }
        }
        TransitionFrontierAction::SyncBlocksApplyPending(a) => {
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
        TransitionFrontierAction::SyncBlocksApplySuccess(a) => {
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
            store.dispatch(TransitionFrontierSyncBlocksSuccessAction {});
            // }
        }
        TransitionFrontierAction::SyncBlocksSuccess(_) => {
            let sync = &store.state.get().transition_frontier.sync;
            let TransitionFrontierSyncState::BlocksSuccess { chain, .. } = sync else { return };
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
            TransitionFrontierSyncLedgerAction::SnarkedPending(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedPeersQuery(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedPeerQueryInit(action) => {
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
            TransitionFrontierSyncLedgerAction::SnarkedPeerQueryRetry(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedPeerQueryPending(_) => {}
            TransitionFrontierSyncLedgerAction::SnarkedPeerQueryError(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedPeerQuerySuccess(action) => {
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
            TransitionFrontierSyncLedgerAction::SnarkedChildHashesReceived(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedChildAccountsReceived(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedSuccess(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedReconstructPending(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedPartsFetchInit(action) => {
                if let Some(stats) = store.service().stats() {
                    let (start, end) = (meta.time(), None);
                    stats.syncing_ledger(SyncingLedger::FetchParts { start, end });
                }
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedPartsFetchPending(_) => {}
            TransitionFrontierSyncLedgerAction::StagedPartsFetchError(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedPartsFetchSuccess(action) => {
                if let Some(stats) = store.service().stats() {
                    let (start, end) = (Timestamp::ZERO, Some(meta.time()));
                    stats.syncing_ledger(SyncingLedger::FetchParts { start, end });
                }
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedPartsApplyInit(action) => {
                if let Some(stats) = store.service().stats() {
                    let (start, end) = (meta.time(), None);
                    stats.syncing_ledger(SyncingLedger::ApplyParts { start, end });
                }
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedPartsApplySuccess(action) => {
                if let Some(stats) = store.service().stats() {
                    let (start, end) = (Timestamp::ZERO, Some(meta.time()));
                    stats.syncing_ledger(SyncingLedger::ApplyParts { start, end });
                }
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedReconstructSuccess(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::Success(_) => {
                store.dispatch(TransitionFrontierSyncLedgerRootSuccessAction {});
            }
        },
    }
}
