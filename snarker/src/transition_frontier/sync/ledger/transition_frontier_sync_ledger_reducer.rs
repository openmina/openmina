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
            TransitionFrontierSyncLedgerAction::SnarkedPending(_) => {
                if let Self::Init { block, .. } = self {
                    let block = block.clone();
                    *self = Self::SnarkedPending {
                        time: meta.time(),
                        block,
                        pending: Default::default(),
                        next_addr: Some(LedgerAddress::root()),
                        end_addr: LedgerAddress::root(),
                    };
                }
            }
            TransitionFrontierSyncLedgerAction::SnarkedPeersQuery(_) => {}
            TransitionFrontierSyncLedgerAction::SnarkedPeerQueryInit(action) => {
                if let Self::SnarkedPending {
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
            TransitionFrontierSyncLedgerAction::SnarkedPeerQueryRetry(action) => {
                if let Self::SnarkedPending { pending, .. } = self {
                    if let Some(pending) = pending.get_mut(&action.address) {
                        pending
                            .attempts
                            .insert(action.peer_id, PeerRpcState::Init { time: meta.time() });
                    }
                }
            }
            TransitionFrontierSyncLedgerAction::SnarkedPeerQueryPending(action) => {
                let Self::SnarkedPending { pending, .. } = self else { return };
                let Some(rpc_state) = pending.get_mut(&action.address)
                    .and_then(|s| s.attempts.get_mut(&action.peer_id)) else { return };

                *rpc_state = PeerRpcState::Pending {
                    time: meta.time(),
                    rpc_id: action.rpc_id,
                };
            }
            TransitionFrontierSyncLedgerAction::SnarkedPeerQueryError(action) => {
                let Some(rpc_state) = self.snarked_ledger_peer_query_get_mut(&action.peer_id, action.rpc_id) else { return };

                *rpc_state = PeerRpcState::Error {
                    time: meta.time(),
                    rpc_id: action.rpc_id,
                    error: action.error.clone(),
                };
            }
            TransitionFrontierSyncLedgerAction::SnarkedPeerQuerySuccess(action) => {
                let Some(rpc_state) = self.snarked_ledger_peer_query_get_mut(&action.peer_id, action.rpc_id) else { return };
                *rpc_state = PeerRpcState::Success {
                    time: meta.time(),
                    rpc_id: action.rpc_id,
                };
            }
            TransitionFrontierSyncLedgerAction::SnarkedChildHashesReceived(action) => {
                let Self::SnarkedPending { pending, next_addr, end_addr, .. } = self else { return };
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
            TransitionFrontierSyncLedgerAction::SnarkedChildAccountsReceived(action) => {
                let Self::SnarkedPending { pending, .. } = self else { return };
                pending.remove(&action.address);
            }
            TransitionFrontierSyncLedgerAction::SnarkedSuccess(_) => {
                let Self::SnarkedPending { block, .. } = self else { return };
                *self = Self::SnarkedSuccess {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
            TransitionFrontierSyncLedgerAction::StagedReconstructPending(_) => {
                let Self::SnarkedSuccess { block, .. } = self else { return };
                *self = Self::StagedReconstructPending {
                    time: meta.time(),
                    block: block.clone(),
                    attempts: Default::default(),
                };
            }
            TransitionFrontierSyncLedgerAction::StagedPartsFetchInit(_) => {}
            TransitionFrontierSyncLedgerAction::StagedPartsFetchPending(action) => {
                let Self::StagedReconstructPending { attempts, .. } = self else { return };
                attempts.insert(
                    action.peer_id,
                    PeerStagedLedgerReconstructState::PartsFetchPending {
                        time: meta.time(),
                        rpc_id: action.rpc_id,
                    },
                );
            }
            TransitionFrontierSyncLedgerAction::StagedPartsFetchError(action) => {
                let Self::StagedReconstructPending { attempts, .. } = self else { return };
                let Some(attempt) = attempts.get_mut(&action.peer_id) else { return };
                let PeerStagedLedgerReconstructState::PartsFetchPending { rpc_id, .. } = &attempt else { return };
                *attempt = PeerStagedLedgerReconstructState::PartsFetchError {
                    time: meta.time(),
                    rpc_id: *rpc_id,
                    error: action.error.clone(),
                };
            }
            TransitionFrontierSyncLedgerAction::StagedPartsFetchSuccess(action) => {
                let Self::StagedReconstructPending { attempts, .. } = self else { return };
                let Some(attempt) = attempts.get_mut(&action.peer_id) else { return };
                *attempt = PeerStagedLedgerReconstructState::PartsFetchSuccess {
                    time: meta.time(),
                    parts: action.parts.clone(),
                };
            }
            TransitionFrontierSyncLedgerAction::StagedPartsApplyInit(_) => {}
            TransitionFrontierSyncLedgerAction::StagedPartsApplySuccess(action) => {
                let Self::StagedReconstructPending { attempts, .. } = self else { return };
                let Some(attempt) = attempts.get_mut(&action.sender) else { return };
                *attempt =
                    PeerStagedLedgerReconstructState::PartsApplySuccess { time: meta.time() };
            }
            TransitionFrontierSyncLedgerAction::StagedReconstructSuccess(_) => {
                let Self::StagedReconstructPending { block, .. } = self else { return };

                *self = Self::StagedReconstructSuccess {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
            TransitionFrontierSyncLedgerAction::Success(_) => {
                let Self::StagedReconstructSuccess { block, .. } = self else { return };

                *self = Self::Success {
                    time: meta.time(),
                    block: block.clone(),
                };
            }
        }
    }
}
