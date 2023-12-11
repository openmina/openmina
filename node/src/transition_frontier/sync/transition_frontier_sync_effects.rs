use p2p::channels::rpc::P2pChannelsRpcRequestSendAction;
use redux::ActionMeta;

use crate::p2p::channels::rpc::P2pRpcRequest;
use crate::transition_frontier::TransitionFrontierService;
use crate::Store;

use super::ledger::snarked::TransitionFrontierSyncLedgerSnarkedPeersQueryAction;
use super::ledger::staged::TransitionFrontierSyncLedgerStagedPartsFetchPendingAction;
use super::ledger::TransitionFrontierSyncLedgerInitAction;
use super::{
    TransitionFrontierSyncBestTipUpdateAction, TransitionFrontierSyncBlocksFetchSuccessAction,
    TransitionFrontierSyncBlocksNextApplyInitAction,
    TransitionFrontierSyncBlocksNextApplyPendingAction,
    TransitionFrontierSyncBlocksNextApplySuccessAction,
    TransitionFrontierSyncBlocksPeerQueryErrorAction,
    TransitionFrontierSyncBlocksPeerQueryInitAction,
    TransitionFrontierSyncBlocksPeerQueryPendingAction,
    TransitionFrontierSyncBlocksPeerQueryRetryAction,
    TransitionFrontierSyncBlocksPeerQuerySuccessAction,
    TransitionFrontierSyncBlocksPeersQueryAction, TransitionFrontierSyncBlocksPendingAction,
    TransitionFrontierSyncBlocksSuccessAction, TransitionFrontierSyncInitAction,
    TransitionFrontierSyncLedgerNextEpochPendingAction,
    TransitionFrontierSyncLedgerNextEpochSuccessAction,
    TransitionFrontierSyncLedgerRootPendingAction, TransitionFrontierSyncLedgerRootSuccessAction,
    TransitionFrontierSyncLedgerStakingPendingAction,
    TransitionFrontierSyncLedgerStakingSuccessAction,
};

impl TransitionFrontierSyncInitAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerStakingPendingAction {});
    }
}

impl TransitionFrontierSyncBestTipUpdateAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>) {
        // if root snarked ledger changed.
        store.dispatch(TransitionFrontierSyncLedgerInitAction {});
        // if root snarked ledger stayed same but root block changed
        // while reconstructing staged ledger.
        store.dispatch(TransitionFrontierSyncLedgerStagedPartsFetchPendingAction {});
        store.dispatch(TransitionFrontierSyncLedgerSnarkedPeersQueryAction {});
        // if we don't need to sync root staged ledger.
        store.dispatch(TransitionFrontierSyncBlocksPeersQueryAction {});
        // if we already have a block ready to be applied.
        store.dispatch(TransitionFrontierSyncBlocksNextApplyInitAction {});

        // TODO(binier): cleanup ledgers
    }
}

impl TransitionFrontierSyncLedgerStakingPendingAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerInitAction {});
    }
}

impl TransitionFrontierSyncLedgerStakingSuccessAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerNextEpochPendingAction {});
    }
}

impl TransitionFrontierSyncLedgerNextEpochPendingAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerInitAction {});
    }
}

impl TransitionFrontierSyncLedgerNextEpochSuccessAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerRootPendingAction {});
    }
}

impl TransitionFrontierSyncLedgerRootPendingAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerInitAction {});
    }
}

impl TransitionFrontierSyncLedgerRootSuccessAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncBlocksPendingAction {});
    }
}

impl TransitionFrontierSyncBlocksPendingAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>) {
        if !store.dispatch(TransitionFrontierSyncBlocksSuccessAction {}) {
            store.dispatch(TransitionFrontierSyncBlocksPeersQueryAction {});
        }
    }
}

impl TransitionFrontierSyncBlocksPeersQueryAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>) {
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
}

impl TransitionFrontierSyncBlocksPeerQueryInitAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        let Some(rpc_id) = store
            .state()
            .p2p
            .get_ready_peer(&self.peer_id)
            .map(|v| v.channels.rpc.next_local_rpc_id())
        else {
            return;
        };

        if store.dispatch(P2pChannelsRpcRequestSendAction {
            peer_id: self.peer_id,
            id: rpc_id,
            request: P2pRpcRequest::Block(self.hash.clone()),
        }) {
            store.dispatch(TransitionFrontierSyncBlocksPeerQueryPendingAction {
                hash: self.hash,
                peer_id: self.peer_id,
                rpc_id,
            });
        }
    }
}

impl TransitionFrontierSyncBlocksPeerQueryRetryAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        let Some(rpc_id) = store
            .state()
            .p2p
            .get_ready_peer(&self.peer_id)
            .map(|v| v.channels.rpc.next_local_rpc_id())
        else {
            return;
        };

        if store.dispatch(P2pChannelsRpcRequestSendAction {
            peer_id: self.peer_id,
            id: rpc_id,
            request: P2pRpcRequest::Block(self.hash.clone()),
        }) {
            store.dispatch(TransitionFrontierSyncBlocksPeerQueryPendingAction {
                hash: self.hash,
                peer_id: self.peer_id,
                rpc_id,
            });
        }
    }
}

impl TransitionFrontierSyncBlocksPeerQueryErrorAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncBlocksPeersQueryAction {});
    }
}

impl TransitionFrontierSyncBlocksPeerQuerySuccessAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncBlocksPeersQueryAction {});
        store.dispatch(TransitionFrontierSyncBlocksFetchSuccessAction {
            hash: self.response.hash,
        });
    }
}

impl TransitionFrontierSyncBlocksFetchSuccessAction {
    pub fn effects<S: redux::Service>(&self, _: &ActionMeta, store: &mut Store<S>) {
        let _ = store;
        // TODO(binier): uncomment once ledger communication is async.
        // store.dispatch(TransitionFrontierSyncBlocksNextApplyInitAction {});
    }
}

impl TransitionFrontierSyncBlocksNextApplyInitAction {
    pub fn effects<S>(&self, _: &ActionMeta, store: &mut Store<S>)
    where
        S: TransitionFrontierService,
    {
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

        store.dispatch(TransitionFrontierSyncBlocksNextApplyPendingAction { hash: hash.clone() });
        store.service.block_apply(block, pred_block).unwrap();

        store.dispatch(TransitionFrontierSyncBlocksNextApplySuccessAction { hash });
    }
}

impl TransitionFrontierSyncBlocksNextApplySuccessAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        // TODO(binier): uncomment once ledger communication is async.
        // if !store.dispatch(TransitionFrontierSyncBlockNextApplyInitAction {}) {
        store.dispatch(TransitionFrontierSyncBlocksSuccessAction {});
        // }
    }
}
