use std::sync::Arc;

use mina_p2p_messages::v2::{self, LedgerHash};
use serde::{Deserialize, Serialize};

use crate::p2p::channels::rpc::{P2pRpcId, StagedLedgerAuxAndPendingCoinbases};
use crate::p2p::PeerId;
use crate::transition_frontier::sync::ledger::snarked::TransitionFrontierSyncLedgerSnarkedState;

use super::{
    PeerStagedLedgerPartsFetchError, PeerStagedLedgerPartsFetchState,
    TransitionFrontierSyncLedgerStagedState,
};

pub type TransitionFrontierSyncLedgerStagedActionWithMeta =
    redux::ActionWithMeta<TransitionFrontierSyncLedgerStagedAction>;
pub type TransitionFrontierSyncLedgerStagedActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierSyncLedgerStagedAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncLedgerStagedAction {
    PartsFetchPending,
    PartsPeerFetchInit,
    PartsPeerFetchPending {
        peer_id: PeerId,
        rpc_id: P2pRpcId,
    },
    PartsPeerFetchError {
        peer_id: PeerId,
        rpc_id: P2pRpcId,
        error: PeerStagedLedgerPartsFetchError,
    },
    PartsPeerFetchSuccess {
        peer_id: PeerId,
        rpc_id: P2pRpcId,
        parts: Arc<StagedLedgerAuxAndPendingCoinbases>,
    },
    PartsPeerInvalid {
        sender: PeerId,
        parts: Arc<StagedLedgerAuxAndPendingCoinbases>,
    },
    PartsPeerValid {
        sender: PeerId,
    },
    PartsFetchSuccess {
        sender: PeerId,
    },
    ReconstructEmpty,
    ReconstructInit,
    ReconstructPending,
    ReconstructError {
        error: String,
    },
    ReconstructSuccess {
        ledger_hash: LedgerHash,
    },
    Success,
}

impl redux::EnablingCondition<crate::State> for TransitionFrontierSyncLedgerStagedAction {
    fn is_enabled(&self, state: &crate::State, _time: redux::Timestamp) -> bool {
        match self {
            TransitionFrontierSyncLedgerStagedAction::PartsFetchPending => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked())
                .map_or(false, |s| match s {
                    TransitionFrontierSyncLedgerSnarkedState::Success { target, .. } => {
                        target.staged.is_some()
                    }
                    _ => false,
                }),
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchInit => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.staged())
                .map_or(false, |staged| {
                    let iter = state.p2p.ready_rpc_peers_iter();
                    staged.filter_available_peers(iter).next().is_some()
                }),
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchPending { .. } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.staged())
                .map_or(false, |s| {
                    matches!(
                        s,
                        TransitionFrontierSyncLedgerStagedState::PartsFetchPending { .. }
                    )
                }),
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchError {
                peer_id,
                rpc_id,
                ..
            } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.staged()?.fetch_attempts()?.get(peer_id))
                .and_then(|s| s.fetch_pending_rpc_id())
                .map_or(false, |fetch_rpc_id| fetch_rpc_id == *rpc_id),
            TransitionFrontierSyncLedgerStagedAction::PartsPeerFetchSuccess {
                peer_id,
                rpc_id,
                ..
            } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.staged()?.fetch_attempts()?.get(peer_id))
                .and_then(|s| s.fetch_pending_rpc_id())
                .map_or(false, |fetch_rpc_id| fetch_rpc_id == *rpc_id),
            TransitionFrontierSyncLedgerStagedAction::PartsPeerInvalid { sender, .. } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.staged()?.fetch_attempts()?.get(sender))
                .map_or(false, |s| match s {
                    PeerStagedLedgerPartsFetchState::Success { parts, .. } => !parts.is_valid(),
                    _ => false,
                }),
            TransitionFrontierSyncLedgerStagedAction::PartsPeerValid { sender } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.staged()?.fetch_attempts()?.get(sender))
                .map_or(false, |s| match s {
                    PeerStagedLedgerPartsFetchState::Success { parts, .. } => parts.is_valid(),
                    _ => false,
                }),
            TransitionFrontierSyncLedgerStagedAction::PartsFetchSuccess { sender } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.staged()?.fetch_attempts()?.get(sender))
                .map_or(false, |s| s.is_valid()),
            TransitionFrontierSyncLedgerStagedAction::ReconstructEmpty => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.snarked())
                .and_then(|s| match s {
                    TransitionFrontierSyncLedgerSnarkedState::Success { target, .. } => {
                        target.clone().with_staged()
                    }
                    _ => None,
                })
                .map_or(false, |target| {
                    let hashes = &target.staged.hashes;
                    target.snarked_ledger_hash == hashes.non_snark.ledger_hash
                        && hashes.non_snark.aux_hash == v2::StagedLedgerHashAuxHash::zero()
                        && hashes.non_snark.pending_coinbase_aux
                            == v2::StagedLedgerHashPendingCoinbaseAux::zero()
                    // TODO(binier): `pending_coinbase_hash` isn't empty hash.
                    // Do we need to check it?
                }),
            TransitionFrontierSyncLedgerStagedAction::ReconstructInit => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.staged())
                .map_or(false, |s| {
                    matches!(
                        s,
                        TransitionFrontierSyncLedgerStagedState::PartsFetchSuccess { .. }
                            | TransitionFrontierSyncLedgerStagedState::ReconstructEmpty { .. }
                    )
                }),
            TransitionFrontierSyncLedgerStagedAction::ReconstructPending => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.staged())
                .map_or(false, |s| {
                    matches!(
                        s,
                        TransitionFrontierSyncLedgerStagedState::PartsFetchSuccess { .. }
                            | TransitionFrontierSyncLedgerStagedState::ReconstructEmpty { .. }
                    )
                }),
            TransitionFrontierSyncLedgerStagedAction::ReconstructError { .. } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.staged())
                .map_or(false, |s| {
                    matches!(
                        s,
                        TransitionFrontierSyncLedgerStagedState::ReconstructPending { .. }
                    )
                }),
            TransitionFrontierSyncLedgerStagedAction::ReconstructSuccess { .. } => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.staged())
                .map_or(false, |s| {
                    matches!(
                        s,
                        TransitionFrontierSyncLedgerStagedState::ReconstructPending { .. }
                    )
                }),
            TransitionFrontierSyncLedgerStagedAction::Success => state
                .transition_frontier
                .sync
                .ledger()
                .and_then(|s| s.staged())
                .map_or(false, |s| {
                    matches!(
                        s,
                        TransitionFrontierSyncLedgerStagedState::ReconstructSuccess { .. }
                    )
                }),
        }
    }
}

use crate::transition_frontier::{
    sync::{ledger::TransitionFrontierSyncLedgerAction, TransitionFrontierSyncAction},
    TransitionFrontierAction,
};

impl From<TransitionFrontierSyncLedgerStagedAction> for crate::Action {
    fn from(value: TransitionFrontierSyncLedgerStagedAction) -> Self {
        Self::TransitionFrontier(TransitionFrontierAction::Sync(
            TransitionFrontierSyncAction::Ledger(TransitionFrontierSyncLedgerAction::Staged(value)),
        ))
    }
}
