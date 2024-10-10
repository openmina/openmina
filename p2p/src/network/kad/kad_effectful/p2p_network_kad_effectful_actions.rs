use openmina_core::ActionEvent;

use multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum P2pNetworkKadEffectfulAction {
    Discovered {
        multiaddr: Multiaddr,
        filter_local: bool,
        peer_id: PeerId,
    },
    MakeRequest {
        multiaddr: Vec<Multiaddr>,
        filter_local: bool,
        peer_id: PeerId,
    },
}

impl From<P2pNetworkKadEffectfulAction> for crate::P2pAction {
    fn from(value: P2pNetworkKadEffectfulAction) -> Self {
        crate::P2pAction::Network(value.into())
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkKadEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}
