use serde::{Deserialize, Serialize};

use shared::requests::RpcId;

use crate::{webrtc, P2pState, PeerId};

pub type P2pConnectionIncomingActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a P2pConnectionIncomingAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionIncomingAction {
    Init(P2pConnectionIncomingInitAction),
    AnswerSdpCreatePending(P2pConnectionIncomingAnswerSdpCreatePendingAction),
    AnswerSdpCreateError(P2pConnectionIncomingAnswerSdpCreateErrorAction),
    AnswerSdpCreateSuccess(P2pConnectionIncomingAnswerSdpCreateSuccessAction),
    AnswerReady(P2pConnectionIncomingAnswerReadyAction),
    AnswerSendSuccess(P2pConnectionIncomingAnswerSendSuccessAction),
    FinalizePending(P2pConnectionIncomingFinalizePendingAction),
    FinalizeError(P2pConnectionIncomingFinalizeErrorAction),
    FinalizeSuccess(P2pConnectionIncomingFinalizeSuccessAction),
    Error(P2pConnectionIncomingErrorAction),
    Success(P2pConnectionIncomingSuccessAction),
}

impl P2pConnectionIncomingAction {
    pub fn peer_id(&self) -> Option<&PeerId> {
        match self {
            Self::Init(v) => Some(&v.opts.peer_id),
            Self::AnswerSdpCreatePending(v) => Some(&v.peer_id),
            Self::AnswerSdpCreateError(v) => Some(&v.peer_id),
            Self::AnswerSdpCreateSuccess(v) => Some(&v.peer_id),
            Self::AnswerReady(v) => Some(&v.peer_id),
            Self::AnswerSendSuccess(v) => Some(&v.peer_id),
            Self::FinalizePending(v) => Some(&v.peer_id),
            Self::FinalizeError(v) => Some(&v.peer_id),
            Self::FinalizeSuccess(v) => Some(&v.peer_id),
            Self::Error(v) => Some(&v.peer_id),
            Self::Success(v) => Some(&v.peer_id),
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct P2pConnectionIncomingInitOpts {
    pub peer_id: PeerId,
    pub signaling: IncomingSignalingMethod,
    pub offer: webrtc::Offer,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionIncomingInitAction {
    pub opts: P2pConnectionIncomingInitOpts,
    pub rpc_id: Option<RpcId>,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionIncomingInitAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .incoming_accept(self.opts.peer_id, &self.opts.offer)
            .is_ok()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionIncomingAnswerSdpCreatePendingAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionIncomingAnswerSdpCreatePendingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionIncomingState::Init { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionIncomingAnswerSdpCreateErrorAction {
    pub peer_id: PeerId,
    pub error: String,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionIncomingAnswerSdpCreateErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionIncomingState::AnswerSdpCreatePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionIncomingAnswerSdpCreateSuccessAction {
    pub peer_id: PeerId,
    pub sdp: String,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionIncomingAnswerSdpCreateSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionIncomingState::AnswerSdpCreatePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionIncomingAnswerReadyAction {
    pub peer_id: PeerId,
    pub answer: webrtc::Answer,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionIncomingAnswerReadyAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionIncomingState::AnswerSdpCreateSuccess { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionIncomingAnswerSendSuccessAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionIncomingAnswerSendSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionIncomingState::AnswerReady { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionIncomingFinalizePendingAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionIncomingFinalizePendingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionIncomingState::AnswerSendSuccess { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionIncomingFinalizeErrorAction {
    pub peer_id: PeerId,
    pub error: String,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionIncomingFinalizeErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionIncomingState::FinalizePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionIncomingFinalizeSuccessAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionIncomingFinalizeSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionIncomingState::FinalizePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionIncomingErrorAction {
    pub peer_id: PeerId,
    pub error: P2pConnectionIncomingError,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionIncomingErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(s)) => match &self.error {
                    P2pConnectionIncomingError::SdpCreateError(_) => {
                        matches!(s, P2pConnectionIncomingState::AnswerSdpCreatePending { .. })
                    }
                    P2pConnectionIncomingError::FinalizeError(_) => {
                        matches!(s, P2pConnectionIncomingState::FinalizePending { .. })
                    }
                },
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionIncomingSuccessAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionIncomingSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionIncomingState::FinalizeSuccess { .. },
                )) => true,
                _ => false,
            })
    }
}

// --- From<LeafAction> for Action impls.
use crate::{
    connection::{P2pConnectionAction, P2pConnectionState},
    P2pPeerStatus,
};

use super::{IncomingSignalingMethod, P2pConnectionIncomingError, P2pConnectionIncomingState};

impl From<P2pConnectionIncomingInitAction> for crate::P2pAction {
    fn from(a: P2pConnectionIncomingInitAction) -> Self {
        Self::Connection(P2pConnectionAction::Incoming(a.into()))
    }
}

impl From<P2pConnectionIncomingAnswerSdpCreatePendingAction> for crate::P2pAction {
    fn from(a: P2pConnectionIncomingAnswerSdpCreatePendingAction) -> Self {
        Self::Connection(P2pConnectionAction::Incoming(a.into()))
    }
}

impl From<P2pConnectionIncomingAnswerSdpCreateErrorAction> for crate::P2pAction {
    fn from(a: P2pConnectionIncomingAnswerSdpCreateErrorAction) -> Self {
        Self::Connection(P2pConnectionAction::Incoming(a.into()))
    }
}

impl From<P2pConnectionIncomingAnswerSdpCreateSuccessAction> for crate::P2pAction {
    fn from(a: P2pConnectionIncomingAnswerSdpCreateSuccessAction) -> Self {
        Self::Connection(P2pConnectionAction::Incoming(a.into()))
    }
}

impl From<P2pConnectionIncomingAnswerReadyAction> for crate::P2pAction {
    fn from(a: P2pConnectionIncomingAnswerReadyAction) -> Self {
        Self::Connection(P2pConnectionAction::Incoming(a.into()))
    }
}

impl From<P2pConnectionIncomingAnswerSendSuccessAction> for crate::P2pAction {
    fn from(a: P2pConnectionIncomingAnswerSendSuccessAction) -> Self {
        Self::Connection(P2pConnectionAction::Incoming(a.into()))
    }
}

impl From<P2pConnectionIncomingFinalizePendingAction> for crate::P2pAction {
    fn from(a: P2pConnectionIncomingFinalizePendingAction) -> Self {
        Self::Connection(P2pConnectionAction::Incoming(a.into()))
    }
}

impl From<P2pConnectionIncomingFinalizeErrorAction> for crate::P2pAction {
    fn from(a: P2pConnectionIncomingFinalizeErrorAction) -> Self {
        Self::Connection(P2pConnectionAction::Incoming(a.into()))
    }
}

impl From<P2pConnectionIncomingFinalizeSuccessAction> for crate::P2pAction {
    fn from(a: P2pConnectionIncomingFinalizeSuccessAction) -> Self {
        Self::Connection(P2pConnectionAction::Incoming(a.into()))
    }
}

impl From<P2pConnectionIncomingErrorAction> for crate::P2pAction {
    fn from(a: P2pConnectionIncomingErrorAction) -> Self {
        Self::Connection(P2pConnectionAction::Incoming(a.into()))
    }
}

impl From<P2pConnectionIncomingSuccessAction> for crate::P2pAction {
    fn from(a: P2pConnectionIncomingSuccessAction) -> Self {
        Self::Connection(P2pConnectionAction::Incoming(a.into()))
    }
}
