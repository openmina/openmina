use mina_p2p_messages::v2::{LedgerHash, MinaBaseAccountBinableArgStableV2};
use serde::{Deserialize, Serialize};

use crate::ledger::{LedgerAddress, LEDGER_DEPTH};
use crate::p2p::channels::rpc::P2pRpcId;
use crate::p2p::PeerId;
use crate::transition_frontier::sync::ledger::TransitionFrontierSyncLedgerState;

use super::{
    PeerLedgerQueryError, PeerLedgerQueryResponse, PeerRpcState,
    TransitionFrontierSyncLedgerSnarkedState,
};

pub type TransitionFrontierSyncLedgerSnarkedActionWithMeta =
    redux::ActionWithMeta<TransitionFrontierSyncLedgerSnarkedAction>;
pub type TransitionFrontierSyncLedgerSnarkedActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierSyncLedgerSnarkedAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncLedgerSnarkedAction {
    Pending,
    PeersQuery,
    PeerQueryInit {
        address: LedgerAddress,
        peer_id: PeerId,
    },
    PeerQueryPending {
        address: LedgerAddress,
        peer_id: PeerId,
        rpc_id: P2pRpcId,
    },
    PeerQueryRetry {
        address: LedgerAddress,
        peer_id: PeerId,
    },
    PeerQueryError {
        peer_id: PeerId,
        rpc_id: P2pRpcId,
        error: PeerLedgerQueryError,
    },
    PeerQuerySuccess {
        peer_id: PeerId,
        rpc_id: P2pRpcId,
        response: PeerLedgerQueryResponse,
    },
    ChildHashesReceived {
        address: LedgerAddress,
        hashes: (LedgerHash, LedgerHash),
        sender: PeerId,
    },
    ChildAccountsReceived {
        address: LedgerAddress,
        accounts: Vec<MinaBaseAccountBinableArgStableV2>,
        sender: PeerId,
    },
    Success,
}

