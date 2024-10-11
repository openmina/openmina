use openmina_core::log::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

use super::{
    best_tip::P2pChannelsBestTipAction,
    best_tip_effectful::P2pChannelsBestTipEffectfulAction,
    rpc::P2pChannelsRpcAction,
    rpc_effectful::P2pChannelsRpcEffectfulAction,
    signaling::{
        exchange::P2pChannelsSignalingExchangeAction,
        exchange_effectful::P2pChannelsSignalingExchangeEffectfulAction,
    },
    snark::P2pChannelsSnarkAction,
    snark_effectful::P2pChannelsSnarkEffectfulAction,
    snark_job_commitment::P2pChannelsSnarkJobCommitmentAction,
    snark_job_commitment_effectful::P2pChannelsSnarkJobCommitmentEffectfulAction,
    streaming_rpc::P2pChannelsStreamingRpcAction,
    streaming_rpc_effectful::P2pChannelsStreamingRpcEffectfulAction,
    transaction::P2pChannelsTransactionAction,
    transaction_effectful::P2pChannelsTransactionEffectfulAction,
    ChannelMsg,
};

#[derive(Serialize, Deserialize, Debug, Clone, openmina_core::ActionEvent)]
pub enum P2pChannelsAction {
    MessageReceived(P2pChannelsMessageReceivedAction),
    SignalingExchange(P2pChannelsSignalingExchangeAction),
    BestTip(P2pChannelsBestTipAction),
    Transaction(P2pChannelsTransactionAction),
    Snark(P2pChannelsSnarkAction),
    SnarkJobCommitment(P2pChannelsSnarkJobCommitmentAction),
    Rpc(P2pChannelsRpcAction),
    StreamingRpc(P2pChannelsStreamingRpcAction),
}

#[derive(Serialize, Deserialize, Debug, Clone, openmina_core::ActionEvent)]
pub enum P2pChannelsEffectfulAction {
    SignalingExchange(P2pChannelsSignalingExchangeEffectfulAction),
    BestTip(P2pChannelsBestTipEffectfulAction),
    Rpc(P2pChannelsRpcEffectfulAction),
    Snark(P2pChannelsSnarkEffectfulAction),
    SnarkJobCommitment(P2pChannelsSnarkJobCommitmentEffectfulAction),
    StreamingRpc(P2pChannelsStreamingRpcEffectfulAction),
    Transaction(P2pChannelsTransactionEffectfulAction),
}

impl P2pChannelsAction {
    pub fn peer_id(&self) -> Option<&PeerId> {
        match self {
            Self::MessageReceived(v) => Some(&v.peer_id),
            Self::SignalingExchange(v) => Some(v.peer_id()),
            Self::BestTip(v) => Some(v.peer_id()),
            Self::Transaction(v) => v.peer_id(),
            Self::Snark(v) => v.peer_id(),
            Self::SnarkJobCommitment(v) => Some(v.peer_id()),
            Self::Rpc(v) => Some(v.peer_id()),
            Self::StreamingRpc(v) => Some(v.peer_id()),
        }
    }
}

impl redux::EnablingCondition<crate::P2pState> for P2pChannelsAction {
    fn is_enabled(&self, state: &crate::P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pChannelsAction::MessageReceived(a) => a.is_enabled(state, time),
            P2pChannelsAction::SignalingExchange(a) => a.is_enabled(state, time),
            P2pChannelsAction::Transaction(a) => a.is_enabled(state, time),
            P2pChannelsAction::BestTip(a) => a.is_enabled(state, time),
            P2pChannelsAction::Snark(a) => a.is_enabled(state, time),
            P2pChannelsAction::SnarkJobCommitment(a) => a.is_enabled(state, time),
            P2pChannelsAction::Rpc(a) => a.is_enabled(state, time),
            P2pChannelsAction::StreamingRpc(a) => a.is_enabled(state, time),
        }
    }
}

impl redux::EnablingCondition<crate::P2pState> for P2pChannelsEffectfulAction {
    fn is_enabled(&self, state: &crate::P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pChannelsEffectfulAction::SignalingExchange(a) => a.is_enabled(state, time),
            P2pChannelsEffectfulAction::BestTip(a) => a.is_enabled(state, time),
            P2pChannelsEffectfulAction::Transaction(a) => a.is_enabled(state, time),
            P2pChannelsEffectfulAction::StreamingRpc(a) => a.is_enabled(state, time),
            P2pChannelsEffectfulAction::SnarkJobCommitment(a) => a.is_enabled(state, time),
            P2pChannelsEffectfulAction::Rpc(a) => a.is_enabled(state, time),
            P2pChannelsEffectfulAction::Snark(a) => a.is_enabled(state, time),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsMessageReceivedAction {
    pub peer_id: PeerId,
    pub message: Box<ChannelMsg>,
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
