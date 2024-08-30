use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    best_tip::P2pChannelsBestTipState,
    rpc::{P2pChannelsRpcState, P2pRpcId},
    snark::P2pChannelsSnarkState,
    snark_job_commitment::P2pChannelsSnarkJobCommitmentState,
    streaming_rpc::P2pChannelsStreamingRpcState,
    transaction::P2pChannelsTransactionState,
    ChannelId,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsState {
    pub best_tip: P2pChannelsBestTipState,
    pub transaction: P2pChannelsTransactionState,
    pub snark: P2pChannelsSnarkState,
    pub snark_job_commitment: P2pChannelsSnarkJobCommitmentState,
    pub rpc: P2pChannelsRpcState,
    pub streaming_rpc: P2pChannelsStreamingRpcState,

    pub(super) next_local_rpc_id: P2pRpcId,
}

impl P2pChannelsState {
    pub fn new(enabled_channels: &BTreeSet<ChannelId>) -> Self {
        Self {
            best_tip: match enabled_channels.contains(&ChannelId::BestTipPropagation) {
                false => P2pChannelsBestTipState::Disabled,
                true => P2pChannelsBestTipState::Enabled,
            },
            snark_job_commitment: match enabled_channels
                .contains(&ChannelId::SnarkJobCommitmentPropagation)
            {
                false => P2pChannelsSnarkJobCommitmentState::Disabled,
                true => P2pChannelsSnarkJobCommitmentState::Enabled,
            },
            transaction: match enabled_channels.contains(&ChannelId::TransactionPropagation) {
                false => P2pChannelsTransactionState::Disabled,
                true => P2pChannelsTransactionState::Enabled,
            },
            snark: match enabled_channels.contains(&ChannelId::SnarkPropagation) {
                false => P2pChannelsSnarkState::Disabled,
                true => P2pChannelsSnarkState::Enabled,
            },
            rpc: match enabled_channels.contains(&ChannelId::Rpc) {
                false => P2pChannelsRpcState::Disabled,
                true => P2pChannelsRpcState::Enabled,
            },
            streaming_rpc: match enabled_channels.contains(&ChannelId::StreamingRpc) {
                false => P2pChannelsStreamingRpcState::Disabled,
                true => P2pChannelsStreamingRpcState::Enabled,
            },

            next_local_rpc_id: 0,
        }
    }

    pub fn next_local_rpc_id(&self) -> P2pRpcId {
        self.next_local_rpc_id
    }

    pub fn rpc_remote_last_responded(&self) -> redux::Timestamp {
        std::cmp::max(
            self.rpc.remote_last_responded(),
            self.streaming_rpc.remote_last_responded(),
        )
    }
}

impl P2pChannelsState {
    pub fn is_channel_ready(&self, chan_id: ChannelId) -> bool {
        match chan_id {
            ChannelId::BestTipPropagation => self.best_tip.is_ready(),
            ChannelId::TransactionPropagation => self.transaction.is_ready(),
            ChannelId::SnarkPropagation => self.snark.is_ready(),
            ChannelId::SnarkJobCommitmentPropagation => self.snark_job_commitment.is_ready(),
            ChannelId::Rpc => self.rpc.is_ready(),
            ChannelId::StreamingRpc => self.rpc.is_ready(),
        }
    }
}
