use mina_p2p_messages::v2::{LedgerHash, MinaBaseAccountBinableArgStableV2};
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::ledger::{LedgerAddress, LEDGER_DEPTH};
use crate::p2p::channels::rpc::P2pRpcId;
use crate::p2p::PeerId;
use crate::transition_frontier::sync::ledger::TransitionFrontierSyncLedgerState;

use super::{
    LedgerAddressQuery, PeerLedgerQueryError, PeerLedgerQueryResponse, PeerRpcState,
    TransitionFrontierSyncLedgerSnarkedState,
};

/// Once we reach subtrees of this height, we begin performing
/// queries to fetch all the accounts in the subtree at once
/// instead of fetching intermediary hashes.
pub const ACCOUNT_SUBTREE_HEIGHT: usize = 6;

pub type TransitionFrontierSyncLedgerSnarkedActionWithMeta =
    redux::ActionWithMeta<TransitionFrontierSyncLedgerSnarkedAction>;
pub type TransitionFrontierSyncLedgerSnarkedActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierSyncLedgerSnarkedAction>;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = trace)]
pub enum TransitionFrontierSyncLedgerSnarkedAction {
    Pending,
    PeersQuery,

    // For NumAccounts query
    PeerQueryNumAccountsInit {
        peer_id: PeerId,
    },
    PeerQueryNumAccountsPending {
        peer_id: PeerId,
        rpc_id: P2pRpcId,
    },
    PeerQueryNumAccountsRetry {
        peer_id: PeerId,
    },
    PeerQueryNumAccountsError {
        peer_id: PeerId,
        rpc_id: P2pRpcId,
        error: PeerLedgerQueryError,
    },
    PeerQueryNumAccountsSuccess {
        peer_id: PeerId,
        rpc_id: P2pRpcId,
        response: PeerLedgerQueryResponse,
    },
    NumAccountsReceived {
        num_accounts: u64,
        contents_hash: LedgerHash,
        sender: PeerId,
    },
    NumAccountsAccepted {
        num_accounts: u64,
        contents_hash: LedgerHash,
        sender: PeerId,
    },
    NumAccountsRejected {
        num_accounts: u64,
        sender: PeerId,
    },
    NumAccountsSuccess {
        num_accounts: u64,
        contents_hash: LedgerHash,
    },

    MerkleTreeSyncPending,

    // For child hashes and content queries
    PeerQueryAddressInit {
        address: LedgerAddress,
        expected_hash: LedgerHash,
        peer_id: PeerId,
    },
    PeerQueryAddressPending {
        address: LedgerAddress,
        peer_id: PeerId,
        rpc_id: P2pRpcId,
    },
    PeerQueryAddressRetry {
        address: LedgerAddress,
        peer_id: PeerId,
    },
    PeerQueryAddressError {
        peer_id: PeerId,
        rpc_id: P2pRpcId,
        error: PeerLedgerQueryError,
    },
    PeerQueryAddressSuccess {
        peer_id: PeerId,
        rpc_id: P2pRpcId,
        response: PeerLedgerQueryResponse,
    },
    ChildHashesReceived {
        address: LedgerAddress,
        hashes: (LedgerHash, LedgerHash),
        sender: PeerId,
    },
    ChildHashesAccepted {
        address: LedgerAddress,
        hashes: (LedgerHash, LedgerHash),
        previous_hashes: (LedgerHash, LedgerHash),
        sender: PeerId,
    },
    ChildHashesRejected {
        address: LedgerAddress,
        hashes: (LedgerHash, LedgerHash),
        sender: PeerId,
    },
    ChildAccountsReceived {
        address: LedgerAddress,
        accounts: Vec<MinaBaseAccountBinableArgStableV2>,
        sender: PeerId,
    },
    ChildAccountsAccepted {
        address: LedgerAddress,
        count: u64,
        sender: PeerId,
    },
    ChildAccountsRejected {
        address: LedgerAddress,
        sender: PeerId,
    },

    MerkleTreeSyncSuccess,
    Success,
}

