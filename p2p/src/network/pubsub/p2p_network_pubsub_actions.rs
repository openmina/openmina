use openmina_core::ActionEvent;

use serde::{Deserialize, Serialize};

use crate::{token::BroadcastAlgorithm, Data, P2pState, PeerId};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum P2pNetworkPubsubAction {
    NewStream {
        incoming: bool,
        peer_id: PeerId,
        protocol: BroadcastAlgorithm,
    },
    IncomingData {
        peer_id: PeerId,
        data: Data,
    },
    Broadcast {
        data: Data,
        topic: String,
    },
}

impl From<P2pNetworkPubsubAction> for crate::P2pAction {
    fn from(value: P2pNetworkPubsubAction) -> Self {
        crate::P2pAction::Network(value.into())
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkPubsubAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}
