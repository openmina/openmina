use serde::{Deserialize, Serialize};

use crate::rpc::{P2pRpcId, P2pRpcOutgoingError, P2pRpcRequest, P2pRpcResponse};

use super::P2pRpcOutgoingStatus;

pub type P2pRpcOutgoingActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pRpcOutgoingAction>;
pub type P2pRpcOutgoingActionWithMeta = redux::ActionWithMeta<P2pRpcOutgoingAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcOutgoingAction {
    Init(P2pRpcOutgoingInitAction),
    Pending(P2pRpcOutgoingPendingAction),
    Error(P2pRpcOutgoingErrorAction),
    Success(P2pRpcOutgoingSuccessAction),
    Finish(P2pRpcOutgoingFinishAction),
}

impl P2pRpcOutgoingAction {
    pub fn peer_id(&self) -> &crate::PeerId {
        match self {
            Self::Init(v) => &v.peer_id,
            Self::Pending(v) => &v.peer_id,
            Self::Error(v) => &v.peer_id,
            Self::Success(v) => &v.peer_id,
            Self::Finish(v) => &v.peer_id,
        }
        // match self {}
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pRpcOutgoingInitAction {
    pub peer_id: crate::PeerId,
    pub rpc_id: P2pRpcId,
    pub request: P2pRpcRequest,
}

impl redux::EnablingCondition<crate::P2pState> for P2pRpcOutgoingInitAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            p.rpc.supports(self.request.kind()) && p.rpc.outgoing.next_req_id() == self.rpc_id
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pRpcOutgoingPendingAction {
    pub peer_id: crate::PeerId,
    pub rpc_id: P2pRpcId,
}

impl redux::EnablingCondition<crate::P2pState> for P2pRpcOutgoingPendingAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        state
            .get_ready_peer(&self.peer_id)
            .and_then(|p| p.rpc.outgoing.get(self.rpc_id))
            .map_or(false, |v| v.is_init())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pRpcOutgoingErrorAction {
    pub peer_id: crate::PeerId,
    pub rpc_id: P2pRpcId,
    pub error: P2pRpcOutgoingError,
}

impl redux::EnablingCondition<crate::P2pState> for P2pRpcOutgoingErrorAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        state
            .get_ready_peer(&self.peer_id)
            .and_then(|p| p.rpc.outgoing.get(self.rpc_id))
            .map_or(false, |v| v.is_pending())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pRpcOutgoingSuccessAction {
    pub peer_id: crate::PeerId,
    pub rpc_id: P2pRpcId,
    pub response: P2pRpcResponse,
}

impl redux::EnablingCondition<crate::P2pState> for P2pRpcOutgoingSuccessAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        state
            .get_ready_peer(&self.peer_id)
            .and_then(|p| p.rpc.outgoing.get(self.rpc_id))
            .map_or(false, |v| match v {
                P2pRpcOutgoingStatus::Pending { request, .. } => {
                    request.kind() == self.response.kind()
                }
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pRpcOutgoingFinishAction {
    pub peer_id: crate::PeerId,
    pub rpc_id: P2pRpcId,
}

impl redux::EnablingCondition<crate::P2pState> for P2pRpcOutgoingFinishAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        state
            .get_ready_peer(&self.peer_id)
            .and_then(|p| p.rpc.outgoing.get(self.rpc_id))
            .map_or(false, |v| v.is_finished())
    }
}

macro_rules! impl_into_p2p_action {
    ($a:ty) => {
        impl From<$a> for crate::P2pAction {
            fn from(value: $a) -> Self {
                Self::Rpc(crate::rpc::P2pRpcAction::Outgoing(value.into()))
            }
        }
    };
}

impl_into_p2p_action!(P2pRpcOutgoingInitAction);
impl_into_p2p_action!(P2pRpcOutgoingPendingAction);
impl_into_p2p_action!(P2pRpcOutgoingErrorAction);
impl_into_p2p_action!(P2pRpcOutgoingSuccessAction);
impl_into_p2p_action!(P2pRpcOutgoingFinishAction);
