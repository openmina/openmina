use serde::{Deserialize, Serialize};

use crate::identity::PublicKey;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pChannelsSignalingExchangeState {
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
        local: SignalingExchangeState,
        /// We are the responders here.
        remote: SignalingExchangeState,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SignalingExchangeState {
    /// Next offer wasn't requested, so we shouldn't receive/send any offers.
    WaitingForRequest {
        time: redux::Timestamp,
    },
    /// Next offer/incoming connection was requested. Offer will be received
    /// once/if peer wants to connect.
    Requested {
        time: redux::Timestamp,
    },
    Offered {
        time: redux::Timestamp,
        offerer_pub_key: PublicKey,
    },
    Answered {
        time: redux::Timestamp,
    },
}

impl P2pChannelsSignalingExchangeState {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }
}
