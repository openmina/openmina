use serde::{Deserialize, Serialize};

use shared::requests::RpcId;

use crate::{connection::RejectionReason, webrtc};

use super::P2pConnectionOutgoingInitOpts;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionOutgoingState {
    Init {
        time: redux::Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        rpc_id: Option<RpcId>,
    },
    OfferSdpCreatePending {
        time: redux::Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        rpc_id: Option<RpcId>,
    },
    OfferSdpCreateSuccess {
        time: redux::Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        sdp: String,
        rpc_id: Option<RpcId>,
    },
    OfferReady {
        time: redux::Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: webrtc::Offer,
        rpc_id: Option<RpcId>,
    },
    OfferSendSuccess {
        time: redux::Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: webrtc::Offer,
        rpc_id: Option<RpcId>,
    },
    AnswerRecvPending {
        time: redux::Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: webrtc::Offer,
        rpc_id: Option<RpcId>,
    },
    AnswerRecvSuccess {
        time: redux::Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: webrtc::Offer,
        answer: webrtc::Answer,
        rpc_id: Option<RpcId>,
    },
    FinalizePending {
        time: redux::Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: webrtc::Offer,
        answer: webrtc::Answer,
        rpc_id: Option<RpcId>,
    },
    FinalizeSuccess {
        time: redux::Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: webrtc::Offer,
        answer: webrtc::Answer,
        rpc_id: Option<RpcId>,
    },
    Error {
        time: redux::Timestamp,
        error: P2pConnectionOutgoingError,
        rpc_id: Option<RpcId>,
    },
    Success {
        time: redux::Timestamp,
        offer: webrtc::Offer,
        answer: webrtc::Answer,
        rpc_id: Option<RpcId>,
    },
}

impl P2pConnectionOutgoingState {
    pub fn rpc_id(&self) -> Option<RpcId> {
        match self {
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionOutgoingError {
    SdpCreateError(String),
    Rejected(RejectionReason),
    RemoteInternalError,
    FinalizeError(String),
}
