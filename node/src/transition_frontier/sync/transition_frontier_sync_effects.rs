use openmina_core::block::ArcBlockWithHash;
use p2p::channels::rpc::P2pChannelsRpcAction;
use redux::ActionMeta;

use crate::p2p::channels::rpc::P2pRpcRequest;
use crate::service::TransitionFrontierSyncLedgerSnarkedService;
use crate::transition_frontier::TransitionFrontierService;
use crate::Store;

use super::ledger::snarked::TransitionFrontierSyncLedgerSnarkedAction;
use super::ledger::staged::TransitionFrontierSyncLedgerStagedAction;
use super::ledger::{SyncLedgerTarget, TransitionFrontierSyncLedgerAction};
use super::{TransitionFrontierSyncAction, TransitionFrontierSyncState};

impl TransitionFrontierSyncAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>)
    where
        S: TransitionFrontierService + TransitionFrontierSyncLedgerSnarkedService,
    {
        match self {
            TransitionFrontierSyncAction::Init { best_tip, .. } => {
                let protocol_state_body = &best_tip.block.header.protocol_state.body;
                let genesis_ledger_hash = &protocol_state_body.blockchain_state.genesis_ledger_hash;
                let staking_epoch_ledger_hash = &protocol_state_body
                    .consensus_state
                    .staking_epoch_data
                    .ledger
                    .hash;
                let next_epoch_ledger_hash = &protocol_state_body
                    .consensus_state
                    .next_epoch_data
                    .ledger
                    .hash;

                // TODO(tizoc): if root ledger matches genesis, should anything special be done?
                // snarked ledger will not need to be synced but staged ledger parts are still
                // required

                if genesis_ledger_hash != staking_epoch_ledger_hash {
                    store.dispatch(TransitionFrontierSyncAction::LedgerStakingPending);
                } else if genesis_ledger_hash != next_epoch_ledger_hash {
                    store.dispatch(TransitionFrontierSyncAction::LedgerNextEpochPending);
                } else {
                    store.dispatch(TransitionFrontierSyncAction::LedgerRootPending);
                }
            }
            TransitionFrontierSyncAction::BestTipUpdate { best_tip, .. } => {
                // TODO(tizoc): this is currently required because how how complicated the BestTipUpdate reducer is,
                // once that is simplified this should be handled in separate actions.
                maybe_copy_ledgers_for_sync(store, best_tip).unwrap();

                // if root snarked ledger changed.
                store.dispatch(TransitionFrontierSyncLedgerAction::Init);
                // if root snarked ledger stayed same but root block changed
                // while reconstructing staged ledger.
                store.dispatch(TransitionFrontierSyncLedgerStagedAction::PartsFetchPending);
                store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::PeersQuery);
                // if we don't need to sync root staged ledger.
                store.dispatch(TransitionFrontierSyncAction::BlocksPeersQuery);
                // if we already have a block ready to be applied.
                store.dispatch(TransitionFrontierSyncAction::BlocksNextApplyInit);

                // TODO(binier): cleanup ledgers
            }
            // TODO(tizoc): this action is never called with the current implementation,
            // either remove it or figure out how to recover it as a reaction to
            // `BestTipUpdate` above. Currently this logic is handled by
            // `maybe_copy_ledgers_for_sync` at the end of this file.
            // Same kind of applies to `LedgerNextEpochPending` and `LedgerRootPending`
            // in some cases, but issue is mostly about `LedgerStakingPending` because
            // it is the one most likely to be affected by the first `BestTipUpdate`
            // action processed by the state machine.
            TransitionFrontierSyncAction::LedgerStakingPending => {
                prepare_staking_epoch_ledger_for_sync(store, &sync_best_tip(store.state()))
                    .unwrap();

                store.dispatch(TransitionFrontierSyncLedgerAction::Init);
            }
            TransitionFrontierSyncAction::LedgerStakingSuccess => {
                if store.dispatch(TransitionFrontierSyncAction::LedgerNextEpochPending) {
                } else if store.dispatch(TransitionFrontierSyncAction::LedgerRootPending) {
                }
            }
            TransitionFrontierSyncAction::LedgerNextEpochPending => {
                prepare_next_epoch_ledger_for_sync(store, &sync_best_tip(store.state())).unwrap();

                store.dispatch(TransitionFrontierSyncLedgerAction::Init);
            }
            TransitionFrontierSyncAction::LedgerNextEpochSuccess => {
                store.dispatch(TransitionFrontierSyncAction::LedgerRootPending);
            }
            TransitionFrontierSyncAction::LedgerRootPending => {
                prepare_transition_frontier_root_ledger_for_sync(
                    store,
                    &sync_best_tip(store.state()),
                )
                .unwrap();

                store.dispatch(TransitionFrontierSyncLedgerAction::Init);
            }
            TransitionFrontierSyncAction::LedgerRootSuccess => {
                store.dispatch(TransitionFrontierSyncAction::BlocksPending);
            }
            TransitionFrontierSyncAction::BlocksPending => {
                if !store.dispatch(TransitionFrontierSyncAction::BlocksSuccess) {
                    store.dispatch(TransitionFrontierSyncAction::BlocksPeersQuery);
                }
            }
            TransitionFrontierSyncAction::BlocksPeersQuery => {
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
                        if store.dispatch(TransitionFrontierSyncAction::BlocksPeerQueryRetry {
                            peer_id,
                            hash: hash.clone(),
                        }) {
                            retry_hashes.pop();
                            continue;
                        }
                    }

                    match store.state().transition_frontier.sync.blocks_fetch_next() {
                        Some(hash) => {
                            store.dispatch(TransitionFrontierSyncAction::BlocksPeerQueryInit {
                                peer_id,
                                hash,
                            });
                        }
                        None if retry_hashes.is_empty() => break,
                        None => {}
                    }
                }
            }
            TransitionFrontierSyncAction::BlocksPeerQueryInit { hash, peer_id } => {
                let Some(rpc_id) = store
                    .state()
                    .p2p
                    .get_ready_peer(peer_id)
                    .map(|v| v.channels.rpc.next_local_rpc_id())
                else {
                    return;
                };

                if store.dispatch(P2pChannelsRpcAction::RequestSend {
                    peer_id: *peer_id,
                    id: rpc_id,
                    request: P2pRpcRequest::Block(hash.clone()),
                }) {
                    store.dispatch(TransitionFrontierSyncAction::BlocksPeerQueryPending {
                        hash: hash.clone(),
                        peer_id: *peer_id,
                        rpc_id,
                    });
                }
            }
            TransitionFrontierSyncAction::BlocksPeerQueryRetry { hash, peer_id } => {
                let Some(rpc_id) = store
                    .state()
                    .p2p
                    .get_ready_peer(peer_id)
                    .map(|v| v.channels.rpc.next_local_rpc_id())
                else {
                    return;
                };

                if store.dispatch(P2pChannelsRpcAction::RequestSend {
                    peer_id: *peer_id,
                    id: rpc_id,
                    request: P2pRpcRequest::Block(hash.clone()),
                }) {
                    store.dispatch(TransitionFrontierSyncAction::BlocksPeerQueryPending {
                        hash: hash.clone(),
                        peer_id: *peer_id,
                        rpc_id,
                    });
                }
            }
            TransitionFrontierSyncAction::BlocksPeerQueryPending { .. } => {}
            TransitionFrontierSyncAction::BlocksPeerQueryError { .. } => {
                store.dispatch(TransitionFrontierSyncAction::BlocksPeersQuery);
            }
            TransitionFrontierSyncAction::BlocksPeerQuerySuccess { response, .. } => {
                store.dispatch(TransitionFrontierSyncAction::BlocksPeersQuery);
                store.dispatch(TransitionFrontierSyncAction::BlocksFetchSuccess {
                    hash: response.hash.clone(),
                });
            }
            TransitionFrontierSyncAction::BlocksFetchSuccess { .. } => {
                let _ = store;
                // TODO(binier): uncomment once ledger communication is async.
                // store.dispatch(TransitionFrontierSyncBlocksNextApplyInitAction {});
            }
            TransitionFrontierSyncAction::BlocksNextApplyInit => {
                let Some((block, pred_block)) = store
                    .state()
                    .transition_frontier
                    .sync
                    .blocks_apply_next()
                    .map(|v| (v.0.clone(), v.1.clone()))
                else {
                    return;
                };
                let hash = block.hash.clone();

                store.dispatch(TransitionFrontierSyncAction::BlocksNextApplyPending {
                    hash: hash.clone(),
                });
                store.service.block_apply(block, pred_block).unwrap();

                store.dispatch(TransitionFrontierSyncAction::BlocksNextApplySuccess { hash });
            }
            TransitionFrontierSyncAction::BlocksNextApplyPending { .. } => {}
            TransitionFrontierSyncAction::BlocksNextApplySuccess { .. } => {
                // TODO(binier): uncomment once ledger communication is async.
                // if !store.dispatch(TransitionFrontierSyncAction::BlockNextApplyInit) {
                store.dispatch(TransitionFrontierSyncAction::BlocksSuccess);
                // }
            }
            TransitionFrontierSyncAction::BlocksSuccess => {}
            TransitionFrontierSyncAction::Ledger(_) => {}
        }
    }
}

