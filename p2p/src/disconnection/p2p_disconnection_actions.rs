use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use super::P2pDisconnectionReason;
use crate::{P2pPeerStatus, P2pState, PeerId};

pub type P2pDisconnectionActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pDisconnectionAction>;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(peer_id), display(reason)), level = info)]
pub enum P2pDisconnectionAction {
    /// Initialize disconnection.
    #[action_event(level = debug)]
    Init {
        peer_id: PeerId,
        reason: P2pDisconnectionReason,
    },
    /// Finish disconnecting from a peer.
    #[action_event(level = debug)]
    Finish { peer_id: PeerId },
}

impl redux::EnablingCondition<P2pState> for P2pDisconnectionAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            P2pDisconnectionAction::Init { peer_id, .. }
            | P2pDisconnectionAction::Finish { peer_id } => {
                state.peers.get(peer_id).map_or(false, |peer| {
                    !matches!(peer.status, P2pPeerStatus::Disconnected { .. })
                })
            }
        }
    }
}
