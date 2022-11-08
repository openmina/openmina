use serde::{Deserialize, Serialize};

use super::outgoing::P2pConnectionOutgoingState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionState {
    Outgoing(P2pConnectionOutgoingState),
    // Incoming(P2pConnectionIncomingState),
}
