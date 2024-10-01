use crate::{network::identify::P2pNetworkIdentify, ConnectionAddr, P2pState, PeerId};
use openmina_macros::ActionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(peer_id), display(addr), debug(info)))]
pub enum P2pIdentifyAction {
    /// Open a new yamux stream to the remote peer to request its identity
    NewRequest {
        peer_id: PeerId,
        addr: ConnectionAddr,
    },
    /// Updates the P2P peer information based on the Identify message sent to us.
    UpdatePeerInformation {
        peer_id: PeerId,
        addr: ConnectionAddr,
        info: Box<P2pNetworkIdentify>,
    },
}

impl redux::EnablingCondition<P2pState> for P2pIdentifyAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            Self::NewRequest { peer_id, .. } => state.get_ready_peer(peer_id).is_some(),
            Self::UpdatePeerInformation { peer_id, .. } => state.get_ready_peer(peer_id).is_some(),
        }
    }
}
