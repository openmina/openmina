use serde::{Deserialize, Serialize};

use openmina_core::requests::RpcId;

use crate::{webrtc, P2pState, PeerId, P2pPeerStatus, connection::P2pConnectionState};

use super::{P2pConnectionWebRTCIncomingInitOpts, P2pConnectionWebRTCIncomingState, P2pConnectionWebRTCIncomingError};

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionWebRTCIncomingAction {
    Init(P2pConnectionWebRTCIncomingInitAction),
    AnswerSdpCreatePending(P2pConnectionWebRTCIncomingAnswerSdpCreatePendingAction),
    AnswerSdpCreateError(P2pConnectionWebRTCIncomingAnswerSdpCreateErrorAction),
    AnswerSdpCreateSuccess(P2pConnectionWebRTCIncomingAnswerSdpCreateSuccessAction),
    AnswerReady(P2pConnectionWebRTCIncomingAnswerReadyAction),
    AnswerSendSuccess(P2pConnectionWebRTCIncomingAnswerSendSuccessAction),
    FinalizePending(P2pConnectionWebRTCIncomingFinalizePendingAction),
    FinalizeError(P2pConnectionWebRTCIncomingFinalizeErrorAction),
    FinalizeSuccess(P2pConnectionWebRTCIncomingFinalizeSuccessAction),
    Timeout(P2pConnectionWebRTCIncomingTimeoutAction),
    Error(P2pConnectionWebRTCIncomingErrorAction),
    Success(P2pConnectionWebRTCIncomingSuccessAction),
    Libp2pReceived(P2pConnectionWebRTCIncomingLibp2pReceivedAction),
}

impl P2pConnectionWebRTCIncomingAction {
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
            Self::Timeout(v) => Some(&v.peer_id),
            Self::Error(v) => Some(&v.peer_id),
            Self::Success(v) => Some(&v.peer_id),
            Self::Libp2pReceived(v) => Some(&v.peer_id),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCIncomingInitAction {
    pub opts: P2pConnectionWebRTCIncomingInitOpts,
    pub rpc_id: Option<RpcId>,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCIncomingInitAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .incoming_accept(self.opts.peer_id, &self.opts.offer)
            .is_ok()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCIncomingAnswerSdpCreatePendingAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCIncomingAnswerSdpCreatePendingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionWebRTCIncomingState::Init { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCIncomingAnswerSdpCreateErrorAction {
    pub peer_id: PeerId,
    pub error: String,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCIncomingAnswerSdpCreateErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionWebRTCIncomingState::AnswerSdpCreatePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCIncomingAnswerSdpCreateSuccessAction {
    pub peer_id: PeerId,
    pub sdp: String,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCIncomingAnswerSdpCreateSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionWebRTCIncomingState::AnswerSdpCreatePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCIncomingAnswerReadyAction {
    pub peer_id: PeerId,
    pub answer: webrtc::Answer,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCIncomingAnswerReadyAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionWebRTCIncomingState::AnswerSdpCreateSuccess { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCIncomingAnswerSendSuccessAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCIncomingAnswerSendSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionWebRTCIncomingState::AnswerReady { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCIncomingFinalizePendingAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCIncomingFinalizePendingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionWebRTCIncomingState::AnswerSendSuccess { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCIncomingFinalizeErrorAction {
    pub peer_id: PeerId,
    pub error: String,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCIncomingFinalizeErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionWebRTCIncomingState::FinalizePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCIncomingFinalizeSuccessAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCIncomingFinalizeSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionWebRTCIncomingState::FinalizePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCIncomingTimeoutAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCIncomingTimeoutAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .and_then(|peer| peer.status.as_connecting()?.as_incoming())
            .is_some()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCIncomingErrorAction {
    pub peer_id: PeerId,
    pub error: P2pConnectionWebRTCIncomingError,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCIncomingErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(s)) => match &self.error {
                    P2pConnectionWebRTCIncomingError::SdpCreateError(_) => {
                        matches!(s, P2pConnectionWebRTCIncomingState::AnswerSdpCreatePending { .. })
                    }
                    P2pConnectionWebRTCIncomingError::FinalizeError(_) => {
                        matches!(s, P2pConnectionWebRTCIncomingState::FinalizePending { .. })
                    }
                    P2pConnectionWebRTCIncomingError::Timeout => true,
                },
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCIncomingSuccessAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCIncomingSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionWebRTCIncomingState::FinalizeSuccess { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCIncomingLibp2pReceivedAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCIncomingLibp2pReceivedAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_libp2p_peer(&self.peer_id).map_or(true, |peer| {
            matches!(&peer.status, P2pPeerStatus::Disconnected { .. })
        })
    }
}

macro_rules! into_p2p_action {
    ($($action:ident),* $(,)?) => {
        $(
            impl From<$action> for crate::P2pAction {
                fn from(value: $action) -> Self {
                    crate::P2pAction::Connection(crate::connection::P2pConnectionAction::WebRTC(crate::connection::webrtc::P2pConnectionWebRTCAction::Incoming(value.into())))
                }
            }
        )*
    };
}

into_p2p_action!(
    P2pConnectionWebRTCIncomingInitAction,
    P2pConnectionWebRTCIncomingAnswerSdpCreatePendingAction,
    P2pConnectionWebRTCIncomingAnswerSdpCreateErrorAction,
    P2pConnectionWebRTCIncomingAnswerSdpCreateSuccessAction,
    P2pConnectionWebRTCIncomingAnswerReadyAction,
    P2pConnectionWebRTCIncomingAnswerSendSuccessAction,
    P2pConnectionWebRTCIncomingFinalizePendingAction,
    P2pConnectionWebRTCIncomingFinalizeErrorAction,
    P2pConnectionWebRTCIncomingFinalizeSuccessAction,
    P2pConnectionWebRTCIncomingTimeoutAction,
    P2pConnectionWebRTCIncomingErrorAction,
    P2pConnectionWebRTCIncomingSuccessAction,
    P2pConnectionWebRTCIncomingLibp2pReceivedAction,
);

