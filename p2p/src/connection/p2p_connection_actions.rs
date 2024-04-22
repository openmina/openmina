use serde::{Deserialize, Serialize};

use crate::P2pState;

use super::{incoming::P2pConnectionIncomingAction, outgoing::P2pConnectionOutgoingAction};

pub type P2pConnectionActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pConnectionAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionAction {
    Outgoing(P2pConnectionOutgoingAction),
    Incoming(P2pConnectionIncomingAction),
}

impl P2pConnectionAction {
    pub fn peer_id(&self) -> Option<&crate::PeerId> {
        match self {
            Self::Outgoing(v) => v.peer_id(),
            Self::Incoming(v) => v.peer_id(),
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pConnectionAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pConnectionAction::Outgoing(action) => action.is_enabled(state, time),
            P2pConnectionAction::Incoming(action) => action.is_enabled(state, time),
        }
    }
}
