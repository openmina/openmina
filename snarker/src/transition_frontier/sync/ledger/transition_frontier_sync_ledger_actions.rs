use std::sync::Arc;

use mina_p2p_messages::v2::{LedgerHash, MinaBaseAccountBinableArgStableV2};
use serde::{Deserialize, Serialize};

use crate::ledger::{LedgerAddress, LEDGER_DEPTH};
use crate::p2p::channels::rpc::{P2pRpcId, StagedLedgerAuxAndPendingCoinbases};
use crate::p2p::PeerId;

use super::{
    PeerLedgerQueryError, PeerLedgerQueryResponse, PeerRpcState, PeerStagedLedgerReconstructState,
    TransitionFrontierSyncLedgerState,
};

pub type TransitionFrontierSyncLedgerActionWithMeta =
    redux::ActionWithMeta<TransitionFrontierSyncLedgerAction>;
pub type TransitionFrontierSyncLedgerActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a TransitionFrontierSyncLedgerAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncLedgerAction {
    Init(TransitionFrontierSyncLedgerInitAction),
    SnarkedLedgerSyncPending(TransitionFrontierSyncLedgerSnarkedLedgerSyncPendingAction),
    SnarkedLedgerSyncPeersQuery(TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction),
    SnarkedLedgerSyncPeerQueryInit(
        TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryInitAction,
    ),
    SnarkedLedgerSyncPeerQueryPending(
        TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryPendingAction,
    ),
    SnarkedLedgerSyncPeerQueryRetry(
        TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryRetryAction,
    ),
    SnarkedLedgerSyncPeerQueryError(
        TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryErrorAction,
    ),
    SnarkedLedgerSyncPeerQuerySuccess(
        TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQuerySuccessAction,
    ),
    SnarkedLedgerSyncChildHashesReceived(
        TransitionFrontierSyncLedgerSnarkedLedgerSyncChildHashesReceivedAction,
    ),
    SnarkedLedgerSyncChildAccountsReceived(
        TransitionFrontierSyncLedgerSnarkedLedgerSyncChildAccountsReceivedAction,
    ),
    SnarkedLedgerSyncSuccess(TransitionFrontierSyncLedgerSnarkedLedgerSyncSuccessAction),
    StagedLedgerReconstructPending(
        TransitionFrontierSyncLedgerStagedLedgerReconstructPendingAction,
    ),
    StagedLedgerPartsFetchInit(TransitionFrontierSyncLedgerStagedLedgerPartsFetchInitAction),
    StagedLedgerPartsFetchPending(TransitionFrontierSyncLedgerStagedLedgerPartsFetchPendingAction),
    StagedLedgerPartsFetchError(TransitionFrontierSyncLedgerStagedLedgerPartsFetchErrorAction),
    StagedLedgerPartsFetchSuccess(TransitionFrontierSyncLedgerStagedLedgerPartsFetchSuccessAction),
    StagedLedgerPartsApplyInit(TransitionFrontierSyncLedgerStagedLedgerPartsApplyInitAction),
    StagedLedgerPartsApplySuccess(TransitionFrontierSyncLedgerStagedLedgerPartsApplySuccessAction),
    StagedLedgerReconstructSuccess(
        TransitionFrontierSyncLedgerStagedLedgerReconstructSuccessAction,
    ),
    Success(TransitionFrontierSyncLedgerSuccessAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerInitAction {}

impl redux::EnablingCondition<crate::State> for TransitionFrontierSyncLedgerInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .map_or(false, |s| {
                matches!(s, TransitionFrontierSyncLedgerState::Init { .. })
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedLedgerSyncPendingAction {}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedLedgerSyncPendingAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .map_or(false, |s| {
                matches!(s, TransitionFrontierSyncLedgerState::Init { .. })
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction {}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        let peers_available = state
            .p2p
            .ready_peers_iter()
            .any(|(_, p)| p.channels.rpc.can_send_request());
        peers_available
            && state
                .transition_frontier
                .sync
                .root_ledger()
                .map_or(false, |s| {
                    s.snarked_ledger_sync_next().is_some()
                        || s.snarked_ledger_sync_retry_iter().next().is_some()
                })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryInitAction {
    pub address: LedgerAddress,
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryInitAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        let check_next_addr =
            state
                .transition_frontier
                .sync
                .root_ledger()
                .map_or(false, |s| match s {
                    TransitionFrontierSyncLedgerState::SnarkedLedgerSyncPending {
                        pending,
                        next_addr,
                        ..
                    } => next_addr.as_ref().map_or(false, |next_addr| {
                        next_addr == &self.address
                            && (next_addr.to_index().0 != 0 || pending.is_empty())
                    }),
                    _ => false,
                });

        let check_peer_available = state
            .p2p
            .get_ready_peer(&self.peer_id)
            .and_then(|p| {
                let sync_best_tip = state.transition_frontier.sync.best_tip()?;
                let peer_best_tip = p.best_tip.as_ref()?;
                Some(p).filter(|_| sync_best_tip.hash == peer_best_tip.hash)
            })
            .map_or(false, |p| p.channels.rpc.can_send_request());
        check_next_addr && check_peer_available
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryRetryAction {
    pub address: LedgerAddress,
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryRetryAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        let check_next_addr = state
            .transition_frontier
            .sync
            .root_ledger()
            .and_then(|s| s.snarked_ledger_sync_retry_iter().next())
            .map_or(false, |addr| addr == self.address);

        let check_peer_available = state
            .p2p
            .get_ready_peer(&self.peer_id)
            .and_then(|p| {
                let sync_best_tip = state.transition_frontier.sync.best_tip()?;
                let peer_best_tip = p.best_tip.as_ref()?;
                Some(p).filter(|_| sync_best_tip.hash == peer_best_tip.hash)
            })
            .map_or(false, |p| p.channels.rpc.can_send_request());
        check_next_addr && check_peer_available
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryPendingAction {
    pub address: LedgerAddress,
    pub peer_id: PeerId,
    pub rpc_id: P2pRpcId,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryPendingAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .map_or(false, |s| match s {
                TransitionFrontierSyncLedgerState::SnarkedLedgerSyncPending { pending, .. } => {
                    pending
                        .iter()
                        .filter_map(|(_, query_state)| query_state.attempts.get(&self.peer_id))
                        .any(|peer_rpc_state| matches!(peer_rpc_state, PeerRpcState::Init { .. }))
                }
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryErrorAction {
    pub peer_id: PeerId,
    pub rpc_id: P2pRpcId,
    pub error: PeerLedgerQueryError,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryErrorAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .map_or(false, |s| {
                s.snarked_ledger_peer_query_get(&self.peer_id, self.rpc_id)
                    .and_then(|(_, s)| s.attempts.get(&self.peer_id))
                    .map_or(false, |s| matches!(s, PeerRpcState::Pending { .. }))
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQuerySuccessAction {
    pub peer_id: PeerId,
    pub rpc_id: P2pRpcId,
    pub response: PeerLedgerQueryResponse,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQuerySuccessAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .map_or(false, |s| {
                // TODO(binier): check if expected response
                // kind is correct.
                s.snarked_ledger_peer_query_get(&self.peer_id, self.rpc_id)
                    .and_then(|(_, s)| s.attempts.get(&self.peer_id))
                    .map_or(false, |s| matches!(s, PeerRpcState::Pending { .. }))
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedLedgerSyncChildHashesReceivedAction {
    pub address: LedgerAddress,
    pub hashes: (LedgerHash, LedgerHash),
    pub sender: PeerId,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedLedgerSyncChildHashesReceivedAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.address.length() < LEDGER_DEPTH - 1
            && state
                .transition_frontier
                .sync
                .root_ledger()
                .and_then(|s| match s {
                    TransitionFrontierSyncLedgerState::SnarkedLedgerSyncPending {
                        pending, ..
                    } => pending.get(&self.address),
                    _ => None,
                })
                .and_then(|s| s.attempts.get(&self.sender))
                .map_or(false, |s| s.is_success())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedLedgerSyncChildAccountsReceivedAction {
    pub address: LedgerAddress,
    pub accounts: Vec<MinaBaseAccountBinableArgStableV2>,
    pub sender: PeerId,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedLedgerSyncChildAccountsReceivedAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .and_then(|s| match s {
                TransitionFrontierSyncLedgerState::SnarkedLedgerSyncPending { pending, .. } => {
                    pending.get(&self.address)
                }
                _ => None,
            })
            .and_then(|s| s.attempts.get(&self.sender))
            // TODO(binier): check if expected response
            // kind is correct.
            .map_or(false, |s| s.is_success())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSnarkedLedgerSyncSuccessAction {}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerSnarkedLedgerSyncSuccessAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .map_or(false, |s| match s {
                TransitionFrontierSyncLedgerState::SnarkedLedgerSyncPending {
                    pending,
                    next_addr,
                    ..
                } => next_addr.is_none() && pending.is_empty(),
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedLedgerReconstructPendingAction {}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedLedgerReconstructPendingAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .map_or(false, |s| {
                matches!(
                    s,
                    TransitionFrontierSyncLedgerState::SnarkedLedgerSyncSuccess { .. }
                )
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedLedgerPartsFetchInitAction {}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedLedgerPartsFetchInitAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        let Some(root_ledger) = state.transition_frontier.sync.root_ledger() else { return false };

        let iter = state.p2p.ready_rpc_peers_iter();
        root_ledger
            .staged_ledger_reconstruct_filter_available_peers(iter)
            .next()
            .is_some()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedLedgerPartsFetchPendingAction {
    pub peer_id: PeerId,
    pub rpc_id: P2pRpcId,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedLedgerPartsFetchPendingAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .map_or(false, |s| {
                matches!(
                    s,
                    TransitionFrontierSyncLedgerState::StagedLedgerReconstructPending { .. }
                )
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedLedgerPartsFetchErrorAction {
    pub peer_id: PeerId,
    pub rpc_id: P2pRpcId,
    pub error: PeerLedgerQueryError,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedLedgerPartsFetchErrorAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .and_then(|s| match s {
                TransitionFrontierSyncLedgerState::StagedLedgerReconstructPending {
                    attempts,
                    ..
                } => attempts.get(&self.peer_id),
                _ => None,
            })
            .and_then(|s| s.fetch_pending_rpc_id())
            .map_or(false, |rpc_id| rpc_id == self.rpc_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedLedgerPartsFetchSuccessAction {
    pub peer_id: PeerId,
    pub rpc_id: P2pRpcId,
    pub parts: Arc<StagedLedgerAuxAndPendingCoinbases>,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedLedgerPartsFetchSuccessAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .and_then(|s| match s {
                TransitionFrontierSyncLedgerState::StagedLedgerReconstructPending {
                    attempts,
                    ..
                } => attempts.get(&self.peer_id),
                _ => None,
            })
            .and_then(|s| s.fetch_pending_rpc_id())
            .map_or(false, |rpc_id| rpc_id == self.rpc_id)
    }
}
// TODO(binier): validate staged ledger hash in fetched parts.

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedLedgerPartsApplyInitAction {
    pub sender: PeerId,
    pub parts: Arc<StagedLedgerAuxAndPendingCoinbases>,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedLedgerPartsApplyInitAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .and_then(|s| match s {
                TransitionFrontierSyncLedgerState::StagedLedgerReconstructPending {
                    attempts,
                    ..
                } => attempts.get(&self.sender),
                _ => None,
            })
            .map_or(false, |s| match s {
                PeerStagedLedgerReconstructState::PartsFetchSuccess { parts, .. } => {
                    Arc::ptr_eq(parts, &self.parts)
                }
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedLedgerPartsApplySuccessAction {
    pub sender: PeerId,
}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedLedgerPartsApplySuccessAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .and_then(|s| match s {
                TransitionFrontierSyncLedgerState::StagedLedgerReconstructPending {
                    attempts,
                    ..
                } => attempts.get(&self.sender),
                _ => None,
            })
            .map_or(false, |s| s.is_fetch_success())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerStagedLedgerReconstructSuccessAction {}

impl redux::EnablingCondition<crate::State>
    for TransitionFrontierSyncLedgerStagedLedgerReconstructSuccessAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .map_or(false, |s| match s {
                TransitionFrontierSyncLedgerState::StagedLedgerReconstructPending {
                    attempts,
                    ..
                } => attempts.iter().any(|(_, s)| s.is_apply_success()),
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierSyncLedgerSuccessAction {}

impl redux::EnablingCondition<crate::State> for TransitionFrontierSyncLedgerSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .transition_frontier
            .sync
            .root_ledger()
            .map_or(false, |s| {
                matches!(
                    s,
                    TransitionFrontierSyncLedgerState::StagedLedgerReconstructSuccess { .. }
                )
            })
    }
}

use crate::transition_frontier::TransitionFrontierAction;

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::TransitionFrontier(TransitionFrontierAction::SyncLedger(value.into()))
            }
        }
    };
}

impl_into_global_action!(TransitionFrontierSyncLedgerInitAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedLedgerSyncPendingAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryInitAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryPendingAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryErrorAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQueryRetryAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedLedgerSyncPeerQuerySuccessAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedLedgerSyncChildHashesReceivedAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedLedgerSyncChildAccountsReceivedAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSnarkedLedgerSyncSuccessAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedLedgerReconstructPendingAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedLedgerPartsFetchInitAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedLedgerPartsFetchPendingAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedLedgerPartsFetchErrorAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedLedgerPartsFetchSuccessAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedLedgerPartsApplyInitAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedLedgerPartsApplySuccessAction);
impl_into_global_action!(TransitionFrontierSyncLedgerStagedLedgerReconstructSuccessAction);
impl_into_global_action!(TransitionFrontierSyncLedgerSuccessAction);
