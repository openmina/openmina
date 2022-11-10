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
}
