use serde::{Deserialize, Serialize};

use super::{P2pRpcId, P2pRpcRequest};

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
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pRpcRemoteState {
    pub pending_requests: Vec<P2pRpcRemotePendingRequestState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pRpcRemotePendingRequestState {
    pub time: redux::Timestamp,
    pub id: P2pRpcId,
    pub request: P2pRpcRequest,
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
}