impl redux::EnablingCondition<crate::State> for TransitionFrontierSyncLedgerSnarkedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        match self {
            TransitionFrontierSyncLedgerSnarkedAction::Pending => {
                state.transition_frontier.sync.ledger().map_or(false, |s| {
                    matches!(s, TransitionFrontierSyncLedgerState::Init { .. })
                })
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeersQuery => {
                let peers_available = state
                    .p2p
                    .ready_peers_iter()
                    .any(|(_, p)| p.channels.rpc.can_send_request());
                peers_available
                    && state
                        .transition_frontier
                        .sync
                        .ledger()
                        .and_then(|s| s.snarked())
                        .map_or(false, |s| {
                            s.sync_next().is_some() || s.sync_retry_iter().next().is_some()
                        })
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryInit { address, peer_id } => {
                None.or_else(|| {
                    let target_best_tip = state.transition_frontier.sync.best_tip()?;
                    let ledger = state.transition_frontier.sync.ledger()?.snarked()?;
                    let target = ledger.target();

                    // This is true if there is a next address that needs to be queried
                    // from a peer and it matches the one requested by this action.
                    let check_next_addr = match ledger {
                        TransitionFrontierSyncLedgerSnarkedState::Pending {
                            pending,
                            next_addr,
                            ..
                        } => next_addr.as_ref().map_or(false, |next_addr| {
                            next_addr == address
                                && (next_addr.to_index().0 != 0 || pending.is_empty())
                        }),
                        _ => false,
                    };

                    let peer = state.p2p.get_ready_peer(peer_id)?;
                    let check_peer_available = {
                        let peer_best_tip = peer.best_tip.as_ref()?;
                        if !peer.channels.rpc.can_send_request() {
                            false
                        } else if target.staged.is_some() {
                            // if peer has same best tip, then he has same root
                            // so we can sync root snarked+staged ledger from that peer.
                            target_best_tip.hash() == peer_best_tip.hash()
                        } else {
                            &target.snarked_ledger_hash == peer_best_tip.snarked_ledger_hash()
                                || &target.snarked_ledger_hash
                                    == peer_best_tip.staking_epoch_ledger_hash()
                                || &target.snarked_ledger_hash
                                    == peer_best_tip.next_epoch_ledger_hash()
                        }
                    };

                    Some(check_next_addr && check_peer_available)
                })
                .unwrap_or(false)
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryRetry { address, peer_id } => {
                None.or_else(|| {
                    let target_best_tip = state.transition_frontier.sync.best_tip()?;
                    let ledger = state.transition_frontier.sync.ledger()?.snarked()?;
                    let target = ledger.target();

                    // This is true if there is next retry address and it
                    // matches the one requested in this action.
                    let check_next_addr = state
                        .transition_frontier
                        .sync
                        .ledger()
                        .and_then(|s| s.snarked()?.sync_retry_iter().next())
                        .map_or(false, |addr| &addr == address);

                    let peer = state.p2p.get_ready_peer(peer_id)?;
                    let check_peer_available = {
                        let peer_best_tip = peer.best_tip.as_ref()?;
                        if !peer.channels.rpc.can_send_request() {
                            false
                        } else if target.staged.is_some() {
                            // if peer has same best tip, then he has same root
                            // so we can sync root snarked+staged ledger from that peer.
                            target_best_tip.hash() == peer_best_tip.hash()
                        } else {
                            &target.snarked_ledger_hash == peer_best_tip.snarked_ledger_hash()
                                || &target.snarked_ledger_hash
                                    == peer_best_tip.staking_epoch_ledger_hash()
                                || &target.snarked_ledger_hash
                                    == peer_best_tip.next_epoch_ledger_hash()
                        }
                    };

                    Some(check_next_addr && check_peer_available)
                })
                .unwrap_or(false)
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryPending { peer_id, .. } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked()?.fetch_pending())
                .map_or(false, |pending| {
                    pending
                        .iter()
                        .filter_map(|(_, query_state)| query_state.attempts.get(peer_id))
                        .any(|peer_rpc_state| matches!(peer_rpc_state, PeerRpcState::Init { .. }))
                }),
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryError {
                peer_id, rpc_id, ..
            } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked())
                .map_or(false, |s| {
                    s.peer_query_get(peer_id, *rpc_id)
                        .and_then(|(_, s)| s.attempts.get(peer_id))
                        .map_or(false, |s| matches!(s, PeerRpcState::Pending { .. }))
                }),
            TransitionFrontierSyncLedgerSnarkedAction::PeerQuerySuccess {
                peer_id, rpc_id, ..
            } => {
                state
                    .transition_frontier
                    .sync
                    .ledger()
                    .and_then(|s| s.snarked())
                    .map_or(false, |s| {
                        // TODO(binier): check if expected response
                        // kind is correct.
                        s.peer_query_get(peer_id, *rpc_id)
                            .and_then(|(_, s)| s.attempts.get(peer_id))
                            .map_or(false, |s| matches!(s, PeerRpcState::Pending { .. }))
                    })
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesReceived {
                address,
                sender,
                ..
            } => {
                address.length() < LEDGER_DEPTH - 1
                    && state
                        .transition_frontier
                        .sync
                        .ledger()
                        .and_then(|s| s.snarked()?.fetch_pending()?.get(address))
                        .and_then(|s| s.attempts.get(sender))
                        .map_or(false, |s| s.is_success())
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsReceived {
                address,
                sender,
                ..
            } => {
                state
                    .transition_frontier
                    .sync
                    .ledger()
                    .and_then(|s| s.snarked()?.fetch_pending()?.get(address))
                    .and_then(|s| s.attempts.get(sender))
                    // TODO(binier): check if expected response
                    // kind is correct.
                    .map_or(false, |s| s.is_success())
            }
            TransitionFrontierSyncLedgerSnarkedAction::Success => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked())
                .map_or(false, |s| match s {
                    TransitionFrontierSyncLedgerSnarkedState::Pending {
                        pending,
                        next_addr,
                        ..
                    } => next_addr.is_none() && pending.is_empty(),
                    _ => false,
                }),
        }
    }
}

use crate::transition_frontier::{
    sync::{ledger::TransitionFrontierSyncLedgerAction, TransitionFrontierSyncAction},
    TransitionFrontierAction,
};

impl From<TransitionFrontierSyncLedgerSnarkedAction> for crate::Action {
    fn from(value: TransitionFrontierSyncLedgerSnarkedAction) -> Self {
        Self::TransitionFrontier(TransitionFrontierAction::Sync(
            TransitionFrontierSyncAction::Ledger(TransitionFrontierSyncLedgerAction::Snarked(
                value,
            )),
        ))
    }
}
