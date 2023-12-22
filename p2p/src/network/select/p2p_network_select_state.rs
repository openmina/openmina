use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use crate::P2pMioService;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkSelectState {
    //
}

impl P2pNetworkSelectAction {
    pub fn effects<Store, S>(&self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService,
    {
        match self {
            Self::IncomingData(a) => {
                dbg!(std::str::from_utf8(&a.data[1..])).unwrap_or_default();
            }
            Self::Init(a) => {
                let _ = dbg!(a);
            }
        }

        let _ = store;
    }
}

impl P2pNetworkSelectState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkSelectAction>) {
        let _ = action;
    }
}
