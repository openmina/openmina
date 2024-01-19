use serde::{Serialize, Deserialize};

use super::{
    incoming::P2pConnectionLibP2pIncomingState, outgoing::P2pConnectionLibP2pOutgoingState,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionLibP2pState {
    Outgoing(P2pConnectionLibP2pOutgoingState),
    Incoming(P2pConnectionLibP2pIncomingState),
}
