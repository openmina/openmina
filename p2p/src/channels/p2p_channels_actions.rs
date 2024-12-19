use openmina_core::log::ActionEvent;
use redux::Callback;
use serde::{Deserialize, Serialize};

use crate::{
    identity::PublicKey,
    webrtc::{EncryptedAnswer, EncryptedOffer, Offer, P2pConnectionResponse},
    P2pState, PeerId,
};

use super::{
    best_tip::P2pChannelsBestTipAction,
    rpc::P2pChannelsRpcAction,
    signaling::{
        discovery::P2pChannelsSignalingDiscoveryAction,
        exchange::P2pChannelsSignalingExchangeAction,
    },
    snark::P2pChannelsSnarkAction,
    snark_job_commitment::P2pChannelsSnarkJobCommitmentAction,
    streaming_rpc::P2pChannelsStreamingRpcAction,
    transaction::P2pChannelsTransactionAction,
    ChannelId, ChannelMsg, MsgId,
};

#[derive(Serialize, Deserialize, Debug, Clone, openmina_core::ActionEvent)]
pub enum P2pChannelsAction {
    MessageReceived(P2pChannelsMessageReceivedAction),
    SignalingDiscovery(P2pChannelsSignalingDiscoveryAction),
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
    InitChannel {
        peer_id: PeerId,
        id: ChannelId,
        on_success: Callback<PeerId>,
    },
    MessageSend {
        peer_id: PeerId,
        msg_id: MsgId,
        msg: ChannelMsg,
    },
    SignalingDiscoveryAnswerDecrypt {
        peer_id: PeerId,
        pub_key: PublicKey,
        answer: EncryptedAnswer,
    },
    SignalingDiscoveryOfferEncryptAndSend {
        peer_id: PeerId,
        pub_key: PublicKey,
        offer: Box<Offer>,
    },
    SignalingExchangeOfferDecrypt {
        peer_id: PeerId,
        pub_key: PublicKey,
        offer: EncryptedOffer,
    },
    SignalingExchangeAnswerEncryptAndSend {
        peer_id: PeerId,
        pub_key: PublicKey,
        answer: Option<P2pConnectionResponse>,
    },
}

impl P2pChannelsAction {
    pub fn peer_id(&self) -> Option<&PeerId> {
        match self {
            Self::MessageReceived(v) => Some(&v.peer_id),
            Self::SignalingDiscovery(v) => Some(v.peer_id()),
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
            P2pChannelsAction::SignalingDiscovery(a) => a.is_enabled(state, time),
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
    fn is_enabled(&self, _state: &crate::P2pState, _time: redux::Timestamp) -> bool {
        true
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
