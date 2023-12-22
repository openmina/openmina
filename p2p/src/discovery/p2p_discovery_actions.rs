use serde::{Deserialize, Serialize};

use crate::{connection::outgoing::P2pConnectionOutgoingInitOpts, P2pState, PeerId};

// use super::{incoming::P2pConnectionIncomingAction, outgoing::P2pConnectionOutgoingAction};

pub type P2pDiscoveryActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pDiscoveryAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pDiscoveryAction {
    Init(P2pDiscoveryInitAction),
    Success(P2pDiscoverySuccessAction),
    KademliaBootstrap(P2pDiscoveryKademliaBootstrapAction),
    KademliaInit(P2pDiscoveryKademliaInitAction),
    KademliaAddRoute(P2pDiscoveryKademliaAddRouteAction),
    KademliaSuccess(P2pDiscoveryKademliaSuccessAction),
    KademliaFailure(P2pDiscoveryKademliaFailureAction),
}

impl redux::EnablingCondition<P2pState> for P2pDiscoveryAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        match self {
            Self::Init(action) => action.is_enabled(state),
            Self::Success(action) => action.is_enabled(state),
            Self::KademliaBootstrap(action) => action.is_enabled(state),
            Self::KademliaAddRoute(action) => action.is_enabled(state),
            Self::KademliaInit(action) => action.is_enabled(state),
            Self::KademliaSuccess(action) => action.is_enabled(state),
            Self::KademliaFailure(action) => action.is_enabled(state),
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
pub struct P2pDiscoveryKademliaAddRouteAction {
    pub peer_id: PeerId,
    pub addresses: Vec<P2pConnectionOutgoingInitOpts>,
}

impl redux::EnablingCondition<P2pState> for P2pDiscoveryKademliaAddRouteAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl From<P2pDiscoveryKademliaAddRouteAction> for crate::P2pAction {
    fn from(a: P2pDiscoveryKademliaAddRouteAction) -> Self {
        Self::Discovery(a.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pDiscoverySuccessAction {
    pub peer_id: PeerId,
    pub peers: Vec<P2pConnectionOutgoingInitOpts>,
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
pub struct P2pDiscoveryKademliaBootstrapAction {}

impl redux::EnablingCondition<P2pState> for P2pDiscoveryKademliaBootstrapAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        !state.kademlia.is_ready && !state.kademlia.is_bootstrapping
    }
}

impl From<P2pDiscoveryKademliaBootstrapAction> for crate::P2pAction {
    fn from(a: P2pDiscoveryKademliaBootstrapAction) -> Self {
        Self::Discovery(a.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pDiscoveryKademliaInitAction {}

impl redux::EnablingCondition<P2pState> for P2pDiscoveryKademliaInitAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.kademlia.is_ready
            && state.kademlia.outgoing_requests < 1
            && !state.already_knows_max_peers()
    }
}

impl From<P2pDiscoveryKademliaInitAction> for crate::P2pAction {
    fn from(a: P2pDiscoveryKademliaInitAction) -> Self {
        Self::Discovery(a.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pDiscoveryKademliaSuccessAction {
    pub peers: Vec<PeerId>,
}

impl redux::EnablingCondition<P2pState> for P2pDiscoveryKademliaSuccessAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl From<P2pDiscoveryKademliaSuccessAction> for crate::P2pAction {
    fn from(a: P2pDiscoveryKademliaSuccessAction) -> Self {
        Self::Discovery(a.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pDiscoveryKademliaFailureAction {
    pub description: String,
}

impl redux::EnablingCondition<P2pState> for P2pDiscoveryKademliaFailureAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}

impl From<P2pDiscoveryKademliaFailureAction> for crate::P2pAction {
    fn from(a: P2pDiscoveryKademliaFailureAction) -> Self {
        Self::Discovery(a.into())
    }
}
