use serde::{Deserialize, Serialize};

use super::outgoing::P2pConnectionOutgoingAction;

pub type P2pConnectionActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pConnectionAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionAction {
    Outgoing(P2pConnectionOutgoingAction),
}

impl P2pConnectionAction {
    pub fn peer_id(&self) -> Option<&crate::PeerId> {
        match self {
            Self::Outgoing(v) => v.peer_id(),
        }
    }
}
