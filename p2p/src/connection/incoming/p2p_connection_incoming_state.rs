use std::time::Duration;

use redux::Timestamp;
use serde::{Deserialize, Serialize};

use openmina_core::requests::RpcId;

use crate::webrtc;

use super::IncomingSignalingMethod;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionIncomingState {
    Init {
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: webrtc::Offer,
        rpc_id: Option<RpcId>,
    },
    AnswerSdpCreatePending {
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: webrtc::Offer,
        rpc_id: Option<RpcId>,
    },
    AnswerSdpCreateSuccess {
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: webrtc::Offer,
        sdp: String,
        rpc_id: Option<RpcId>,
    },
    AnswerReady {
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: webrtc::Offer,
        answer: webrtc::Answer,
        rpc_id: Option<RpcId>,
    },
    AnswerSendSuccess {
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: webrtc::Offer,
        answer: webrtc::Answer,
        rpc_id: Option<RpcId>,
    },
    FinalizePending {
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: webrtc::Offer,
        answer: webrtc::Answer,
        rpc_id: Option<RpcId>,
    },
    FinalizeSuccess {
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: webrtc::Offer,
        answer: webrtc::Answer,
        rpc_id: Option<RpcId>,
    },
    Error {
        time: redux::Timestamp,
        error: P2pConnectionIncomingError,
        rpc_id: Option<RpcId>,
    },
    Success {
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: webrtc::Offer,
        answer: webrtc::Answer,
        rpc_id: Option<RpcId>,
    },
    Libp2pReceived {
        time: redux::Timestamp,
    },
}

impl P2pConnectionIncomingState {
    pub fn time(&self) -> Timestamp {
        match self {
            Self::Init { time, .. } => *time,
            Self::AnswerSdpCreatePending { time, .. } => *time,
            Self::AnswerSdpCreateSuccess { time, .. } => *time,
            Self::AnswerReady { time, .. } => *time,
            Self::AnswerSendSuccess { time, .. } => *time,
            Self::FinalizePending { time, .. } => *time,
            Self::FinalizeSuccess { time, .. } => *time,
            Self::Error { time, .. } => *time,
            Self::Success { time, .. } => *time,
            Self::Libp2pReceived { time } => *time,
        }
    }

    pub fn rpc_id(&self) -> Option<RpcId> {
        match self {
            Self::Init { rpc_id, .. } => *rpc_id,
            Self::AnswerSdpCreatePending { rpc_id, .. } => *rpc_id,
            Self::AnswerSdpCreateSuccess { rpc_id, .. } => *rpc_id,
            Self::AnswerReady { rpc_id, .. } => *rpc_id,
            Self::AnswerSendSuccess { rpc_id, .. } => *rpc_id,
            Self::FinalizePending { rpc_id, .. } => *rpc_id,
            Self::FinalizeSuccess { rpc_id, .. } => *rpc_id,
            Self::Error { rpc_id, .. } => *rpc_id,
            Self::Success { rpc_id, .. } => *rpc_id,
            Self::Libp2pReceived { .. } => None,
        }
    }

    pub fn is_timed_out(&self, now: Timestamp) -> bool {
        !matches!(self, Self::Error { .. })
            && now
                .checked_sub(self.time())
                .map_or(false, |dur| dur >= Duration::from_secs(30))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionIncomingError {
    SdpCreateError(String),
    FinalizeError(String),
    Timeout,
}