impl redux::EnablingCondition<crate::State> for TransitionFrontierSyncLedgerSnarkedAction {
    fn is_enabled(&self, state: &crate::State, _time: redux::Timestamp) -> bool {
        match self {
            TransitionFrontierSyncLedgerSnarkedAction::Pending => {
                state.transition_frontier.sync.ledger().map_or(false, |s| {
                    matches!(s, TransitionFrontierSyncLedgerState::Init { .. })
                })
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeersQuery => {
                // This condition passes if:
                // - there are available peers to query
                // - there is a snarked ledger to sync
                // - there are either queued num_accounts or address queries
                //   or queries to retry
                let peers_available = state
                    .p2p
                    .ready_peers_iter()
                    .any(|(_, p)| p.channels.rpc.can_send_request());
                let sync_next_available = state
                    .transition_frontier
                    .sync
                    .ledger()
                    .and_then(|s| s.snarked())
                    .map_or(false, |s| {
                        s.is_num_accounts_query_next()
                            || s.sync_address_next().is_some()
                            || s.sync_address_retry_iter().next().is_some()
                    });
                peers_available && sync_next_available
            }

            // num accounts
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsInit { peer_id } => None
                .or_else(|| {
                    let target_best_tip = state.transition_frontier.sync.best_tip()?;
                    let ledger = state.transition_frontier.sync.ledger()?.snarked()?;
                    let target = ledger.target();

                    let check_num_accounts = matches!(
                        ledger,
                        TransitionFrontierSyncLedgerSnarkedState::NumAccountsPending { .. }
                    );

                    let peer = state.p2p.get_ready_peer(peer_id)?;
                    let check_peer_available = check_peer_available(peer, target, target_best_tip);

                    Some(check_num_accounts && check_peer_available)
                })
                .unwrap_or(false),
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsPending {
                peer_id,
                ..
            } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked()?.num_accounts_pending())
                .map_or(false, |pending| {
                    pending
                        .attempts
                        .get(peer_id)
                        .map(|peer_rpc_state| matches!(peer_rpc_state, PeerRpcState::Init { .. }))
                        .unwrap_or(false)
                }),
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsRetry { peer_id } => {
                None.or_else(|| {
                    let target_best_tip = state.transition_frontier.sync.best_tip()?;
                    let ledger = state.transition_frontier.sync.ledger()?.snarked()?;
                    let target = ledger.target();

                    let check_num_accounts = matches!(
                        ledger,
                        TransitionFrontierSyncLedgerSnarkedState::NumAccountsPending { .. }
                    );

                    let peer = state.p2p.get_ready_peer(peer_id)?;
                    let check_peer_available = check_peer_available(peer, target, target_best_tip);

                    Some(check_num_accounts && check_peer_available)
                })
                .unwrap_or(false)
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsError {
                peer_id,
                rpc_id,
                ..
            } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked())
                .map_or(false, |s| {
                    s.peer_num_account_query_get(peer_id, *rpc_id)
                        .and_then(|s| s.attempts.get(peer_id))
                        .map_or(false, |s| matches!(s, PeerRpcState::Pending { .. }))
                }),
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryNumAccountsSuccess {
                peer_id,
                rpc_id,
                ..
            } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked())
                .map_or(false, |s| {
                    // TODO(tizoc): check if expected response kind is correct.
                    s.peer_num_account_query_get(peer_id, *rpc_id)
                        .and_then(|s| s.attempts.get(peer_id))
                        .map_or(false, |s| matches!(s, PeerRpcState::Pending { .. }))
                }),
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsReceived { sender, .. }
            | TransitionFrontierSyncLedgerSnarkedAction::NumAccountsAccepted { sender, .. }
            | TransitionFrontierSyncLedgerSnarkedAction::NumAccountsRejected { sender, .. } => {
                state
                    .transition_frontier
                    .sync
                    .ledger()
                    .and_then(|s| s.snarked()?.num_accounts_pending())
                    .and_then(|s| s.attempts.get(sender))
                    .map_or(false, |s| s.is_success())
            }
            TransitionFrontierSyncLedgerSnarkedAction::NumAccountsSuccess { .. } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked()?.num_accounts_pending())
                .is_some(),

