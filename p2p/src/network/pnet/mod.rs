pub mod p2p_network_pnet_action;
pub use self::p2p_network_pnet_action::P2pNetworkPnetAction;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPnetState {}
