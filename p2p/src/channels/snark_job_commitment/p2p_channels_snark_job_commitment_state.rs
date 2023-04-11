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
        /// Last sent commitment index.
        last_sent_index: u64,
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

    pub fn next_send_index_and_limit(&self) -> (u64, u8) {
        match self {
            Self::Ready {
                remote,
                last_sent_index,
                ..
            } => match remote {
                SnarkJobCommitmentPropagationState::Requested {
                    requested_limit, ..
                } => (*last_sent_index + 1, *requested_limit),
                _ => (*last_sent_index + 1, 0),
            },
            _ => (0, 0),
        }
    }
}
