use mina_p2p_messages::v2::MinaLedgerSyncLedgerQueryStableV1;
use p2p::channels::rpc::{P2pChannelsRpcRequestSendAction, P2pRpcRequest};
use p2p::PeerId;
use redux::ActionMeta;

use crate::ledger::{
    LedgerAddress, LedgerChildAccountsAddAction, LedgerChildHashesAddAction, LedgerId, LEDGER_DEPTH,
};
use crate::Store;

use super::{
    PeerLedgerQueryResponse, TransitionFrontierSyncLedgerInitAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryErrorAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryInitAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryPendingAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryRetryAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQuerySuccessAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPendingAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncSuccessAction,
};

// SnarkedLedgerSyncSuccess(TransitionFrontierSyncLedgerSnarkedLedgerSyncSuccessAction),
// StagedLedgerPartsFetchInit(TransitionFrontierSyncLedgerStagedLedgerPartsFetchInitAction),
// StagedLedgerPartsFetchPending(TransitionFrontierSyncLedgerStagedLedgerPartsFetchPendingAction),
// StagedLedgerPartsFetchSuccess(TransitionFrontierSyncLedgerStagedLedgerPartsFetchSuccessAction),
// StagedLedgerReconstructInit(TransitionFrontierSyncLedgerStagedLedgerReconstructInitAction),
// StagedLedgerReconstructPending(
//     TransitionFrontierSyncLedgerStagedLedgerReconstructPendingAction,
// ),
// StagedLedgerReconstructSuccess(
//     TransitionFrontierSyncLedgerStagedLedgerReconstructSuccessAction,
// ),
// Success(TransitionFrontierSyncLedgerSuccessAction),

impl TransitionFrontierSyncLedgerInitAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerSnarkedLedgerSyncPendingAction {});
    }
}

impl TransitionFrontierSyncLedgerSnarkedLedgerSyncPendingAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction {});
    }
}

impl TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        // TODO(binier): make sure they have the ledger we want to query.
        let peer_ids = store
            .state()
            .p2p
            .ready_peers_iter()
            .filter(|(_, p)| p.channels.rpc.can_send_request())
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();

        let mut retry_addresses = store
            .state()
            .transition_frontier
            .sync
            .root_ledger()
            .map_or(vec![], |s| s.snarked_ledger_sync_retry_iter().collect());
        retry_addresses.reverse();

        for peer_id in peer_ids {
            if let Some(address) = retry_addresses.pop() {
                if store.dispatch(
                    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryRetryAction {
                        peer_id,
                        address,
                    },
                ) {
                    continue;
                }
            }

            let address = store
                .state()
                .transition_frontier
                .sync
                .root_ledger()
                .and_then(|s| s.snarked_ledger_sync_next());
            match address {
                Some(address) => {
                    store.dispatch(
                        TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryInitAction {
                            peer_id,
                            address,
                        },
                    );
                }
                None => break,
            }
        }
    }
}

fn query_peer_init<S: redux::Service>(
    store: &mut Store<S>,
    peer_id: PeerId,
    address: LedgerAddress,
) {
    let Some((ledger_hash, rpc_id)) = None.or_else(|| {
            let state = store.state();
            let root_ledger = state.transition_frontier.sync.root_ledger()?;
            let ledger_hash = root_ledger.snarked_ledger_hash();

            let p = store.state().p2p.get_ready_peer(&peer_id)?;
            let rpc_id = p.channels.rpc.next_local_rpc_id();

            Some((ledger_hash, rpc_id))
        }) else { return };

    let query = if address.length() >= LEDGER_DEPTH - 1 {
        MinaLedgerSyncLedgerQueryStableV1::WhatContents(address.clone().into())
    } else {
        MinaLedgerSyncLedgerQueryStableV1::WhatChildHashes(address.clone().into())
    };

    if store.dispatch(P2pChannelsRpcRequestSendAction {
        peer_id,
        id: rpc_id,
        request: P2pRpcRequest::LedgerQuery(ledger_hash, query),
    }) {
        store.dispatch(
            TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryPendingAction {
                address,
                peer_id,
                rpc_id,
            },
        );
    }
}

impl TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryInitAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        query_peer_init(store, self.peer_id, self.address);
    }
}

impl TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryRetryAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        query_peer_init(store, self.peer_id, self.address);
    }
}

impl TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryErrorAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction {});
    }
}

impl TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQuerySuccessAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        let Some(root_ledger) = store.state().transition_frontier.sync.root_ledger() else { return };
        let Some((addr, _)) = root_ledger.snarked_ledger_peer_query_get(&self.peer_id, self.rpc_id) else { return };

        match self.response {
            PeerLedgerQueryResponse::ChildHashes(left, right) => {
                store.dispatch(LedgerChildHashesAddAction {
                    ledger_id: LedgerId::root_snarked_ledger(root_ledger.snarked_ledger_hash()),
                    parent: addr.clone(),
                    hashes: (left, right),
                });
            }
            PeerLedgerQueryResponse::Accounts(accounts) => {
                store.dispatch(LedgerChildAccountsAddAction {
                    ledger_id: LedgerId::root_snarked_ledger(root_ledger.snarked_ledger_hash()),
                    parent: addr.clone(),
                    accounts,
                });
            }
        }

        if !store.dispatch(TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction {}) {
            store.dispatch(TransitionFrontierSyncLedgerSnarkedLedgerSyncSuccessAction {});
        }
    }
}
