use std::time::Duration;

use redux::Timestamp;
use serde::{Deserialize, Serialize};

use shared::requests::RpcId;

use crate::{connection::RejectionReason, webrtc};

use super::P2pConnectionOutgoingInitOpts;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionOutgoingState {
    Init {
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        rpc_id: Option<RpcId>,
    },
    OfferSdpCreatePending {
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        rpc_id: Option<RpcId>,
    },
    OfferSdpCreateSuccess {
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        sdp: String,
        rpc_id: Option<RpcId>,
    },
    OfferReady {
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: webrtc::Offer,
        rpc_id: Option<RpcId>,
    },
    OfferSendSuccess {
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: webrtc::Offer,
        rpc_id: Option<RpcId>,
    },
    AnswerRecvPending {
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: webrtc::Offer,
        rpc_id: Option<RpcId>,
    },
    AnswerRecvSuccess {
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: webrtc::Offer,
        answer: webrtc::Answer,
        rpc_id: Option<RpcId>,
    },
    FinalizePending {
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: Option<webrtc::Offer>,
        answer: Option<webrtc::Answer>,
        rpc_id: Option<RpcId>,
    },
    FinalizeSuccess {
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: Option<webrtc::Offer>,
        answer: Option<webrtc::Answer>,
        rpc_id: Option<RpcId>,
    },
    Error {
        time: Timestamp,
        error: P2pConnectionOutgoingError,
        rpc_id: Option<RpcId>,
    },
    Success {
        time: Timestamp,
        offer: Option<webrtc::Offer>,
        answer: Option<webrtc::Answer>,
        rpc_id: Option<RpcId>,
    },
}

impl P2pConnectionOutgoingState {
    pub fn time(&self) -> Timestamp {
        match self {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionOutgoingError {
    SdpCreateError(String),
    Rejected(RejectionReason),
    RemoteInternalError,
    FinalizeError(String),
    Timeout,
}
