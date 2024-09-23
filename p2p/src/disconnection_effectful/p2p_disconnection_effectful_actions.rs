use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{P2pPeerStatus, P2pState, PeerId};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(peer_id), display(reason)), level = info)]
pub enum P2pDisconnectionEffectfulAction {
    /// Initialize disconnection.
    Init { peer_id: PeerId },
}

impl redux::EnablingCondition<P2pState> for P2pDisconnectionEffectfulAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            P2pDisconnectionEffectfulAction::Init { peer_id } => {
                state.peers.get(peer_id).map_or(false, |peer| {
                    !matches!(peer.status, P2pPeerStatus::Disconnected { .. })
                })
            }
        }
    }
}
