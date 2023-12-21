pub mod pnet;
pub use self::pnet::{P2pNetworkPnetAction, P2pNetworkPnetState};

pub mod connection;
pub use self::connection::{P2pNetworkConnectionAction, P2pNetworkConnectionState};

mod p2p_network_action;
pub use self::p2p_network_action::P2pNetworkAction;

mod service;
pub use self::service::*;

use redux::ActionMeta;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkState {
    pub connection: P2pNetworkConnectionState,
    pub pnet: P2pNetworkPnetState,
}

impl P2pNetworkAction {
    pub fn effects<Store, S>(&self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService,
    {
        match self {
            Self::Connection(v) => v.effects(meta, store),
            Self::Pnet(_) => {}
        }
    }
}
