use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkRpcState {}

impl P2pNetworkRpcState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkRpcAction>) {
        let _ = action;
    }
}

impl P2pNetworkRpcAction {
    pub fn effects<Store, S>(&self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        let _ = store;
        match self {
            Self::Init(a) => {
                //
                let _ = a;
            }
            Self::IncomingData(a) => {
                dbg!(&a.peer_id, a.stream_id, &a.data);
            }
            Self::OutgoingData(_) => {}
        }
    }
}
