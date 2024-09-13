use crate::{P2pState, PeerId};
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum P2pNetworkPubsubEffectfulAction {
    Sign { author: PeerId, topic: String },
    IncomingData { peer_id: PeerId, seen_limit: usize },
}

impl From<P2pNetworkPubsubEffectfulAction> for crate::P2pAction {
    fn from(value: P2pNetworkPubsubEffectfulAction) -> Self {
        crate::P2pAction::Network(value.into())
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkPubsubEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}
