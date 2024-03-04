use serde::{Deserialize, Serialize};

use super::channels::P2pChannelsAction;
use super::connection::P2pConnectionAction;
use super::disconnection::P2pDisconnectionAction;
use super::discovery::P2pDiscoveryAction;
use super::network::P2pNetworkAction;
use super::peer::P2pPeerAction;
use crate::listen::P2pListenAction;

pub type P2pActionWithMeta = redux::ActionWithMeta<P2pAction>;
pub type P2pActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pAction>;

#[derive(Serialize, Deserialize, Debug, Clone, derive_more::From)]
pub enum P2pAction {
    Listen(P2pListenAction),
    Connection(P2pConnectionAction),
    Disconnection(P2pDisconnectionAction),
    Discovery(P2pDiscoveryAction),
    Channels(P2pChannelsAction),
    Peer(P2pPeerAction),
    Network(P2pNetworkAction),
}
