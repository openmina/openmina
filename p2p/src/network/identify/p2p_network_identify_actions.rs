use crate::{
    network::identify::stream::P2pNetworkIdentifyStreamAction, P2pAction, P2pNetworkAction,
    P2pState,
};
use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

/// Identify actions.
#[derive(Debug, Clone, Serialize, Deserialize, derive_more::From)]
pub enum P2pNetworkIdentifyAction {
    Stream(P2pNetworkIdentifyStreamAction),
}

impl EnablingCondition<P2pState> for P2pNetworkIdentifyAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pNetworkIdentifyAction::Stream(action) => action.is_enabled(state, time),
        }
    }
}

impl From<P2pNetworkIdentifyAction> for P2pAction {
    fn from(value: P2pNetworkIdentifyAction) -> Self {
        P2pNetworkAction::Identify(value).into()
    }
}
