use std::net::SocketAddr;

use malloc_size_of_derive::MallocSizeOf;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use mina_core::requests::RpcId;

use crate::{webrtc, P2pTimeouts};

use super::IncomingSignalingMethod;

#[derive(Serialize, Deserialize, Debug, Clone, MallocSizeOf)]
pub enum P2pConnectionIncomingState {
    Init {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: Box<webrtc::Offer>,
        rpc_id: Option<RpcId>,
    },
    AnswerSdpCreatePending {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: Box<webrtc::Offer>,
        rpc_id: Option<RpcId>,
    },
    AnswerSdpCreateSuccess {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: Box<webrtc::Offer>,
        sdp: String,
        rpc_id: Option<RpcId>,
    },
    AnswerReady {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: Box<webrtc::Offer>,
        answer: Box<webrtc::Answer>,
        rpc_id: Option<RpcId>,
    },
    AnswerSendSuccess {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: Box<webrtc::Offer>,
        answer: Box<webrtc::Answer>,
        rpc_id: Option<RpcId>,
    },
    FinalizePending {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: Box<webrtc::Offer>,
        answer: Box<webrtc::Answer>,
        rpc_id: Option<RpcId>,
    },
    FinalizeSuccess {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: Box<webrtc::Offer>,
        answer: Box<webrtc::Answer>,
        rpc_id: Option<RpcId>,
    },
    Error {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: redux::Timestamp,
        error: P2pConnectionIncomingError,
        rpc_id: Option<RpcId>,
    },
    Success {
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: redux::Timestamp,
        signaling: IncomingSignalingMethod,
        offer: Box<webrtc::Offer>,
        answer: Box<webrtc::Answer>,
        rpc_id: Option<RpcId>,
    },
    FinalizePendingLibp2p {
        #[ignore_malloc_size_of = "doesn't allocate"]
        addr: SocketAddr,
        #[with_malloc_size_of_func = "measurement::socket_vec"]
        close_duplicates: Vec<SocketAddr>,
        #[ignore_malloc_size_of = "doesn't allocate"]
        time: redux::Timestamp,
    },
    Libp2pReceived {
        #[ignore_malloc_size_of = "doesn't allocate"]
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
            Self::FinalizePendingLibp2p { time, .. } => *time,
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
            Self::FinalizePendingLibp2p { .. } | Self::Libp2pReceived { .. } => None,
        }
    }

    pub fn is_timed_out(&self, now: Timestamp, timeouts: &P2pTimeouts) -> bool {
        !matches!(self, Self::Error { .. })
            && now
                .checked_sub(self.time())
                .and_then(|dur| timeouts.incoming_connection_timeout.map(|to| dur >= to))
                .unwrap_or(false)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error, MallocSizeOf)]
pub enum P2pConnectionIncomingError {
    #[error("error creating SDP: {0}")]
    SdpCreateError(String),
    #[error("finalization error: {0}")]
    FinalizeError(String),
    #[error("connection authentication failed")]
    ConnectionAuthError,
    #[error("timeout error")]
    Timeout,
}

mod measurement {
    use std::{mem, net::SocketAddr};

    use malloc_size_of::MallocSizeOfOps;

    pub fn socket_vec(val: &Vec<SocketAddr>, _ops: &mut MallocSizeOfOps) -> usize {
        val.capacity() * mem::size_of::<SocketAddr>()
    }
}
