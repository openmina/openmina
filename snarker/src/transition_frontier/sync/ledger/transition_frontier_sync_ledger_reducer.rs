use crate::ledger::{ledger_empty_hash_at_depth, LedgerAddress};

use super::{
    LedgerQueryPending, PeerLedgerQueryResponse, PeerRpcState, TransitionFrontierSyncLedgerAction,
    TransitionFrontierSyncLedgerActionWithMetaRef, TransitionFrontierSyncLedgerState,
};

impl TransitionFrontierSyncLedgerState {
    pub fn reducer(&mut self, action: TransitionFrontierSyncLedgerActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            TransitionFrontierSyncLedgerAction::Init(_) => {}
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPending(_) => {
                if let Self::Init { block, .. } = self {
                    let block = block.clone();
                    *self = Self::SnarkedLedgerSyncPending {
                        time: meta.time(),
                        block,
                        pending: Default::default(),
                        next_addr: Some(LedgerAddress::first(0)),
                    };
                }
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeersQuery(_) => {}
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQueryInit(action) => {
                if let Self::SnarkedLedgerSyncPending {
                    pending, next_addr, ..
                } = self
                {
                    pending.insert(
                        action.address.clone(),
                        LedgerQueryPending {
                            time: meta.time(),
                            attempts: std::iter::once((
                                action.peer_id,
                                PeerRpcState::Init { time: meta.time() },
                            ))
                            .collect(),
                        },
                    );
                    *next_addr = next_addr
                        .as_ref()
                        .map(|addr| dbg!(dbg!(addr).next_or_next_depth()))
                }
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQueryPending(action) => {
                let Self::SnarkedLedgerSyncPending { pending, .. } = self else { return };
                let Some(rpc_state) = pending.get_mut(&action.address)
                    .and_then(|s| s.attempts.get_mut(&action.peer_id)) else { return };

                *rpc_state = PeerRpcState::Pending {
                    time: meta.time(),
                    rpc_id: action.rpc_id,
                };
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQuerySuccess(action) => {
                let Some((addr, _)) = self.snarked_ledger_peer_query_get(&action.peer_id, action.rpc_id) else { return };
                let addr = addr.clone();
                let Self::SnarkedLedgerSyncPending { next_addr, pending, .. } = self else { return };
                pending.remove(dbg!(&addr));

                match &action.response {
                    PeerLedgerQueryResponse::ChildHashes(_, hash) => {
                        let empty_hash = ledger_empty_hash_at_depth(addr.length() + 1);
                        if hash == &empty_hash {
                            *next_addr = dbg!(Some(addr.next_depth()));
                        }
                    }
                    PeerLedgerQueryResponse::Accounts(list) => {
                        if list.is_empty() || list.len() % 2 == 1 {
                            *next_addr = None;
                        }
                    }
                }
            }
            _ => todo!(),
        }
    }
}
