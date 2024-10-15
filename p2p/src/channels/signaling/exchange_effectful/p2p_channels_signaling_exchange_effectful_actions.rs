use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{
    channels::{signaling::exchange::SignalingExchangeChannelMsg, P2pChannelsEffectfulAction},
    connection::P2pConnectionResponse,
    identity::PublicKey,
    webrtc::EncryptedOffer,
    P2pState, PeerId,
};

#[derive(Debug, Clone, Serialize, Deserialize, ActionEvent)]
#[action_event(fields(display(peer_id)))]
pub enum P2pChannelsSignalingExchangeEffectfulAction {
    Init {
        peer_id: PeerId,
    },
    MessageSend {
        peer_id: PeerId,
        message: SignalingExchangeChannelMsg,
    },
    OfferDecrypt {
        peer_id: PeerId,
        pub_key: PublicKey,
        offer: EncryptedOffer,
    },
    AnswerEncryptAndSend {
        peer_id: PeerId,
        pub_key: PublicKey,
        answer: Option<P2pConnectionResponse>,
    },
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSignalingExchangeEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl From<P2pChannelsSignalingExchangeEffectfulAction> for crate::P2pEffectfulAction {
    fn from(action: P2pChannelsSignalingExchangeEffectfulAction) -> crate::P2pEffectfulAction {
        crate::P2pEffectfulAction::Channels(P2pChannelsEffectfulAction::SignalingExchange(action))
    }
}
