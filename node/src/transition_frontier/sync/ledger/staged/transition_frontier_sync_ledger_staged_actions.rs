use std::sync::Arc;

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

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncLedgerStagedAction {
    PartsFetchPending(TransitionFrontierSyncLedgerStagedPartsFetchPendingAction),
    PartsPeerFetchInit(TransitionFrontierSyncLedgerStagedPartsPeerFetchInitAction),
    PartsPeerFetchPending(TransitionFrontierSyncLedgerStagedPartsPeerFetchPendingAction),
    PartsPeerFetchError(TransitionFrontierSyncLedgerStagedPartsPeerFetchErrorAction),
    PartsPeerFetchSuccess(TransitionFrontierSyncLedgerStagedPartsPeerFetchSuccessAction),
    PartsPeerInvalid(TransitionFrontierSyncLedgerStagedPartsPeerInvalidAction),
    PartsPeerValid(TransitionFrontierSyncLedgerStagedPartsPeerValidAction),
    PartsFetchSuccess(TransitionFrontierSyncLedgerStagedPartsFetchSuccessAction),
    ReconstructEmpty(TransitionFrontierSyncLedgerStagedReconstructEmptyAction),
    ReconstructInit(TransitionFrontierSyncLedgerStagedReconstructInitAction),
    ReconstructPending(TransitionFrontierSyncLedgerStagedReconstructPendingAction),
    ReconstructError(TransitionFrontierSyncLedgerStagedReconstructErrorAction),
    ReconstructSuccess(TransitionFrontierSyncLedgerStagedReconstructSuccessAction),
    Success(TransitionFrontierSyncLedgerStagedSuccessAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedPartsFetchPendingAction {}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedPartsFetchPendingAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.snarked())
            .map_or(false, |s| match s {
                TransitionFrontierSyncLedgerSnarkedState::Success { target, .. } => {
                    target.staged.is_some()
                }
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedPartsPeerFetchInitAction {}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedPartsPeerFetchInitAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.staged())
            .map_or(false, |staged| {
                let iter = state.p2p.ready_rpc_peers_iter();
                staged.filter_available_peers(iter).next().is_some()
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedPartsPeerFetchPendingAction {
    pub peer_id: PeerId,
    pub rpc_id: P2pRpcId,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedPartsPeerFetchPendingAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.staged())
            .map_or(false, |s| {
                matches!(
                    s,
                    TransitionFrontierSyncLedgerStagedState::PartsFetchPending { .. }
                )
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedPartsPeerFetchErrorAction {
    pub peer_id: PeerId,
    pub rpc_id: P2pRpcId,
    pub error: PeerStagedLedgerPartsFetchError,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedPartsPeerFetchErrorAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.staged()?.fetch_attempts()?.get(&self.peer_id))
            .and_then(|s| s.fetch_pending_rpc_id())
            .map_or(false, |rpc_id| rpc_id == self.rpc_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedPartsPeerFetchSuccessAction {
    pub peer_id: PeerId,
    pub rpc_id: P2pRpcId,
    pub parts: Arc<StagedLedgerAuxAndPendingCoinbases>,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedPartsPeerFetchSuccessAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.staged()?.fetch_attempts()?.get(&self.peer_id))
            .and_then(|s| s.fetch_pending_rpc_id())
            .map_or(false, |rpc_id| rpc_id == self.rpc_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedPartsPeerInvalidAction {
    pub sender: PeerId,
    pub parts: Arc<StagedLedgerAuxAndPendingCoinbases>,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedPartsPeerInvalidAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.staged()?.fetch_attempts()?.get(&self.sender))
            .map_or(false, |s| match s {
                PeerStagedLedgerPartsFetchState::Success { parts, .. } => !parts.is_valid(),
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedPartsPeerValidAction {
    pub sender: PeerId,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedPartsPeerValidAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.staged()?.fetch_attempts()?.get(&self.sender))
            .map_or(false, |s| match s {
                PeerStagedLedgerPartsFetchState::Success { parts, .. } => parts.is_valid(),
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedPartsFetchSuccessAction {
    pub sender: PeerId,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedPartsFetchSuccessAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.staged()?.fetch_attempts()?.get(&self.sender))
            .map_or(false, |s| s.is_valid())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedReconstructEmptyAction {}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedReconstructEmptyAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
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
                let empty_hash = &[0; 32];
                target.snarked_ledger_hash == hashes.non_snark.ledger_hash
                    && hashes.non_snark.aux_hash.as_ref() == empty_hash
                    && hashes.non_snark.pending_coinbase_aux.as_ref() == empty_hash
                // TODO(binier): `pending_coinbase_hash` isn't empty hash.
                // Do we need to check it?
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedReconstructInitAction {}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedReconstructInitAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
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
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedReconstructPendingAction {}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedReconstructPendingAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
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
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedReconstructErrorAction {
    pub error: String,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedReconstructErrorAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.staged())
            .map_or(false, |s| {
                matches!(
                    s,
                    TransitionFrontierSyncLedgerStagedState::ReconstructPending { .. }
                )
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedReconstructSuccessAction {}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedReconstructSuccessAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.staged())
            .map_or(false, |s| {
                matches!(
                    s,
                    TransitionFrontierSyncLedgerStagedState::ReconstructPending { .. }
                )
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedSuccessAction {}

impl redux::EnablingCondition<crate::State> for TransitionFrontierSyncLedgerStagedSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .ledger()
            .and_then(|s| s.staged())
            .map_or(false, |s| {
                matches!(
                    s,
                    TransitionFrontierSyncLedgerStagedState::ReconstructSuccess { .. }
                )
            })
    }
}

use crate::transition_frontier::{
    sync::{ledger::TransitionFrontierSyncLedgerAction, TransitionFrontierSyncAction},
    TransitionFrontierAction,
};

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::TransitionFrontier(TransitionFrontierAction::Sync(
                    TransitionFrontierSyncAction::Ledger(
                        TransitionFrontierSyncLedgerAction::Staged(value.into()),
                    ),
                ))
            }
        }
    };
}

impl_into_global_action!(TransitionFrontierSyncLedgerStagedPartsFetchPendingAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedPartsPeerFetchInitAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedPartsPeerFetchPendingAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedPartsPeerFetchErrorAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedPartsPeerFetchSuccessAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedPartsPeerInvalidAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedPartsPeerValidAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedPartsFetchSuccessAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedReconstructEmptyAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedReconstructInitAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedReconstructPendingAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedReconstructErrorAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedReconstructSuccessAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedSuccessAction);
