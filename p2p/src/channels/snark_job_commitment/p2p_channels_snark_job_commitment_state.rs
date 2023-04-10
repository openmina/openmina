use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pChannelsSnarkJobCommitmentState {
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
        local: SnarkJobCommitmentPropagationState,
        /// We are the responders here.
        remote: SnarkJobCommitmentPropagationState,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkJobCommitmentPropagationState {
    WaitingForRequest {
        time: redux::Timestamp,
    },
    Requested {
        time: redux::Timestamp,
        requested_limit: u8,
    },
    Responding {
        time: redux::Timestamp,
        requested_limit: u8,
        promised_count: u8,
        current_count: u8,
    },
    Responded {
        time: redux::Timestamp,
        count: u8,
    },
}

impl P2pChannelsSnarkJobCommitmentState {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }
}
