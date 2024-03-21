use p2p::channels::rpc::P2pChannelsRpcAction;
use redux::ActionMeta;

use crate::p2p::channels::rpc::P2pRpcRequest;
use crate::service::TransitionFrontierSyncLedgerSnarkedService;
use crate::transition_frontier::TransitionFrontierService;
use crate::Store;

use super::ledger::snarked::TransitionFrontierSyncLedgerSnarkedAction;
use super::ledger::staged::TransitionFrontierSyncLedgerStagedAction;
use super::ledger::TransitionFrontierSyncLedgerAction;
use super::TransitionFrontierSyncAction;

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
            TransitionFrontierSyncAction::BestTipUpdate { .. } => {
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
            TransitionFrontierSyncAction::LedgerStakingPending => {
                // The staking ledger is equal to the genesis ledger with changes on top, so
                // we use it as a base to save work during synchronization.
                let best_tip = store.state().transition_frontier.sync.best_tip().unwrap();
                let target = super::ledger::SyncLedgerTarget::staking_epoch(best_tip);
                let origin = best_tip.genesis_ledger_hash().clone();
                let target = target.snarked_ledger_hash;
                // TODO: for the async ledger this should be handled in intermediary action
                store
                    .service()
                    .copy_snarked_ledger_contents(origin, target, false)
                    .unwrap();
                store.dispatch(TransitionFrontierSyncLedgerAction::Init);
            }
            TransitionFrontierSyncAction::LedgerStakingSuccess => {
                if store.dispatch(TransitionFrontierSyncAction::LedgerNextEpochPending) {
                } else if store.dispatch(TransitionFrontierSyncAction::LedgerRootPending) {
                }
            }
            TransitionFrontierSyncAction::LedgerNextEpochPending => {
                // The next epoch ledger is equal to the staking ledger with changes on top, so
                // we use it as a base to save work during synchronization.
                let sync = &store.state().transition_frontier.sync;
                let best_tip = sync.best_tip().unwrap();
                let root_block = sync.root_block().unwrap();
                let Some(next_epoch_sync) =
                    super::ledger::SyncLedgerTarget::next_epoch(best_tip, root_block)
                else {
                    return;
                };
                let origin =
                    super::ledger::SyncLedgerTarget::staking_epoch(best_tip).snarked_ledger_hash;
                let target = next_epoch_sync.snarked_ledger_hash;
                // TODO: for the async ledger this should be handled in intermediary action
                store
                    .service()
                    .copy_snarked_ledger_contents(origin, target, false)
                    .unwrap();

                store.dispatch(TransitionFrontierSyncLedgerAction::Init);
            }
            TransitionFrontierSyncAction::LedgerNextEpochSuccess => {
                store.dispatch(TransitionFrontierSyncAction::LedgerRootPending);
            }
            TransitionFrontierSyncAction::LedgerRootPending => {
                // The transition frontier root ledger is equal to the next epoch ledger with changes
                // on top, so we use it as a base to save work during synchronization.
                let sync = &store.state().transition_frontier.sync;
                let best_tip = sync.best_tip().unwrap();
                let root_block = sync.root_block().unwrap();
                let next_epoch_sync =
                    super::ledger::SyncLedgerTarget::next_epoch(best_tip, root_block)
                        .unwrap_or_else(|| {
                            super::ledger::SyncLedgerTarget::staking_epoch(best_tip)
                        });
                let origin = next_epoch_sync.snarked_ledger_hash;
                let target = root_block.snarked_ledger_hash().clone();
                // TODO: for the async ledger this should be handled in intermediary action
                store
                    .service()
                    .copy_snarked_ledger_contents(origin, target, false)
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
