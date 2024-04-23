use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::P2pTimeouts;

use super::{P2pRpcId, P2pRpcKind, P2pRpcRequest};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pChannelsRpcState {
    Disabled,
    Enabled,
    Init {
        time: redux::Timestamp,
    },
    Pending {
        time: redux::Timestamp,
    },
    Ready {
        time: redux::Timestamp,
        /// We are the requestors here.
        local: P2pRpcLocalState,
        /// We are the responders here.
        remote: P2pRpcRemoteState,
        next_local_rpc_id: P2pRpcId,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcLocalState {
    WaitingForRequest {
        time: redux::Timestamp,
    },
    Requested {
        time: redux::Timestamp,
        id: P2pRpcId,
        request: P2pRpcRequest,
    },
    Responded {
        time: redux::Timestamp,
        id: P2pRpcId,
        request: P2pRpcRequest,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pRpcRemoteState {
    pub pending_requests: VecDeque<P2pRpcRemotePendingRequestState>,
    pub last_responded: redux::Timestamp,
}

static EMPTY_REMOTE_REQUESTS: VecDeque<P2pRpcRemotePendingRequestState> = VecDeque::new();

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pRpcRemotePendingRequestState {
    pub time: redux::Timestamp,
    pub id: P2pRpcId,
    pub request: P2pRpcRequest,
    /// If a given rpc request requires a response from some async component,
    /// e.g. ledger, then this field will be set to `true` once request
    /// to that async component is initiated.
    pub is_pending: bool,
}

impl P2pChannelsRpcState {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    pub fn next_local_rpc_id(&self) -> P2pRpcId {
        match self {
            Self::Ready {
                next_local_rpc_id, ..
            } => *next_local_rpc_id,
            _ => 0,
        }
    }

    pub fn can_send_request(&self) -> bool {
        match self {
            Self::Ready { local, .. } => matches!(
                local,
                P2pRpcLocalState::WaitingForRequest { .. } | P2pRpcLocalState::Responded { .. }
            ),
            _ => false,
        }
    }

    pub fn is_timed_out(
        &self,
        rpc_id: P2pRpcId,
        now: redux::Timestamp,
        config: &P2pTimeouts,
    ) -> bool {
        match self {
            Self::Ready {
                local: P2pRpcLocalState::Requested { time, id, request },
                ..
            } => {
                rpc_id == *id
                    && request
                        .kind()
                        .timeout(config)
                        .and_then(|timeout| {
                            let dur = now.checked_sub(*time)?;
                            Some(dur >= timeout)
                        })
                        .unwrap_or(false)
            }
            _ => false,
        }
    }

    pub fn pending_local_rpc_id(&self) -> Option<P2pRpcId> {
        match self {
            Self::Ready {
                local: P2pRpcLocalState::Requested { id, .. },
                ..
            } => Some(*id),
            _ => None,
        }
    }

    pub fn pending_local_rpc(&self) -> Option<&P2pRpcRequest> {
        match self {
            Self::Ready {
                local: P2pRpcLocalState::Requested { request, .. },
                ..
            } => Some(request),
            _ => None,
        }
    }

    pub fn pending_local_rpc_kind(&self) -> Option<P2pRpcKind> {
        self.pending_local_rpc().map(|req| req.kind())
    }

    pub fn local_responded_request(&self) -> Option<(P2pRpcId, &P2pRpcRequest)> {
        match self {
            Self::Ready {
                local: P2pRpcLocalState::Responded { id, request, .. },
                ..
            } => Some((*id, request)),
            _ => None,
        }
    }

    fn remote_requests(&self) -> impl Iterator<Item = &P2pRpcRemotePendingRequestState> {
        match self {
            Self::Ready { remote, .. } => remote.pending_requests.iter(),
            _ => EMPTY_REMOTE_REQUESTS.iter(),
        }
    }

    pub fn remote_todo_requests_iter(
        &self,
    ) -> impl Iterator<Item = &P2pRpcRemotePendingRequestState> {
        self.remote_requests().filter(|req| !req.is_pending)
    }

    pub fn remote_pending_requests_iter(
        &self,
    ) -> impl Iterator<Item = &P2pRpcRemotePendingRequestState> {
        self.remote_requests().filter(|req| req.is_pending)
    }

    pub fn remote_last_responded(&self) -> redux::Timestamp {
        match self {
            Self::Ready { remote, .. } => remote.last_responded,
            _ => redux::Timestamp::ZERO,
        }
    }
}
