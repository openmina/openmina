use p2p::channels::rpc::P2pChannelsRpcAction;
use redux::ActionMeta;

use crate::p2p::channels::rpc::P2pRpcRequest;
use crate::transition_frontier::TransitionFrontierService;
use crate::Store;

use super::ledger::snarked::TransitionFrontierSyncLedgerSnarkedAction;
use super::ledger::staged::TransitionFrontierSyncLedgerStagedAction;
use super::ledger::TransitionFrontierSyncLedgerAction;
use super::TransitionFrontierSyncAction;

impl TransitionFrontierSyncAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>)
    where
        S: TransitionFrontierService,
    {
        match self {
            TransitionFrontierSyncAction::Init { .. } => {
                store.dispatch(TransitionFrontierSyncAction::LedgerStakingPending);
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
                store.dispatch(TransitionFrontierSyncLedgerAction::Init);
            }
            TransitionFrontierSyncAction::LedgerStakingSuccess => {
                if store.dispatch(TransitionFrontierSyncAction::LedgerNextEpochPending) {
                } else if store.dispatch(TransitionFrontierSyncAction::LedgerRootPending) {
                }
            }
            TransitionFrontierSyncAction::LedgerNextEpochPending => {
                store.dispatch(TransitionFrontierSyncLedgerAction::Init);
            }
            TransitionFrontierSyncAction::LedgerNextEpochSuccess => {
                store.dispatch(TransitionFrontierSyncAction::LedgerRootPending);
            }
            TransitionFrontierSyncAction::LedgerRootPending => {
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
