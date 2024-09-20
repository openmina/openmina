use super::{
    incoming::P2pConnectionIncomingAction,
    incoming_effectful::P2pConnectionIncomingEffectfulAction,
    outgoing::P2pConnectionOutgoingAction,
    outgoing_effectful::P2pConnectionOutgoingEffectfulAction,
};
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum P2pConnectionAction {
    Outgoing(P2pConnectionOutgoingAction),
    Incoming(P2pConnectionIncomingAction),
}

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum P2pConnectionEffectfulAction {
    Outgoing(P2pConnectionOutgoingEffectfulAction),
    Incoming(P2pConnectionIncomingEffectfulAction),
}

impl redux::EnablingCondition<crate::P2pState> for P2pConnectionAction {
    fn is_enabled(&self, state: &crate::P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pConnectionAction::Outgoing(a) => a.is_enabled(state, time),
            P2pConnectionAction::Incoming(a) => a.is_enabled(state, time),
        }
    }
}

impl redux::EnablingCondition<crate::P2pState> for P2pConnectionEffectfulAction {
    fn is_enabled(&self, state: &crate::P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pConnectionEffectfulAction::Outgoing(a) => a.is_enabled(state, time),
            P2pConnectionEffectfulAction::Incoming(a) => a.is_enabled(state, time),
        }
    }
}
