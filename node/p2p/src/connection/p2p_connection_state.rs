use serde::{Deserialize, Serialize};
use shared::requests::RpcId;

use super::outgoing::P2pConnectionOutgoingState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionState {
    Outgoing(P2pConnectionOutgoingState),
    // Incoming(P2pConnectionIncomingState),
}

impl P2pConnectionState {
    pub fn outgoing_rpc_id(&self) -> Option<RpcId> {
        match self {
            Self::Outgoing(v) => v.rpc_id(),
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            Self::Outgoing(P2pConnectionOutgoingState::Error { .. }) => true,
            Self::Outgoing(_) => false,
        }
    }

    pub fn is_success(&self) -> bool {
        match self {
            Self::Outgoing(P2pConnectionOutgoingState::Success { .. }) => true,
            Self::Outgoing(_) => false,
        }
    }
}
