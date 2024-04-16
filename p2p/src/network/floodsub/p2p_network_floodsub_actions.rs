use crate::{
    network::floodsub::stream::P2pNetworkFloodsubStreamAction, P2pAction, P2pNetworkAction,
    P2pState,
};
use openmina_core::ActionEvent;
use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, derive_more::From, ActionEvent)]

pub enum P2pNetworkFloodsubAction {
    Stream(P2pNetworkFloodsubStreamAction),
}

impl EnablingCondition<P2pState> for P2pNetworkFloodsubAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pNetworkFloodsubAction::Stream(action) => action.is_enabled(state, time),
        }
    }
}

impl From<P2pNetworkFloodsubAction> for P2pAction {
    fn from(value: P2pNetworkFloodsubAction) -> Self {
        P2pNetworkAction::Floodsub(value).into()
    }
}
