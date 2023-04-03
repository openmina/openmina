use serde::{Deserialize, Serialize};

use shared::requests::RpcId;

use crate::connection::incoming::P2pConnectionIncomingState;
use crate::connection::P2pConnectionErrorResponse;
use crate::{webrtc, P2pState, PeerId};

use super::{P2pConnectionOutgoingError, P2pConnectionOutgoingInitOpts};

pub type P2pConnectionOutgoingActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a P2pConnectionOutgoingAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionOutgoingAction {
    RandomInit(P2pConnectionOutgoingRandomInitAction),
    Init(P2pConnectionOutgoingInitAction),
    Reconnect(P2pConnectionOutgoingReconnectAction),
    OfferSdpCreatePending(P2pConnectionOutgoingOfferSdpCreatePendingAction),
    OfferSdpCreateError(P2pConnectionOutgoingOfferSdpCreateErrorAction),
    OfferSdpCreateSuccess(P2pConnectionOutgoingOfferSdpCreateSuccessAction),
    OfferReady(P2pConnectionOutgoingOfferReadyAction),
    OfferSendSuccess(P2pConnectionOutgoingOfferSendSuccessAction),
    AnswerRecvPending(P2pConnectionOutgoingAnswerRecvPendingAction),
    AnswerRecvError(P2pConnectionOutgoingAnswerRecvErrorAction),
    AnswerRecvSuccess(P2pConnectionOutgoingAnswerRecvSuccessAction),
    FinalizePending(P2pConnectionOutgoingFinalizePendingAction),
    FinalizeError(P2pConnectionOutgoingFinalizeErrorAction),
    FinalizeSuccess(P2pConnectionOutgoingFinalizeSuccessAction),
    Error(P2pConnectionOutgoingErrorAction),
    Success(P2pConnectionOutgoingSuccessAction),
}

impl P2pConnectionOutgoingAction {
    pub fn peer_id(&self) -> Option<&PeerId> {
        match self {
            Self::RandomInit(_) => None,
            Self::Init(v) => Some(&v.opts.peer_id),
            Self::Reconnect(v) => Some(&v.opts.peer_id),
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
            Self::Error(v) => Some(&v.peer_id),
            Self::Success(v) => Some(&v.peer_id),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingRandomInitAction {}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingRandomInitAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        !state.already_has_min_peers() && !state.initial_unused_peers().is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingInitAction {
    pub opts: P2pConnectionOutgoingInitOpts,
    pub rpc_id: Option<RpcId>,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingInitAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        !state.already_has_min_peers() && !state.peers.contains_key(&self.opts.peer_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingReconnectAction {
    pub opts: P2pConnectionOutgoingInitOpts,
    pub rpc_id: Option<RpcId>,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingReconnectAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        if state.already_has_min_peers() {
            return false;
        }

        state
            .peers
            .iter()
            .filter_map(|(id, p)| match &p.status {
                P2pPeerStatus::Connecting(s) => match s {
                    P2pConnectionState::Outgoing(P2pConnectionOutgoingState::Error {
                        time,
                        ..
                    }) => Some((*time, id, &p.dial_opts)),
                    P2pConnectionState::Incoming(P2pConnectionIncomingState::Error {
                        time,
                        ..
                    }) => Some((*time, id, &p.dial_opts)),
                    _ => None,
                },
                P2pPeerStatus::Disconnected { time } => Some((*time, id, &p.dial_opts)),
                P2pPeerStatus::Ready(_) => None,
            })
            .min_by_key(|(time, ..)| *time)
            .filter(|(_, id, _)| *id == &self.opts.peer_id)
            .filter(|(.., opts)| opts.as_ref().map_or(true, |opts| opts == &self.opts))
            .is_some()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingOfferSdpCreatePendingAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingOfferSdpCreatePendingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::Init { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingOfferSdpCreateErrorAction {
    pub peer_id: PeerId,
    pub error: String,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingOfferSdpCreateErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::OfferSdpCreatePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingOfferSdpCreateSuccessAction {
    pub peer_id: PeerId,
    pub sdp: String,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingOfferSdpCreateSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::OfferSdpCreatePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingOfferReadyAction {
    pub peer_id: PeerId,
    pub offer: webrtc::Offer,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingOfferReadyAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::OfferSdpCreateSuccess { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingOfferSendSuccessAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingOfferSendSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::OfferReady { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingAnswerRecvPendingAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingAnswerRecvPendingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::OfferSendSuccess { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingAnswerRecvErrorAction {
    pub peer_id: PeerId,
    pub error: P2pConnectionErrorResponse,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingAnswerRecvErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::AnswerRecvPending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingAnswerRecvSuccessAction {
    pub peer_id: PeerId,
    pub answer: webrtc::Answer,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingAnswerRecvSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::AnswerRecvPending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingFinalizePendingAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingFinalizePendingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::AnswerRecvSuccess { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingFinalizeErrorAction {
    pub peer_id: PeerId,
    pub error: String,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingFinalizeErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::FinalizePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingFinalizeSuccessAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingFinalizeSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::FinalizePending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingErrorAction {
    pub peer_id: PeerId,
    pub error: P2pConnectionOutgoingError,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(s)) => match &self.error {
                    P2pConnectionOutgoingError::SdpCreateError(_) => {
                        matches!(s, P2pConnectionOutgoingState::OfferSdpCreatePending { .. })
                    }
                    P2pConnectionOutgoingError::Rejected(_) => {
                        matches!(s, P2pConnectionOutgoingState::AnswerRecvPending { .. })
                    }
                    P2pConnectionOutgoingError::RemoteInternalError => {
                        matches!(s, P2pConnectionOutgoingState::AnswerRecvPending { .. })
                    }
                    P2pConnectionOutgoingError::FinalizeError(_) => {
                        matches!(s, P2pConnectionOutgoingState::FinalizePending { .. })
                    }
                },
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingSuccessAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::FinalizeSuccess { .. },
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

use super::P2pConnectionOutgoingState;

impl From<P2pConnectionOutgoingRandomInitAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingRandomInitAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingInitAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingInitAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingReconnectAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingReconnectAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingOfferSdpCreatePendingAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingOfferSdpCreatePendingAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingOfferSdpCreateErrorAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingOfferSdpCreateErrorAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingOfferSdpCreateSuccessAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingOfferSdpCreateSuccessAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingOfferReadyAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingOfferReadyAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingOfferSendSuccessAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingOfferSendSuccessAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingAnswerRecvPendingAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingAnswerRecvPendingAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingAnswerRecvErrorAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingAnswerRecvErrorAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingAnswerRecvSuccessAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingAnswerRecvSuccessAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingFinalizePendingAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingFinalizePendingAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingFinalizeErrorAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingFinalizeErrorAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingFinalizeSuccessAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingFinalizeSuccessAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingErrorAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingErrorAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingSuccessAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingSuccessAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}
