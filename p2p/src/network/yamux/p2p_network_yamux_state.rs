use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkYamuxState {}

impl P2pNetworkYamuxAction {
    pub fn effects<Store, S>(&self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        match self {
            Self::IncomingData(a) => {
                let _ = (a, store);
                unimplemented!()
            }
            Self::OutgoingData(a) => {
                let _ = a;
                unimplemented!()
            }
        }
    }
}

impl P2pNetworkYamuxState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkYamuxAction>) {
        match action.action() {
            P2pNetworkYamuxAction::IncomingData(a) => {
                let _ = a;
                unimplemented!()
            }
            P2pNetworkYamuxAction::OutgoingData(a) => {
                let _ = a;
                unimplemented!()
            }
        }
    }
}
