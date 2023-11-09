use serde::{Deserialize, Serialize};

pub type P2pDisconnectionActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pDisconnectionAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pDisconnectionAction {
    Init(P2pDisconnectionInitAction),
    Finish(P2pDisconnectionFinishAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pDisconnectionInitAction {
    pub peer_id: crate::PeerId,
}

impl redux::EnablingCondition<crate::P2pState> for P2pDisconnectionInitAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Disconnected { .. } => false,
                _ => true,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pDisconnectionFinishAction {
    pub peer_id: crate::PeerId,
}

impl redux::EnablingCondition<crate::P2pState> for P2pDisconnectionFinishAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Disconnected { .. } => false,
                _ => true,
            })
    }
}

// --- From<LeafAction> for Action impls.
use crate::P2pPeerStatus;

impl From<P2pDisconnectionInitAction> for crate::P2pAction {
    fn from(a: P2pDisconnectionInitAction) -> Self {
        Self::Disconnection(P2pDisconnectionAction::Init(a.into()))
    }
}

impl From<P2pDisconnectionFinishAction> for crate::P2pAction {
    fn from(a: P2pDisconnectionFinishAction) -> Self {
        Self::Disconnection(P2pDisconnectionAction::Finish(a.into()))
    }
}
