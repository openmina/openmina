use serde::{Deserialize, Serialize};

use shared::requests::RpcId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionOutgoingState {
    Init {
        time: redux::Timestamp,
        addrs: Vec<libp2p::Multiaddr>,
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

impl Default for P2pConnectionOutgoingState {
    fn default() -> Self {
        Self::Init {
            time: redux::Timestamp::ZERO,
            addrs: vec![],
            rpc_id: None,
        }
    }
}
