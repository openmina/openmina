use crate::ledger::{ledger_empty_hash_at_depth, LEDGER_DEPTH};

use super::{
    LedgerQueryPending, PeerRpcState, TransitionFrontierSyncLedgerSnarkedAction,
    TransitionFrontierSyncLedgerSnarkedActionWithMetaRef, TransitionFrontierSyncLedgerSnarkedState,
};

impl TransitionFrontierSyncLedgerSnarkedState {
    pub fn reducer(&mut self, action: TransitionFrontierSyncLedgerSnarkedActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            TransitionFrontierSyncLedgerSnarkedAction::Pending => {
                // handled in parent reducer.
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeersQuery => {}
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryInit { address, peer_id } => {
                if let Self::Pending {
                    pending,
                    next_addr,
                    end_addr,
                    ..
                } = self
                {
                    pending.insert(
                        address.clone(),
                        LedgerQueryPending {
                            time: meta.time(),
                            attempts: std::iter::once((
                                *peer_id,
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
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryRetry { address, peer_id } => {
                if let Self::Pending { pending, .. } = self {
                    if let Some(pending) = pending.get_mut(address) {
                        pending
                            .attempts
                            .insert(*peer_id, PeerRpcState::Init { time: meta.time() });
                    }
                }
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryPending {
                address,
                peer_id,
                rpc_id,
            } => {
                let Self::Pending { pending, .. } = self else {
                    return;
                };
                let Some(rpc_state) = pending
                    .get_mut(address)
                    .and_then(|s| s.attempts.get_mut(peer_id))
                else {
                    return;
                };

                *rpc_state = PeerRpcState::Pending {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                };
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryError {
                peer_id,
                rpc_id,
                error,
            } => {
                let Some(rpc_state) = self.peer_query_get_mut(peer_id, *rpc_id) else {
                    return;
                };

                *rpc_state = PeerRpcState::Error {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                    error: error.clone(),
                };
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQuerySuccess {
                peer_id, rpc_id, ..
            } => {
                let Some(rpc_state) = self.peer_query_get_mut(peer_id, *rpc_id) else {
                    return;
                };
                *rpc_state = PeerRpcState::Success {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                };
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesReceived {
                address,
                hashes,
                ..
            } => {
                let Self::Pending {
                    pending,
                    next_addr,
                    end_addr,
                    ..
                } = self
                else {
                    return;
                };
                let addr = address;
                pending.remove(addr);
                let (left, right) = hashes;

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
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsReceived {
                address, ..
            } => {
                let Self::Pending { pending, .. } = self else {
                    return;
                };
                pending.remove(address);
            }
            TransitionFrontierSyncLedgerSnarkedAction::Success => {
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
