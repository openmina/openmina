pub mod pnet;
pub use self::pnet::*;

pub mod connection;
pub use self::connection::*;

mod p2p_network_actions;
pub use self::p2p_network_actions::*;

mod service;
pub use self::service::*;

use redux::ActionMeta;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkState {
    pub connection: P2pNetworkConnectionState,
    pub pnet: P2pNetworkPnetState,
}

impl P2pNetworkState {
    pub fn new(chain_id: &str) -> Self {
        let pnet_key = {
            use blake2::{
                digest::{generic_array::GenericArray, Update, VariableOutput},
                Blake2bVar,
            };

            let mut key = GenericArray::default();
            Blake2bVar::new(32)
                .expect("valid constant")
                .chain(b"/coda/0.0.1/")
                .chain(chain_id.as_bytes())
                .finalize_variable(&mut key)
                .expect("good buffer size");
            key.into()
        };

        P2pNetworkState {
            connection: P2pNetworkConnectionState::default(),
            pnet: P2pNetworkPnetState::new(pnet_key),
        }
    }
}

impl P2pNetworkAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService + P2pCryptoService,
        P2pNetworkPnetAction: redux::EnablingCondition<S>,
    {
        match self {
            Self::Connection(v) => v.effects(meta, store),
            Self::Pnet(v) => v.effects(meta, store),
        }
    }
}
