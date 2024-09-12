use redux::Timestamp;

use crate::block_producer::BlockProducerAction;
use crate::consensus::ConsensusAction;
use crate::ledger::LEDGER_DEPTH;
use crate::p2p::channels::best_tip::P2pChannelsBestTipAction;
use crate::snark_pool::{SnarkPoolAction, SnarkWork};
use crate::stats::sync::SyncingLedger;
use crate::{Store, TransactionPoolAction};

use super::genesis::TransitionFrontierGenesisAction;
use super::sync::ledger::snarked::{
    TransitionFrontierSyncLedgerSnarkedAction, ACCOUNT_SUBTREE_HEIGHT,
};
use super::sync::ledger::staged::TransitionFrontierSyncLedgerStagedAction;
use super::sync::ledger::{
    transition_frontier_sync_ledger_init_effects,
    transition_frontier_sync_ledger_snarked_success_effects,
    transition_frontier_sync_ledger_staged_success_effects, TransitionFrontierSyncLedgerAction,
};
use super::sync::{TransitionFrontierSyncAction, TransitionFrontierSyncState};
use super::{TransitionFrontierAction, TransitionFrontierActionWithMeta, TransitionFrontierState};

// TODO(refactor): all service accesses are for stats, how should that be handled?

pub fn transition_frontier_effects<S: crate::Service>(
    store: &mut Store<S>,
    action: TransitionFrontierActionWithMeta,
) {
    let (action, meta) = action.split();

    match action {
        TransitionFrontierAction::Genesis(a) => {
            // TODO(refactor): this should be handled by a callback and removed from here
            // whenever any of these is going to happen, genesisinject must happen first
            match &a {
                TransitionFrontierGenesisAction::Produce
                | TransitionFrontierGenesisAction::ProveSuccess { .. } => {
                    store.dispatch(TransitionFrontierAction::GenesisInject);
                }
                _ => {}
            }
        }
        TransitionFrontierAction::GenesisEffect(a) => {
            a.effects(&meta, store);
        }
        TransitionFrontierAction::GenesisInject => {
            synced_effects(&meta, store);
        }
        TransitionFrontierAction::Sync(a) => {
            match a {
                TransitionFrontierSyncAction::Init {
                    ref best_tip,
                    ref root_block,
                    ..
                } => {
                    if let Some(stats) = store.service.stats() {
                        stats.new_sync_target(meta.time(), best_tip, root_block);
                        if let TransitionFrontierSyncState::BlocksPending { chain, .. } =
                            &store.state.get().transition_frontier.sync
                        {
                            stats.syncing_blocks_init(chain);
                        }
                    }
                }
                TransitionFrontierSyncAction::BestTipUpdate {
                    ref best_tip,
                    ref root_block,
                    ..
                } => {
                    if let Some(stats) = store.service.stats() {
                        stats.new_sync_target(meta.time(), best_tip, root_block);
                        if let Some(target) =
                            store.state.get().transition_frontier.sync.ledger_target()
                        {
                            stats.syncing_ledger(
                                target.kind,
                                SyncingLedger::Init {
                                    snarked_ledger_hash: target.snarked_ledger_hash.clone(),
                                    staged_ledger_hash: target
                                        .staged
                                        .as_ref()
                                        .map(|v| v.hashes.non_snark.ledger_hash.clone()),
                                },
                            );
                        }
                        if let TransitionFrontierSyncState::BlocksPending { chain, .. } =
                            &store.state.get().transition_frontier.sync
                        {
                            stats.syncing_blocks_init(chain);
                        }
                    }
                }
                TransitionFrontierSyncAction::LedgerStakingPending => {
                    if let Some(stats) = store.service.stats() {
                        if let Some(target) =
                            store.state.get().transition_frontier.sync.ledger_target()
                        {
                            stats.syncing_ledger(
                                target.kind,
                                SyncingLedger::Init {
                                    snarked_ledger_hash: target.snarked_ledger_hash.clone(),
                                    staged_ledger_hash: target
                                        .staged
                                        .as_ref()
                                        .map(|v| v.hashes.non_snark.ledger_hash.clone()),
                                },
                            );
                        }
                    }
                }
                TransitionFrontierSyncAction::LedgerStakingSuccess => {}
                TransitionFrontierSyncAction::LedgerNextEpochPending => {
                    if let Some(stats) = store.service.stats() {
                        if let Some(target) =
                            store.state.get().transition_frontier.sync.ledger_target()
                        {
                            stats.syncing_ledger(
                                target.kind,
                                SyncingLedger::Init {
                                    snarked_ledger_hash: target.snarked_ledger_hash.clone(),
                                    staged_ledger_hash: target
                                        .staged
                                        .as_ref()
                                        .map(|v| v.hashes.non_snark.ledger_hash.clone()),
                                },
                            );
                        }
                    }
                }
                TransitionFrontierSyncAction::LedgerNextEpochSuccess => {}
                TransitionFrontierSyncAction::LedgerRootPending => {
                    if let Some(stats) = store.service.stats() {
                        if let Some(target) =
                            store.state.get().transition_frontier.sync.ledger_target()
                        {
                            stats.syncing_ledger(
                                target.kind,
                                SyncingLedger::Init {
                                    snarked_ledger_hash: target.snarked_ledger_hash.clone(),
                                    staged_ledger_hash: target
                                        .staged
                                        .as_ref()
                                        .map(|v| v.hashes.non_snark.ledger_hash.clone()),
                                },
                            );
                        }
                    }
                }
                TransitionFrontierSyncAction::LedgerRootSuccess => {}
                TransitionFrontierSyncAction::BlocksPending => {
                    if let Some(stats) = store.service.stats() {
                        if let TransitionFrontierSyncState::BlocksPending { chain, .. } =
                            &store.state.get().transition_frontier.sync
                        {
                            stats.syncing_blocks_init(chain);
                        }
                    }
                }
                TransitionFrontierSyncAction::BlocksPeersQuery => {}
                TransitionFrontierSyncAction::BlocksPeerQueryInit { .. } => {}
                TransitionFrontierSyncAction::BlocksPeerQueryRetry { .. } => {}
                TransitionFrontierSyncAction::BlocksPeerQueryPending { ref hash, .. } => {
                    if let Some(stats) = store.service.stats() {
                        if let Some(state) =
                            store.state.get().transition_frontier.sync.block_state(hash)
                        {
                            stats.syncing_block_update(state);
                        }
                    }
                }
                TransitionFrontierSyncAction::BlocksPeerQueryError { .. } => {}
                TransitionFrontierSyncAction::BlocksPeerQuerySuccess { .. } => {}
                TransitionFrontierSyncAction::BlocksFetchSuccess { ref hash } => {
                    if let Some(stats) = store.service.stats() {
                        if let Some(state) =
                            store.state.get().transition_frontier.sync.block_state(hash)
                        {
                            stats.syncing_block_update(state);
                        }
                    }
                }
                TransitionFrontierSyncAction::BlocksNextApplyInit => {}
                TransitionFrontierSyncAction::BlocksNextApplyPending { ref hash } => {
                    if let Some(stats) = store.service.stats() {
                        if let Some(state) =
                            store.state.get().transition_frontier.sync.block_state(hash)
                        {
                            stats.syncing_block_update(state);
                        }
                    }
                }
                TransitionFrontierSyncAction::BlocksNextApplyError { .. } => {}
                TransitionFrontierSyncAction::BlocksNextApplySuccess { ref hash } => {
                    if let Some(stats) = store.service.stats() {
                        if let Some(state) =
                            store.state.get().transition_frontier.sync.block_state(hash)
                        {
                            stats.syncing_block_update(state);
                        }
                    }
                }
                TransitionFrontierSyncAction::BlocksSuccess => {
                    store.dispatch(TransitionFrontierSyncAction::CommitInit);
                }
                TransitionFrontierSyncAction::CommitInit => {}
                TransitionFrontierSyncAction::CommitPending => {}
                TransitionFrontierSyncAction::CommitSuccess { result } => {
                    // TODO(refactor): needs to be moved to the reducer in the sync module,
                    // but that will result in extra cloning until the reducers
                    // take the action by value instead of reference
                    let own_peer_id = store.state().p2p.my_id();
                    let transition_frontier = &store.state.get().transition_frontier;
                    let TransitionFrontierSyncState::CommitSuccess { chain, .. } =
                        &transition_frontier.sync
                    else {
                        return;
                    };
                    let Some(best_tip) = chain.last() else {
                        return;
                    };
                    let orphaned_snarks = transition_frontier
                        .best_chain
                        .iter()
                        .rev()
                        .take_while(|b1| {
                            let height_diff =
                                best_tip.height().saturating_sub(b1.height()) as usize;
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

                    store.dispatch(TransitionFrontierAction::Synced {
                        needed_protocol_states: result.needed_protocol_states,
                    });
                    store.dispatch(SnarkPoolAction::JobsUpdate {
                        jobs: result.available_jobs,
                        orphaned_snarks,
                    });
                    return;
                }
                TransitionFrontierSyncAction::Ledger(ref a) => {
                    handle_transition_frontier_sync_ledger_action(a.clone(), &meta, store)
                }
            }
            a.effects(&meta, store);
        }
        TransitionFrontierAction::Synced { .. } => {
            synced_effects(&meta, store);
        }
        TransitionFrontierAction::SyncFailed { .. } => {
            // TODO(SEC): disconnect/blacklist peers that caused this.
        }
    }
}

fn synced_effects<S: crate::Service>(
    meta: &redux::ActionMeta,
    store: &mut redux::Store<crate::State, S, crate::Action>,
) {
    let TransitionFrontierState {
        best_chain,
        chain_diff,
        ..
    } = &store.state.get().transition_frontier;

    let Some(best_tip) = best_chain.last() else {
        return;
    };
    if let Some(stats) = store.service.stats() {
        stats.new_best_chain(meta.time(), best_chain);
    }

    let chain_diff = chain_diff.clone();

    // publish new best tip.
    let best_tip = best_tip.clone();
    for peer_id in store.state().p2p.ready_peers() {
        store.dispatch(P2pChannelsBestTipAction::ResponseSend {
            peer_id,
            best_tip: best_tip.clone(),
        });
    }

    let best_tip_hash = best_tip.staged_ledger_hash().clone();
    store.dispatch(ConsensusAction::Prune);
    store.dispatch(BlockProducerAction::BestTipUpdate { best_tip });
    store.dispatch(TransactionPoolAction::BestTipChanged {
        best_tip_hash: best_tip_hash.clone(),
    });
    if let Some(diff) = chain_diff {
        store.dispatch(TransactionPoolAction::ApplyTransitionFrontierDiff {
            best_tip_hash,
            diff,
        });
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
        TransitionFrontierSyncLedgerAction::Init => {
            transition_frontier_sync_ledger_init_effects(meta, store);
        }
        TransitionFrontierSyncLedgerAction::Snarked(a) => {
            match a {
                TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressInit {
                    ref address,
                    ..
                } => {
                    if let Some(stats) = store.service.stats() {
                        let (start, end) = (meta.time(), meta.time());
                        if let Some(kind) = store
                            .state
                            .get()
                            .transition_frontier
                            .sync
                            .ledger_target_kind()
                        {
                            if address.length() < LEDGER_DEPTH - ACCOUNT_SUBTREE_HEIGHT {
                                stats.syncing_ledger(
                                    kind,
                                    SyncingLedger::FetchHashes { start, end },
                                );
                            } else {
                                stats.syncing_ledger(
                                    kind,
                                    SyncingLedger::FetchAccounts { start, end },
                                );
                            }
                        }
                    }
                }
                TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressSuccess {
                    peer_id,
                    rpc_id,
                    ref response,
                } => {
                    if let Some(stats) = store.service.stats() {
                        if let Some((kind, start, end)) = store
                            .state
                            .get()
                            .transition_frontier
                            .sync
                            .ledger()
                            .and_then(|s| s.snarked())
                            .and_then(|s| {
                                Some((s.target().kind, s.peer_address_query_get(&peer_id, rpc_id)?))
                            })
                            .map(|(kind, (_, s))| (kind, s.time, meta.time()))
                        {
                            if response.is_child_hashes() {
                                stats.syncing_ledger(
                                    kind,
                                    SyncingLedger::FetchHashes { start, end },
                                );
                            } else if response.is_child_accounts() {
                                stats.syncing_ledger(
                                    kind,
                                    SyncingLedger::FetchAccounts { start, end },
                                );
                            }
                        }
                    }
                }
                TransitionFrontierSyncLedgerSnarkedAction::Success => {
                    transition_frontier_sync_ledger_snarked_success_effects(meta, store);
                }
                _ => {}
            }
            a.effects(meta, store);
        }
        TransitionFrontierSyncLedgerAction::Staged(a) => {
            // TODO(refactor): these should be handled with callbacks or something
            match a {
                TransitionFrontierSyncLedgerStagedAction::PartsFetchPending => {
                    if let Some(stats) = store.service.stats() {
                        if let Some(kind) = store
                            .state
                            .get()
                            .transition_frontier
                            .sync
                            .ledger_target_kind()
                        {
                            let (start, end) = (meta.time(), None);
                            stats.syncing_ledger(kind, SyncingLedger::FetchParts { start, end });
                        }
                    }
                }
                TransitionFrontierSyncLedgerStagedAction::PartsFetchSuccess { .. } => {
                    if let Some(stats) = store.service.stats() {
                        let (start, end) = (Timestamp::ZERO, Some(meta.time()));
                        if let Some(kind) = store
                            .state
                            .get()
                            .transition_frontier
                            .sync
                            .ledger_target_kind()
                        {
                            stats.syncing_ledger(kind, SyncingLedger::FetchParts { start, end });
                        }
                    }
                }
                TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchError {
                    ref error, ..
                } => {
                    if let Some(stats) = store.service.stats() {
                        stats.staging_ledger_fetch_failure(error, meta.time());
                    }
                }
                TransitionFrontierSyncLedgerStagedAction::ReconstructInit { .. } => {
                    if let Some(stats) = store.service.stats() {
                        let (start, end) = (meta.time(), None);
                        if let Some(kind) = store
                            .state
                            .get()
                            .transition_frontier
                            .sync
                            .ledger_target_kind()
                        {
                            stats.syncing_ledger(kind, SyncingLedger::ApplyParts { start, end });
                        }
                    }
                }
                TransitionFrontierSyncLedgerStagedAction::ReconstructSuccess { .. } => {
                    if let Some(stats) = store.service.stats() {
                        let (start, end) = (Timestamp::ZERO, Some(meta.time()));
                        if let Some(kind) = store
                            .state
                            .get()
                            .transition_frontier
                            .sync
                            .ledger_target_kind()
                        {
                            stats.syncing_ledger(kind, SyncingLedger::ApplyParts { start, end });
                        }
                    }
                }
                TransitionFrontierSyncLedgerStagedAction::Success => {
                    // TODO(refactor): this one in particular must be a callback, others
                    // are just stats updates
                    transition_frontier_sync_ledger_staged_success_effects(meta, store);
                }
                _ => {}
            }
        }
        TransitionFrontierSyncLedgerAction::Success => {
            match &store.state().transition_frontier.sync {
                TransitionFrontierSyncState::StakingLedgerPending { .. } => {
                    store.dispatch(TransitionFrontierSyncAction::LedgerStakingSuccess);
                }
                TransitionFrontierSyncState::NextEpochLedgerPending { .. } => {
                    store.dispatch(TransitionFrontierSyncAction::LedgerNextEpochSuccess);
                }
                TransitionFrontierSyncState::RootLedgerPending { .. } => {
                    store.dispatch(TransitionFrontierSyncAction::LedgerRootSuccess);
                }
                _ => {}
            }
        }
    }
}
