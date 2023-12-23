use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use crate::{P2pCryptoService, P2pMioService};

use super::*;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseState {}

impl P2pNetworkNoiseAction {
    pub fn effects<Store, S>(&self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService + P2pCryptoService,
    {
        match self {
            Self::Init(a) => {
                let _ = store;
                dbg!(a.addr, a.incoming);
            }
            Self::IncomingData(a) => {
                dbg!(a.addr, a.data.len());
            }
        }
    }
}

impl P2pNetworkNoiseState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkNoiseAction>) {
        let _ = action;
    }
}
