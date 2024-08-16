use serde::{Deserialize, Serialize};

use crate::P2pTimeouts;

use super::{
    staged_ledger_parts::StagedLedgerPartsReceiveProgress, P2pStreamingRpcId, P2pStreamingRpcKind,
    P2pStreamingRpcReceiveProgress, P2pStreamingRpcRequest, P2pStreamingRpcResponse,
    P2pStreamingRpcResponseFull, P2pStreamingRpcSendProgress,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pChannelsStreamingRpcState {
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
        local: P2pStreamingRpcLocalState,
        /// We are the responders here.
        remote: P2pStreamingRpcRemoteState,
        remote_last_responded: redux::Timestamp,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pStreamingRpcLocalState {
    WaitingForRequest {
        time: redux::Timestamp,
    },
    Requested {
        time: redux::Timestamp,
        id: P2pStreamingRpcId,
        request: Box<P2pStreamingRpcRequest>,
        progress: P2pStreamingRpcReceiveProgress,
    },
    Responded {
        time: redux::Timestamp,
        id: P2pStreamingRpcId,
        request: Box<P2pStreamingRpcRequest>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pStreamingRpcRemoteState {
    WaitingForRequest {
        time: redux::Timestamp,
    },
    Requested {
        time: redux::Timestamp,
        id: P2pStreamingRpcId,
        request: Box<P2pStreamingRpcRequest>,
        progress: P2pStreamingRpcSendProgress,
    },
    Responded {
        time: redux::Timestamp,
        id: P2pStreamingRpcId,
        request: Box<P2pStreamingRpcRequest>,
    },
}

impl P2pChannelsStreamingRpcState {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    pub fn can_send_request(&self) -> bool {
        match self {
            Self::Ready { local, .. } => matches!(
                local,
                P2pStreamingRpcLocalState::WaitingForRequest { .. }
                    | P2pStreamingRpcLocalState::Responded { .. }
            ),
            _ => false,
        }
    }

    pub fn is_timed_out(
        &self,
        rpc_id: P2pStreamingRpcId,
        now: redux::Timestamp,
        config: &P2pTimeouts,
    ) -> bool {
        match self {
            Self::Ready {
                local:
                    P2pStreamingRpcLocalState::Requested {
                        time, id, request, ..
                    },
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

    pub fn pending_local_rpc_id(&self) -> Option<P2pStreamingRpcId> {
        match self {
            Self::Ready {
                local: P2pStreamingRpcLocalState::Requested { id, .. },
                ..
            } => Some(*id),
            _ => None,
        }
    }

    pub fn pending_local_rpc(&self) -> Option<&P2pStreamingRpcRequest> {
        match self {
            Self::Ready {
                local: P2pStreamingRpcLocalState::Requested { request, .. },
                ..
            } => Some(request),
            _ => None,
        }
    }

    pub fn pending_local_rpc_kind(&self) -> Option<P2pStreamingRpcKind> {
        self.pending_local_rpc().map(|req| req.kind())
    }

    pub(super) fn local_done_response(&self) -> Option<P2pStreamingRpcResponseFull> {
        match self {
            Self::Ready {
                local:
                    P2pStreamingRpcLocalState::Requested {
                        progress:
                            P2pStreamingRpcReceiveProgress::StagedLedgerParts(
                                StagedLedgerPartsReceiveProgress::Success { data, .. },
                            ),
                        ..
                    },
                ..
            } => Some(data.clone().into()),
            _ => None,
        }
    }

    pub fn local_responded_request(&self) -> Option<(P2pStreamingRpcId, &P2pStreamingRpcRequest)> {
        match self {
            Self::Ready {
                local: P2pStreamingRpcLocalState::Responded { id, request, .. },
                ..
            } => Some((*id, request)),
            _ => None,
        }
    }

    #[allow(unused)]
    pub(super) fn remote_request(&self) -> Option<&P2pStreamingRpcRequest> {
        match self {
            Self::Ready {
                remote: P2pStreamingRpcRemoteState::Requested { request, .. },
                ..
            } => Some(request),
            _ => None,
        }
    }

    pub fn remote_todo_request(&self) -> Option<(P2pStreamingRpcId, &P2pStreamingRpcRequest)> {
        match self {
            Self::Ready {
                remote:
                    P2pStreamingRpcRemoteState::Requested {
                        id,
                        request,
                        progress,
                        ..
                    },
                ..
            } if progress.external_data_todo() => Some((*id, request)),
            _ => None,
        }
    }

    pub fn remote_pending_request(&self) -> Option<(P2pStreamingRpcId, &P2pStreamingRpcRequest)> {
        match self {
            Self::Ready {
                remote:
                    P2pStreamingRpcRemoteState::Requested {
                        id,
                        request,
                        progress,
                        ..
                    },
                ..
            } if progress.external_data_pending() => Some((*id, request)),
            _ => None,
        }
    }

    pub fn remote_next_msg(&self) -> Option<P2pStreamingRpcResponse> {
        match self {
            Self::Ready {
                remote: P2pStreamingRpcRemoteState::Requested { progress, .. },
                ..
            } => progress.next_msg(),
            _ => None,
        }
    }

    pub fn remote_last_responded(&self) -> redux::Timestamp {
        match self {
            Self::Ready {
                remote_last_responded,
                ..
            } => *remote_last_responded,
            _ => redux::Timestamp::ZERO,
        }
    }
}
