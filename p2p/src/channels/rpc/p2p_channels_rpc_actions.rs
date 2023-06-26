use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

use super::{P2pChannelsRpcState, P2pRpcId, P2pRpcLocalState, P2pRpcRequest, P2pRpcResponse};

pub type P2pChannelsRpcActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pChannelsRpcAction>;

pub const MAX_P2P_RPC_REMOTE_CONCURRENT_REQUESTS: usize = 5;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pChannelsRpcAction {
    Init(P2pChannelsRpcInitAction),
    Pending(P2pChannelsRpcPendingAction),
    Ready(P2pChannelsRpcReadyAction),

    RequestSend(P2pChannelsRpcRequestSendAction),
    ResponseReceived(P2pChannelsRpcResponseReceivedAction),

    RequestReceived(P2pChannelsRpcRequestReceivedAction),
    ResponseSend(P2pChannelsRpcResponseSendAction),
}

impl P2pChannelsRpcAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::Init(v) => &v.peer_id,
            Self::Pending(v) => &v.peer_id,
            Self::Ready(v) => &v.peer_id,
            Self::RequestSend(v) => &v.peer_id,
            Self::ResponseReceived(v) => &v.peer_id,
            Self::RequestReceived(v) => &v.peer_id,
            Self::ResponseSend(v) => &v.peer_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsRpcInitAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsRpcInitAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            matches!(&p.channels.rpc, P2pChannelsRpcState::Enabled)
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsRpcPendingAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsRpcPendingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            matches!(&p.channels.rpc, P2pChannelsRpcState::Init { .. })
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsRpcReadyAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsRpcReadyAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            matches!(&p.channels.rpc, P2pChannelsRpcState::Pending { .. })
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsRpcRequestSendAction {
    pub peer_id: PeerId,
    pub id: P2pRpcId,
    pub request: P2pRpcRequest,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsRpcRequestSendAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_ready_peer(&self.peer_id)
            .map_or(false, |p| match &p.channels.rpc {
                P2pChannelsRpcState::Ready {
                    local,
                    next_local_rpc_id,
                    ..
                } => {
                    *next_local_rpc_id == self.id
                        && matches!(
                            local,
                            P2pRpcLocalState::WaitingForRequest { .. }
                                | P2pRpcLocalState::Responded { .. }
                        )
                }
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsRpcResponseReceivedAction {
    pub peer_id: PeerId,
    pub id: P2pRpcId,
    pub response: Option<P2pRpcResponse>,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsRpcResponseReceivedAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        // TODO(binier): use consensus to enforce that peer doesn't send
        // us inferrior block than it has in the past.
        state
            .get_ready_peer(&self.peer_id)
            .map_or(false, |p| match &p.channels.rpc {
                P2pChannelsRpcState::Ready { local, .. } => match local {
                    // TODO(binier): validate that response corresponds to request.
                    P2pRpcLocalState::Requested { id, .. } => *id == self.id,
                    _ => false,
                },
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsRpcRequestReceivedAction {
    pub peer_id: PeerId,
    pub id: P2pRpcId,
    pub request: P2pRpcRequest,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsRpcRequestReceivedAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_ready_peer(&self.peer_id)
            .map_or(false, |p| match &p.channels.rpc {
                P2pChannelsRpcState::Ready { remote, .. } => {
                    remote.pending_requests.len() < MAX_P2P_RPC_REMOTE_CONCURRENT_REQUESTS
                        && remote.pending_requests.iter().all(|v| v.id != self.id)
                }
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsRpcResponseSendAction {
    pub peer_id: PeerId,
    pub id: P2pRpcId,
    pub response: Option<P2pRpcResponse>,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsRpcResponseSendAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_ready_peer(&self.peer_id)
            .map_or(false, |p| match &p.channels.rpc {
                P2pChannelsRpcState::Ready { remote, .. } => {
                    // TODO(binier): validate that response corresponds to request.
                    remote
                        .pending_requests
                        .iter()
                        .find(|v| v.id == self.id)
                        .is_some()
                }
                _ => false,
            })
    }
}

// --- From<LeafAction> for Action impls.

use crate::channels::P2pChannelsAction;

impl From<P2pChannelsRpcInitAction> for crate::P2pAction {
    fn from(a: P2pChannelsRpcInitAction) -> Self {
        Self::Channels(P2pChannelsAction::Rpc(a.into()))
    }
}

impl From<P2pChannelsRpcPendingAction> for crate::P2pAction {
    fn from(a: P2pChannelsRpcPendingAction) -> Self {
        Self::Channels(P2pChannelsAction::Rpc(a.into()))
    }
}

impl From<P2pChannelsRpcReadyAction> for crate::P2pAction {
    fn from(a: P2pChannelsRpcReadyAction) -> Self {
        Self::Channels(P2pChannelsAction::Rpc(a.into()))
    }
}

impl From<P2pChannelsRpcRequestSendAction> for crate::P2pAction {
    fn from(a: P2pChannelsRpcRequestSendAction) -> Self {
        Self::Channels(P2pChannelsAction::Rpc(a.into()))
    }
}

impl From<P2pChannelsRpcResponseReceivedAction> for crate::P2pAction {
    fn from(a: P2pChannelsRpcResponseReceivedAction) -> Self {
        Self::Channels(P2pChannelsAction::Rpc(a.into()))
    }
}

impl From<P2pChannelsRpcRequestReceivedAction> for crate::P2pAction {
    fn from(a: P2pChannelsRpcRequestReceivedAction) -> Self {
        Self::Channels(P2pChannelsAction::Rpc(a.into()))
    }
}

impl From<P2pChannelsRpcResponseSendAction> for crate::P2pAction {
    fn from(a: P2pChannelsRpcResponseSendAction) -> Self {
        Self::Channels(P2pChannelsAction::Rpc(a.into()))
    }
}
