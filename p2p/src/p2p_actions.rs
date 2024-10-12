use openmina_macros::ActionEvent;
use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

use crate::channels::P2pChannelsEffectfulAction;
use crate::connection::P2pConnectionEffectfulAction;
use crate::disconnection_effectful::P2pDisconnectionEffectfulAction;
use crate::P2pNetworkEffectfulAction;

use super::channels::P2pChannelsAction;
use super::connection::P2pConnectionAction;
use super::disconnection::P2pDisconnectionAction;
use super::identify::P2pIdentifyAction;
use super::network::P2pNetworkAction;
use super::peer::P2pPeerAction;
use super::P2pState;

#[derive(Serialize, Deserialize, Debug, Clone, derive_more::From, ActionEvent)]
pub enum P2pAction {
    Initialization(P2pInitializeAction),
    Connection(P2pConnectionAction),
    Disconnection(P2pDisconnectionAction),
    Identify(P2pIdentifyAction),
    Channels(P2pChannelsAction),
    Peer(P2pPeerAction),
    Network(P2pNetworkAction),
}

#[derive(Serialize, Deserialize, Debug, Clone, derive_more::From, ActionEvent)]
pub enum P2pEffectfulAction {
    Initialize,
    Channels(P2pChannelsEffectfulAction),
    Connection(P2pConnectionEffectfulAction),
    Disconnection(P2pDisconnectionEffectfulAction),
    Network(P2pNetworkEffectfulAction),
}

#[derive(Serialize, Deserialize, Debug, Clone, derive_more::From, ActionEvent)]
#[action_event(level = info, fields(display(chain_id)))]
pub enum P2pInitializeAction {
    /// Initializes p2p layer.
    #[action_event(level = info)]
    Initialize { chain_id: openmina_core::ChainId },
}

impl EnablingCondition<P2pState> for P2pInitializeAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        // this action cannot be called for initialized p2p state
        false
    }
}

impl redux::EnablingCondition<crate::P2pState> for P2pAction {
    fn is_enabled(&self, state: &crate::P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pAction::Initialization(a) => a.is_enabled(state, time),
            P2pAction::Connection(a) => a.is_enabled(state, time),
            P2pAction::Disconnection(a) => a.is_enabled(state, time),
            P2pAction::Channels(a) => a.is_enabled(state, time),
            P2pAction::Peer(a) => a.is_enabled(state, time),
            P2pAction::Identify(a) => a.is_enabled(state, time),
            P2pAction::Network(a) => a.is_enabled(state, time),
        }
    }
}

impl redux::EnablingCondition<crate::P2pState> for P2pEffectfulAction {
    fn is_enabled(&self, state: &crate::P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pEffectfulAction::Channels(a) => a.is_enabled(state, time),
            P2pEffectfulAction::Connection(a) => a.is_enabled(state, time),
            P2pEffectfulAction::Disconnection(a) => a.is_enabled(state, time),
            P2pEffectfulAction::Network(a) => a.is_enabled(state, time),
            P2pEffectfulAction::Initialize => true,
        }
    }
}

impl From<redux::AnyAction> for P2pAction {
    fn from(action: redux::AnyAction) -> Self {
        *action.0.downcast::<Self>().expect("Downcast failed")
    }
}

impl From<redux::AnyAction> for P2pEffectfulAction {
    fn from(action: redux::AnyAction) -> Self {
        *action.0.downcast::<Self>().expect("Downcast failed")
    }
}
