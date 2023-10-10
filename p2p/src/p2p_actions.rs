use serde::{Deserialize, Serialize};

use super::channels::P2pChannelsAction;
use super::connection::P2pConnectionAction;
use super::disconnection::P2pDisconnectionAction;
use super::discovery::P2pDiscoveryAction;
use super::peer::P2pPeerAction;

pub type P2pActionWithMeta = redux::ActionWithMeta<P2pAction>;
pub type P2pActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pAction {
    Connection(P2pConnectionAction),
    Disconnection(P2pDisconnectionAction),
    Discovery(P2pDiscoveryAction),
    Channels(P2pChannelsAction),
    Peer(P2pPeerAction),
}