            TransitionFrontierSyncLedgerSnarkedAction::MerkleTreeSyncPending => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked())
                .map_or(false, |s| {
                    matches!(
                        s,
                        TransitionFrontierSyncLedgerSnarkedState::NumAccountsSuccess { .. }
                    )
                }),
            TransitionFrontierSyncLedgerSnarkedAction::MerkleTreeSyncSuccess => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked())
                .map_or(false, |s| match s {
                    TransitionFrontierSyncLedgerSnarkedState::MerkleTreeSyncPending {
                        queue,
                        pending_addresses: pending,
                        ..
                    } => queue.is_empty() && pending.is_empty(),
                    _ => false,
                }),

            // hashes and contents
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressInit {
                address,
                peer_id,
                expected_hash: _,
            } => {
                None.or_else(|| {
                    let target_best_tip = state.transition_frontier.sync.best_tip()?;
                    let ledger = state.transition_frontier.sync.ledger()?.snarked()?;
                    let target = ledger.target();

                    // This is true if there is a next address that needs to be queried
                    // from a peer and it matches the one requested by this action.
                    let check_next_addr = match ledger {
                        TransitionFrontierSyncLedgerSnarkedState::MerkleTreeSyncPending {
                            queue,
                            pending_addresses: pending,
                            ..
                        } => queue.front().map_or(false, |query| {
                            let LedgerAddressQuery {
                                address: next_addr, ..
                            } = query;

                            next_addr == address
                                && (next_addr.to_index().0 != 0 || pending.is_empty())
                        }),
                        _ => false,
                    };

                    let peer = state.p2p.get_ready_peer(peer_id)?;
                    let check_peer_available = check_peer_available(peer, target, target_best_tip);

                    Some(check_next_addr && check_peer_available)
                })
                .unwrap_or(false)
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressRetry {
                address,
                peer_id,
            } => {
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
                        .and_then(|s| s.snarked()?.sync_address_retry_iter().next())
                        .map_or(false, |addr| &addr == address);

                    let peer = state.p2p.get_ready_peer(peer_id)?;
                    let check_peer_available = check_peer_available(peer, target, target_best_tip);

                    Some(check_next_addr && check_peer_available)
                })
                .unwrap_or(false)
            }
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressPending {
                peer_id, ..
            } => state
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
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressError {
                peer_id,
                rpc_id,
                ..
            } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked())
                .map_or(false, |s| {
                    s.peer_address_query_get(peer_id, *rpc_id)
                        .and_then(|(_, s)| s.attempts.get(peer_id))
                        .map_or(false, |s| matches!(s, PeerRpcState::Pending { .. }))
                }),
            TransitionFrontierSyncLedgerSnarkedAction::PeerQueryAddressSuccess {
                peer_id,
                rpc_id,
                ..
            } => {
                state
                    .transition_frontier
                    .sync
                    .ledger()
                    .and_then(|s| s.snarked())
                    .map_or(false, |s| {
                        // TODO(binier): check if expected response kind is correct.
                        s.peer_address_query_get(peer_id, *rpc_id)
                            .and_then(|(_, s)| s.attempts.get(peer_id))
                            .map_or(false, |s| matches!(s, PeerRpcState::Pending { .. }))
                    })
            }
            TransitionFrontierSyncLedgerSnarkedAction::ChildHashesReceived {
                address,
                sender,
                ..
            }
            | TransitionFrontierSyncLedgerSnarkedAction::ChildHashesAccepted {
                address,
                sender,
                ..
            }
            | TransitionFrontierSyncLedgerSnarkedAction::ChildHashesRejected {
                address,
                sender,
                ..
            } => {
                address.length() < LEDGER_DEPTH - ACCOUNT_SUBTREE_HEIGHT
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
            }
            | TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsRejected {
                address,
                sender,
            } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked()?.fetch_pending()?.get(address))
                .and_then(|s| s.attempts.get(sender))
                .map_or(false, |s| s.is_success()),
            TransitionFrontierSyncLedgerSnarkedAction::ChildAccountsAccepted {
                address,
                count,
                sender,
            } => {
                *count > 0
                    && state
                        .transition_frontier
                        .sync
                        .ledger()
                        .and_then(|s| s.snarked()?.fetch_pending()?.get(address))
                        .and_then(|s| s.attempts.get(sender))
                        .map_or(false, |s| s.is_success())
            }
            TransitionFrontierSyncLedgerSnarkedAction::Success => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked())
                .map_or(false, |s| {
                    matches!(
                        s,
                        TransitionFrontierSyncLedgerSnarkedState::MerkleTreeSyncSuccess { .. }
                    )
                }),
        }
    }
}

fn check_peer_available(
    peer: &p2p::P2pPeerStatusReady,
    target: &crate::transition_frontier::sync::ledger::SyncLedgerTarget,
    target_best_tip: &openmina_core::block::BlockWithHash<
        std::sync::Arc<mina_p2p_messages::v2::MinaBlockBlockStableV2>,
    >,
) -> bool {
    None.or_else(|| {
        let peer_best_tip = peer.best_tip.as_ref()?;
        let available = if !peer.channels.rpc.can_send_request() {
            false
        } else if target.staged.is_some() {
            // if peer has same best tip, then he has same root
            // so we can sync root snarked+staged ledger from that peer.
            target_best_tip.hash() == peer_best_tip.hash()
        } else {
            &target.snarked_ledger_hash == peer_best_tip.snarked_ledger_hash()
                || &target.snarked_ledger_hash == peer_best_tip.staking_epoch_ledger_hash()
                || &target.snarked_ledger_hash == peer_best_tip.next_epoch_ledger_hash()
        };

        Some(available)
    })
    .unwrap_or(false)
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
