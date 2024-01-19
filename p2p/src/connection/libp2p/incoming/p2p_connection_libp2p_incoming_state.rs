use derive_more::From;
use redux::Timestamp;
use serde::{Serialize, Deserialize};

use crate::{P2pConnectionId, PeerId, connection::ConnectionState};

#[derive(Serialize, Deserialize, Debug, Clone, From, Default)]
pub enum P2pConnectionLibP2pIncomingState {
    #[default]
    Default,
    Success(P2pConnectionLibP2pIncomingSuccessState),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionLibP2pIncomingInitState {
    pub connection_id: P2pConnectionId,
    pub time: Timestamp,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionLibP2pIncomingSuccessState {
    pub peer_id: PeerId,
    pub time: Timestamp,
}

impl ConnectionState for P2pConnectionLibP2pIncomingState {
    fn is_success(&self) -> bool {
        match self {
            P2pConnectionLibP2pIncomingState::Success(_) => true,
            _ => false,
        }
    }

    fn is_error(&self) -> bool {
        false
    }

    fn rpc_id(&self) -> Option<openmina_core::requests::RpcId> {
        None
    }

    fn time(&self) -> Timestamp {
        match self {
            P2pConnectionLibP2pIncomingState::Default => Timestamp::ZERO,
            P2pConnectionLibP2pIncomingState::Success(s) => s.time,
        }
    }
}
