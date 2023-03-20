use serde::{Deserialize, Serialize};

use shared::requests::RpcId;

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
    FinalizeError {
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: webrtc::Offer,
        answer: webrtc::Answer,
        error: String,
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
        error: String,
        rpc_id: Option<RpcId>,
    },
    Success {
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: webrtc::Offer,
        answer: webrtc::Answer,
        rpc_id: Option<RpcId>,
    },
}

impl P2pConnectionIncomingState {
    pub fn rpc_id(&self) -> Option<RpcId> {
        match self {
            Self::Init { rpc_id, .. } => *rpc_id,
            Self::AnswerSdpCreatePending { rpc_id, .. } => *rpc_id,
            Self::AnswerSdpCreateSuccess { rpc_id, .. } => *rpc_id,
            Self::AnswerReady { rpc_id, .. } => *rpc_id,
            Self::AnswerSendSuccess { rpc_id, .. } => *rpc_id,
            Self::FinalizePending { rpc_id, .. } => *rpc_id,
            Self::FinalizeError { rpc_id, .. } => *rpc_id,
            Self::FinalizeSuccess { rpc_id, .. } => *rpc_id,
            Self::Error { rpc_id, .. } => *rpc_id,
            Self::Success { rpc_id, .. } => *rpc_id,
        }
    }
}
