use mina_p2p_messages::v2::MinaLedgerSyncLedgerQueryStableV1;
use p2p::channels::rpc::{P2pChannelsRpcRequestSendAction, P2pRpcRequest};
use p2p::PeerId;
use redux::ActionMeta;

use crate::ledger::{LedgerAddress, LEDGER_DEPTH};
use crate::Store;

use super::{
    PeerLedgerQueryResponse, TransitionFrontierSyncLedgerInitAction,
    TransitionFrontierSyncLedgerService,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncChildAccountsReceivedAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncChildHashesReceivedAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryErrorAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryInitAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryPendingAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryRetryAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQuerySuccessAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPendingAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncSuccessAction,
    TransitionFrontierSyncLedgerStagedLedgerPartsApplyInitAction,
    TransitionFrontierSyncLedgerStagedLedgerPartsApplySuccessAction,
    TransitionFrontierSyncLedgerStagedLedgerPartsFetchErrorAction,
    TransitionFrontierSyncLedgerStagedLedgerPartsFetchInitAction,
    TransitionFrontierSyncLedgerStagedLedgerPartsFetchPendingAction,
    TransitionFrontierSyncLedgerStagedLedgerPartsFetchSuccessAction,
    TransitionFrontierSyncLedgerStagedLedgerReconstructPendingAction,
    TransitionFrontierSyncLedgerStagedLedgerReconstructSuccessAction,
    TransitionFrontierSyncLedgerSuccessAction,
};

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

        Some((ledger_hash.clone(), rpc_id))
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
        let mut peer_ids = store
            .state()
            .p2p
            .ready_peers_iter()
            .filter(|(_, p)| p.channels.rpc.can_send_request())
            .map(|(id, p)| (*id, p.connected_since))
            .collect::<Vec<_>>();
        peer_ids.sort_by(|(_, t1), (_, t2)| t2.cmp(t1));

        let mut retry_addresses = store
            .state()
            .transition_frontier
            .sync
            .root_ledger()
            .map_or(vec![], |s| s.snarked_ledger_sync_retry_iter().collect());
        retry_addresses.reverse();

        for (peer_id, _) in peer_ids {
            if let Some(address) = retry_addresses.last() {
                if store.dispatch(
                    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryRetryAction {
                        peer_id,
                        address: address.clone(),
                    },
                ) {
                    retry_addresses.pop();
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
                None if retry_addresses.is_empty() => break,
                None => {}
            }
        }
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
        let address = addr.clone();

        match self.response {
            PeerLedgerQueryResponse::ChildHashes(left, right) => {
                store.dispatch(
                    TransitionFrontierSyncLedgerSnarkedLedgerSyncChildHashesReceivedAction {
                        address,
                        hashes: (left, right),
                        sender: self.peer_id,
                    },
                );
            }
            PeerLedgerQueryResponse::Accounts(accounts) => {
                store.dispatch(
                    TransitionFrontierSyncLedgerSnarkedLedgerSyncChildAccountsReceivedAction {
                        address,
                        accounts,
                        sender: self.peer_id,
                    },
                );
            }
        }
    }
}

impl TransitionFrontierSyncLedgerSnarkedLedgerSyncChildHashesReceivedAction {
    pub fn effects<S>(self, _: &ActionMeta, store: &mut Store<S>)
    where
        S: TransitionFrontierSyncLedgerService,
    {
        let Some(block) = store.state().transition_frontier.sync.root_ledger() else { return };
        let snarked_ledger_hash = block.snarked_ledger_hash().clone();
        store
            .service
            .hashes_set(snarked_ledger_hash, &self.address, self.hashes)
            .unwrap();

        if !store.dispatch(TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction {}) {
            store.dispatch(TransitionFrontierSyncLedgerSnarkedLedgerSyncSuccessAction {});
        }
    }
}

impl TransitionFrontierSyncLedgerSnarkedLedgerSyncChildAccountsReceivedAction {
    pub fn effects<S>(self, _: &ActionMeta, store: &mut Store<S>)
    where
        S: TransitionFrontierSyncLedgerService,
    {
        let Some(block) = store.state().transition_frontier.sync.root_ledger() else { return };
        let snarked_ledger_hash = block.snarked_ledger_hash().clone();
        store
            .service
            .accounts_set(snarked_ledger_hash, &self.address, self.accounts)
            .unwrap();

        if !store.dispatch(TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction {}) {
            store.dispatch(TransitionFrontierSyncLedgerSnarkedLedgerSyncSuccessAction {});
        }
    }
}

impl TransitionFrontierSyncLedgerSnarkedLedgerSyncSuccessAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerStagedLedgerReconstructPendingAction {});
    }
}

impl TransitionFrontierSyncLedgerStagedLedgerReconstructPendingAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerStagedLedgerPartsFetchInitAction {});
    }
}

impl TransitionFrontierSyncLedgerStagedLedgerPartsFetchInitAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        let state = store.state();
        let Some(root_ledger) = state.transition_frontier.sync.root_ledger() else { return };
        let root_block_hash = root_ledger.block().hash.clone();

        let ready_peers = root_ledger
            .staged_ledger_reconstruct_filter_available_peers(state.p2p.ready_rpc_peers_iter())
            .collect::<Vec<_>>();

        for (peer_id, rpc_id) in ready_peers {
            if store.dispatch(P2pChannelsRpcRequestSendAction {
                peer_id,
                id: rpc_id,
                request: P2pRpcRequest::StagedLedgerAuxAndPendingCoinbasesAtBlock(
                    root_block_hash.clone(),
                ),
            }) {
                store.dispatch(
                    TransitionFrontierSyncLedgerStagedLedgerPartsFetchPendingAction {
                        peer_id,
                        rpc_id,
                    },
                );
                break;
            }
        }
    }
}

impl TransitionFrontierSyncLedgerStagedLedgerPartsFetchErrorAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerStagedLedgerPartsFetchInitAction {});
    }
}

impl TransitionFrontierSyncLedgerStagedLedgerPartsFetchSuccessAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(
            TransitionFrontierSyncLedgerStagedLedgerPartsApplyInitAction {
                sender: self.peer_id,
                parts: self.parts,
            },
        );
    }
}

impl TransitionFrontierSyncLedgerStagedLedgerPartsApplyInitAction {
    pub fn effects<S>(self, _: &ActionMeta, store: &mut Store<S>)
    where
        S: TransitionFrontierSyncLedgerService,
    {
        let Some(block) = store.state().transition_frontier.sync.root_ledger() else { return };
        // TODO(binier): dispatch error action if error.
        store
            .service
            .staged_ledger_reconstruct(block.snarked_ledger_hash().clone(), self.parts)
            .expect("staged ledger reconstruct failed");
        store.dispatch(
            TransitionFrontierSyncLedgerStagedLedgerPartsApplySuccessAction {
                sender: self.sender,
            },
        );
    }
}

impl TransitionFrontierSyncLedgerStagedLedgerPartsApplySuccessAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerStagedLedgerReconstructSuccessAction {});
    }
}

impl TransitionFrontierSyncLedgerStagedLedgerReconstructSuccessAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerSuccessAction {});
    }
}
