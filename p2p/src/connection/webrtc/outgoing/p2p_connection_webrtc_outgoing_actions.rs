use serde::{Deserialize, Serialize};

use openmina_core::requests::RpcId;

use crate::connection::webrtc::P2pConnectionWebRTCErrorResponse;
use crate::{webrtc, P2pPeerStatus, P2pState, PeerId};

use super::*;

pub type P2pConnectionWebRTCOutgoingActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a P2pConnectionWebRTCOutgoingAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionWebRTCOutgoingAction {
    Init(P2pConnectionWebRTCOutgoingInitAction),
    OfferSdpCreatePending(P2pConnectionWebRTCOutgoingOfferSdpCreatePendingAction),
    OfferSdpCreateError(P2pConnectionWebRTCOutgoingOfferSdpCreateErrorAction),
    OfferSdpCreateSuccess(P2pConnectionWebRTCOutgoingOfferSdpCreateSuccessAction),
    OfferReady(P2pConnectionWebRTCOutgoingOfferReadyAction),
    OfferSendSuccess(P2pConnectionWebRTCOutgoingOfferSendSuccessAction),
    AnswerRecvPending(P2pConnectionWebRTCOutgoingAnswerRecvPendingAction),
    AnswerRecvError(P2pConnectionWebRTCOutgoingAnswerRecvErrorAction),
    AnswerRecvSuccess(P2pConnectionWebRTCOutgoingAnswerRecvSuccessAction),
    FinalizePending(P2pConnectionWebRTCOutgoingFinalizePendingAction),
    FinalizeError(P2pConnectionWebRTCOutgoingFinalizeErrorAction),
    FinalizeSuccess(P2pConnectionWebRTCOutgoingFinalizeSuccessAction),
    Timeout(P2pConnectionWebRTCOutgoingTimeoutAction),
    Error(P2pConnectionWebRTCOutgoingErrorAction),
    Success(P2pConnectionWebRTCOutgoingSuccessAction),
}

