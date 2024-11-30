use crate::{pubsub::pb::Message, ConnectionAddr, P2pState, PeerId};
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum P2pNetworkPubsubEffectfulAction {
    Sign {
        author: PeerId,
        topic: String,
        message: Message,
    },
    IncomingData {
        peer_id: PeerId,
        seen_limit: usize,
        addr: ConnectionAddr,
        messages: Vec<Message>,
    },
}

impl From<P2pNetworkPubsubEffectfulAction> for crate::P2pEffectfulAction {
    fn from(value: P2pNetworkPubsubEffectfulAction) -> crate::P2pEffectfulAction {
        crate::P2pEffectfulAction::Network(crate::P2pNetworkEffectfulAction::Pubsub(value))
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkPubsubEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}
