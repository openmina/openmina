use serde::{Deserialize, Serialize};

use crate::PeerId;

use super::{libp2p::P2pConnectionLibP2pAction, webrtc::P2pConnectionWebRTCAction};

pub type P2pConnectionActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pConnectionAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionAction {
    LibP2p(P2pConnectionLibP2pAction),
    WebRTC(P2pConnectionWebRTCAction),
}

impl P2pConnectionAction {
    pub fn peer_id(&self) -> Option<&PeerId> {
        match self {
            P2pConnectionAction::LibP2p(v) => v.peer_id(),
            P2pConnectionAction::WebRTC(v) => v.peer_id(),
        }
    }
}
