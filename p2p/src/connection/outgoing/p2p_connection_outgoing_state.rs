use malloc_size_of_derive::MallocSizeOf;
use redux::{Callback, Timestamp};
use serde::{Deserialize, Serialize};

use openmina_core::requests::RpcId;

use crate::{connection::RejectionReason, webrtc, P2pTimeouts, PeerId};

use super::P2pConnectionOutgoingInitOpts;

#[derive(Serialize, Deserialize, Debug, Clone, MallocSizeOf)]
pub enum P2pConnectionOutgoingState {
    Init {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        rpc_id: Option<RpcId>,
        #[ignore_malloc_size_of = "negligible"]
        on_success: Option<Callback<(PeerId, Option<RpcId>)>>,
    },
    OfferSdpCreatePending {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        rpc_id: Option<RpcId>,
        #[ignore_malloc_size_of = "negligible"]
        on_success: Option<Callback<(PeerId, Option<RpcId>)>>,
    },
    OfferSdpCreateSuccess {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        sdp: String,
        rpc_id: Option<RpcId>,
        #[ignore_malloc_size_of = "negligible"]
        on_success: Option<Callback<(PeerId, Option<RpcId>)>>,
    },
    OfferReady {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: Box<webrtc::Offer>,
        rpc_id: Option<RpcId>,
        #[ignore_malloc_size_of = "negligible"]
        on_success: Option<Callback<(PeerId, Option<RpcId>)>>,
    },
    OfferSendSuccess {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: Box<webrtc::Offer>,
        rpc_id: Option<RpcId>,
        #[ignore_malloc_size_of = "negligible"]
        on_success: Option<Callback<(PeerId, Option<RpcId>)>>,
    },
    AnswerRecvPending {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: Box<webrtc::Offer>,
        rpc_id: Option<RpcId>,
        #[ignore_malloc_size_of = "negligible"]
        on_success: Option<Callback<(PeerId, Option<RpcId>)>>,
    },
    AnswerRecvSuccess {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: Box<webrtc::Offer>,
        answer: Box<webrtc::Answer>,
        rpc_id: Option<RpcId>,
        #[ignore_malloc_size_of = "negligible"]
        on_success: Option<Callback<(PeerId, Option<RpcId>)>>,
    },
    FinalizePending {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: Option<Box<webrtc::Offer>>,
        answer: Option<Box<webrtc::Answer>>,
        rpc_id: Option<RpcId>,
        #[ignore_malloc_size_of = "negligible"]
        on_success: Option<Callback<(PeerId, Option<RpcId>)>>,
    },
    FinalizeSuccess {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        offer: Option<Box<webrtc::Offer>>,
        answer: Option<Box<webrtc::Answer>>,
        rpc_id: Option<RpcId>,
        #[ignore_malloc_size_of = "negligible"]
        on_success: Option<Callback<(PeerId, Option<RpcId>)>>,
    },
    Error {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: Timestamp,
        error: P2pConnectionOutgoingError,
        rpc_id: Option<RpcId>,
    },
    Success {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: Timestamp,
        offer: Option<Box<webrtc::Offer>>,
        answer: Option<Box<webrtc::Answer>>,
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

    pub fn is_timed_out(&self, now: Timestamp, timeouts: &P2pTimeouts) -> bool {
        !matches!(self, Self::Error { .. })
            && now
                .checked_sub(self.time())
                .and_then(|dur| timeouts.outgoing_connection_timeout.map(|to| dur >= to))
                .unwrap_or(false)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error, MallocSizeOf)]
pub enum P2pConnectionOutgoingError {
    #[error("error creating SDP: {0}")]
    SdpCreateError(String),
    #[error("rejected: {0}")]
    Rejected(RejectionReason),
    #[error("remote signal decryption failed")]
    RemoteSignalDecryptionFailed,
    #[error("remote internal error")]
    RemoteInternalError,
    #[error("finalization error: {0}")]
    FinalizeError(String),
    #[error("connection authorization error")]
    ConnectionAuthError,
    #[error("timeout error")]
    Timeout,
}
