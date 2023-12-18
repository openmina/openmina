use redux::Timestamp;

use crate::ledger::LEDGER_DEPTH;
use crate::p2p::channels::best_tip::P2pChannelsBestTipResponseSendAction;
use crate::snark_pool::{SnarkPoolJobsUpdateAction, SnarkWork};
use crate::stats::sync::SyncingLedger;
use crate::Store;

use super::sync::ledger::snarked::TransitionFrontierSyncLedgerSnarkedAction;
use super::sync::ledger::staged::TransitionFrontierSyncLedgerStagedAction;
use super::sync::ledger::TransitionFrontierSyncLedgerAction;
use super::sync::{
    TransitionFrontierSyncAction, TransitionFrontierSyncLedgerNextEpochSuccessAction,
    TransitionFrontierSyncLedgerRootSuccessAction,
    TransitionFrontierSyncLedgerStakingSuccessAction, TransitionFrontierSyncState,
};
use super::{
    TransitionFrontierAction, TransitionFrontierActionWithMeta, TransitionFrontierSyncedAction,
};

pub fn transition_frontier_effects<S: crate::Service>(
    store: &mut Store<S>,
    action: TransitionFrontierActionWithMeta,
) {
    let (action, meta) = action.split();

    match action {
        TransitionFrontierAction::Sync(a) => match a {
            TransitionFrontierSyncAction::Init(a) => {
                if let Some(stats) = store.service.stats() {
                    stats.new_sync_target(meta.time(), &a.best_tip);
                    if let Some(root) = store.state.get().transition_frontier.sync.root_block() {
                        stats.syncing_ledger(SyncingLedger::Init {
                            snarked_ledger_hash: root.snarked_ledger_hash().clone(),
                            staged_ledger_hash: root.staged_ledger_hash().clone(),
                        });
                    }
                    if let TransitionFrontierSyncState::BlocksPending { chain, .. } =
                        &store.state.get().transition_frontier.sync
                    {
                        stats.syncing_blocks_init(chain);
                    }
                }
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::BestTipUpdate(a) => {
                if let Some(stats) = store.service.stats() {
                    stats.new_sync_target(meta.time(), &a.best_tip);
                    if let Some(root) = store.state.get().transition_frontier.sync.root_block() {
                        stats.syncing_ledger(SyncingLedger::Init {
                            snarked_ledger_hash: root.snarked_ledger_hash().clone(),
                            staged_ledger_hash: root.staged_ledger_hash().clone(),
                        });
                    }
                    if let TransitionFrontierSyncState::BlocksPending { chain, .. } =
                        &store.state.get().transition_frontier.sync
                    {
                        stats.syncing_blocks_init(chain);
                    }
                }
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::LedgerStakingPending(a) => {
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::LedgerStakingSuccess(a) => {
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::LedgerNextEpochPending(a) => {
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::LedgerNextEpochSuccess(a) => {
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::LedgerRootPending(a) => {
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::LedgerRootSuccess(a) => {
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::BlocksPending(a) => {
                if let Some(stats) = store.service.stats() {
                    if let TransitionFrontierSyncState::BlocksPending { chain, .. } =
                        &store.state.get().transition_frontier.sync
                    {
                        stats.syncing_blocks_init(chain);
                    }
                }
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::BlocksPeersQuery(a) => {
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::BlocksPeerQueryInit(a) => {
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::BlocksPeerQueryRetry(a) => {
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::BlocksPeerQueryPending(a) => {
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
            TransitionFrontierSyncAction::BlocksPeerQueryError(a) => {
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::BlocksPeerQuerySuccess(a) => {
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::BlocksFetchSuccess(a) => {
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
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::BlocksNextApplyInit(a) => {
                a.effects(&meta, store);
            }
            TransitionFrontierSyncAction::BlocksNextApplyPending(a) => {
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
            TransitionFrontierSyncAction::BlocksNextApplySuccess(a) => {
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
                // TODO(tizoc): push new snarked roots here?
                a.effects(&meta, store);
            }
            // Bootstrap/Catchup is practically complete at this point.
            // This effect is where the finalization part needs to be
            // executed, which is mostly to grab some data that we need
            // from previous chain, before it's discarded after dispatching
            // `TransitionFrontierSyncedAction`.
            TransitionFrontierSyncAction::BlocksSuccess(_) => {
                let transition_frontier = &store.state.get().transition_frontier;
                let sync = &transition_frontier.sync;
                let TransitionFrontierSyncState::BlocksSuccess { chain, .. } = sync else {
                    return;
                };
                let Some(root_block) = chain.first() else {
                    return;
                };
                let Some(best_tip) = chain.last() else {
                    return;
                };
                let ledgers_to_keep = chain
                    .iter()
                    .flat_map(|b| [b.snarked_ledger_hash(), b.staged_ledger_hash()])
                    .cloned()
                    .collect();

                let own_peer_id = store.state().p2p.my_id();
                let orphaned_snarks = transition_frontier
                    .best_chain
                    .iter()
                    .rev()
                    .take_while(|b1| {
                        let height_diff = best_tip.height().saturating_sub(b1.height()) as usize;
                        if height_diff == 0 {
                            best_tip.hash() != b1.hash()
                        } else if let Some(index) = chain.len().checked_sub(height_diff + 1) {
                            chain.get(index).map_or(true, |b2| b1.hash() != b2.hash())
                        } else {
                            true
                        }
                    })
                    .flat_map(|v| v.completed_works_iter())
                    .map(|v| SnarkWork {
                        work: v.clone().into(),
                        received_t: meta.time(),
                        sender: own_peer_id,
                    })
                    .collect();

                match (transition_frontier.best_chain.first(), sync.root_block()) {
                    (Some(current_root), Some(new_root)) => {
                        let current_root = current_root.clone();
                        let new_root = new_root.clone();
                        if current_root.snarked_ledger_hash() != new_root.snarked_ledger_hash() {
                            let protocol_states = store
                                .state()
                                .transition_frontier
                                .needed_protocol_states
                                .clone();
                            if let Err(error) = store.service.push_snarked_ledger(
                                &protocol_states,
                                &current_root,
                                &new_root,
                            ) {
                                // TODO(tizoc): don't panic here
                                panic!(
                                    "Failed to push a new snarked ledger {}  -> {}: {}",
                                    current_root.snarked_ledger_hash().to_string(),
                                    new_root.snarked_ledger_hash().to_string(),
                                    error
                                );
                            }
                        }
                    }
                    _ => {}
                };

                let res = store.service.commit(ledgers_to_keep, root_block, best_tip);
                let needed_protocol_states = res.needed_protocol_states;
                let jobs = res.available_jobs;
                store.dispatch(TransitionFrontierSyncedAction {
                    needed_protocol_states,
                });
                store.dispatch(SnarkPoolJobsUpdateAction {
                    jobs,
                    orphaned_snarks,
                });
            }
            TransitionFrontierSyncAction::Ledger(a) => {
                handle_transition_frontier_sync_ledger_action(a, &meta, store)
            }
        },
        TransitionFrontierAction::Synced(_) => {
            let Some(best_tip) = store.state.get().transition_frontier.best_tip() else {
                return;
            };
            if let Some(stats) = store.service.stats() {
                stats.new_best_tip(meta.time(), best_tip);
            }

            // publish new best tip.
            let best_tip = best_tip.clone();
            for peer_id in store.state().p2p.ready_peers() {
                store.dispatch(P2pChannelsBestTipResponseSendAction {
                    peer_id,
                    best_tip: best_tip.clone(),
                });
            }
        }
    }
}

// Handling of the actions related to the synchronization of a target ledger
// in either one of the epoch ledgers or the root of the transition frontier
// happens here. These are part of the bootstrap process and should not happen
// again unless the node needs to re-bootstrap (either because of a reorg or
// a long desync).
fn handle_transition_frontier_sync_ledger_action<S: crate::Service>(
    action: TransitionFrontierSyncLedgerAction,
    meta: &redux::ActionMeta,
    store: &mut redux::Store<crate::State, S, crate::Action>,
) {
    match action {
        TransitionFrontierSyncLedgerAction::Init(action) => {
            action.effects(meta, store);
        }
        TransitionFrontierSyncLedgerAction::Snarked(a) => match a {
            TransitionFrontierSyncLedgerSnarkedAction::Pending(action) => {
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeersQuery(action) => {
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryInit(action) => {
                if let Some(stats) = store.service().stats() {
                    let (start, end) = (meta.time(), meta.time());
                    if action.address.length() < LEDGER_DEPTH - 1 {
                        stats.syncing_ledger(SyncingLedger::FetchHashes { start, end });
                    } else {
                        stats.syncing_ledger(SyncingLedger::FetchAccounts { start, end });
                    }
                }
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryPending(_) => {}
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryRetry(action) => {
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryError(action) => {
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQuerySuccess(action) => {
                if let Some(stats) = store.service.stats() {
                    if let Some((start, end)) = store
                        .state
                        .get()
                        .transition_frontier
                        .sync
                        .ledger()
                        .and_then(|s| s.snarked()?.peer_query_get(&action.peer_id, action.rpc_id))
                        .map(|(_, s)| (s.time, meta.time()))
                    {
                        if action.response.is_child_hashes() {
                            stats.syncing_ledger(SyncingLedger::FetchHashes { start, end });
                        } else if action.response.is_child_accounts() {
                            stats.syncing_ledger(SyncingLedger::FetchAccounts { start, end });
                        }
                    }
                }
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesReceived(action) => {
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsReceived(action) => {
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerSnarkedAction::Success(action) => {
                action.effects(meta, store);
            }
        },
        TransitionFrontierSyncLedgerAction::Staged(a) => match a {
            TransitionFrontierSyncLedgerStagedAction::PartsFetchPending(action) => {
                if let Some(stats) = store.service().stats() {
                    let (start, end) = (meta.time(), None);
                    stats.syncing_ledger(SyncingLedger::FetchParts { start, end });
                }
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit(action) => {
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchPending(_) => {}
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchError(action) => {
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchSuccess(action) => {
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerInvalid(action) => {
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerStagedAction::PartsPeerValid(action) => {
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerStagedAction::PartsFetchSuccess(action) => {
                if let Some(stats) = store.service().stats() {
                    let (start, end) = (Timestamp::ZERO, Some(meta.time()));
                    stats.syncing_ledger(SyncingLedger::FetchParts { start, end });
                }
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructEmpty(action) => {
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructInit(action) => {
                if let Some(stats) = store.service().stats() {
                    let (start, end) = (meta.time(), None);
                    stats.syncing_ledger(SyncingLedger::ApplyParts { start, end });
                }
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerStagedAction::ReconstructPending(_) => {}
            TransitionFrontierSyncLedgerStagedAction::ReconstructError(_) => {}
            TransitionFrontierSyncLedgerStagedAction::ReconstructSuccess(action) => {
                if let Some(stats) = store.service().stats() {
                    let (start, end) = (Timestamp::ZERO, Some(meta.time()));
                    stats.syncing_ledger(SyncingLedger::ApplyParts { start, end });
                }
                action.effects(meta, store);
            }
            TransitionFrontierSyncLedgerStagedAction::Success(action) => {
                action.effects(meta, store);
            }
        },
        TransitionFrontierSyncLedgerAction::Success(_) => {
            // TODO(tizoc): should check that ledger is success?
            match &store.state().transition_frontier.sync {
                TransitionFrontierSyncState::StakingLedgerPending { .. } => {
                    store.dispatch(TransitionFrontierSyncLedgerStakingSuccessAction {});
                }
                TransitionFrontierSyncState::NextEpochLedgerPending { .. } => {
                    store.dispatch(TransitionFrontierSyncLedgerNextEpochSuccessAction {});
                }
                TransitionFrontierSyncState::RootLedgerPending { .. } => {
                    store.dispatch(TransitionFrontierSyncLedgerRootSuccessAction {});
                }
                other => println!(
                    "@@@@@@ sync ledger success, but should not for this state {:?}",
                    other
                ),
            }
        }
    }
}
