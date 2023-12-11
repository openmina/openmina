use crate::ledger::{ledger_empty_hash_at_depth, LEDGER_DEPTH};

use super::{
    LedgerQueryPending, PeerRpcState, TransitionFrontierSyncLedgerSnarkedAction,
    TransitionFrontierSyncLedgerSnarkedActionWithMetaRef, TransitionFrontierSyncLedgerSnarkedState,
};

impl TransitionFrontierSyncLedgerSnarkedState {
    pub fn reducer(&mut self, action: TransitionFrontierSyncLedgerSnarkedActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            TransitionFrontierSyncLedgerSnarkedAction::Pending(_) => {
                // handled in parent reducer.
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeersQuery(_) => {}
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryInit(action) => {
                if let Self::Pending {
                    pending,
                    next_addr,
                    end_addr,
                    ..
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
                        .map(|addr| {
                            addr.next()
                                .filter(|addr| {
                                    let mut end_addr = end_addr.clone();
                                    while end_addr.length() < addr.length() {
                                        end_addr = end_addr.child_right();
                                    }
                                    while end_addr.length() > addr.length() {
                                        let Some(addr) = end_addr.parent() else {
                                            return true;
                                        };
                                        end_addr = addr;
                                    }
                                    addr <= &end_addr
                                })
                                .unwrap_or_else(|| addr.next_depth())
                        })
                        .filter(|addr| addr.length() < LEDGER_DEPTH);
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryRetry(action) => {
                if let Self::Pending { pending, .. } = self {
                    if let Some(pending) = pending.get_mut(&action.address) {
                        pending
                            .attempts
                            .insert(action.peer_id, PeerRpcState::Init { time: meta.time() });
                    }
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryPending(action) => {
                let Self::Pending { pending, .. } = self else {
                    return;
                };
                let Some(rpc_state) = pending
                    .get_mut(&action.address)
                    .and_then(|s| s.attempts.get_mut(&action.peer_id))
                else {
                    return;
                };

                *rpc_state = PeerRpcState::Pending {
                    time: meta.time(),
                    rpc_id: action.rpc_id,
                };
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryError(action) => {
                let Some(rpc_state) = self.peer_query_get_mut(&action.peer_id, action.rpc_id)
                else {
                    return;
                };

                *rpc_state = PeerRpcState::Error {
                    time: meta.time(),
                    rpc_id: action.rpc_id,
                    error: action.error.clone(),
                };
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQuerySuccess(action) => {
                let Some(rpc_state) = self.peer_query_get_mut(&action.peer_id, action.rpc_id)
                else {
                    return;
                };
                *rpc_state = PeerRpcState::Success {
                    time: meta.time(),
                    rpc_id: action.rpc_id,
                };
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesReceived(action) => {
                let Self::Pending {
                    pending,
                    next_addr,
                    end_addr,
                    ..
                } = self
                else {
                    return;
                };
                let addr = &action.address;
                pending.remove(&addr);
                let (left, right) = &action.hashes;

                let empty_hash = ledger_empty_hash_at_depth(addr.length() + 1);
                if right == &empty_hash {
                    *next_addr =
                        Some(addr.next_depth()).filter(|addr| addr.length() < LEDGER_DEPTH);
                    let addr = match left == &empty_hash {
                        true => addr.child_left(),
                        false => addr.child_right(),
                    };
                    if addr.length() > end_addr.length()
                        || (addr.length() == end_addr.length()
                            && addr.to_index() < end_addr.to_index())
                    {
                        *end_addr = addr.prev().unwrap_or(addr);
                    }
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsReceived(action) => {
                let Self::Pending { pending, .. } = self else {
                    return;
                };
                pending.remove(&action.address);
            }
            TransitionFrontierSyncLedgerSnarkedAction::Success(_) => {
                let Self::Pending { target, .. } = self else {
                    return;
                };
                *self = Self::Success {
                    time: meta.time(),
                    target: target.clone(),
                };
            }
        }
    }
}
