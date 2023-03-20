use serde::{Deserialize, Serialize};

use shared::requests::RpcId;

use super::P2pConnectionOutgoingInitOpts;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionOutgoingState {
    Init {
        time: redux::Timestamp,
        opts: P2pConnectionOutgoingInitOpts,
        rpc_id: Option<RpcId>,
    },
    Pending {
        time: redux::Timestamp,
        rpc_id: Option<RpcId>,
    },
    Error {
        time: redux::Timestamp,
        error: String,
        rpc_id: Option<RpcId>,
    },
    Success {
        time: redux::Timestamp,
        rpc_id: Option<RpcId>,
    },
}

impl P2pConnectionOutgoingState {
    pub fn rpc_id(&self) -> Option<RpcId> {
        match self {
            Self::Init { rpc_id, .. } => *rpc_id,
            Self::Pending { rpc_id, .. } => *rpc_id,
            Self::Error { rpc_id, .. } => *rpc_id,
            Self::Success { rpc_id, .. } => *rpc_id,
        }
    }
}
