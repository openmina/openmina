use openmina_core::block::ArcBlockWithHash;
use p2p::channels::rpc::{P2pChannelsRpcAction, P2pRpcId};
use p2p::PeerId;
use redux::ActionMeta;

use crate::ledger::write::{LedgerWriteAction, LedgerWriteRequest};
use crate::p2p::channels::rpc::P2pRpcRequest;
use crate::service::TransitionFrontierSyncLedgerSnarkedService;
use crate::{p2p_ready, Service, Store};

use super::ledger::snarked::TransitionFrontierSyncLedgerSnarkedAction;
use super::ledger::staged::TransitionFrontierSyncLedgerStagedAction;
use super::ledger::{SyncLedgerTarget, TransitionFrontierSyncLedgerAction};
use super::{TransitionFrontierSyncAction, TransitionFrontierSyncState};

impl TransitionFrontierSyncAction {
    pub fn effects<S>(&self, meta: &ActionMeta, store: &mut Store<S>)
    where
        S: Service,
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
                let p2p = p2p_ready!(store.state().p2p, meta.time());
                // TODO(binier): make sure they have the ledger we want to query.
                let mut peer_ids = p2p
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
                let p2p = p2p_ready!(store.state().p2p, meta.time());
                let Some(rpc_id) = p2p
                    .get_ready_peer(peer_id)
                    .map(|v| v.channels.next_local_rpc_id())
                else {
                    return;
                };

                store.dispatch(P2pChannelsRpcAction::RequestSend {
                    peer_id: *peer_id,
                    id: rpc_id,
                    request: Box::new(P2pRpcRequest::Block(hash.clone())),
                    on_init: Some(redux::callback!(
                        on_send_p2p_block_rpc_request(
                            (peer_id: PeerId, rpc_id: P2pRpcId, request: P2pRpcRequest)
                        ) -> crate::Action {
                            let P2pRpcRequest::Block(hash) = request else {
                                unreachable!()
                            };
                            TransitionFrontierSyncAction::BlocksPeerQueryPending {
                                hash,
                                peer_id,
                                rpc_id,
                            }
                        }
                    )),
                });
            }
            TransitionFrontierSyncAction::BlocksPeerQueryRetry { hash, peer_id } => {
                let p2p = p2p_ready!(store.state().p2p, meta.time());
                let Some(rpc_id) = p2p
                    .get_ready_peer(peer_id)
                    .map(|v| v.channels.next_local_rpc_id())
                else {
                    return;
                };

                store.dispatch(P2pChannelsRpcAction::RequestSend {
                    peer_id: *peer_id,
                    id: rpc_id,
                    request: Box::new(P2pRpcRequest::Block(hash.clone())),
                    on_init: Some(redux::callback!(
                        on_send_p2p_block_rpc_request_retry(
                            (peer_id: PeerId, rpc_id: P2pRpcId, request: P2pRpcRequest)
                        ) -> crate::Action {
                            let P2pRpcRequest::Block(hash) = request else {
                                unreachable!()
                            };
                            TransitionFrontierSyncAction::BlocksPeerQueryPending {
                                hash,
                                peer_id,
                                rpc_id,
                            }
                        }
                    )),
                });
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
                store.dispatch(TransitionFrontierSyncAction::BlocksNextApplyInit {});
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

                if let Some(stats) = store.service.stats() {
                    stats.block_producer().block_apply_start(meta.time(), &hash);
                }

                store.dispatch(LedgerWriteAction::Init {
                    request: LedgerWriteRequest::BlockApply { block, pred_block },
                    on_init: redux::callback!(
                        on_block_next_apply_init(request: LedgerWriteRequest) -> crate::Action {
                            let LedgerWriteRequest::BlockApply {
                                block,
                                pred_block: _,
                            } = request
                            else {
                                unreachable!()
                            };
                            let hash = block.hash().clone();
                            TransitionFrontierSyncAction::BlocksNextApplyPending { hash }
                        }
                    ),
                });
            }
            TransitionFrontierSyncAction::BlocksNextApplyPending { .. } => {}
            TransitionFrontierSyncAction::BlocksNextApplySuccess { hash } => {
                if let Some(stats) = store.service.stats() {
                    stats.block_producer().block_apply_end(meta.time(), hash);
                }

                if !store.dispatch(TransitionFrontierSyncAction::BlocksNextApplyInit) {
                    store.dispatch(TransitionFrontierSyncAction::BlocksSuccess);
                }
            }
            TransitionFrontierSyncAction::BlocksSuccess => {}
            // Bootstrap/Catchup is practically complete at this point.
            // This effect is where the finalization part needs to be
            // executed, which is mostly to grab some data that we need
            // from previous chain, before it's discarded after dispatching
            // `TransitionFrontierSyncedAction`.
            TransitionFrontierSyncAction::CommitInit => {
                let transition_frontier = &store.state.get().transition_frontier;
                let TransitionFrontierSyncState::BlocksSuccess {
                    chain,
                    root_snarked_ledger_updates,
                    needed_protocol_states,
                    ..
                } = &transition_frontier.sync
                else {
                    return;
                };
                let Some(new_root) = chain.first() else {
                    return;
                };
                let Some(new_best_tip) = chain.last() else {
                    return;
                };
                let ledgers_to_keep = chain
                    .iter()
                    .flat_map(|b| {
                        [
                            b.snarked_ledger_hash(),
                            b.staged_ledger_hash(),
                            b.staking_epoch_ledger_hash(),
                            b.next_epoch_ledger_hash(),
                        ]
                    })
                    .cloned()
                    .collect();
                let mut root_snarked_ledger_updates = root_snarked_ledger_updates.clone();
                if transition_frontier
                    .best_chain
                    .iter()
                    .any(|b| b.hash() == new_root.hash())
                {
                    root_snarked_ledger_updates
                        .extend_with_needed(new_root, &transition_frontier.best_chain);
                }

                let needed_protocol_states = if root_snarked_ledger_updates.is_empty() {
                    // We don't need protocol states unless we need to
                    // recreate some snarked ledgers during `commit`.
                    Default::default()
                } else {
                    needed_protocol_states
                        .iter()
                        .chain(&transition_frontier.needed_protocol_states)
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                };

                store.dispatch(LedgerWriteAction::Init {
                    request: LedgerWriteRequest::Commit {
                        ledgers_to_keep,
                        root_snarked_ledger_updates,
                        needed_protocol_states,
                        new_root: new_root.clone(),
                        new_best_tip: new_best_tip.clone(),
                    },
                    on_init: redux::callback!(
                        on_frontier_commit_init(_request: LedgerWriteRequest) -> crate::Action {
                            TransitionFrontierSyncAction::CommitPending
                        }
                    ),
                });
            }
            TransitionFrontierSyncAction::CommitPending => {}
            TransitionFrontierSyncAction::CommitSuccess { .. } => {
                unreachable!("handled in parent effects to avoid cloning")
            }
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
    S: TransitionFrontierSyncLedgerSnarkedService,
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
    S: TransitionFrontierSyncLedgerSnarkedService,
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
    S: TransitionFrontierSyncLedgerSnarkedService,
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
    S: TransitionFrontierSyncLedgerSnarkedService,
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