// Helper functions

/// Gets from the current state the best tip sync target
fn sync_best_tip(state: &crate::State) -> ArcBlockWithHash {
    state.transition_frontier.sync.best_tip().unwrap().clone()
}

/// For snarked ledger sync targets, copy the previous snarked ledger if required
fn maybe_copy_ledgers_for_sync<S>(
    store: &mut Store<S>,
    best_tip: &ArcBlockWithHash,
) -> Result<bool, String>
where
    S: TransitionFrontierService + TransitionFrontierSyncLedgerSnarkedService,
{
    let sync = &store.state().transition_frontier.sync;

    match sync {
        TransitionFrontierSyncState::StakingLedgerPending(_) => {
            prepare_staking_epoch_ledger_for_sync(store, best_tip)
        }
        TransitionFrontierSyncState::NextEpochLedgerPending(_) => {
            prepare_next_epoch_ledger_for_sync(store, best_tip)
        }

        TransitionFrontierSyncState::RootLedgerPending(_) => {
            prepare_transition_frontier_root_ledger_for_sync(store, best_tip)
        }
        _ => Ok(true),
    }
}

/// Copies (if necessary) the genesis ledger into the sync ledger state
/// for the staking epoch ledger to use as a starting point.
fn prepare_staking_epoch_ledger_for_sync<S>(
    store: &mut Store<S>,
    best_tip: &ArcBlockWithHash,
) -> Result<bool, String>
where
    S: TransitionFrontierService + TransitionFrontierSyncLedgerSnarkedService,
{
    let target = SyncLedgerTarget::staking_epoch(best_tip).snarked_ledger_hash;
    let origin = best_tip.genesis_ledger_hash().clone();

    store
        .service()
        .copy_snarked_ledger_contents_for_sync(origin, target, false)
}

