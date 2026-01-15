use serde::{Deserialize, Serialize};

use crate::identity::PublicKey;

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pChannelsSignalingDiscoveryState {
    Disabled,
    Enabled,
    Init {
        time: redux::Timestamp,
    },
    Pending {
        time: redux::Timestamp,
    },
    Ready {
        time: redux::Timestamp,
        /// We are the requestors here.
        local: SignalingDiscoveryState,
        /// We are the responders here.
        remote: SignalingDiscoveryState,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SignalingDiscoveryState {
    WaitingForRequest {
        time: redux::Timestamp,
    },
    Requested {
        time: redux::Timestamp,
    },
    DiscoveryRequested {
        time: redux::Timestamp,
    },
    Discovered {
        time: redux::Timestamp,
        target_public_key: PublicKey,
    },
    DiscoveredRejected {
        time: redux::Timestamp,
        target_public_key: PublicKey,
    },
    DiscoveredAccepted {
        time: redux::Timestamp,
        target_public_key: PublicKey,
    },
    Answered {
        time: redux::Timestamp,
    },
}

impl P2pChannelsSignalingDiscoveryState {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }
}
