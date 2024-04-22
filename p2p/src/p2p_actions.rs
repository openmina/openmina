use serde::{Deserialize, Serialize};

use crate::P2pState;

use super::channels::P2pChannelsAction;
use super::connection::P2pConnectionAction;
use super::disconnection::P2pDisconnectionAction;
use super::discovery::P2pDiscoveryAction;
use super::network::P2pNetworkAction;
use super::peer::P2pPeerAction;

pub type P2pActionWithMeta = redux::ActionWithMeta<P2pAction>;
pub type P2pActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pAction>;

#[derive(Serialize, Deserialize, Debug, Clone, derive_more::From)]
#[allow(clippy::large_enum_variant)]
pub enum P2pAction {
    Connection(P2pConnectionAction),
    Disconnection(P2pDisconnectionAction),
    Discovery(P2pDiscoveryAction),
    Channels(P2pChannelsAction),
    Peer(P2pPeerAction),
    Network(P2pNetworkAction),
}

impl redux::EnablingCondition<P2pState> for P2pAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pAction::Connection(action) => action.is_enabled(state, time),
            P2pAction::Disconnection(action) => action.is_enabled(state, time),
            P2pAction::Discovery(action) => action.is_enabled(state, time),
            P2pAction::Channels(action) => action.is_enabled(state, time),
            P2pAction::Peer(action) => action.is_enabled(state, time),
            P2pAction::Network(action) => action.is_enabled(state, time),
        }
    }
}
