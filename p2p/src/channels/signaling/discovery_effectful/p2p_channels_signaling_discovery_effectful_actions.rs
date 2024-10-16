use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{
    channels::{signaling::discovery::SignalingDiscoveryChannelMsg, P2pChannelsEffectfulAction},
    connection::Offer,
    identity::PublicKey,
    webrtc::EncryptedAnswer,
    P2pState, PeerId,
};

#[derive(Debug, Clone, Serialize, Deserialize, ActionEvent)]
#[action_event(fields(display(peer_id)))]
pub enum P2pChannelsSignalingDiscoveryEffectfulAction {
    Init {
        peer_id: PeerId,
    },
    MessageSend {
        peer_id: PeerId,
        message: SignalingDiscoveryChannelMsg,
    },
    OfferEncryptAndSend {
        peer_id: PeerId,
        pub_key: PublicKey,
        offer: Box<Offer>,
    },
    AnswerDecrypt {
        peer_id: PeerId,
        pub_key: PublicKey,
        answer: EncryptedAnswer,
    },
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSignalingDiscoveryEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl From<P2pChannelsSignalingDiscoveryEffectfulAction> for crate::P2pEffectfulAction {
    fn from(action: P2pChannelsSignalingDiscoveryEffectfulAction) -> crate::P2pEffectfulAction {
        crate::P2pEffectfulAction::Channels(P2pChannelsEffectfulAction::SignalingDiscovery(action))
    }
}
