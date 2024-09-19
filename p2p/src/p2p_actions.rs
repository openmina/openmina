use openmina_macros::ActionEvent;
use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

use super::channels::P2pChannelsAction;
use super::connection::P2pConnectionAction;
use super::disconnection::P2pDisconnectionAction;
use super::identify::P2pIdentifyAction;
use super::network::P2pNetworkAction;
use super::peer::P2pPeerAction;
use super::P2pState;

pub type P2pActionWithMeta = redux::ActionWithMeta<P2pAction>;
pub type P2pActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pAction>;

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

impl From<redux::AnyAction> for P2pAction {
    fn from(action: redux::AnyAction) -> Self {
        *action.0.downcast::<Self>().expect("Downcast failed")
    }
}
