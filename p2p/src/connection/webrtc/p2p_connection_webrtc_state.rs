use openmina_core::requests::RpcId;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use super::incoming::{P2pConnectionWebRTCIncomingInitOpts, P2pConnectionWebRTCIncomingState};
use super::outgoing::{P2pConnectionOutgoingInitOpts, P2pConnectionOutgoingState, P2pConnectionWebRTCOutgoingState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionWebRTCState {
    Outgoing(P2pConnectionWebRTCOutgoingState),
    Incoming(P2pConnectionWebRTCIncomingState),
}

impl P2pConnectionWebRTCWebRTCState {
    pub fn outgoing_init() -> Self {
        Self::Outgoing(P2pConnectionWebRTCOutgoingState::Init {
            time: Timestamp::ZERO,
            rpc_id: None,
        })
    }

    pub fn incoming_init(opts: &P2pConnectionWebRTCIncomingInitOpts) -> Self {
        Self::Incoming(P2pConnectionWebRTCIncomingState::Init {
            time: Timestamp::ZERO,
            signaling: opts.signaling.clone(),
            offer: opts.offer.clone(),
            rpc_id: None,
        })
    }

    pub fn as_outgoing(&self) -> Option<&P2pConnectionWebRTCOutgoingState> {
        match self {
            Self::Outgoing(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_incoming(&self) -> Option<&P2pConnectionWebRTCIncomingState> {
        match self {
            Self::Incoming(v) => Some(v),
            _ => None,
        }
    }

    pub fn rpc_id(&self) -> Option<RpcId> {
        match self {
            Self::Outgoing(v) => v.rpc_id(),
            Self::Incoming(v) => v.rpc_id(),
        }
    }

    pub fn is_timed_out(&self, now: Timestamp) -> bool {
        match self {
            Self::Outgoing(v) => v.is_timed_out(now),
            Self::Incoming(v) => v.is_timed_out(now),
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            Self::Outgoing(P2pConnectionWebRTCOutgoingState::Error { .. }) => true,
            Self::Outgoing(_) => false,
            Self::Incoming(P2pConnectionWebRTCIncomingState::Error { .. }) => true,
            Self::Incoming(_) => false,
        }
    }

    pub fn is_success(&self) -> bool {
        match self {
            Self::Outgoing(P2pConnectionWebRTCOutgoingState::Success { .. }) => true,
            Self::Outgoing(_) => false,
            Self::Incoming(P2pConnectionWebRTCIncomingState::Success { .. }) => true,
            Self::Incoming(P2pConnectionWebRTCIncomingState::Libp2pReceived { .. }) => true,
            Self::Incoming(_) => false,
        }
    }
}
