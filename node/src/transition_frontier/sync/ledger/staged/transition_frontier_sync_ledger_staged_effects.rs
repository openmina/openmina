use redux::ActionMeta;

use crate::p2p::channels::rpc::{P2pChannelsRpcRequestSendAction, P2pRpcRequest};
use crate::Store;

use super::{
    TransitionFrontierSyncLedgerStagedPartsFetchPendingAction,
    TransitionFrontierSyncLedgerStagedPartsFetchSuccessAction,
    TransitionFrontierSyncLedgerStagedPartsPeerFetchErrorAction,
    TransitionFrontierSyncLedgerStagedPartsPeerFetchInitAction,
    TransitionFrontierSyncLedgerStagedPartsPeerFetchPendingAction,
    TransitionFrontierSyncLedgerStagedPartsPeerFetchSuccessAction,
    TransitionFrontierSyncLedgerStagedPartsPeerInvalidAction,
    TransitionFrontierSyncLedgerStagedPartsPeerValidAction,
    TransitionFrontierSyncLedgerStagedReconstructEmptyAction,
    TransitionFrontierSyncLedgerStagedReconstructErrorAction,
    TransitionFrontierSyncLedgerStagedReconstructInitAction,
    TransitionFrontierSyncLedgerStagedReconstructPendingAction,
    TransitionFrontierSyncLedgerStagedReconstructSuccessAction,
    TransitionFrontierSyncLedgerStagedService, TransitionFrontierSyncLedgerStagedSuccessAction,
};

impl TransitionFrontierSyncLedgerStagedPartsFetchPendingAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerStagedPartsPeerFetchInitAction {});
    }
}

impl TransitionFrontierSyncLedgerStagedPartsPeerFetchInitAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        let state = store.state();
        let Some(staged_ledger) =
            None.or_else(|| state.transition_frontier.sync.ledger()?.staged())
        else {
            return;
        };
        let block_hash = staged_ledger.target().staged.block_hash.clone();

        let ready_peers = staged_ledger
            .filter_available_peers(state.p2p.ready_rpc_peers_iter())
            .collect::<Vec<_>>();

        for (peer_id, rpc_id) in ready_peers {
            // TODO(binier): maybe
            if store.dispatch(P2pChannelsRpcRequestSendAction {
                peer_id,
                id: rpc_id,
                request: P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(
                    block_hash.clone(),
                ),
            }) {
                store.dispatch(
                    TransitionFrontierSyncLedgerStagedPartsPeerFetchPendingAction {
                        peer_id,
                        rpc_id,
                    },
                );
                break;
            }
        }
    }
}

impl TransitionFrontierSyncLedgerStagedPartsPeerFetchErrorAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerStagedPartsPeerFetchInitAction {});
    }
}

impl TransitionFrontierSyncLedgerStagedPartsPeerFetchSuccessAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        if !store.dispatch(TransitionFrontierSyncLedgerStagedPartsPeerValidAction {
            sender: self.peer_id,
        }) {
            store.dispatch(TransitionFrontierSyncLedgerStagedPartsPeerInvalidAction {
                sender: self.peer_id,
                parts: self.parts,
            });
        }
    }
}

impl TransitionFrontierSyncLedgerStagedPartsPeerInvalidAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerStagedPartsPeerFetchInitAction {});
    }
}

impl TransitionFrontierSyncLedgerStagedPartsPeerValidAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerStagedPartsFetchSuccessAction {
            sender: self.sender,
        });
    }
}

impl TransitionFrontierSyncLedgerStagedPartsFetchSuccessAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerStagedReconstructInitAction {});
    }
}

impl TransitionFrontierSyncLedgerStagedReconstructEmptyAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerStagedReconstructInitAction {});
    }
}

impl TransitionFrontierSyncLedgerStagedReconstructInitAction {
    pub fn effects<S>(self, _: &ActionMeta, store: &mut Store<S>)
    where
        S: TransitionFrontierSyncLedgerStagedService,
    {
        let ledger_state = store.state().transition_frontier.sync.ledger();
        let Some((target, parts)) = ledger_state.and_then(|s| s.staged()?.target_with_parts())
        else {
            return;
        };
        let snarked_ledger_hash = target.snarked_ledger_hash.clone();
        let parts = parts.cloned();

        store.dispatch(TransitionFrontierSyncLedgerStagedReconstructPendingAction {});

        match store
            .service
            .staged_ledger_reconstruct(snarked_ledger_hash, parts)
        {
            Err(error) => {
                store.dispatch(TransitionFrontierSyncLedgerStagedReconstructErrorAction { error });
            }
            Ok(_) => {
                store.dispatch(TransitionFrontierSyncLedgerStagedReconstructSuccessAction {});
            }
        }
    }
}

impl TransitionFrontierSyncLedgerStagedReconstructSuccessAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerStagedSuccessAction {});
    }
}
