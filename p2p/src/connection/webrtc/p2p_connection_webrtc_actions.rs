use serde::{Deserialize, Serialize};

use super::{incoming::P2pConnectionWebRTCIncomingAction, outgoing::P2pConnectionWebRTCOutgoingAction};

pub type P2pConnectionActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pConnectionWebRTCAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionWebRTCAction {
    Outgoing(P2pConnectionWebRTCOutgoingAction),
    Incoming(P2pConnectionWebRTCIncomingAction),
}

impl P2pConnectionWebRTCAction {
    pub fn peer_id(&self) -> Option<&crate::PeerId> {
        match self {
            Self::Outgoing(v) => v.peer_id(),
            Self::Incoming(v) => v.peer_id(),
        }
    }
}
