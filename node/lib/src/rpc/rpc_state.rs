use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;

use super::RpcId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcRequestState {
    pub req: RpcRequest,
    pub status: RpcRequestStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcRequest {
    GetState,
    P2pConnectionOutgoing(P2pConnectionOutgoingInitOpts),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcRequestStatus {
    Init {
        time: redux::Timestamp,
    },
    Pending {
        time: redux::Timestamp,
    },
    Error {
        time: redux::Timestamp,
        error: String,
    },
    Success {
        time: redux::Timestamp,
    },
}

impl RpcRequestStatus {
    pub fn is_init(&self) -> bool {
        matches!(self, Self::Init { .. })
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Error { .. } | Self::Success { .. })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcState {
    pub requests: BTreeMap<RpcId, RpcRequestState>,
}

impl RpcState {
    pub fn new() -> Self {
        Self {
            requests: Default::default(),
        }
    }
}
