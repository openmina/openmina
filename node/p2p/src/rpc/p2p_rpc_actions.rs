use serde::{Deserialize, Serialize};

use super::outgoing::P2pRpcOutgoingAction;

pub type P2pRpcActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pRpcAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcAction {
    Outgoing(P2pRpcOutgoingAction),
}

impl P2pRpcAction {
    pub fn peer_id(&self) -> &crate::PeerId {
        match self {
            Self::Outgoing(v) => v.peer_id(),
        }
    }
}