/// Copies (if necessary) the staking ledger into the sync ledger state
/// for the next epoch ledger to use as a starting point.
fn prepare_next_epoch_ledger_for_sync<S>(
    store: &mut Store<S>,
    best_tip: &ArcBlockWithHash,
) -> Result<bool, String>
where
    S: TransitionFrontierService + TransitionFrontierSyncLedgerSnarkedService,
{
    let sync = &store.state().transition_frontier.sync;
    let root_block = sync.root_block().unwrap();
    let Some(next_epoch_sync) = SyncLedgerTarget::next_epoch(best_tip, root_block) else {
        return Ok(false);
    };
    let target = next_epoch_sync.snarked_ledger_hash;
    let origin = SyncLedgerTarget::staking_epoch(best_tip).snarked_ledger_hash;

    store
        .service()
        .copy_snarked_ledger_contents_for_sync(origin, target, false)
}

/// Copies (if necessary) the next epoch ledger into the sync ledger state
/// for the transition frontier root ledger to use as a starting point.
fn prepare_transition_frontier_root_ledger_for_sync<S>(
    store: &mut Store<S>,
    best_tip: &ArcBlockWithHash,
) -> Result<bool, String>
where
    S: TransitionFrontierService + TransitionFrontierSyncLedgerSnarkedService,
{
    let sync = &store.state().transition_frontier.sync;
    let root_block = sync.root_block().unwrap();
    let next_epoch_sync = SyncLedgerTarget::next_epoch(best_tip, root_block)
        .unwrap_or_else(|| SyncLedgerTarget::staking_epoch(best_tip));
    let target = root_block.snarked_ledger_hash().clone();
    let origin = next_epoch_sync.snarked_ledger_hash;

    store
        .service()
        .copy_snarked_ledger_contents_for_sync(origin, target, false)
}
