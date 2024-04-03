use openmina_core::{action_info, log::ActionEvent};
use serde::{Deserialize, Serialize};

use super::P2pDisconnectionReason;
use crate::{P2pPeerStatus, P2pState, PeerId};

pub type P2pDisconnectionActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pDisconnectionAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pDisconnectionAction {
    Init {
        peer_id: PeerId,
        reason: P2pDisconnectionReason,
    },
    Finish {
        peer_id: PeerId,
    },
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

impl ActionEvent for P2pDisconnectionAction {
    fn action_event<T>(&self, context: &T)
    where
        T: openmina_core::log::EventContext,
    {
        match self {
            P2pDisconnectionAction::Init { peer_id, reason } => action_info!(
                context,
                peer_id = display(peer_id),
                reason = display(reason)
            ),
            P2pDisconnectionAction::Finish { peer_id } => {
                action_info!(context, peer_id = display(peer_id))
            }
        }
    }
}
