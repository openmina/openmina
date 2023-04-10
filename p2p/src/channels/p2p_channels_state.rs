use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{snark_job_commitment::P2pChannelsSnarkJobCommitmentState, ChannelId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsState {
    pub snark_job_commitment: P2pChannelsSnarkJobCommitmentState,
}

impl P2pChannelsState {
    pub fn new(enabled_channels: &BTreeSet<ChannelId>) -> Self {
        Self {
            snark_job_commitment: match enabled_channels
                .contains(&ChannelId::SnarkJobCommitmentPropagation)
            {
                false => P2pChannelsSnarkJobCommitmentState::Disabled,
                true => P2pChannelsSnarkJobCommitmentState::Enabled,
            },
        }
    }
}

impl P2pChannelsState {
    pub fn is_channel_ready(&self, chan_id: ChannelId) -> bool {
        match chan_id {
            ChannelId::SnarkJobCommitmentPropagation => self.snark_job_commitment.is_ready(),
        }
    }
}
