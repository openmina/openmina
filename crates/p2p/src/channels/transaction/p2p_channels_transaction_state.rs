use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pChannelsTransactionState {
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
        local: TransactionPropagationState,
        /// We are the responders here.
        remote: TransactionPropagationState,
        /// Last sent snark index.
        next_send_index: u64,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionPropagationState {
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

impl P2pChannelsTransactionState {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    pub fn can_send_request(&self) -> bool {
        matches!(
            self,
            Self::Ready {
                local: TransactionPropagationState::WaitingForRequest { .. }
                    | TransactionPropagationState::Responded { .. },
                ..
            }
        )
    }

    pub fn next_send_index_and_limit(&self) -> (u64, u8) {
        match self {
            Self::Ready {
                remote,
                next_send_index,
                ..
            } => match remote {
                TransactionPropagationState::Requested {
                    requested_limit, ..
                } => (*next_send_index, *requested_limit),
                _ => (*next_send_index, 0),
            },
            _ => (0, 0),
        }
    }
}
