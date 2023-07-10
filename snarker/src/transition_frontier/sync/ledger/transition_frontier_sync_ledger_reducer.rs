use crate::ledger::{ledger_empty_hash_at_depth, LedgerAddress, LEDGER_DEPTH};

use super::{
    LedgerQueryPending, PeerRpcState, PeerStagedLedgerReconstructState,
    TransitionFrontierSyncLedgerAction, TransitionFrontierSyncLedgerActionWithMetaRef,
    TransitionFrontierSyncLedgerState,
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
                        next_addr: Some(LedgerAddress::root()),
                        end_addr: LedgerAddress::root(),
                    };
                }
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeersQuery(_) => {}
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQueryInit(action) => {
                if let Self::SnarkedLedgerSyncPending {
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
                                        let Some(addr) = end_addr.parent() else { return true };
                                        end_addr = addr;
                                    }
                                    addr <= &end_addr
                                })
                                .unwrap_or_else(|| addr.next_depth())
                        })
                        .filter(|addr| addr.length() < LEDGER_DEPTH);
                }
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQueryRetry(action) => {
                if let Self::SnarkedLedgerSyncPending { pending, .. } = self {
                    if let Some(pending) = pending.get_mut(&action.address) {
                        pending
                            .attempts
                            .insert(action.peer_id, PeerRpcState::Init { time: meta.time() });
                    }
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
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQueryError(action) => {
                let Some(rpc_state) = self.snarked_ledger_peer_query_get_mut(&action.peer_id, action.rpc_id) else { return };

                *rpc_state = PeerRpcState::Error {
                    time: meta.time(),
                    rpc_id: action.rpc_id,
                    error: action.error.clone(),
                };
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQuerySuccess(action) => {
                let Some(rpc_state) = self.snarked_ledger_peer_query_get_mut(&action.peer_id, action.rpc_id) else { return };
                *rpc_state = PeerRpcState::Success {
                    time: meta.time(),
                    rpc_id: action.rpc_id,
                };
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncChildHashesReceived(action) => {
                let Self::SnarkedLedgerSyncPending { pending, next_addr, end_addr, .. } = self else { return };
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
                        dbg!(&end_addr);
                    }
                }
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncChildAccountsReceived(action) => {
                let Self::SnarkedLedgerSyncPending { pending, .. } = self else { return };
                pending.remove(&action.address);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncSuccess(_) => {
                let Self::SnarkedLedgerSyncPending { block, .. } = self else { return };
                *self = Self::SnarkedLedgerSyncSuccess {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerReconstructPending(_) => {
                let Self::SnarkedLedgerSyncSuccess { block, .. } = self else { return };
                *self = Self::StagedLedgerReconstructPending {
                    time: meta.time(),
                    block: block.clone(),
                    attempts: Default::default(),
                };
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsFetchInit(_) => {}
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsFetchPending(action) => {
                let Self::StagedLedgerReconstructPending { attempts, .. } = self else { return };
                attempts.insert(
                    action.peer_id,
                    PeerStagedLedgerReconstructState::PartsFetchPending {
                        time: meta.time(),
                        rpc_id: action.rpc_id,
                    },
                );
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsFetchError(action) => {
                let Self::StagedLedgerReconstructPending { attempts, .. } = self else { return };
                let Some(attempt) = attempts.get_mut(&action.peer_id) else { return };
                let PeerStagedLedgerReconstructState::PartsFetchPending { rpc_id, .. } = &attempt else { return };
                *attempt = PeerStagedLedgerReconstructState::PartsFetchError {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                    error: action.error.clone(),
                };
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsFetchSuccess(action) => {
                let Self::StagedLedgerReconstructPending { attempts, .. } = self else { return };
                let Some(attempt) = attempts.get_mut(&action.peer_id) else { return };
                *attempt = PeerStagedLedgerReconstructState::PartsFetchSuccess {
                    time: meta.time(),
                    parts: action.parts.clone(),
                };
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsApplyInit(_) => {}
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsApplySuccess(action) => {
                let Self::StagedLedgerReconstructPending { attempts, .. } = self else { return };
                let Some(attempt) = attempts.get_mut(&action.sender) else { return };
                *attempt =
                    PeerStagedLedgerReconstructState::PartsApplySuccess { time: meta.time() };
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerReconstructSuccess(_) => {
                let Self::StagedLedgerReconstructPending { block, .. } = self else { return };

                *self = Self::StagedLedgerReconstructSuccess {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
            TransitionFrontierSyncLedgerAction::Success(_) => {
                let Self::StagedLedgerReconstructSuccess { block, .. } = self else { return };

                *self = Self::Success {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
        }
    }
}
