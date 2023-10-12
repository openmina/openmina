use mina_p2p_messages::v2;
use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

// use super::{incoming::P2pConnectionIncomingAction, outgoing::P2pConnectionOutgoingAction};

pub type P2pDiscoveryActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pDiscoveryAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pDiscoveryAction {
    Init(P2pDiscoveryInitAction),
    Success(P2pDiscoverySuccessAction),
    Timeout(P2pDiscoveryTimeoutAction),
}

impl redux::EnablingCondition<P2pState> for P2pDiscoveryAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        match self {
            Self::Init(action) => action.is_enabled(state),
            Self::Success(action) => action.is_enabled(state),
            Self::Timeout(action) => action.is_enabled(state),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pDiscoveryInitAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pDiscoveryInitAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).is_some()
    }
}

impl From<P2pDiscoveryInitAction> for crate::P2pAction {
    fn from(a: P2pDiscoveryInitAction) -> Self {
        Self::Discovery(a.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pDiscoverySuccessAction {
    pub peer_id: PeerId,
    pub peers: Vec<v2::NetworkPeerPeerStableV1>,
}

impl redux::EnablingCondition<P2pState> for P2pDiscoverySuccessAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl From<P2pDiscoverySuccessAction> for crate::P2pAction {
    fn from(a: P2pDiscoverySuccessAction) -> Self {
        Self::Discovery(a.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pDiscoveryTimeoutAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pDiscoveryTimeoutAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl From<P2pDiscoveryTimeoutAction> for crate::P2pAction {
    fn from(a: P2pDiscoveryTimeoutAction) -> Self {
        Self::Discovery(a.into())
    }
}
