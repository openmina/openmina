use serde::{Deserialize, Serialize};

use crate::PeerId;

use super::{
    incoming::P2pConnectionLibP2pIncomingAction, outgoing::P2pConnectionLibP2pOutgoingAction,
};

pub type P2pConnectionLibP2pActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a P2pConnectionLibP2pAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionLibP2pAction {
    Outgoing(P2pConnectionLibP2pOutgoingAction),
    Incoming(P2pConnectionLibP2pIncomingAction),
}

impl P2pConnectionLibP2pAction {
    pub fn peer_id(&self) -> Option<&PeerId> {
        match self {
            P2pConnectionLibP2pAction::Outgoing(v) => v.peer_id(),
            P2pConnectionLibP2pAction::Incoming(v) => v.peer_id(),
        }
    }
}
