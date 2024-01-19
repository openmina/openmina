use std::time::Duration;

use redux::Timestamp;
use serde::{Deserialize, Serialize};

use openmina_core::requests::RpcId;

use crate::{
    connection::{webrtc::RejectionReason, ConnectionState},
    webrtc,
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum P2pConnectionWebRTCOutgoingState {
    #[default]
    Default,
    Init {
        time: Timestamp,
        rpc_id: Option<RpcId>,
    },
    OfferSdpCreatePending {
        time: Timestamp,
        rpc_id: Option<RpcId>,
    },
    OfferSdpCreateSuccess {
        time: Timestamp,
        sdp: String,
        rpc_id: Option<RpcId>,
    },
    OfferReady {
        time: Timestamp,
        offer: webrtc::Offer,
        rpc_id: Option<RpcId>,
    },
    OfferSendSuccess {
        time: Timestamp,
        offer: webrtc::Offer,
        rpc_id: Option<RpcId>,
    },
    AnswerRecvPending {
        time: Timestamp,
        offer: webrtc::Offer,
        rpc_id: Option<RpcId>,
    },
    AnswerRecvSuccess {
        time: Timestamp,
        offer: webrtc::Offer,
        answer: webrtc::Answer,
        rpc_id: Option<RpcId>,
    },
    FinalizePending {
        time: Timestamp,
        offer: Option<webrtc::Offer>,
        answer: Option<webrtc::Answer>,
        rpc_id: Option<RpcId>,
    },
    FinalizeSuccess {
        time: Timestamp,
        offer: Option<webrtc::Offer>,
        answer: Option<webrtc::Answer>,
        rpc_id: Option<RpcId>,
    },
    Error {
        time: Timestamp,
        error: P2pConnectionWebRTCOutgoingError,
        rpc_id: Option<RpcId>,
    },
    Success {
        time: Timestamp,
        offer: Option<webrtc::Offer>,
        answer: Option<webrtc::Answer>,
        rpc_id: Option<RpcId>,
    },
}

impl P2pConnectionWebRTCOutgoingState {
    pub fn time(&self) -> Timestamp {
        match self {
            Self::Default => Timestamp::ZERO,
            Self::Init { time, .. } => *time,
            Self::OfferSdpCreatePending { time, .. } => *time,
            Self::OfferSdpCreateSuccess { time, .. } => *time,
            Self::OfferReady { time, .. } => *time,
            Self::OfferSendSuccess { time, .. } => *time,
            Self::AnswerRecvPending { time, .. } => *time,
            Self::AnswerRecvSuccess { time, .. } => *time,
            Self::FinalizePending { time, .. } => *time,
            Self::FinalizeSuccess { time, .. } => *time,
            Self::Error { time, .. } => *time,
            Self::Success { time, .. } => *time,
        }
    }

    pub fn rpc_id(&self) -> Option<RpcId> {
        match self {
            Self::Default => None,
            Self::Init { rpc_id, .. } => *rpc_id,
            Self::OfferSdpCreatePending { rpc_id, .. } => *rpc_id,
            Self::OfferSdpCreateSuccess { rpc_id, .. } => *rpc_id,
            Self::OfferReady { rpc_id, .. } => *rpc_id,
            Self::OfferSendSuccess { rpc_id, .. } => *rpc_id,
            Self::AnswerRecvPending { rpc_id, .. } => *rpc_id,
            Self::AnswerRecvSuccess { rpc_id, .. } => *rpc_id,
            Self::FinalizePending { rpc_id, .. } => *rpc_id,
            Self::FinalizeSuccess { rpc_id, .. } => *rpc_id,
            Self::Error { rpc_id, .. } => *rpc_id,
            Self::Success { rpc_id, .. } => *rpc_id,
        }
    }

    pub fn is_timed_out(&self, now: Timestamp) -> bool {
        !matches!(self, Self::Error { .. })
            && now
                .checked_sub(self.time())
                .map_or(false, |dur| dur >= Duration::from_secs(30))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error)]
pub enum P2pConnectionWebRTCOutgoingError {
    #[error("error creating SDP: {0}")]
    SdpCreateError(String),
    #[error("rejected: {0}")]
    Rejected(RejectionReason),
    #[error("remote internal error")]
    RemoteInternalError,
    #[error("error finalizing: {0}")]
    FinalizeError(String),
    #[error("timeout")]
    Timeout,
}

impl ConnectionState for P2pConnectionWebRTCOutgoingState {
    fn is_success(&self) -> bool {
        matches!(self, P2pConnectionWebRTCOutgoingState::Success { .. })
    }

    fn is_error(&self) -> bool {
        matches!(self, P2pConnectionWebRTCOutgoingState::Error { .. })
    }

    fn rpc_id(&self) -> Option<RpcId> {
        match self {
            Self::Default => None,
            Self::Init { rpc_id, .. } => *rpc_id,
            Self::OfferSdpCreatePending { rpc_id, .. } => *rpc_id,
            Self::OfferSdpCreateSuccess { rpc_id, .. } => *rpc_id,
            Self::OfferReady { rpc_id, .. } => *rpc_id,
            Self::OfferSendSuccess { rpc_id, .. } => *rpc_id,
            Self::AnswerRecvPending { rpc_id, .. } => *rpc_id,
            Self::AnswerRecvSuccess { rpc_id, .. } => *rpc_id,
            Self::FinalizePending { rpc_id, .. } => *rpc_id,
            Self::FinalizeSuccess { rpc_id, .. } => *rpc_id,
            Self::Error { rpc_id, .. } => *rpc_id,
            Self::Success { rpc_id, .. } => *rpc_id,
        }
    }

    fn time(&self) -> Timestamp {
        match self {
            P2pConnectionWebRTCOutgoingState::Default => Timestamp::ZERO,
            P2pConnectionWebRTCOutgoingState::Init { time, .. } => *time,
            P2pConnectionWebRTCOutgoingState::OfferSdpCreatePending { time, .. } => *time,
            P2pConnectionWebRTCOutgoingState::OfferSdpCreateSuccess { time, .. } => *time,
            P2pConnectionWebRTCOutgoingState::OfferReady { time, .. } => *time,
            P2pConnectionWebRTCOutgoingState::OfferSendSuccess { time, .. } => *time,
            P2pConnectionWebRTCOutgoingState::AnswerRecvPending { time, .. } => *time,
            P2pConnectionWebRTCOutgoingState::AnswerRecvSuccess { time, .. } => *time,
            P2pConnectionWebRTCOutgoingState::FinalizePending { time, .. } => *time,
            P2pConnectionWebRTCOutgoingState::FinalizeSuccess { time, .. } => *time,
            P2pConnectionWebRTCOutgoingState::Error { time, .. } => *time,
            P2pConnectionWebRTCOutgoingState::Success { time, .. } => *time,
        }
    }
}
