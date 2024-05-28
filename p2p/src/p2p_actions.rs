use openmina_macros::ActionEvent;
use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

use super::channels::P2pChannelsAction;
use super::connection::P2pConnectionAction;
use super::disconnection::P2pDisconnectionAction;
use super::discovery::P2pDiscoveryAction;
#[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
use super::network::P2pNetworkAction;
use super::peer::P2pPeerAction;
#[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
use crate::identify::P2pIdentifyAction;
use crate::P2pState;

pub type P2pActionWithMeta = redux::ActionWithMeta<P2pAction>;
pub type P2pActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pAction>;

#[derive(Serialize, Deserialize, Debug, Clone, derive_more::From, ActionEvent)]
pub enum P2pAction {
    Initialization(P2pInitializeAction),
    Connection(P2pConnectionAction),
    Disconnection(P2pDisconnectionAction),
    Discovery(P2pDiscoveryAction),
    #[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
    Identify(P2pIdentifyAction),
    Channels(P2pChannelsAction),
    Peer(P2pPeerAction),
    #[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
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
