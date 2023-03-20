use serde::{Deserialize, Serialize};

use super::connection::P2pConnectionAction;
use super::disconnection::P2pDisconnectionAction;

pub type P2pActionWithMeta = redux::ActionWithMeta<P2pAction>;
pub type P2pActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pAction {
    Connection(P2pConnectionAction),
    Disconnection(P2pDisconnectionAction),
    PeerReady(P2pPeerReadyAction),
}

// TODO(binier): feels out of place. Maybe move somewhere else?
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerReadyAction {
    pub peer_id: crate::PeerId,
}

impl redux::EnablingCondition<crate::P2pState> for P2pPeerReadyAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |p| p.status.is_connecting_success())
    }
}

impl From<P2pPeerReadyAction> for crate::P2pAction {
    fn from(value: P2pPeerReadyAction) -> Self {
        Self::PeerReady(value)
    }
}
