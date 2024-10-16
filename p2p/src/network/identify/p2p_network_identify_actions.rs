use crate::{
    network::identify::{
        stream::P2pNetworkIdentifyStreamAction,
        stream_effectful::P2pNetworkIdentifyStreamEffectfulAction,
    },
    P2pAction, P2pEffectfulAction, P2pNetworkAction, P2pNetworkEffectfulAction, P2pState,
};
use openmina_core::ActionEvent;
use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

/// Identify actions.
#[derive(Debug, Clone, Serialize, Deserialize, derive_more::From, ActionEvent)]
pub enum P2pNetworkIdentifyAction {
    Stream(P2pNetworkIdentifyStreamAction),
}

#[derive(Debug, Clone, Serialize, Deserialize, derive_more::From, ActionEvent)]
pub enum P2pNetworkIdentifyEffectfulAction {
    Stream(P2pNetworkIdentifyStreamEffectfulAction),
}

impl EnablingCondition<P2pState> for P2pNetworkIdentifyAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pNetworkIdentifyAction::Stream(action) => action.is_enabled(state, time),
        }
    }
}

impl EnablingCondition<P2pState> for P2pNetworkIdentifyEffectfulAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pNetworkIdentifyEffectfulAction::Stream(action) => action.is_enabled(state, time),
        }
    }
}

impl From<P2pNetworkIdentifyAction> for P2pAction {
    fn from(value: P2pNetworkIdentifyAction) -> Self {
        P2pNetworkAction::Identify(value).into()
    }
}

impl From<P2pNetworkIdentifyEffectfulAction> for P2pEffectfulAction {
    fn from(value: P2pNetworkIdentifyEffectfulAction) -> P2pEffectfulAction {
        P2pEffectfulAction::Network(P2pNetworkEffectfulAction::Identify(value))
    }
}
