use openmina_core::log::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

use super::{
    best_tip::P2pChannelsBestTipAction, rpc::P2pChannelsRpcAction, snark::P2pChannelsSnarkAction,
    snark_job_commitment::P2pChannelsSnarkJobCommitmentAction,
    transaction::P2pChannelsTransactionAction, ChannelMsg,
};

pub type P2pChannelsActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pChannelsAction>;

#[derive(Serialize, Deserialize, Debug, Clone, openmina_core::ActionEvent)]
pub enum P2pChannelsAction {
    MessageReceived(P2pChannelsMessageReceivedAction),

    BestTip(P2pChannelsBestTipAction),
    Transaction(P2pChannelsTransactionAction),
    Snark(P2pChannelsSnarkAction),
    SnarkJobCommitment(P2pChannelsSnarkJobCommitmentAction),
    Rpc(P2pChannelsRpcAction),
}

impl P2pChannelsAction {
    pub fn peer_id(&self) -> Option<&PeerId> {
        match self {
            Self::MessageReceived(v) => Some(&v.peer_id),
            Self::BestTip(v) => Some(v.peer_id()),
            Self::Transaction(v) => v.peer_id(),
            Self::Snark(v) => v.peer_id(),
            Self::SnarkJobCommitment(v) => Some(v.peer_id()),
            Self::Rpc(v) => Some(v.peer_id()),
        }
    }
}

impl redux::EnablingCondition<crate::P2pState> for P2pChannelsAction {
    fn is_enabled(&self, state: &crate::P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pChannelsAction::MessageReceived(a) => a.is_enabled(state, time),
            P2pChannelsAction::BestTip(a) => a.is_enabled(state, time),
            P2pChannelsAction::Snark(a) => a.is_enabled(state, time),
            P2pChannelsAction::SnarkJobCommitment(a) => a.is_enabled(state, time),
            P2pChannelsAction::Rpc(a) => a.is_enabled(state, time),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsMessageReceivedAction {
    pub peer_id: PeerId,
    pub message: ChannelMsg,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsMessageReceivedAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
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

impl ActionEvent for P2pChannelsMessageReceivedAction {
    fn action_event<T>(&self, _context: &T)
    where
        T: openmina_core::log::EventContext,
    {
    }
}
