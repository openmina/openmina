use std::time::Duration;

use openmina_core::requests::RpcId;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::connection::ConnectionState;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum P2pConnectionLibP2pOutgoingState {
    #[default]
    Default,
    Init(P2pConnectionLibP2pOutgoingInitState),
    FinalizePending(P2pConnectionLibP2pOutgoingFinalizePendingState),
    Success(P2pConnectionLibP2pOutgoingSuccessState),
    Error(P2pConnectionLibP2pOutgoingErrorState),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionLibP2pOutgoingInitState {
    pub time: Timestamp,
    pub rpc_id: Option<RpcId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionLibP2pOutgoingFinalizePendingState {
    pub time: Timestamp,
    pub rpc_id: Option<RpcId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionLibP2pOutgoingSuccessState {
    pub time: Timestamp,
    pub rpc_id: Option<RpcId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionLibP2pOutgoingErrorState {
    pub time: Timestamp,
    pub error: P2pConnectionLibP2pOutgoingError,
    pub rpc_id: Option<RpcId>,
}

impl P2pConnectionLibP2pOutgoingState {
    pub fn time(&self) -> Timestamp {
        match self {
            P2pConnectionLibP2pOutgoingState::Default => Timestamp::ZERO,
            P2pConnectionLibP2pOutgoingState::Init(v) => v.time,
            P2pConnectionLibP2pOutgoingState::FinalizePending(v) => v.time,
            P2pConnectionLibP2pOutgoingState::Success(v) => v.time,
            P2pConnectionLibP2pOutgoingState::Error(v) => v.time,
        }
    }

    pub fn is_timed_out(&self, now: Timestamp, timeout: Duration) -> bool {
        !matches!(self, Self::Error { .. })
            && now
                .checked_sub(self.time())
                .map_or(false, |dur| dur >= timeout)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error)]
pub enum P2pConnectionLibP2pOutgoingError {
    #[error("connection finalization error: {0}")]
    FinalizeError(String),
    #[error("timeout")]
    Timeout,
}

impl ConnectionState for P2pConnectionLibP2pOutgoingState {
    fn is_success(&self) -> bool {
        matches!(self, P2pConnectionLibP2pOutgoingState::Success(_))
    }

    fn is_error(&self) -> bool {
        matches!(self, P2pConnectionLibP2pOutgoingState::Error(_))
    }

    fn rpc_id(&self) -> Option<RpcId> {
        match self {
            P2pConnectionLibP2pOutgoingState::Default => None,
            P2pConnectionLibP2pOutgoingState::Init(v) => v.rpc_id,
            P2pConnectionLibP2pOutgoingState::FinalizePending(v) => v.rpc_id,
            P2pConnectionLibP2pOutgoingState::Success(v) => v.rpc_id,
            P2pConnectionLibP2pOutgoingState::Error(v) => v.rpc_id,
        }
    }

    fn time(&self) -> Timestamp {
        match self {
            P2pConnectionLibP2pOutgoingState::Default => Timestamp::ZERO,
            P2pConnectionLibP2pOutgoingState::Init(v) => v.time,
            P2pConnectionLibP2pOutgoingState::FinalizePending(v) => v.time,
            P2pConnectionLibP2pOutgoingState::Success(v) => v.time,
            P2pConnectionLibP2pOutgoingState::Error(v) => v.time,
        }
    }
}
