use std::collections::BTreeMap;

use redux::Timestamp;
use serde::{Deserialize, Serialize};

use shared::requests::PendingRequests;

use crate::rpc::{P2pRpcIdType, P2pRpcKind, P2pRpcOutgoingError, P2pRpcRequest, P2pRpcResponse};

use super::P2pRpcRequestor;

type P2pRpcOutgoingContainer = PendingRequests<P2pRpcIdType, P2pRpcOutgoingStatus>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pRpcOutgoingStats {
    pub last_requested: Timestamp,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct P2pRpcOutgoingState {
    list: P2pRpcOutgoingContainer,
    /// TODO(binier): maybe use stack based array like for menu.
    pub stats: BTreeMap<P2pRpcKind, P2pRpcOutgoingStats>,
}

impl std::ops::Deref for P2pRpcOutgoingState {
    type Target = P2pRpcOutgoingContainer;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl std::ops::DerefMut for P2pRpcOutgoingState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcOutgoingStatus {
    Init {
        time: redux::Timestamp,
        request: P2pRpcRequest,
        requestor: P2pRpcRequestor,
    },
    Pending {
        time: redux::Timestamp,
        request: P2pRpcRequest,
        requestor: P2pRpcRequestor,
    },
    Received {
        time: redux::Timestamp,
        request: P2pRpcRequest,
        requestor: P2pRpcRequestor,
        response: P2pRpcResponse,
    },
    Error {
        time: redux::Timestamp,
        request: P2pRpcRequest,
        requestor: P2pRpcRequestor,
        response: Option<P2pRpcResponse>,
        error: P2pRpcOutgoingError,
    },
    Success {
        time: redux::Timestamp,
        request: P2pRpcRequest,
        requestor: P2pRpcRequestor,
        response: P2pRpcResponse,
    },
}

impl P2pRpcOutgoingStatus {
    pub fn is_init(&self) -> bool {
        matches!(self, Self::Init { .. })
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    pub fn is_received(&self) -> bool {
        matches!(self, Self::Received { .. })
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Error { .. } | Self::Success { .. })
    }

    pub fn request(&self) -> &P2pRpcRequest {
        match self {
            Self::Init { request, .. } => request,
            Self::Pending { request, .. } => request,
            Self::Received { request, .. } => request,
            Self::Error { request, .. } => request,
            Self::Success { request, .. } => request,
        }
    }
}
