use serde::{Deserialize, Serialize};

use super::P2pDisconnectionReason;

pub type P2pDisconnectionActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pDisconnectionAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pDisconnectionAction {
    Init(P2pDisconnectionInitAction),
    Finish(P2pDisconnectionFinishAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pDisconnectionInitAction {
    pub peer_id: crate::PeerId,
    pub reason: P2pDisconnectionReason,
}

impl redux::EnablingCondition<crate::P2pState> for P2pDisconnectionInitAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| !peer.is_disconnected())
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
            .map_or(false, |peer| !peer.is_disconnected()) // TODO(akoptelov): review this
    }
}

// --- From<LeafAction> for Action impls.

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
