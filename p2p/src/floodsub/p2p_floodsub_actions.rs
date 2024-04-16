use crate::{network::floodsub::P2pNetworkFloodsub, P2pState, PeerId};
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(peer_id), debug(addr)))]
pub enum P2pFloodsubAction {
    /// Opens the outbound stream
    NewOutboundStream { peer_id: PeerId, addr: SocketAddr },
}

impl redux::EnablingCondition<P2pState> for P2pFloodsubAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            Self::NewOutboundStream { peer_id, .. } => state.get_ready_peer(peer_id).is_some(),
        }
    }
}
