use openmina_core::requests::RpcId;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use super::incoming::{P2pConnectionIncomingInitOpts, P2pConnectionIncomingState};
use super::outgoing::{P2pConnectionOutgoingInitOpts, P2pConnectionOutgoingState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionState {
    Outgoing(P2pConnectionOutgoingState),
    Incoming(P2pConnectionIncomingState),
}

impl P2pConnectionState {
    pub fn outgoing_init(opts: &P2pConnectionOutgoingInitOpts) -> Self {
        Self::Outgoing(P2pConnectionOutgoingState::Init {
            time: Timestamp::ZERO,
            opts: opts.clone(),
            rpc_id: None,
        })
    }

    pub fn incoming_init(opts: &P2pConnectionIncomingInitOpts) -> Self {
        Self::Incoming(P2pConnectionIncomingState::Init {
            time: Timestamp::ZERO,
            signaling: opts.signaling.clone(),
            offer: opts.offer.clone(),
            rpc_id: None,
        })
    }

    pub fn as_outgoing(&self) -> Option<&P2pConnectionOutgoingState> {
        match self {
            Self::Outgoing(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_incoming(&self) -> Option<&P2pConnectionIncomingState> {
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
            Self::Outgoing(P2pConnectionOutgoingState::Error { .. }) => true,
            Self::Outgoing(_) => false,
            Self::Incoming(P2pConnectionIncomingState::Error { .. }) => true,
            Self::Incoming(_) => false,
        }
    }

    pub fn is_success(&self) -> bool {
        match self {
            Self::Outgoing(P2pConnectionOutgoingState::Success { .. }) => true,
            Self::Outgoing(_) => false,
            Self::Incoming(P2pConnectionIncomingState::Success { .. }) => true,
            Self::Incoming(P2pConnectionIncomingState::Libp2pReceived { .. }) => true,
            Self::Incoming(_) => false,
        }
    }
}
