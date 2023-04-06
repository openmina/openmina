use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pChannelsSnarkJobCommitmentState {
    Disabled,
    Enabled,
    Init { time: redux::Timestamp },
    Pending { time: redux::Timestamp },
    Ready { time: redux::Timestamp },
}
