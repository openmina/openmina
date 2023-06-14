use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

use super::{
    best_tip::P2pChannelsBestTipAction, rpc::P2pChannelsRpcAction,
    snark_job_commitment::P2pChannelsSnarkJobCommitmentAction, ChannelMsg,
};

pub type P2pChannelsActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pChannelsAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pChannelsAction {
    MessageReceived(P2pChannelsMessageReceivedAction),

    BestTip(P2pChannelsBestTipAction),
    SnarkJobCommitment(P2pChannelsSnarkJobCommitmentAction),
    Rpc(P2pChannelsRpcAction),
}

impl P2pChannelsAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::MessageReceived(v) => &v.peer_id,
            Self::BestTip(v) => v.peer_id(),
            Self::SnarkJobCommitment(v) => v.peer_id(),
            Self::Rpc(v) => v.peer_id(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsMessageReceivedAction {
    pub peer_id: PeerId,
    pub message: ChannelMsg,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsMessageReceivedAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            p.channels.is_channel_ready(self.message.channel_id())
        })
    }
}

impl From<P2pChannelsMessageReceivedAction> for crate::P2pAction {
    fn from(a: P2pChannelsMessageReceivedAction) -> Self {
        Self::Channels(P2pChannelsAction::MessageReceived(a))
    }
}