impl P2pConnectionWebRTCOutgoingAction {
    pub fn peer_id(&self) -> Option<&PeerId> {
        match self {
            Self::Init(v) => Some(&v.peer_id),
            Self::OfferSdpCreatePending(v) => Some(&v.peer_id),
            Self::OfferSdpCreateError(v) => Some(&v.peer_id),
            Self::OfferSdpCreateSuccess(v) => Some(&v.peer_id),
            Self::OfferReady(v) => Some(&v.peer_id),
            Self::OfferSendSuccess(v) => Some(&v.peer_id),
            Self::AnswerRecvPending(v) => Some(&v.peer_id),
            Self::AnswerRecvError(v) => Some(&v.peer_id),
            Self::AnswerRecvSuccess(v) => Some(&v.peer_id),
            Self::FinalizePending(v) => Some(&v.peer_id),
            Self::FinalizeError(v) => Some(&v.peer_id),
            Self::FinalizeSuccess(v) => Some(&v.peer_id),
            Self::Timeout(v) => Some(&v.peer_id),
            Self::Error(v) => Some(&v.peer_id),
            Self::Success(v) => Some(&v.peer_id),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCOutgoingInitAction {
    pub peer_id: PeerId,
    pub rpc_id: Option<RpcId>,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCOutgoingInitAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        !state.already_has_min_peers()
            && state.get_webrtc_peer(&self.peer_id).map_or(false, |peer| {
                matches!(
                    &peer.status,
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionWebRTCOutgoingState::Default
                            | P2pConnectionWebRTCOutgoingState::Error { .. }
                    ))
                )
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCOutgoingOfferSdpCreatePendingAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCOutgoingOfferSdpCreatePendingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_webrtc_peer(&self.peer_id).map_or(false, |peer| {
            matches!(
                &peer.status,
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionWebRTCOutgoingState::Init { .. }
                ))
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCOutgoingOfferSdpCreateErrorAction {
    pub peer_id: PeerId,
    pub error: String,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCOutgoingOfferSdpCreateErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionWebRTCOutgoingState::OfferSdpCreatePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCOutgoingOfferSdpCreateSuccessAction {
    pub peer_id: PeerId,
    pub sdp: String,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCOutgoingOfferSdpCreateSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionWebRTCOutgoingState::OfferSdpCreatePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCOutgoingOfferReadyAction {
    pub peer_id: PeerId,
    pub offer: webrtc::Offer,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCOutgoingOfferReadyAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionWebRTCOutgoingState::OfferSdpCreateSuccess { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCOutgoingOfferSendSuccessAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCOutgoingOfferSendSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionWebRTCOutgoingState::OfferReady { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCOutgoingAnswerRecvPendingAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCOutgoingAnswerRecvPendingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionWebRTCOutgoingState::OfferSendSuccess { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCOutgoingAnswerRecvErrorAction {
    pub peer_id: PeerId,
    pub error: P2pConnectionWebRTCErrorResponse,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCOutgoingAnswerRecvErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionWebRTCOutgoingState::AnswerRecvPending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCOutgoingAnswerRecvSuccessAction {
    pub peer_id: PeerId,
    pub answer: webrtc::Answer,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCOutgoingAnswerRecvSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionWebRTCOutgoingState::AnswerRecvPending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCOutgoingFinalizePendingAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCOutgoingFinalizePendingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(v)) => match v {
                    P2pConnectionWebRTCOutgoingState::AnswerRecvSuccess { .. } => true,
                    _ => false,
                },
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCOutgoingFinalizeErrorAction {
    pub peer_id: PeerId,
    pub error: String,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCOutgoingFinalizeErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionWebRTCOutgoingState::FinalizePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCOutgoingFinalizeSuccessAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCOutgoingFinalizeSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionWebRTCOutgoingState::FinalizePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCOutgoingTimeoutAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCOutgoingTimeoutAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .and_then(|peer| peer.status.as_connecting()?.as_outgoing())
            .is_some()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCOutgoingErrorAction {
    pub peer_id: PeerId,
    pub error: P2pConnectionWebRTCOutgoingError,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCOutgoingErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(s)) => match &self.error {
                    P2pConnectionWebRTCOutgoingError::SdpCreateError(_) => {
                        matches!(
                            s,
                            P2pConnectionWebRTCOutgoingState::OfferSdpCreatePending { .. }
                        )
                    }
                    P2pConnectionWebRTCOutgoingError::Rejected(_) => {
                        matches!(
                            s,
                            P2pConnectionWebRTCOutgoingState::AnswerRecvPending { .. }
                        )
                    }
                    P2pConnectionWebRTCOutgoingError::RemoteInternalError => {
                        matches!(
                            s,
                            P2pConnectionWebRTCOutgoingState::AnswerRecvPending { .. }
                        )
                    }
                    P2pConnectionWebRTCOutgoingError::FinalizeError(_) => {
                        matches!(s, P2pConnectionWebRTCOutgoingState::FinalizePending { .. })
                    }
                    P2pConnectionWebRTCOutgoingError::Timeout => true,
                },
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionWebRTCOutgoingSuccessAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionWebRTCOutgoingSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_webrtc_peer(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionWebRTCOutgoingState::FinalizeSuccess { .. },
                )) => true,
                _ => false,
            })
    }
}

// --- From<LeafAction> for Action impls.
use crate::connection::P2pConnectionState;

use super::P2pConnectionWebRTCOutgoingState;

macro_rules! into_p2p_action {
    ($($action:ident),* $(,)?) => {
        $(
            impl From<$action> for crate::P2pAction {
                fn from(value: $action) -> Self {
                    crate::P2pAction::Connection(crate::connection::P2pConnectionAction::WebRTC(crate::connection::webrtc::P2pConnectionWebRTCAction::Outgoing(value.into())))
                }
            }
        )*
    };
}

into_p2p_action!(
    P2pConnectionWebRTCOutgoingInitAction,
    P2pConnectionWebRTCOutgoingOfferSdpCreatePendingAction,
    P2pConnectionWebRTCOutgoingOfferSdpCreateErrorAction,
    P2pConnectionWebRTCOutgoingOfferSdpCreateSuccessAction,
    P2pConnectionWebRTCOutgoingOfferReadyAction,
    P2pConnectionWebRTCOutgoingOfferSendSuccessAction,
    P2pConnectionWebRTCOutgoingAnswerRecvPendingAction,
    P2pConnectionWebRTCOutgoingAnswerRecvErrorAction,
    P2pConnectionWebRTCOutgoingAnswerRecvSuccessAction,
    P2pConnectionWebRTCOutgoingFinalizePendingAction,
    P2pConnectionWebRTCOutgoingFinalizeErrorAction,
    P2pConnectionWebRTCOutgoingFinalizeSuccessAction,
    P2pConnectionWebRTCOutgoingTimeoutAction,
    P2pConnectionWebRTCOutgoingErrorAction,
    P2pConnectionWebRTCOutgoingSuccessAction,
);
